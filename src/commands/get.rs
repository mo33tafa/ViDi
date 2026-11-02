//! `get` command: a single, simple, uniform way to see any information.
//!
//! Every piece of data vidi can show is available through one pattern:
//!
//!     vidi get <what> [target] [-f|--full]
//!
//! e.g. `vidi get cpu`, `vidi get memory`, `vidi get serialnumber`,
//! `vidi get process 1234`, `vidi get service ssh`. Run `vidi get list`
//! to print the full catalog. This is a thin router: it reuses the exact
//! same collectors as the advanced command groups (`system`, `hardware`,
//! `network`, `users`, `processes`, `services`, and the standalone tools),
//! so the output and `--json` behavior are identical either way.

use crate::cli::{
    HardwareArgs, HardwareCmd, NetworkArgs, NetworkCmd, ProcessArgs, ProcessCmd, ServiceArgs,
    ServiceCmd, SystemArgs, SystemCmd, UserArgs, UserCmd,
};
use crate::commands::{hardware, network, platform, processes, services, system, tools, users};
use crate::output;
use anyhow::{anyhow, Result};
use colored::Colorize;
use serde::Serialize;
use sysinfo::{Disks, System};

pub fn run(what: String, target: Option<String>, full: bool, json: bool) -> Result<()> {
    let key = what.trim().to_lowercase().replace(['-', '_', ' '], "");

    match key.as_str() {
        // --- system ---------------------------------------------------
        "os" | "system" | "platform" | "info" => system::run(
            SystemArgs {
                cmd: SystemCmd::Info,
            },
            json,
        ),
        "environment" | "env" | "envvars" => system::run(
            SystemArgs {
                cmd: SystemCmd::Environment,
            },
            json,
        ),
        "time" | "date" | "datetime" | "clock" | "timezone" | "ntp" => time_report(json),

        // --- hardware ---------------------------------------------------
        "hardware" | "hw" => hardware_overview(full, json),
        "cpu" | "processor" => hardware::run(
            HardwareArgs {
                cmd: HardwareCmd::Cpu,
            },
            json,
        ),
        "memory" | "ram" => hardware::run(
            HardwareArgs {
                cmd: HardwareCmd::Memory,
            },
            json,
        ),
        "disk" | "disks" | "storage" => hardware::run(
            HardwareArgs {
                cmd: HardwareCmd::Storage,
            },
            json,
        ),
        "temperature" | "temp" | "sensors" => hardware::run(
            HardwareArgs {
                cmd: HardwareCmd::Temperature,
            },
            json,
        ),
        "adapters" | "nic" | "networkadapters" => hardware::run(
            HardwareArgs {
                cmd: HardwareCmd::Adapters,
            },
            json,
        ),
        "serial" | "serialnumber" => serial_number(json),

        // --- network ------------------------------------------------------
        "network" | "interfaces" | "ifaces" | "ip" => network::run(
            NetworkArgs {
                cmd: NetworkCmd::Interfaces,
            },
            json,
        ),
        "connections" | "conns" => network::run(
            NetworkArgs {
                cmd: NetworkCmd::Connections,
            },
            json,
        ),
        "ports" | "listening" => network::run(
            NetworkArgs {
                cmd: NetworkCmd::Listening,
            },
            json,
        ),
        "routes" | "route" => network::run(
            NetworkArgs {
                cmd: NetworkCmd::Routes,
            },
            json,
        ),
        "dns" => network::run(
            NetworkArgs {
                cmd: NetworkCmd::Dns,
            },
            json,
        ),
        "arp" | "neighbors" => network::run(
            NetworkArgs {
                cmd: NetworkCmd::Arp,
            },
            json,
        ),

        // --- users ----------------------------------------------------
        "users" | "accounts" => users::run(UserArgs { cmd: UserCmd::List }, json),
        "user" | "whoami" | "me" => users::run(
            UserArgs {
                cmd: UserCmd::Current,
            },
            json,
        ),
        "active" | "sessions" | "who" => users::run(
            UserArgs {
                cmd: UserCmd::Active,
            },
            json,
        ),
        "groups" => users::run(
            UserArgs {
                cmd: UserCmd::Groups { user: target },
            },
            json,
        ),

        // --- processes --------------------------------------------------
        "processes" | "ps" | "tasks" => processes::run(
            ProcessArgs {
                cmd: ProcessCmd::List {
                    count: 50,
                    sort: "cpu".into(),
                },
            },
            json,
        ),
        "process" | "pid" => {
            let raw = target.ok_or_else(|| anyhow!("usage: vidi get process <pid>"))?;
            let pid: u32 = raw
                .parse()
                .map_err(|_| anyhow!("'{raw}' is not a valid PID"))?;
            processes::run(
                ProcessArgs {
                    cmd: ProcessCmd::Detail { pid },
                },
                json,
            )
        }

        // --- services ---------------------------------------------------
        "services" => services::run(
            ServiceArgs {
                cmd: ServiceCmd::List,
            },
            json,
        ),
        "service" => match target {
            Some(name) => services::run(
                ServiceArgs {
                    cmd: ServiceCmd::Detail { name },
                },
                json,
            ),
            None => services::run(
                ServiceArgs {
                    cmd: ServiceCmd::List,
                },
                json,
            ),
        },

        // --- diagnostics / standalone tools ------------------------------
        "health" => tools::health(json),
        "diagnose" | "diagnostics" => tools::diagnose("all".into(), json),
        "analyze" | "analysis" | "threats" => tools::analyze("all".into(), json),
        "hash" | "checksum" => {
            let path = target.ok_or_else(|| anyhow!("usage: vidi get hash <file-path>"))?;
            tools::hash(path, json)
        }

        // --- catalog ------------------------------------------------------
        "list" | "help" | "commands" | "features" => {
            print_catalog();
            Ok(())
        }

        other => {
            output::warn(&format!(
                "unknown target '{other}'. Run `vidi get list` to see all available options."
            ));
            Ok(())
        }
    }
}

