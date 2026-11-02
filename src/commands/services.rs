//! `services` command group. Service management is inherently OS-specific, so
//! this delegates to the platform's native service manager:
//!   - Linux  : systemctl
//!   - Windows: sc / PowerShell Get-Service

use crate::cli::{ServiceArgs, ServiceCmd};
use crate::commands::platform;
use crate::output;
use anyhow::{bail, Result};
use serde::Serialize;

#[derive(Serialize)]
struct ServiceEntry {
    name: String,
    status: String,
    description: String,
}

fn list_linux() -> Vec<ServiceEntry> {
    let mut out = Vec::new();
    if let Some(text) = platform::capture(
        "systemctl",
        &[
            "list-units",
            "--type=service",
            "--all",
            "--no-legend",
            "--plain",
        ],
    ) {
        for line in text.lines() {
            let mut it = line.split_whitespace();
            let name = it.next().unwrap_or("").to_string();
            let _load = it.next().unwrap_or("");
            let active = it.next().unwrap_or("").to_string();
            let sub = it.next().unwrap_or("");
            let desc: String = it.collect::<Vec<_>>().join(" ");
            if !name.is_empty() {
                out.push(ServiceEntry {
                    name,
                    status: format!("{active}/{sub}"),
                    description: desc,
                });
            }
        }
    }
    out
}

fn list_macos() -> Vec<ServiceEntry> {
    let mut out = Vec::new();
    if let Some(text) = platform::capture("launchctl", &["list"]) {
        for line in text.lines().skip(1) {
            let cols: Vec<&str> = line.split_whitespace().collect();
            if cols.len() >= 3 {
                out.push(ServiceEntry {
                    name: cols[2].to_string(),
                    status: if cols[0] == "-" {
                        "stopped".into()
                    } else {
                        "running".into()
                    },
                    description: format!("exit={}", cols[1]),
                });
            }
        }
    }
    out
}

fn list_windows() -> Vec<ServiceEntry> {
    let mut out = Vec::new();
    if let Some(text) = platform::capture(
        "powershell",
        &[
            "-NoProfile",
            "-Command",
            "Get-Service | Select-Object Name,Status,DisplayName | ConvertTo-Csv -NoTypeInformation",
        ],
    ) {
        for line in text.lines().skip(1) {
            let cols: Vec<String> = line
                .split(',')
                .map(|c| c.trim_matches('"').to_string())
                .collect();
            if cols.len() >= 3 {
                out.push(ServiceEntry {
                    name: cols[0].clone(),
                    status: cols[1].clone(),
                    description: cols[2].clone(),
                });
            }
        }
    }
    out
}

fn collect() -> Vec<ServiceEntry> {
    match platform::os_family() {
        "linux" => list_linux(),
        "macos" => list_macos(),
        "windows" => list_windows(),
        other => {
            output::warn(&format!(
                "Service enumeration is not supported on \"{other}\"."
            ));
            Vec::new()
        }
    }
}

fn control(action: &str, name: &str) -> Result<()> {
    match platform::os_family() {
        "linux" => platform::run("systemctl", &[action, name]),
        "windows" => {
            let verb = match action {
                "start" => "start",
                "stop" => "stop",
                "restart" => "restart",
                _ => bail!("unsupported action \"{action}\""),
            };
            platform::run("sc", &[verb, name])
        }
        "macos" => {
            let sub = match action {
                "start" => "start",
                "stop" => "stop",
                "restart" => "kickstart",
                _ => bail!("unsupported action \"{action}\""),
            };
            platform::run("launchctl", &[sub, name])
        }
        other => bail!("Service control is not supported on \"{other}\"."),
    }
}

pub fn run(args: ServiceArgs, json: bool) -> Result<()> {
    match args.cmd {
        ServiceCmd::List => {
            let svcs = collect();
            if json {
                return output::print_json(&svcs);
            }
            output::report(&format!("Services ({} total)", svcs.len()));
            for s in &svcs {
                output::field(&s.name, &s.status);
            }
        }
        ServiceCmd::Status { status } => {
            let want = status.to_lowercase();
            let svcs: Vec<ServiceEntry> = collect()
                .into_iter()
                .filter(|s| want == "all" || s.status.to_lowercase().contains(&want))
                .collect();
            if json {
                return output::print_json(&svcs);
            }
            output::report(&format!(
                "Services with status \"{status}\" ({})",
                svcs.len()
            ));
            for s in &svcs {
                output::field(&s.name, &s.status);
            }
        }
        ServiceCmd::Detail { name } => {
            let detail = match platform::os_family() {
                "linux" => platform::capture("systemctl", &["status", &name, "--no-pager"]),
                "windows" => platform::capture("sc", &["query", &name]),
                "macos" => platform::capture("launchctl", &["list", &name]),
                _ => None,
            };
            match detail {
                Some(d) => {
                    if json {
                        return output::print_json(
                            &serde_json::json!({ "name": name, "detail": d }),
                        );
                    }
                    output::report(&format!("Service: {name}"));
                    println!("{d}");
                }
                None => bail!("Could not query service \"{name}\"."),
            }
        }
        ServiceCmd::Start { name } => {
            control("start", &name)?;
            output::note(&format!("Started {name}."));
        }
        ServiceCmd::Stop { name } => {
            control("stop", &name)?;
            output::note(&format!("Stopped {name}."));
        }
        ServiceCmd::Restart { name } => {
            control("restart", &name)?;
            output::note(&format!("Restarted {name}."));
        }
        ServiceCmd::Export { output: path } => {
            let svcs = collect();
            output::write_json(&path, &svcs)?;
            output::note(&format!("{} services exported to {path}", svcs.len()));
        }
    }
    Ok(())
}
