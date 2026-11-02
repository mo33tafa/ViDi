//! `network` command group: interfaces, connections, listening ports, routes,
//! DNS, ARP and connectivity testing.

use crate::cli::{NetworkArgs, NetworkCmd};
use crate::commands::platform;
use crate::output;
use anyhow::{bail, Result};
use serde::Serialize;
use sysinfo::Networks;

#[derive(Serialize)]
struct Interface {
    name: String,
    mac: String,
    received: u64,
    transmitted: u64,
    total_received: u64,
    total_transmitted: u64,
}

fn interfaces() -> Vec<Interface> {
    let networks = Networks::new_with_refreshed_list();
    networks
        .iter()
        .map(|(name, data)| Interface {
            name: name.clone(),
            mac: data.mac_address().to_string(),
            received: data.received(),
            transmitted: data.transmitted(),
            total_received: data.total_received(),
            total_transmitted: data.total_transmitted(),
        })
        .collect()
}

/// Run the first available command from a list of candidates.
fn first_available(candidates: &[(&str, &[&str])]) -> Option<String> {
    for (cmd, args) in candidates {
        if let Some(out) = platform::capture(cmd, args) {
            return Some(out);
        }
    }
    None
}

pub fn run(args: NetworkArgs, json: bool) -> Result<()> {
    match args.cmd {
        NetworkCmd::Interfaces => {
            let ifaces = interfaces();
            if json {
                return output::print_json(&ifaces);
            }
            output::report(&format!("Network Interfaces ({})", ifaces.len()));
            for i in &ifaces {
                output::section(&i.name);
                output::field("MAC", &i.mac);
                output::field("Total Received", output::bytes(i.total_received));
                output::field("Total Transmitted", output::bytes(i.total_transmitted));
            }
        }
        NetworkCmd::Connections => {
            let out = first_available(&[
                ("ss", &["-tunap"]),
                ("netstat", &["-anp"]),
                ("netstat", &["-ano"]),
            ]);
            print_raw("Active Connections", out, json)?;
        }
        NetworkCmd::Listening => {
            let out = first_available(&[
                ("ss", &["-tunlp"]),
                ("netstat", &["-anp"]),
                ("netstat", &["-ano"]),
            ]);
            print_raw("Listening Ports", out, json)?;
        }
        NetworkCmd::Routes => {
            let out = first_available(&[
                ("ip", &["route"]),
                ("netstat", &["-rn"]),
                ("route", &["print"]),
            ]);
            print_raw("Routing Table", out, json)?;
        }
        NetworkCmd::Dns => {
            let out = match platform::os_family() {
                "windows" => platform::capture("ipconfig", &["/all"]),
                _ => std::fs::read_to_string("/etc/resolv.conf").ok(),
            };
            print_raw("DNS Configuration", out, json)?;
        }
        NetworkCmd::Arp => {
            let out = first_available(&[("ip", &["neigh"]), ("arp", &["-a"])]);
            print_raw("ARP / Neighbor Table", out, json)?;
        }
        NetworkCmd::Test { target } => {
            let count_flag = if platform::os_family() == "windows" {
                "-n"
            } else {
                "-c"
            };
            output::report(&format!("Connectivity Test: {target}"));
            let ok = platform::run("ping", &[count_flag, "4", &target]).is_ok();
            if json {
                return output::print_json(
                    &serde_json::json!({ "target": target, "reachable": ok }),
                );
            }
            if ok {
                output::note("Host is reachable.");
            } else {
                output::warn("Host is unreachable, or the ping tool is unavailable.");
            }
        }
        NetworkCmd::Export { output: path } => {
            let ifaces = interfaces();
            output::write_json(&path, &ifaces)?;
            output::note(&format!("Network information exported to {path}"));
        }
    }
    Ok(())
}

fn print_raw(title: &str, out: Option<String>, json: bool) -> Result<()> {
    match out {
        Some(text) => {
            if json {
                output::print_json(&serde_json::json!({ "title": title, "raw": text }))
            } else {
                output::report(title);
                println!("{text}");
                Ok(())
            }
        }
        None => bail!("No supported tool available to collect \"{title}\" on this platform."),
    }
}
