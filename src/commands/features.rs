//! `features` command: print an overview of all available command groups.

use colored::Colorize;

pub fn print() {
    println!("{}", "=== vidi Features ===".bold().cyan());
    println!(
        "  {}  the simplest way to see anything: `vidi get cpu`, `vidi get memory`, ... (run `vidi get list` for all options)\n",
        "get".green().bold()
    );
    let groups = [
        ("system", "OS, boot, uptime, environment, performance"),
        (
            "processes",
            "list, tree, detail, search, memory, io, kill, export",
        ),
        (
            "services",
            "list, status, detail, start, stop, restart, export",
        ),
        (
            "network",
            "interfaces, connections, listening, routes, dns, arp, test",
        ),
        (
            "hardware",
            "cpu, memory, storage, temperature, adapters, export",
        ),
        ("users", "current, active, list, groups, export"),
        ("snapshot", "collect a full system snapshot to JSON"),
        ("hash", "calculate the SHA-256 hash of a file"),
        ("diagnose", "run cpu/memory/disk/network diagnostics"),
        ("analyze", "heuristic threat/anomaly analysis"),
        ("health", "generate a system health report"),
        ("monitor", "real-time process/resource monitoring"),
        ("investigate", "collect a labelled forensic snapshot"),
    ];
    for (name, desc) in groups {
        println!("  {:<12} {}", name.green().bold(), desc);
    }
    println!(
        "\n  Add {} to any command for machine-readable output.",
        "--json".yellow()
    );
}
