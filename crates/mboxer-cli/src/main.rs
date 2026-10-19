//! `mboxer-cli` command line: scan (task 002), index (task 003) and search with filters (tasks 003/004).

use std::collections::HashSet;
use std::error::Error;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use clap::{Args, Parser, Subcommand};
use mboxer_core::mail_parser::{DateTime, Message};
use mboxer_core::{
    BuildOptions, IndexStatus, Mailbox, MboxIndex, MessageSummary, Opened, PartInfo, Progress,
    RawMessage, SearchQuery, Sort, build, parse, part_tree, status, summarize,
};

#[derive(Parser)]
#[command(
    name = "mboxer-cli",
    version,
    about = "Tools for large Gmail Takeout mbox archives"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Walk an mbox once: count messages, report throughput, show a sample.
    Scan(ScanArgs),
    /// Build the sidecar full-text index next to an mbox (skipped when it is up to date).
    #[command(after_help = index_help())]
    Index(IndexArgs),
    /// Search an indexed mbox.
    Search(SearchArgs),
}

#[derive(Args)]
struct ScanArgs {
    /// Path to the mbox file.
    mbox: PathBuf,
    /// Print the headers and MIME tree of the first N messages.
    #[arg(long, default_value_t = 3)]
    sample: usize,
    /// Stop after N messages.
    #[arg(long)]
    limit: Option<u64>,
    /// No progress output on stderr.
    #[arg(long)]
    quiet: bool,
}

/// The sidecar folder and the fallback index home (D-005) as this platform spells them.
#[cfg(windows)]
const INDEX_PLACES: (&str, &str) = (".mboxer\\<file name>\\", "%LOCALAPPDATA%\\mboxer\\index\\");
#[cfg(target_os = "macos")]
const INDEX_PLACES: (&str, &str) = (
    ".mboxer/<file name>/",
    "~/Library/Application Support/mboxer/index/",
);
#[cfg(not(any(windows, target_os = "macos")))]
const INDEX_PLACES: (&str, &str) = (
    ".mboxer/<file name>/",
    "$XDG_DATA_HOME/mboxer/index/ (~/.local/share by default)",
);

fn index_help() -> String {
    let (sidecar, fallback) = INDEX_PLACES;
    format!(
        "The index goes to {sidecar} next to the mbox. When that folder is not writable (a read-only \
share, a DVD), it goes under {fallback} instead, or under MBOXER_INDEX_HOME when that variable is \
set; `search` looks in both places."
    )
}

#[derive(Args)]
struct IndexArgs {
    /// Path to the mbox file.
    mbox: PathBuf,
    /// Rebuild even when the existing index matches the mbox.
    #[arg(long)]
    force: bool,
    /// Indexing threads (default: up to 4).
    #[arg(long)]
    threads: Option<usize>,
    /// No progress output on stderr.
    #[arg(long)]
    quiet: bool,
}

#[derive(Args)]
struct SearchArgs {
    /// Path to the mbox file (its index must exist, see `mboxer-cli index`).
    mbox: PathBuf,
    /// Free-text query over subject, body, addresses and attachment names; words are AND-ed,
    /// "quotes" make a phrase. Optional when a filter is given.
    #[arg(num_args = 0..)]
    query: Vec<String>,
    /// Words that must appear in the sender (name or address).
    #[arg(long, value_name = "WORDS")]
    from: Option<String>,
    /// Words that must appear in a recipient (To or Cc).
    #[arg(long, value_name = "WORDS")]
    to: Option<String>,
    /// Words that must appear in the subject.
    #[arg(long, value_name = "WORDS")]
    subject: Option<String>,
    /// Only messages dated on or after this day (YYYY-MM-DD, UTC) or RFC 3339 instant.
    #[arg(long, value_name = "DATE", value_parser = parse_date)]
    after: Option<i64>,
    /// Only messages dated before this day (YYYY-MM-DD, UTC, exclusive) or RFC 3339 instant.
    #[arg(long, value_name = "DATE", value_parser = parse_date)]
    before: Option<i64>,
    /// Exact Gmail label (repeatable; every one must be present).
    #[arg(long, value_name = "NAME")]
    label: Vec<String>,
    /// Only messages with an attachment.
    #[arg(long, conflicts_with = "no_attachment")]
    has_attachment: bool,
    /// Only messages without an attachment.
    #[arg(long)]
    no_attachment: bool,
    /// Number of hits to print.
    #[arg(long, default_value_t = 20)]
    limit: usize,
    /// Skip the first N hits (paging).
    #[arg(long, default_value_t = 0, value_name = "N")]
    offset: usize,
    /// Print the hit count only.
    #[arg(long)]
    count: bool,
    /// Order by match quality instead of date.
    #[arg(long)]
    relevance: bool,
}

