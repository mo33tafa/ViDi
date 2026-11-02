//! `processes` command group: list, tree, detail, search, memory, io, kill.

use crate::cli::{ProcessArgs, ProcessCmd};
use crate::output;
use anyhow::{anyhow, Result};
use colored::Colorize;
use serde::Serialize;
use sysinfo::{Pid, ProcessRefreshKind, System};

#[derive(Serialize, Clone)]
struct ProcInfo {
    pid: u32,
    parent: Option<u32>,
    name: String,
    cpu: f32,
    memory: u64,
    status: String,
    start_time: u64,
    read_bytes: u64,
    written_bytes: u64,
}

/// Refresh processes twice so CPU usage percentages are meaningful.
fn sampled_system() -> System {
    let mut sys = System::new_all();
    sys.refresh_processes_specifics(ProcessRefreshKind::everything());
    std::thread::sleep(std::time::Duration::from_millis(200));
    sys.refresh_processes_specifics(ProcessRefreshKind::everything());
    sys
}

fn snapshot(sys: &System) -> Vec<ProcInfo> {
    sys.processes()
        .iter()
        .map(|(pid, p)| {
            let du = p.disk_usage();
            ProcInfo {
                pid: pid.as_u32(),
                parent: p.parent().map(|pp| pp.as_u32()),
                name: p.name().to_string(),
                cpu: p.cpu_usage(),
                memory: p.memory(),
                status: p.status().to_string(),
                start_time: p.start_time(),
                read_bytes: du.total_read_bytes,
                written_bytes: du.total_written_bytes,
            }
        })
        .collect()
}

fn print_table(rows: &[ProcInfo]) {
    println!(
        "{}",
        format!(
            "  {:>7}  {:>6}  {:>10}  {}",
            "PID", "CPU%", "MEMORY", "NAME"
        )
        .bold()
        .to_string()
    );
    for r in rows {
        println!(
            "  {:>7}  {:>5.1}  {:>10}  {}",
            r.pid,
            r.cpu,
            output::bytes(r.memory),
            r.name
        );
    }
}

