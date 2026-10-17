//! `mboxer` command line: scan (task 002), index and search (tasks 003/004).

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
    name = "mboxer",
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
    /// Path to the mbox file (its index must exist, see `mboxer index`).
    mbox: PathBuf,
    /// Free-text query; words are AND-ed, "quotes" make a phrase.
    #[arg(required = true, num_args = 1..)]
    query: Vec<String>,
    /// Number of hits to print.
    #[arg(long, default_value_t = 20)]
    limit: usize,
    /// Order by match quality instead of date.
    #[arg(long)]
    relevance: bool,
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
    match status(&args.mbox)? {
        IndexStatus::Fresh(meta) if !args.force => {
            writeln!(
                out,
                "index is up to date: {} messages, built {} (use --force to rebuild)",
                meta.message_count,
                DateTime::from_timestamp(meta.built_at_unix).to_rfc3339()
            )?;
            return Ok(());
        }
        IndexStatus::Fresh(_) => {}
        IndexStatus::Missing => {
            if !args.quiet {
                eprintln!("no index yet, building");
            }
        }
        IndexStatus::Stale(reason) => {
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
    writeln!(out, "index:             {}", report.index_dir.display())?;
    writeln!(out, "messages:          {}", report.messages)?;
    writeln!(out, "parse failures:    {}", report.parse_failures)?;
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
        Opened::Missing => return Err("no index for this mbox; run `mboxer index` first".into()),
        Opened::Stale(reason) => {
            return Err(format!("index is stale ({reason}); run `mboxer index`").into());
        }
    };
    let open_elapsed = open_start.elapsed();

    let text = args.query.join(" ");
    let query = SearchQuery {
        text: Some(text.clone()),
        limit: args.limit,
        sort: if args.relevance {
            Sort::Relevance
        } else {
            Sort::DateDesc
        },
        ..SearchQuery::default()
    };
    let search_start = Instant::now();
    let result = index.search(&query)?;
    let search_elapsed = search_start.elapsed();

    writeln!(out, "query:             {text}")?;
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
