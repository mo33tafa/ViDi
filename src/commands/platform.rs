//! Small helpers for shelling out to platform-native tools when a piece of
//! information is not exposed cross-platform by `sysinfo`.

use std::process::Command;

/// Current OS family as a short string.
pub fn os_family() -> &'static str {
    std::env::consts::OS
}

/// Friendly OS family label (Unix-like vs Windows), for display purposes.
pub fn os_family_label() -> &'static str {
    match os_family() {
        "linux" => "Unix-like (Linux)",
        "macos" => "Unix-like (macOS)",
        "windows" => "Windows",
        "freebsd" | "openbsd" | "netbsd" | "dragonfly" => "Unix-like (BSD)",
        other => other,
    }
}

/// Run a command and return trimmed stdout, or `None` if it fails / is missing.
pub fn capture(cmd: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(cmd).args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

/// Run a command for its side effects, returning Ok on success.
pub fn run(cmd: &str, args: &[&str]) -> anyhow::Result<()> {
    let status = Command::new(cmd)
        .args(args)
        .status()
        .map_err(|e| anyhow::anyhow!("failed to launch `{cmd}`: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("`{cmd}` exited with status {status}");
    }
}