/// Best-effort hardware serial number lookup (device serial, not disk serial).
fn serial_number(json: bool) -> Result<()> {
    let raw = match platform::os_family() {
        "linux" => platform::capture("cat", &["/sys/class/dmi/id/product_serial"])
            .or_else(|| platform::capture("dmidecode", &["-s", "system-serial-number"])),
        "macos" => platform::capture("system_profiler", &["SPHardwareDataType"]).and_then(|s| {
            s.lines()
                .find(|l| l.contains("Serial Number"))
                .and_then(|l| l.split(':').nth(1))
                .map(|s| s.trim().to_string())
        }),
        "windows" => platform::capture("wmic", &["bios", "get", "serialnumber"])
            .and_then(|s| s.lines().nth(1).map(|l| l.trim().to_string()))
            .or_else(|| {
                platform::capture(
                    "powershell",
                    &[
                        "-NoProfile",
                        "-Command",
                        "(Get-CimInstance Win32_BIOS).SerialNumber",
                    ],
                )
            }),
        _ => None,
    }
    .filter(|s| !s.trim().is_empty());

    match raw {
        Some(serial) => {
            let serial = serial.trim().to_string();
            if json {
                return output::print_json(&serde_json::json!({ "serial_number": serial }));
            }
            output::report("Serial Number");
            output::field("Serial Number", &serial);
        }
        None => {
            if json {
                return output::print_json(&serde_json::json!({
                    "serial_number": null,
                    "note": "unavailable (may require elevated privileges / not exposed on this platform)",
                }));
            }
            output::report("Serial Number");
            output::note("could not read the serial number (try running as root/Administrator)");
        }
    }
    Ok(())
}

// --- `get hardware` overview ------------------------------------------------

#[derive(Serialize)]
struct RamSlot {
    locator: String,
    size: String,
    manufacturer: Option<String>,
    part_number: Option<String>,
    speed: Option<String>,
}

#[derive(Serialize)]
struct SwapArea {
    device: String,
    size_bytes: u64,
    used_bytes: u64,
}

/// Parse `dmidecode -t 17` to enumerate physical RAM slots/modules.
/// Returns `None` when the tool is unavailable (non-Linux, missing binary,
/// or insufficient privileges) rather than an empty list, so callers can
/// tell "no data" apart from "zero populated slots".
fn ram_slots() -> Option<Vec<RamSlot>> {
    if platform::os_family() != "linux" {
        return None;
    }
    let raw = platform::capture("dmidecode", &["-t", "17"])?;
    let mut slots = Vec::new();
    for block in raw.split("\n\n") {
        if !block.lines().any(|l| l.trim() == "Memory Device") {
            continue;
        }
        let field = |key: &str| -> Option<String> {
            block.lines().find_map(|l| {
                l.trim()
                    .strip_prefix(&format!("{key}:"))
                    .map(|v| v.trim().to_string())
            })
        };
        let size = field("Size").unwrap_or_else(|| "unknown".into());
        if size.eq_ignore_ascii_case("no module installed") {
            continue;
        }
        slots.push(RamSlot {
            locator: field("Locator").unwrap_or_else(|| "unknown".into()),
            size,
            manufacturer: field("Manufacturer").filter(|s| !s.is_empty() && s != "Not Specified"),
            part_number: field("Part Number").filter(|s| !s.is_empty() && s != "Not Specified"),
            speed: field("Speed").filter(|s| !s.is_empty() && s != "Unknown"),
        });
    }
    Some(slots)
}