/// Parses `--after` / `--before`: a day (`2026-01-13`, UTC midnight) or a full RFC 3339 instant.
fn parse_date(text: &str) -> Result<i64, String> {
    let text = text.trim();
    let instant = if text.contains('T') {
        text.to_owned()
    } else {
        format!("{text}T00:00:00Z")
    };
    DateTime::parse_rfc3339(&instant)
        .filter(DateTime::is_valid)
        .map(|date| date.to_timestamp())
        .ok_or_else(|| format!("`{text}` is not a date (expected YYYY-MM-DD or RFC 3339)"))
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Scan(args) => scan(&args),
        Command::Index(args) => index(&args),
        Command::Search(args) => search(&args),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

#[derive(Default)]
struct Stats {
    count: u64,
    largest: Option<(u64, u64)>,
    parse_failures: u64,
    first_parse_failures: Vec<u64>,
    labels: HashSet<String>,
    with_attachment: u64,
}

fn scan(args: &ScanArgs) -> Result<(), Box<dyn Error>> {
    let total_bytes = std::fs::metadata(&args.mbox)?.len();
    let mut reader = mboxer_core::MboxReader::open(&args.mbox)?;
    let start = Instant::now();
    let mut last_report = start;
    let mut stats = Stats::default();
    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    while let Some(item) = reader.next() {
        let raw = item?;
        stats.count += 1;
        if stats.largest.is_none_or(|(_, len)| raw.len > len) {
            stats.largest = Some((raw.offset, raw.len));
        }
        match parse(&raw) {
            Some(message) => {
                let summary = summarize(&raw, &message);
                for label in &summary.labels {
                    if !stats.labels.contains(label) {
                        stats.labels.insert(label.clone());
                    }
                }
                if summary.has_attachment {
                    stats.with_attachment += 1;
                }
                if stats.count as usize <= args.sample {
                    print_sample(&mut out, stats.count, &raw, &message, &summary)?;
                }
            }
            None => {
                stats.parse_failures += 1;
                if stats.first_parse_failures.len() < 10 {
                    stats.first_parse_failures.push(raw.offset);
                }
            }
        }
        if args.limit.is_some_and(|limit| stats.count >= limit) {
            break;
        }
        if !args.quiet && last_report.elapsed() >= Duration::from_secs(2) {
            progress(reader.position(), total_bytes, stats.count, start.elapsed());
            last_report = Instant::now();
        }
    }
    if !args.quiet {
        eprintln!();
    }

    let elapsed = start.elapsed();
    let bytes = reader.position();
    writeln!(out, "file:              {}", args.mbox.display())?;
    writeln!(out, "messages:          {}", stats.count)?;
    writeln!(
        out,
        "bytes read:        {} ({:.2} GB of {:.2} GB)",
        bytes,
        gb(bytes),
        gb(total_bytes)
    )?;
    writeln!(
        out,
        "elapsed:           {:.1} s ({:.0} MB/s)",
        elapsed.as_secs_f64(),
        mb_per_s(bytes, elapsed)
    )?;
    if let Some((offset, len)) = stats.largest {
        writeln!(out, "largest message:   {len} bytes at offset {offset}")?;
    }
    writeln!(out, "with attachment:   {}", stats.with_attachment)?;
    writeln!(out, "distinct labels:   {}", stats.labels.len())?;
    writeln!(out, "parse failures:    {}", stats.parse_failures)?;
    for offset in &stats.first_parse_failures {
        writeln!(out, "  unparsed message at offset {offset}")?;
    }
    Ok(())
}

fn index(args: &IndexArgs) -> Result<(), Box<dyn Error>> {
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    // An index whose state cannot be told is rebuilt under `--force`; an
    // mbox that cannot be read fails the build itself.
    match status(&args.mbox) {
        Err(err) if args.force => {
            if !args.quiet {
                eprintln!("rebuilding: {err}");
            }
        }
        Err(err) => return Err(err.into()),
        Ok(IndexStatus::Fresh(meta)) if !args.force => {
            writeln!(
                out,
                "index is up to date: {} messages, built {} (use --force to rebuild)",
                meta.message_count,
                DateTime::from_timestamp(meta.built_at_unix).to_rfc3339()
            )?;
            return Ok(());
        }
        Ok(IndexStatus::Fresh(_)) => {}
        Ok(IndexStatus::Missing) => {
            if !args.quiet {
                eprintln!("no index yet, building");
            }
        }
        Ok(IndexStatus::Stale(reason)) => {
            if !args.quiet {
                eprintln!("rebuilding: {reason}");
            }
        }
    }

    let mut options = BuildOptions::default();
    if let Some(threads) = args.threads {
        options.threads = threads.max(1);
    }
    let start = Instant::now();
    let mut last_report = start;
    let quiet = args.quiet;
    let report = build(
        &args.mbox,
        &options,
        |p: Progress| {
            if !quiet && last_report.elapsed() >= Duration::from_secs(2) {
                progress(p.bytes, p.total_bytes, p.messages, start.elapsed());
                last_report = Instant::now();
            }
        },
        &AtomicBool::new(false),
    )?;
    if !quiet {
        eprintln!();
    }
    writeln!(out, "file:              {}", args.mbox.display())?;
    writeln!(
        out,
        "index:             {}{}",
        report.index_dir.display(),
        if report.fallback {
            " (fallback: the mbox folder is not writable)"
        } else {
            ""
        }
    )?;
    writeln!(out, "messages:          {}", report.messages)?;
    writeln!(out, "parse failures:    {}", report.parse_failures)?;
    writeln!(out, "skipped:           {}", report.skipped)?;
    for offset in &report.skipped_offsets {
        writeln!(
            out,
            "  message at offset {offset} is over the {}-byte limit, not indexed",
            options.max_message_bytes
        )?;
    }
    writeln!(
        out,
        "elapsed:           {:.1} s",
        report.elapsed.as_secs_f64()
    )?;
    writeln!(
        out,
        "index size:        {} bytes ({:.2} GB)",
        report.index_bytes,
        gb(report.index_bytes)
    )?;
    writeln!(out, "threads:           {}", options.threads)?;
    Ok(())
}

fn search(args: &SearchArgs) -> Result<(), Box<dyn Error>> {
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let open_start = Instant::now();
    let index = match MboxIndex::open(&args.mbox)? {
        Opened::Ready(index) => index,
        Opened::Missing => {
            return Err("no index for this mbox; run `mboxer-cli index` first".into());
        }
        Opened::Stale(reason) => {
            return Err(format!("index is stale ({reason}); run `mboxer-cli index`").into());
        }
    };
    let open_elapsed = open_start.elapsed();

    let text = args.query.join(" ");
    let query = SearchQuery {
        text: Some(text.clone()),
        from: args.from.clone(),
        to: args.to.clone(),
        subject: args.subject.clone(),
        after: args.after,
        before: args.before,
        labels: args.label.clone(),
        any_labels: Vec::new(),
        has_attachment: match (args.has_attachment, args.no_attachment) {
            (true, _) => Some(true),
            (_, true) => Some(false),
            _ => None,
        },
        group_threads: false,
        prefix_last_word: false,
        limit: if args.count { 0 } else { args.limit },
        offset: args.offset,
        sort: if args.relevance {
            Sort::Relevance
        } else {
            Sort::DateDesc
        },
    };
    let search_start = Instant::now();
    let result = index.search(&query)?;
    let search_elapsed = search_start.elapsed();

    writeln!(
        out,
        "query:             {}",
        if text.is_empty() { "-" } else { &text }
    )?;
    writeln!(out, "filters:           {}", describe_filters(&query))?;
    writeln!(out, "indexed messages:  {}", index.message_count())?;
    writeln!(out, "hits:              {}", result.total)?;
    writeln!(out, "open:              {:.1} ms", ms(open_elapsed))?;
    writeln!(out, "search:            {:.1} ms", ms(search_elapsed))?;
    for warning in &result.warnings {
        writeln!(out, "warning:           {warning}")?;
    }
    if !result.hits.is_empty() {
        writeln!(out)?;
    }
    for hit in &result.hits {
        let date = hit
            .date
            .map(|d| DateTime::from_timestamp(d).to_rfc3339())
            .unwrap_or_else(|| "-".to_owned());
        let mut line = format!(
            "{date} | {} | {} | offset {} ({} B)",
            hit.from.join(", "),
            hit.subject.as_deref().unwrap_or("-"),
            hit.offset,
            hit.len
        );
        if hit.has_attachment {
            line.push_str(" | attachment");
        }
        if let Some(score) = hit.score {
            line.push_str(&format!(" | score {score:.2}"));
        }
        writeln!(out, "{line}")?;
    }
    Ok(())
}

/// The active structured filters of `query` as one line, `-` when there are none.
fn describe_filters(query: &SearchQuery) -> String {
    let mut parts = Vec::new();
    if let Some(from) = &query.from {
        parts.push(format!("from \"{from}\""));
    }
    if let Some(to) = &query.to {
        parts.push(format!("to \"{to}\""));
    }
    if let Some(subject) = &query.subject {
        parts.push(format!("subject \"{subject}\""));
    }
    if let Some(after) = query.after {
        parts.push(format!(
            "after {}",
            DateTime::from_timestamp(after).to_rfc3339()
        ));
    }
    if let Some(before) = query.before {
        parts.push(format!(
            "before {}",
            DateTime::from_timestamp(before).to_rfc3339()
        ));
    }
    for label in &query.labels {
        parts.push(format!("label \"{label}\""));
    }
    match query.has_attachment {
        Some(true) => parts.push("has attachment".to_owned()),
        Some(false) => parts.push("no attachment".to_owned()),
        None => {}
    }
    if parts.is_empty() {
        "-".to_owned()
    } else {
        parts.join(", ")
    }
}

fn progress(position: u64, total: u64, count: u64, elapsed: Duration) {
    let percent = if total == 0 {
        100.0
    } else {
        position as f64 * 100.0 / total as f64
    };
    eprint!(
        "\r{:.2} / {:.2} GB  {:5.1}%  {} messages  {:.0} MB/s   ",
        gb(position),
        gb(total),
        percent,
        count,
        mb_per_s(position, elapsed)
    );
}

fn gb(bytes: u64) -> f64 {
    bytes as f64 / 1e9
}

fn ms(elapsed: Duration) -> f64 {
    elapsed.as_secs_f64() * 1e3
}

fn mb_per_s(bytes: u64, elapsed: Duration) -> f64 {
    let secs = elapsed.as_secs_f64();
    if secs == 0.0 {
        0.0
    } else {
        bytes as f64 / 1e6 / secs
    }
}

fn print_sample(
    out: &mut impl Write,
    index: u64,
    raw: &RawMessage,
    message: &Message<'_>,
    summary: &MessageSummary,
) -> std::io::Result<()> {
    writeln!(
        out,
        "--- message #{index} at offset {} ({} bytes) ---",
        raw.offset, raw.len
    )?;
    writeln!(out, "Envelope: {}", raw.envelope)?;
    writeln!(
        out,
        "Date:     {}",
        message
            .date()
            .map(|d| d.to_rfc3339())
            .unwrap_or_else(|| "-".to_owned())
    )?;
    writeln!(out, "From:     {}", join_mailboxes(&summary.from))?;
    writeln!(out, "To:       {}", join_mailboxes(&summary.to))?;
    if !summary.cc.is_empty() {
        writeln!(out, "Cc:       {}", join_mailboxes(&summary.cc))?;
    }
    writeln!(
        out,
        "Subject:  {}",
        summary.subject.as_deref().unwrap_or("-")
    )?;
    writeln!(out, "Labels:   {}", summary.labels.join(", "))?;
    writeln!(
        out,
        "Thread:   {}",
        summary
            .gmail_thread_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "-".to_owned())
    )?;
    writeln!(out, "Parts:")?;
    for part in part_tree(message) {
        writeln!(out, "  {}", format_part(&part))?;
    }
    writeln!(out)
}