pub fn run(args: ProcessArgs, json: bool) -> Result<()> {
    match args.cmd {
        ProcessCmd::List { count, sort } => {
            let sys = sampled_system();
            let mut rows = snapshot(&sys);
            sort_rows(&mut rows, &sort);
            rows.truncate(count);
            if json {
                return output::print_json(&rows);
            }
            output::report(&format!("Processes (top {count} by {sort})"));
            print_table(&rows);
        }
        ProcessCmd::Memory { top } => {
            let sys = sampled_system();
            let mut rows = snapshot(&sys);
            rows.sort_by(|a, b| b.memory.cmp(&a.memory));
            rows.truncate(top);
            if json {
                return output::print_json(&rows);
            }
            output::report(&format!("Top {top} Processes by Memory"));
            print_table(&rows);
        }
        ProcessCmd::Io { top } => {
            let sys = sampled_system();
            let mut rows = snapshot(&sys);
            rows.sort_by(|a, b| {
                (b.read_bytes + b.written_bytes).cmp(&(a.read_bytes + a.written_bytes))
            });
            rows.truncate(top);
            if json {
                return output::print_json(&rows);
            }
            output::report(&format!("Top {top} Processes by I/O"));
            println!(
                "  {:>7}  {:>12}  {:>12}  {}",
                "PID".bold(),
                "READ".bold(),
                "WRITTEN".bold(),
                "NAME".bold()
            );
            for r in &rows {
                println!(
                    "  {:>7}  {:>12}  {:>12}  {}",
                    r.pid,
                    output::bytes(r.read_bytes),
                    output::bytes(r.written_bytes),
                    r.name
                );
            }
        }
        ProcessCmd::Search { pattern } => {
            let sys = sampled_system();
            let pat = pattern.to_lowercase();
            let mut rows: Vec<ProcInfo> = snapshot(&sys)
                .into_iter()
                .filter(|r| r.name.to_lowercase().contains(&pat))
                .collect();
            rows.sort_by(|a, b| a.name.cmp(&b.name));
            if json {
                return output::print_json(&rows);
            }
            output::report(&format!(
                "Processes matching \"{pattern}\" ({} found)",
                rows.len()
            ));
            print_table(&rows);
        }
        ProcessCmd::Detail { pid } => {
            let sys = sampled_system();
            let p = sys
                .process(Pid::from_u32(pid))
                .ok_or_else(|| anyhow!("no process with PID {pid}"))?;
            let du = p.disk_usage();
            if json {
                return output::print_json(&serde_json::json!({
                    "pid": pid,
                    "name": p.name(),
                    "parent": p.parent().map(|x| x.as_u32()),
                    "status": p.status().to_string(),
                    "cpu": p.cpu_usage(),
                    "memory": p.memory(),
                    "virtual_memory": p.virtual_memory(),
                    "start_time": p.start_time(),
                    "run_time": p.run_time(),
                    "exe": p.exe().map(|x| x.display().to_string()),
                    "cwd": p.cwd().map(|x| x.display().to_string()),
                    "cmd": p.cmd(),
                    "read_bytes": du.total_read_bytes,
                    "written_bytes": du.total_written_bytes,
                }));
            }
            output::report(&format!("Process Detail (PID {pid})"));
            output::field("Name", p.name());
            output::field("Status", p.status());
            output::field("Parent PID", p.parent().map(|x| x.as_u32()).unwrap_or(0));
            output::field("CPU", format!("{:.1}%", p.cpu_usage()));
            output::field("Memory", output::bytes(p.memory()));
            output::field("Virtual Memory", output::bytes(p.virtual_memory()));
            output::field("Start Time", output::timestamp(p.start_time()));
            output::field("Run Time", output::duration(p.run_time()));
            if let Some(exe) = p.exe() {
                output::field("Executable", exe.display());
            }
            if let Some(cwd) = p.cwd() {
                output::field("Working Dir", cwd.display());
            }
            output::field("Command", p.cmd().join(" "));
            output::field("Disk Read", output::bytes(du.total_read_bytes));
            output::field("Disk Written", output::bytes(du.total_written_bytes));
        }
        ProcessCmd::Tree { pid } => {
            let sys = sampled_system();
            let rows = snapshot(&sys);
            if json {
                return output::print_json(&rows);
            }
            output::report("Process Tree");
            match pid {
                Some(root) => print_tree(&rows, root, 0),
                None => {
                    // Roots: processes whose parent is absent or not present.
                    let pids: std::collections::HashSet<u32> = rows.iter().map(|r| r.pid).collect();
                    for r in &rows {
                        let is_root = match r.parent {
                            None => true,
                            Some(pp) => !pids.contains(&pp),
                        };
                        if is_root {
                            print_tree(&rows, r.pid, 0);
                        }
                    }
                }
            }
        }
        ProcessCmd::Kill { pid, force } => {
            let sys = System::new_all();
            let p = sys
                .process(Pid::from_u32(pid))
                .ok_or_else(|| anyhow!("no process with PID {pid}"))?;
            let ok = if force {
                p.kill_with(sysinfo::Signal::Kill)
                    .unwrap_or_else(|| p.kill())
            } else {
                p.kill_with(sysinfo::Signal::Term)
                    .unwrap_or_else(|| p.kill())
            };
            if ok {
                output::note(&format!("Signal sent to PID {pid} ({})", p.name()));
            } else {
                return Err(anyhow!("failed to signal PID {pid}"));
            }
        }
        ProcessCmd::Export { output: path } => {
            let sys = sampled_system();
            let rows = snapshot(&sys);
            output::write_json(&path, &rows)?;
            output::note(&format!("{} processes exported to {path}", rows.len()));
        }
    }
    Ok(())
}

fn sort_rows(rows: &mut [ProcInfo], key: &str) {
    match key {
        "memory" | "mem" => rows.sort_by(|a, b| b.memory.cmp(&a.memory)),
        "pid" => rows.sort_by(|a, b| a.pid.cmp(&b.pid)),
        "name" => rows.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
        _ => rows.sort_by(|a, b| {
            b.cpu
                .partial_cmp(&a.cpu)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
    }
}

fn print_tree(rows: &[ProcInfo], pid: u32, depth: usize) {
    if let Some(r) = rows.iter().find(|r| r.pid == pid) {
        let indent = "  ".repeat(depth + 1);
        println!("{indent}{} {}", format!("[{}]", r.pid).dimmed(), r.name);
        for child in rows.iter().filter(|c| c.parent == Some(pid)) {
            if child.pid != pid {
                print_tree(rows, child.pid, depth + 1);
            }
        }
    }
}
