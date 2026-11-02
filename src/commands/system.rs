//! `system` command group: OS info, environment, performance.

use crate::cli::{SystemArgs, SystemCmd};
use crate::commands::platform;
use crate::output;
use anyhow::Result;
use serde::Serialize;
use sysinfo::System;

#[derive(Serialize)]
struct SystemInfo {
    hostname: Option<String>,
    os_name: Option<String>,
    os_version: Option<String>,
    long_os_version: Option<String>,
    kernel_version: Option<String>,
    os_family: String,
    cpu_arch: Option<String>,
    uptime_secs: u64,
    boot_time: u64,
}

fn collect() -> SystemInfo {
    SystemInfo {
        hostname: System::host_name(),
        os_name: System::name(),
        os_version: System::os_version(),
        long_os_version: System::long_os_version(),
        kernel_version: System::kernel_version(),
        os_family: platform::os_family_label().to_string(),
        cpu_arch: System::cpu_arch(),
        uptime_secs: System::uptime(),
        boot_time: System::boot_time(),
    }
}

pub fn run(args: SystemArgs, json: bool) -> Result<()> {
    match args.cmd {
        SystemCmd::Info | SystemCmd::Os => {
            let info = collect();
            if json {
                return output::print_json(&info);
            }
            output::report("System Information");
            output::field("Hostname", info.hostname.as_deref().unwrap_or("unknown"));
            output::field("OS", info.os_name.as_deref().unwrap_or("unknown"));
            output::field(
                "OS Version",
                info.long_os_version
                    .or(info.os_version)
                    .as_deref()
                    .unwrap_or("unknown"),
            );
            output::field("OS Family", &info.os_family);
            output::field(
                "Kernel",
                info.kernel_version.as_deref().unwrap_or("unknown"),
            );
            output::field(
                "Architecture",
                info.cpu_arch.as_deref().unwrap_or("unknown"),
            );
            output::field("Boot Time", output::timestamp(info.boot_time));
            output::field("Uptime", output::duration(info.uptime_secs));
        }
        SystemCmd::Environment => {
            let vars: std::collections::BTreeMap<String, String> = std::env::vars().collect();
            if json {
                return output::print_json(&vars);
            }
            output::report("Environment Variables");
            for (k, v) in &vars {
                output::field(k, v);
            }
        }
        SystemCmd::Performance => {
            let mut sys = System::new_all();
            sys.refresh_cpu();
            std::thread::sleep(std::time::Duration::from_millis(300));
            sys.refresh_cpu();
            sys.refresh_memory();
            let load = System::load_average();
            let global = sys.global_cpu_info().cpu_usage();
            if json {
                return output::print_json(&serde_json::json!({
                    "cpu_usage_percent": global,
                    "load_average": { "one": load.one, "five": load.five, "fifteen": load.fifteen },
                    "total_memory": sys.total_memory(),
                    "used_memory": sys.used_memory(),
                    "total_swap": sys.total_swap(),
                    "used_swap": sys.used_swap(),
                }));
            }
            output::report("Performance Counters");
            output::field("CPU Usage", format!("{global:.1}%"));
            output::field(
                "Load Average",
                format!("{:.2} {:.2} {:.2}", load.one, load.five, load.fifteen),
            );
            output::field(
                "Memory",
                format!(
                    "{} / {}",
                    output::bytes(sys.used_memory()),
                    output::bytes(sys.total_memory())
                ),
            );
            output::field(
                "Swap",
                format!(
                    "{} / {}",
                    output::bytes(sys.used_swap()),
                    output::bytes(sys.total_swap())
                ),
            );
        }
        SystemCmd::Export { output: path } => {
            let info = collect();
            output::write_json(&path, &info)?;
            output::note(&format!("System information exported to {path}"));
        }
    }
    Ok(())
}
