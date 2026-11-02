//! `hardware` command group: cpu, memory, storage, temperature, adapters.

use crate::cli::{HardwareArgs, HardwareCmd};
use crate::output;
use anyhow::Result;
use serde::Serialize;
use sysinfo::{Components, Disks, Networks, System};

#[derive(Serialize)]
struct CpuInfo {
    brand: String,
    vendor: String,
    logical_cores: usize,
    physical_cores: Option<usize>,
    frequency_mhz: u64,
    arch: Option<String>,
}

#[derive(Serialize)]
struct DiskInfo {
    name: String,
    mount_point: String,
    file_system: String,
    kind: String,
    total: u64,
    available: u64,
    removable: bool,
}

pub fn run(args: HardwareArgs, json: bool) -> Result<()> {
    match args.cmd {
        HardwareCmd::Cpu => {
            let sys = System::new_all();
            let first = sys.cpus().first();
            let info = CpuInfo {
                brand: first.map(|c| c.brand().to_string()).unwrap_or_default(),
                vendor: first.map(|c| c.vendor_id().to_string()).unwrap_or_default(),
                logical_cores: sys.cpus().len(),
                physical_cores: sys.physical_core_count(),
                frequency_mhz: first.map(|c| c.frequency()).unwrap_or(0),
                arch: System::cpu_arch(),
            };
            if json {
                return output::print_json(&info);
            }
            output::report("CPU");
            output::field("Brand", &info.brand);
            output::field("Vendor", &info.vendor);
            output::field("Architecture", info.arch.as_deref().unwrap_or("unknown"));
            output::field("Logical Cores", info.logical_cores);
            output::field(
                "Physical Cores",
                info.physical_cores
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "unknown".into()),
            );
            output::field("Frequency", format!("{} MHz", info.frequency_mhz));
        }
        HardwareCmd::Memory => {
            let sys = System::new_all();
            if json {
                return output::print_json(&serde_json::json!({
                    "total": sys.total_memory(),
                    "used": sys.used_memory(),
                    "available": sys.available_memory(),
                    "free": sys.free_memory(),
                    "total_swap": sys.total_swap(),
                    "used_swap": sys.used_swap(),
                }));
            }
            output::report("Memory (RAM)");
            output::field("Total", output::bytes(sys.total_memory()));
            output::field("Used", output::bytes(sys.used_memory()));
            output::field("Available", output::bytes(sys.available_memory()));
            output::field("Free", output::bytes(sys.free_memory()));
            output::field("Total Swap", output::bytes(sys.total_swap()));
            output::field("Used Swap", output::bytes(sys.used_swap()));
        }
        HardwareCmd::Storage => {
            let disks = Disks::new_with_refreshed_list();
            let rows: Vec<DiskInfo> = disks
                .iter()
                .map(|d| DiskInfo {
                    name: d.name().to_string_lossy().to_string(),
                    mount_point: d.mount_point().display().to_string(),
                    file_system: d.file_system().to_string_lossy().to_string(),
                    kind: format!("{:?}", d.kind()),
                    total: d.total_space(),
                    available: d.available_space(),
                    removable: d.is_removable(),
                })
                .collect();
            if json {
                return output::print_json(&rows);
            }
            output::report(&format!("Storage Devices ({})", rows.len()));
            for d in &rows {
                output::section(&d.mount_point);
                output::field("Device", &d.name);
                output::field("Filesystem", &d.file_system);
                output::field("Kind", &d.kind);
                output::field("Total", output::bytes(d.total));
                output::field("Available", output::bytes(d.available));
                output::field("Used", output::bytes(d.total.saturating_sub(d.available)));
                output::field("Removable", if d.removable { "yes" } else { "no" });
            }
        }
        HardwareCmd::Temperature => {
            let components = Components::new_with_refreshed_list();
            if json {
                let rows: Vec<_> = components
                    .iter()
                    .map(|c| serde_json::json!({ "label": c.label(), "temperature": c.temperature() }))
                    .collect();
                return output::print_json(&rows);
            }
            output::report("Temperature Sensors");
            if components.is_empty() {
                output::note("No temperature sensors available on this platform.");
            }
            for c in components.iter() {
                output::field(c.label(), format!("{:.1} °C", c.temperature()));
            }
        }
        HardwareCmd::Adapters => {
            let networks = Networks::new_with_refreshed_list();
            if json {
                let rows: Vec<_> = networks
                    .iter()
                    .map(|(n, d)| serde_json::json!({ "name": n, "mac": d.mac_address().to_string() }))
                    .collect();
                return output::print_json(&rows);
            }
            output::report("Network Adapters");
            for (name, data) in networks.iter() {
                output::field(name, data.mac_address());
            }
        }
        HardwareCmd::Export { output: path } => {
            let sys = System::new_all();
            let disks = Disks::new_with_refreshed_list();
            let payload = serde_json::json!({
                "cpu": {
                    "brand": sys.cpus().first().map(|c| c.brand().to_string()),
                    "logical_cores": sys.cpus().len(),
                    "physical_cores": sys.physical_core_count(),
                },
                "memory": {
                    "total": sys.total_memory(),
                    "used": sys.used_memory(),
                },
                "disks": disks.iter().map(|d| serde_json::json!({
                    "mount": d.mount_point().display().to_string(),
                    "total": d.total_space(),
                    "available": d.available_space(),
                })).collect::<Vec<_>>(),
            });
            output::write_json(&path, &payload)?;
            output::note(&format!("Hardware information exported to {path}"));
        }
    }
    Ok(())
}
