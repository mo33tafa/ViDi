//! Cross-cutting tools: snapshot, hash, diagnose, analyze, health, monitor,
//! investigate.

use crate::output;
use anyhow::{Context, Result};
use colored::Colorize;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::io::Read;
use sysinfo::{Disks, Networks, ProcessRefreshKind, System};

#[derive(Serialize)]
struct Snapshot {
    generated_at: String,
    hostname: Option<String>,
    os: Option<String>,
    os_version: Option<String>,
    kernel: Option<String>,
    uptime_secs: u64,
    cpu_count: usize,
    total_memory: u64,
    used_memory: u64,
    processes: Vec<ProcSummary>,
    disks: Vec<DiskSummary>,
    interfaces: Vec<String>,
}

#[derive(Serialize)]
struct ProcSummary {
    pid: u32,
    name: String,
    cpu: f32,
    memory: u64,
}

#[derive(Serialize)]
struct DiskSummary {
    mount: String,
    total: u64,
    available: u64,
}

fn build_snapshot() -> Snapshot {
    let mut sys = System::new_all();
    sys.refresh_processes_specifics(ProcessRefreshKind::everything());
    std::thread::sleep(std::time::Duration::from_millis(200));
    sys.refresh_processes_specifics(ProcessRefreshKind::everything());
    sys.refresh_memory();

    let mut processes: Vec<ProcSummary> = sys
        .processes()
        .iter()
        .map(|(pid, p)| ProcSummary {
            pid: pid.as_u32(),
            name: p.name().to_string(),
            cpu: p.cpu_usage(),
            memory: p.memory(),
        })
        .collect();
    processes.sort_by(|a, b| {
        b.cpu
            .partial_cmp(&a.cpu)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let disks = Disks::new_with_refreshed_list();
    let disk_rows = disks
        .iter()
        .map(|d| DiskSummary {
            mount: d.mount_point().display().to_string(),
            total: d.total_space(),
            available: d.available_space(),
        })
        .collect();

    let networks = Networks::new_with_refreshed_list();
    let interfaces = networks.iter().map(|(n, _)| n.clone()).collect();

    Snapshot {
        generated_at: chrono::Local::now().to_rfc3339(),
        hostname: System::host_name(),
        os: System::name(),
        os_version: System::os_version(),
        kernel: System::kernel_version(),
        uptime_secs: System::uptime(),
        cpu_count: sys.cpus().len(),
        total_memory: sys.total_memory(),
        used_memory: sys.used_memory(),
        processes,
        disks: disk_rows,
        interfaces,
    }
}

pub fn snapshot(path: String, json: bool) -> Result<()> {
    let snap = build_snapshot();
    output::write_json(&path, &snap)?;
    if json {
        return output::print_json(
            &serde_json::json!({ "output": path, "processes": snap.processes.len() }),
        );
    }
    output::report("System Snapshot");
    output::field("Saved to", &path);
    output::field("Processes captured", snap.processes.len());
    output::field("Disks captured", snap.disks.len());
    Ok(())
}

pub fn hash(path: String, json: bool) -> Result<()> {
    let mut file = std::fs::File::open(&path).with_context(|| format!("opening {path}"))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = file
            .read(&mut buf)
            .with_context(|| format!("reading {path}"))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let digest = format!("{:x}", hasher.finalize());
    if json {
        return output::print_json(&serde_json::json!({ "path": path, "sha256": digest }));
    }
    output::report("File Hash (SHA-256)");
    output::field("Path", &path);
    output::field("SHA-256", &digest);
    Ok(())
}

pub fn diagnose(test_type: String, json: bool) -> Result<()> {
    let mut sys = System::new_all();
    sys.refresh_cpu();
    std::thread::sleep(std::time::Duration::from_millis(300));
    sys.refresh_cpu();
    sys.refresh_memory();

    let cpu = sys.global_cpu_info().cpu_usage();
    let mem_pct = pct(sys.used_memory(), sys.total_memory());
    let disks = Disks::new_with_refreshed_list();
    let disk_pct = disks
        .iter()
        .map(|d| {
            pct(
                d.total_space().saturating_sub(d.available_space()),
                d.total_space(),
            )
        })
        .fold(0.0_f32, f32::max);

    let mut results = Vec::new();
    let t = test_type.to_lowercase();
    if t == "all" || t == "cpu" {
        results.push(("CPU load", cpu, verdict(cpu, 85.0, 95.0)));
    }
    if t == "all" || t == "memory" || t == "mem" {
        results.push(("Memory usage", mem_pct, verdict(mem_pct, 85.0, 95.0)));
    }
    if t == "all" || t == "disk" {
        results.push(("Disk usage (max)", disk_pct, verdict(disk_pct, 85.0, 95.0)));
    }

    if json {
        let rows: Vec<_> = results
            .iter()
            .map(|(name, val, v)| serde_json::json!({ "test": name, "value_pct": val, "verdict": v }))
            .collect();
        return output::print_json(&rows);
    }
    output::report(&format!("Diagnostics ({test_type})"));
    for (name, val, v) in &results {
        output::field(name, format!("{val:.1}%  [{}]", colorize_verdict(v)));
    }
    Ok(())
}

pub fn analyze(analysis_type: String, json: bool) -> Result<()> {
    let mut sys = System::new_all();
    sys.refresh_processes_specifics(ProcessRefreshKind::everything());
    std::thread::sleep(std::time::Duration::from_millis(200));
    sys.refresh_processes_specifics(ProcessRefreshKind::everything());

    // Heuristic, non-authoritative indicators for an operator to review.
    let mut findings: Vec<String> = Vec::new();
    let t = analysis_type.to_lowercase();

    if t == "all" || t == "processes" {
        for (pid, p) in sys.processes() {
            if p.cpu_usage() > 80.0 {
                findings.push(format!(
                    "High CPU process: {} (PID {}, {:.0}%)",
                    p.name(),
                    pid.as_u32(),
                    p.cpu_usage()
                ));
            }
            if p.exe().map(|e| e.parent().is_none()).unwrap_or(false) {
                findings.push(format!(
                    "Process with unusual exe path: {} (PID {})",
                    p.name(),
                    pid.as_u32()
                ));
            }
        }
    }

    if json {
        return output::print_json(
            &serde_json::json!({ "analysis": analysis_type, "findings": findings }),
        );
    }
    output::report(&format!("Threat Analysis ({analysis_type})"));
    output::note("Heuristic indicators only \u{2013} review manually, not a verdict.");
    if findings.is_empty() {
        output::field("Result", "no notable indicators found");
    } else {
        for f in &findings {
            output::field("Indicator", f);
        }
    }
    Ok(())
}

pub fn health(json: bool) -> Result<()> {
    let mut sys = System::new_all();
    sys.refresh_cpu();
    std::thread::sleep(std::time::Duration::from_millis(300));
    sys.refresh_cpu();
    sys.refresh_memory();

    let cpu = sys.global_cpu_info().cpu_usage();
    let mem_pct = pct(sys.used_memory(), sys.total_memory());
    let load = System::load_average();
    let disks = Disks::new_with_refreshed_list();
    let disk_pct = disks
        .iter()
        .map(|d| {
            pct(
                d.total_space().saturating_sub(d.available_space()),
                d.total_space(),
            )
        })
        .fold(0.0_f32, f32::max);

    let overall = [cpu, mem_pct, disk_pct]
        .iter()
        .cloned()
        .fold(0.0_f32, f32::max);
    let status = verdict(overall, 85.0, 95.0);

    if json {
        return output::print_json(&serde_json::json!({
            "status": status,
            "cpu_pct": cpu,
            "memory_pct": mem_pct,
            "disk_pct": disk_pct,
            "load_average": { "one": load.one, "five": load.five, "fifteen": load.fifteen },
        }));
    }
    output::report("System Health Report");
    output::field("Overall Status", colorize_verdict(status));
    output::field("CPU", format!("{cpu:.1}%"));
    output::field("Memory", format!("{mem_pct:.1}%"));
    output::field("Disk (max)", format!("{disk_pct:.1}%"));
    output::field(
        "Load Average",
        format!("{:.2} {:.2} {:.2}", load.one, load.five, load.fifteen),
    );
    output::field("Uptime", output::duration(System::uptime()));
    Ok(())
}

pub fn monitor(duration: u64, interval: u64) -> Result<()> {
    let mut sys = System::new_all();
    let interval = interval.max(100);
    let iterations = (duration * 1000 / interval).max(1);
    output::report(&format!(
        "Real-time Monitor ({duration}s, every {interval}ms)"
    ));
    for i in 0..iterations {
        sys.refresh_processes_specifics(ProcessRefreshKind::everything());
        sys.refresh_cpu();
        sys.refresh_memory();
        let cpu = sys.global_cpu_info().cpu_usage();
        let mem = pct(sys.used_memory(), sys.total_memory());

        let mut procs: Vec<(u32, String, f32)> = sys
            .processes()
            .iter()
            .map(|(pid, p)| (pid.as_u32(), p.name().to_string(), p.cpu_usage()))
            .collect();
        procs.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        procs.truncate(5);

        println!(
            "\n[{}/{}] {}  CPU {:.1}%  MEM {:.1}%",
            i + 1,
            iterations,
            chrono::Local::now().format("%H:%M:%S"),
            cpu,
            mem
        );
        for (pid, name, c) in &procs {
            println!("    {:>7}  {:>5.1}%  {}", pid, c, name);
        }
        std::thread::sleep(std::time::Duration::from_millis(interval));
    }
    Ok(())
}

pub fn investigate(name: String, json: bool) -> Result<()> {
    let id = uuid::Uuid::new_v4();
    let safe_name = safe_label(&name);
    let file = format!("investigation-{safe_name}-{}.json", &id.to_string()[..8]);
    let snap = build_snapshot();
    let payload = serde_json::json!({
        "investigation_id": id.to_string(),
        "name": name,
        "started_at": chrono::Local::now().to_rfc3339(),
        "snapshot": snap,
    });
    output::write_json(&file, &payload)?;
    if json {
        return output::print_json(
            &serde_json::json!({ "investigation_id": id.to_string(), "output": file }),
        );
    }
    output::report("Forensic Investigation");
    output::field("Name", &name);
    output::field("Investigation ID", id);
    output::field("Evidence file", &file);
    Ok(())
}

// --- helpers ---------------------------------------------------------------

fn safe_label(label: &str) -> String {
    let value: String = label
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.') {
                c
            } else {
                '_'
            }
        })
        .take(64)
        .collect();
    if value.is_empty() {
        "investigation".to_string()
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::safe_label;

    #[test]
    fn sanitizes_investigation_labels() {
        assert_eq!(safe_label("../evidence/report"), ".._evidence_report");
        assert_eq!(safe_label(""), "investigation");
        assert!(safe_label(&"x".repeat(100)).len() <= 64);
    }
}

fn pct(part: u64, whole: u64) -> f32 {
    if whole == 0 {
        0.0
    } else {
        (part as f64 / whole as f64 * 100.0) as f32
    }
}

fn verdict(value: f32, warn: f32, crit: f32) -> &'static str {
    if value >= crit {
        "critical"
    } else if value >= warn {
        "warning"
    } else {
        "ok"
    }
}

fn colorize_verdict(v: &str) -> String {
    match v {
        "critical" => v.red().bold().to_string(),
        "warning" => v.yellow().bold().to_string(),
        _ => v.green().bold().to_string(),
    }
}