/// Enumerate active swap areas from `/proc/swaps` (Linux only).
fn swap_areas() -> Vec<SwapArea> {
    let Ok(content) = std::fs::read_to_string("/proc/swaps") else {
        return Vec::new();
    };
    content
        .lines()
        .skip(1)
        .filter_map(|line| {
            let cols: Vec<&str> = line.split_whitespace().collect();
            if cols.len() < 4 {
                return None;
            }
            let size_kb: u64 = cols[2].parse().ok()?;
            let used_kb: u64 = cols[3].parse().ok()?;
            Some(SwapArea {
                device: cols[0].to_string(),
                size_bytes: size_kb * 1024,
                used_bytes: used_kb * 1024,
            })
        })
        .collect()
}

/// Reduce a partition device name to its parent block device, e.g.
/// `sda1` -> `sda`, `nvme0n1p3` -> `nvme0n1`, `mmcblk0p1` -> `mmcblk0`.
fn parent_disk_name(dev: &str) -> String {
    let name = dev.rsplit('/').next().unwrap_or(dev);
    if let Some(idx) = name.rfind('p') {
        let tail = &name[idx + 1..];
        let head_last_is_digit = name[..idx]
            .chars()
            .next_back()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false);
        if !tail.is_empty() && tail.chars().all(|c| c.is_ascii_digit()) && head_last_is_digit {
            return name[..idx].to_string();
        }
    }
    name.trim_end_matches(|c: char| c.is_ascii_digit())
        .to_string()
}

