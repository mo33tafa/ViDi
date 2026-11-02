//! `users` command group: current user, active sessions, local accounts, groups.

use crate::cli::{UserArgs, UserCmd};
use crate::commands::platform;
use crate::output;
use anyhow::Result;
use serde::Serialize;
use sysinfo::Users;

#[derive(Serialize)]
struct UserEntry {
    name: String,
    groups: Vec<String>,
}

fn current_user() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .or_else(|_| std::env::var("LOGNAME"))
        .unwrap_or_else(|_| "unknown".into())
}

pub fn run(args: UserArgs, json: bool) -> Result<()> {
    match args.cmd {
        UserCmd::Current => {
            let name = current_user();
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .unwrap_or_default();
            let shell = std::env::var("SHELL").unwrap_or_default();
            if json {
                return output::print_json(&serde_json::json!({
                    "name": name, "home": home, "shell": shell,
                }));
            }
            output::report("Current User");
            output::field("Name", &name);
            if !home.is_empty() {
                output::field("Home", &home);
            }
            if !shell.is_empty() {
                output::field("Shell", &shell);
            }
        }
        UserCmd::Active => {
            let out = match platform::os_family() {
                "windows" => platform::capture("query", &["user"]),
                _ => platform::capture("who", &[]),
            };
            match out {
                Some(text) => {
                    if json {
                        return output::print_json(&serde_json::json!({ "raw": text }));
                    }
                    output::report("Active Sessions");
                    println!("{text}");
                }
                None => {
                    output::report("Active Sessions");
                    output::note("No session information available on this platform.");
                }
            }
        }
        UserCmd::List => {
            let users = Users::new_with_refreshed_list();
            let rows: Vec<UserEntry> = users
                .iter()
                .map(|u| UserEntry {
                    name: u.name().to_string(),
                    groups: u.groups().iter().map(|g| g.name().to_string()).collect(),
                })
                .collect();
            if json {
                return output::print_json(&rows);
            }
            output::report(&format!("Local Accounts ({})", rows.len()));
            for u in &rows {
                output::field(&u.name, u.groups.join(", "));
            }
        }
        UserCmd::Groups { user } => {
            let target = user.unwrap_or_else(current_user);
            let groups = if platform::os_family() == "windows" {
                platform::capture("whoami", &["/groups"])
            } else {
                platform::capture("id", &["-Gn", &target])
            };
            match groups {
                Some(text) => {
                    if json {
                        return output::print_json(
                            &serde_json::json!({ "user": target, "groups": text }),
                        );
                    }
                    output::report(&format!("Groups for {target}"));
                    println!("{text}");
                }
                None => output::warn(&format!("Could not resolve groups for \"{target}\".")),
            }
        }
        UserCmd::Export { output: path } => {
            let users = Users::new_with_refreshed_list();
            let rows: Vec<UserEntry> = users
                .iter()
                .map(|u| UserEntry {
                    name: u.name().to_string(),
                    groups: u.groups().iter().map(|g| g.name().to_string()).collect(),
                })
                .collect();
            output::write_json(&path, &rows)?;
            output::note(&format!("{} accounts exported to {path}", rows.len()));
        }
    }
    Ok(())
}