fn format_part(part: &PartInfo) -> String {
    let mut line = format!("{}{}", "  ".repeat(part.depth), part.content_type);
    if let Some(encoding) = &part.encoding {
        line.push_str(&format!(" [{encoding}]"));
    }
    if let Some(name) = &part.name {
        line.push_str(&format!(" \"{name}\""));
    }
    line.push_str(&format!(" {} B", part.size));
    line
}

fn join_mailboxes(mailboxes: &[Mailbox]) -> String {
    if mailboxes.is_empty() {
        return "-".to_owned();
    }
    mailboxes
        .iter()
        .map(Mailbox::display)
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_day_is_utc_midnight() {
        assert_eq!(parse_date("2026-01-13"), Ok(1_768_262_400));
        assert_eq!(parse_date(" 1970-01-02 "), Ok(86_400));
    }

    #[test]
    fn an_rfc3339_instant_passes_through() {
        assert_eq!(parse_date("2026-01-13T07:30:04+01:00"), Ok(1_768_285_804));
        assert_eq!(parse_date("2026-01-13T00:00:00Z"), parse_date("2026-01-13"));
    }

    #[test]
    fn garbage_is_rejected() {
        for bad in [
            "2026-13-01",
            "yesterday",
            "2026-01",
            "",
            "2026-01-13T25:00:00Z",
        ] {
            assert!(parse_date(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn filters_are_described_or_dashed() {
        assert_eq!(describe_filters(&SearchQuery::default()), "-");
        let query = SearchQuery {
            from: Some("anna".into()),
            subject: Some("számla".into()),
            after: Some(0),
            labels: vec!["Inbox".into()],
            has_attachment: Some(false),
            ..SearchQuery::default()
        };
        assert_eq!(
            describe_filters(&query),
            "from \"anna\", subject \"számla\", after 1970-01-01T00:00:00Z, label \"Inbox\", no attachment"
        );
    }
}
