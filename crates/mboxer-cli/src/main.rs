//! `mboxer` command line: scan (task 002), index and search (task 004).

use std::collections::HashSet;
use std::error::Error;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{Duration, Instant};

use clap::{Args, Parser, Subcommand};
use mboxer_core::mail_parser::Message;
use mboxer_core::{
    Mailbox, MboxReader, MessageSummary, PartInfo, RawMessage, parse, part_tree, summarize,
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

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Scan(args) => scan(&args),
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
    let mut reader = MboxReader::open(&args.mbox)?;
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
        .map(|m| match (&m.name, &m.address) {
            (Some(name), Some(address)) => format!("{name} <{address}>"),
            (None, Some(address)) => address.clone(),
            (Some(name), None) => name.clone(),
            (None, None) => "?".to_owned(),
        })
        .collect::<Vec<_>>()
        .join(", ")
}
