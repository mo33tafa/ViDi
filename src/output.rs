//! Shared output helpers: human-readable report formatting + JSON export.
//!
//! `--json` output always stays in English (stable keys/values) for machine
//! consumption; the human-readable path below is also plain English, laid
//! out as a short report (title + generation time) so it's always clear
//! what was run and when.

use anyhow::{Context, Result};
use colored::Colorize;
use serde::Serialize;
use std::io::Write;

/// Print the main report banner for a command: title + generation time.
/// Every top-level command prints exactly one of these, at the top of its
/// output, so it's always clear *what* was run and *when*.
pub fn report(title: &str) {
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let line = "=".repeat(56);
    println!("{}", line.cyan());
    println!(
        "{}",
        format!(" {title}   |   Generated: {now}").bold().cyan()
    );
    println!("{}", line.cyan());
}

/// Print a section divider inside a report (e.g. one disk / one interface).
pub fn section(title: impl std::fmt::Display) {
    println!("{}", format!("-- {title} --").bold().cyan());
}

/// Print a `label: value` line with aligned, colored label.
pub fn field(label: &str, value: impl std::fmt::Display) {
    println!("  {:<22} {}", format!("{label}:").green(), value);
}

/// Print an informational note.
pub fn note(msg: &str) {
    println!("  {}", msg.dimmed());
}

/// Print an error to stderr.
pub fn error(msg: &str) {
    eprintln!("{} {}", "[error]".bold().red(), msg);
}

/// Print a warning.
pub fn warn(msg: &str) {
    println!("{} {}", "[warn]".bold().yellow(), msg);
}

/// Serialize a value to pretty JSON and print it to stdout.
pub fn print_json<T: Serialize>(value: &T) -> Result<()> {
    let s = serde_json::to_string_pretty(value).context("serializing JSON output")?;
    println!("{s}");
    Ok(())
}

/// Serialize a value to pretty JSON and write it to `path`.
pub fn write_json<T: Serialize>(path: &str, value: &T) -> Result<()> {
    let s = serde_json::to_string_pretty(value).context("serializing JSON output")?;
    let mut f = std::fs::File::create(path).with_context(|| format!("creating {path}"))?;
    f.write_all(s.as_bytes())
        .with_context(|| format!("writing {path}"))?;
    Ok(())
}

/// Human-readable byte size (base-1024).
pub fn bytes(n: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
    let mut size = n as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{n} B")
    } else {
        format!("{size:.2} {}", UNITS[unit])
    }
}

/// Format a Unix timestamp (seconds) as a readable local datetime.
pub fn timestamp(secs: u64) -> String {
    use chrono::{Local, TimeZone};
    match Local.timestamp_opt(secs as i64, 0).single() {
        Some(dt) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
        None => secs.to_string(),
    }
}

/// Format a duration in seconds as `Nd Nh Nm Ns`.
pub fn duration(secs: u64) -> String {
    let d = secs / 86_400;
    let h = (secs % 86_400) / 3_600;
    let m = (secs % 3_600) / 60;
    let s = secs % 60;
    let mut parts = Vec::new();
    if d > 0 {
        parts.push(format!("{d}d"));
    }
    if h > 0 || d > 0 {
        parts.push(format!("{h}h"));
    }
    parts.push(format!("{m}m"));
    parts.push(format!("{s}s"));
    parts.join(" ")
}