/// Parse `KEY="value"` pairs as produced by `lsblk -P`.
fn parse_pairs(line: &str) -> Vec<(String, String)> {
    let bytes = line.as_bytes();
    let mut i = 0;
    let mut out = Vec::new();
    while i < bytes.len() {
        while i < bytes.len() && bytes[i] == b' ' {
            i += 1;
        }
        let key_start = i;
        while i < bytes.len() && bytes[i] != b'=' {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        let key = line[key_start..i].to_string();
        i += 1; // skip '='
        if i < bytes.len() && bytes[i] == b'"' {
            i += 1;
            let val_start = i;
            while i < bytes.len() && bytes[i] != b'"' {
                i += 1;
            }
            let val = line[val_start..i].to_string();
            i += 1; // skip closing quote
            out.push((key, val));
        }
    }
    out
}

/// Best-effort model/serial lookup for a disk device (Linux, via `lsblk`).
fn disk_model_serial(dev_name: &str) -> (Option<String>, Option<String>) {
    if platform::os_family() != "linux" {
        return (None, None);
    }
    let parent = parent_disk_name(dev_name);
    let path = format!("/dev/{parent}");
    let Some(raw) = platform::capture("lsblk", &["-ndo", "NAME,MODEL,SERIAL", "-P", &path]) else {
        return (None, None);
    };
    let pairs = parse_pairs(raw.lines().next().unwrap_or(""));
    let mut model = None;
    let mut serial = None;
    for (k, v) in pairs {
        let v = v.trim().to_string();
        if v.is_empty() {
            continue;
        }
        match k.as_str() {
            "MODEL" => model = Some(v),
            "SERIAL" => serial = Some(v),
            _ => {}
        }
    }
    (model, serial)
}

fn hardware_overview(full: bool, json: bool) -> Result<()> {
    let mut sys = System::new_all();
    sys.refresh_cpu();
    std::thread::sleep(std::time::Duration::from_millis(200));
    sys.refresh_cpu();
    sys.refresh_memory();

    // CPU
    let first_cpu = sys.cpus().first();
    let cpu_brand = first_cpu.map(|c| c.brand().to_string()).unwrap_or_default();
    let cpu_vendor = first_cpu
        .map(|c| c.vendor_id().to_string())
        .unwrap_or_default();
    let cpu_freq_mhz = first_cpu.map(|c| c.frequency()).unwrap_or(0);
    let logical_cores = sys.cpus().len();
    let physical_cores = sys.physical_core_count();
    let cpu_usage = sys.global_cpu_info().cpu_usage();

    // RAM
    let total_mem = sys.total_memory();
    let used_mem = sys.used_memory();
    let mem_pct = if total_mem > 0 {
        used_mem as f64 / total_mem as f64 * 100.0
    } else {
        0.0
    };
    let ram_slots = ram_slots();

    // Swap
    let total_swap = sys.total_swap();
    let used_swap = sys.used_swap();
    let swap_pct = if total_swap > 0 {
        used_swap as f64 / total_swap as f64 * 100.0
    } else {
        0.0
    };
    let swap_areas = swap_areas();

    // Disks
    let disks = Disks::new_with_refreshed_list();
    struct DiskRow {
        name: String,
        mount: String,
        total: u64,
        available: u64,
        used: u64,
        model: Option<String>,
        serial: Option<String>,
    }
    let disk_rows: Vec<DiskRow> = disks
        .iter()
        .map(|d| {
            let total = d.total_space();
            let available = d.available_space();
            let name = d.name().to_string_lossy().to_string();
            let (model, serial) = if full {
                disk_model_serial(&name)
            } else {
                (None, None)
            };
            DiskRow {
                name,
                mount: d.mount_point().display().to_string(),
                total,
                available,
                used: total.saturating_sub(available),
                model,
                serial,
            }
        })
        .collect();
    let disk_total: u64 = disk_rows.iter().map(|r| r.total).sum();
    let disk_available: u64 = disk_rows.iter().map(|r| r.available).sum();
    let disk_used: u64 = disk_rows.iter().map(|r| r.used).sum();

    if json {
        let disk_json: Vec<_> = disk_rows
            .iter()
            .map(|r| {
                serde_json::json!({
                    "device": r.name, "mount_point": r.mount, "total": r.total,
                    "available": r.available, "used": r.used,
                    "model": r.model, "serial": r.serial,
                })
            })
            .collect();
        let ram_slots_json = ram_slots.as_ref().map(|slots| {
            slots
                .iter()
                .map(|s| {
                    serde_json::json!({
                        "locator": s.locator, "size": s.size,
                        "manufacturer": s.manufacturer, "part_number": s.part_number, "speed": s.speed,
                    })
                })
                .collect::<Vec<_>>()
        });
        let swap_json: Vec<_> = swap_areas
            .iter()
            .map(|a| serde_json::json!({ "device": a.device, "size": a.size_bytes, "used": a.used_bytes }))
            .collect();
        return output::print_json(&serde_json::json!({
            "cpu": {
                "brand": cpu_brand, "vendor": cpu_vendor, "frequency_mhz": cpu_freq_mhz,
                "logical_cores": logical_cores, "physical_cores": physical_cores, "usage_pct": cpu_usage,
            },
            "ram": {
                "total": total_mem, "used": used_mem, "usage_pct": mem_pct,
                "slot_count": ram_slots.as_ref().map(|s| s.len()), "slots": ram_slots_json,
            },
            "swap": {
                "total": total_swap, "used": used_swap, "usage_pct": swap_pct,
                "area_count": swap_areas.len(), "areas": swap_json,
            },
            "disks": {
                "total": disk_total, "available": disk_available, "used": disk_used, "devices": disk_json,
            },
        }));
    }

    output::report("Hardware Overview");

    output::section("CPU");
    output::field(
        "Brand",
        if cpu_brand.is_empty() {
            "unknown".to_string()
        } else {
            cpu_brand
        },
    );
    if full {
        output::field(
            "Vendor",
            if cpu_vendor.is_empty() {
                "unknown".to_string()
            } else {
                cpu_vendor
            },
        );
        output::field(
            "Physical Cores",
            physical_cores
                .map(|n| n.to_string())
                .unwrap_or_else(|| "unknown".into()),
        );
    }
    output::field("Logical Cores", logical_cores);
    output::field("Frequency", format!("{cpu_freq_mhz} MHz"));
    output::field("Current Usage", format!("{cpu_usage:.1}%"));

    output::section("RAM");
    output::field("Total", output::bytes(total_mem));
    output::field(
        "Used",
        format!("{} ({mem_pct:.1}%)", output::bytes(used_mem)),
    );
    match &ram_slots {
        Some(slots) => {
            output::field("Slots", slots.len());
            if full {
                for s in slots {
                    output::field(
                        &format!("  {}", s.locator),
                        format!(
                            "{}, {}, {}",
                            s.size,
                            s.manufacturer.as_deref().unwrap_or("unknown manufacturer"),
                            s.part_number.as_deref().unwrap_or("unknown part number"),
                        ),
                    );
                }
            }
        }
        None => output::note("RAM slot detail requires `dmidecode` (root) on this platform."),
    }

    output::section("Swap");
    if total_swap == 0 {
        output::field("Total", "none configured");
    } else {
        output::field("Total", output::bytes(total_swap));
        output::field(
            "Used",
            format!("{} ({swap_pct:.1}%)", output::bytes(used_swap)),
        );
    }
    output::field("Swap Areas", swap_areas.len());
    if full {
        for a in &swap_areas {
            output::field(
                &a.device,
                format!(
                    "{} used / {} total",
                    output::bytes(a.used_bytes),
                    output::bytes(a.size_bytes)
                ),
            );
        }
    }

    output::section(&format!("Disks ({})", disk_rows.len()));
    output::field("Total Capacity", output::bytes(disk_total));
    output::field("Free", output::bytes(disk_available));
    output::field("Used", output::bytes(disk_used));
    for r in &disk_rows {
        output::field(
            &r.mount,
            format!(
                "{} used / {} total, {} free  [{}]",
                output::bytes(r.used),
                output::bytes(r.total),
                output::bytes(r.available),
                r.name
            ),
        );
        if full {
            output::field("  Model", r.model.as_deref().unwrap_or("unknown"));
            output::field("  Serial", r.serial.as_deref().unwrap_or("unknown"));
        }
    }
    Ok(())
}

// --- `get time` --------------------------------------------------------

#[derive(Serialize)]
struct NtpStatus {
    source: Option<String>,
    synchronized: bool,
    detail: Option<String>,
}

/// Best-effort NTP/time-sync status lookup across common Linux daemons and
/// Windows time services. Returns `source: None` when nothing is detected.
fn ntp_status() -> NtpStatus {
    match platform::os_family() {
        "linux" => {
            if let Some(raw) = platform::capture("timedatectl", &["show"]) {
                let map: std::collections::HashMap<&str, &str> =
                    raw.lines().filter_map(|l| l.split_once('=')).collect();
                if !map.is_empty() {
                    let synchronized = map.get("NTPSynchronized").copied() == Some("yes")
                        || map.get("SystemClockSynchronized").copied() == Some("yes");
                    let ntp_enabled = map.get("NTP").copied() == Some("yes");
                    return NtpStatus {
                        source: Some("systemd-timesyncd (timedatectl)".to_string()),
                        synchronized,
                        detail: Some(format!(
                            "NTP enabled: {}",
                            if ntp_enabled { "yes" } else { "no" }
                        )),
                    };
                }
            }
            if let Some(raw) = platform::capture("chronyc", &["tracking"]) {
                let synchronized = !raw.to_lowercase().contains("not synchronised")
                    && !raw.to_lowercase().contains("not synchronized");
                let leap = raw
                    .lines()
                    .find(|l| l.to_lowercase().starts_with("leap status"))
                    .and_then(|l| l.split(':').nth(1))
                    .map(|s| s.trim().to_string());
                return NtpStatus {
                    source: Some("chrony".to_string()),
                    synchronized,
                    detail: leap,
                };
            }
            if let Some(raw) = platform::capture("ntpq", &["-p"]) {
                let synchronized = raw.lines().any(|l| l.trim_start().starts_with('*'));
                return NtpStatus {
                    source: Some("ntpd (ntpq)".to_string()),
                    synchronized,
                    detail: None,
                };
            }
            NtpStatus {
                source: None,
                synchronized: false,
                detail: None,
            }
        }
        "macos" => match platform::capture("systemsetup", &["-getusingnetworktime"]) {
            Some(raw) => {
                let synchronized = raw.to_lowercase().contains(": on");
                NtpStatus {
                    source: Some("systemsetup".to_string()),
                    synchronized,
                    detail: Some(raw),
                }
            }
            None => NtpStatus {
                source: None,
                synchronized: false,
                detail: None,
            },
        },
        "windows" => match platform::capture("w32tm", &["/query", "/status"]) {
            Some(raw) => {
                let synchronized = raw.to_lowercase().contains("source:")
                    && !raw.to_lowercase().contains("free-running");
                let source = raw
                    .lines()
                    .find(|l| l.to_lowercase().trim_start().starts_with("source"))
                    .and_then(|l| l.split(':').nth(1))
                    .map(|s| s.trim().to_string())
                    .or_else(|| Some("w32tm".to_string()));
                NtpStatus {
                    source,
                    synchronized,
                    detail: None,
                }
            }
            None => NtpStatus {
                source: None,
                synchronized: false,
                detail: None,
            },
        },
        _ => NtpStatus {
            source: None,
            synchronized: false,
            detail: None,
        },
    }
}

/// Best-effort IANA timezone name (e.g. `Asia/Tehran`), independent of the
/// numeric UTC offset already shown alongside it.
fn timezone_name() -> Option<String> {
    if let Ok(tz) = std::env::var("TZ") {
        if !tz.trim().is_empty() {
            return Some(tz);
        }
    }
    if let Ok(link) = std::fs::read_link("/etc/localtime") {
        let s = link.to_string_lossy();
        if let Some(idx) = s.find("zoneinfo/") {
            return Some(s[idx + "zoneinfo/".len()..].to_string());
        }
    }
    if platform::os_family() == "linux" {
        if let Ok(content) = std::fs::read_to_string("/etc/timezone") {
            let t = content.trim();
            if !t.is_empty() {
                return Some(t.to_string());
            }
        }
        if let Some(raw) = platform::capture("timedatectl", &["show", "-p", "Timezone", "--value"])
        {
            if !raw.is_empty() {
                return Some(raw);
            }
        }
    }
    None
}

fn time_report(json: bool) -> Result<()> {
    let local = chrono::Local::now();
    let utc = chrono::Utc::now();
    let tz_name = timezone_name();
    let ntp = ntp_status();

    if json {
        return output::print_json(&serde_json::json!({
            "local_time": local.format("%Y-%m-%d %H:%M:%S").to_string(),
            "utc_time": utc.format("%Y-%m-%d %H:%M:%S").to_string(),
            "utc_offset": local.format("%:z").to_string(),
            "timezone": tz_name,
            "day_of_week": local.format("%A").to_string(),
            "day_of_year": local.format("%j").to_string(),
            "iso_week": local.format("%V").to_string(),
            "unix_timestamp": local.timestamp(),
            "ntp": ntp,
        }));
    }

    output::report("Date & Time");
    output::field("Local Time", local.format("%Y-%m-%d %H:%M:%S").to_string());
    output::field("Day Of Week", local.format("%A").to_string());
    output::field("UTC Time", utc.format("%Y-%m-%d %H:%M:%S").to_string());
    output::field("UTC Offset", local.format("%:z").to_string());
    output::field("Timezone", tz_name.as_deref().unwrap_or("unknown"));
    output::field("Day Of Year", local.format("%j").to_string());
    output::field("ISO Week", local.format("%V").to_string());
    output::field("Unix Timestamp", local.timestamp());
    match ntp.source {
        Some(source) => {
            output::field("NTP Source", &source);
            output::field(
                "NTP Synchronized",
                if ntp.synchronized { "yes" } else { "no" },
            );
            if let Some(detail) = &ntp.detail {
                output::field("NTP Detail", detail);
            }
        }
        None => output::note("No NTP service detected (checked timedatectl / chronyc / ntpq)."),
    }
    Ok(())
}

fn print_catalog() {
    output::report("vidi get <what> [target] - available targets");
    let groups: [(&str, &[&str]); 7] = [
        ("System", &["os", "environment", "time"]),
        (
            "Hardware",
            &[
                "hardware [--full]",
                "cpu",
                "memory",
                "disk",
                "temperature",
                "adapters",
                "serialnumber",
            ],
        ),
        (
            "Network",
            &["network", "connections", "ports", "routes", "dns", "arp"],
        ),
        ("Users", &["user", "users", "active", "groups [user]"]),
        ("Processes", &["processes", "process <pid>"]),
        ("Services", &["services", "service [name]"]),
        (
            "Diagnostics",
            &["health", "diagnose", "analyze", "hash <file>"],
        ),
    ];
    for (group, items) in groups {
        println!("  {}", group.bold());
        for item in items {
            println!("    vidi get {}", item.green());
        }
    }
    println!(
        "\n  Add {} to any command for machine-readable output.",
        "--json".yellow()
    );
}
