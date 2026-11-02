//! Command-line surface (clap derive). Mirrors the original vidi command tree.

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "vidi",
    version,
    about = "vidi - a cross-platform system investigation & diagnostics tool",
    long_about = "vidi collects and analyzes real system information, providing \
evidence-based findings for troubleshooting, security auditing, and forensic analysis."
)]
pub struct Cli {
    /// Print output as machine-readable JSON instead of human-readable text.
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// The simplest way to see anything: `vidi get <what>` (e.g. `vidi get cpu`,
    /// `vidi get memory`, `vidi get serialnumber`). Run `vidi get list` for all options.
    Get {
        /// What to display. Run `vidi get list` to see all available options.
        what: String,
        /// Optional extra value some targets need (e.g. a PID, service name, or file path).
        target: Option<String>,
        /// Show more detail where supported (e.g. disk model/serial, RAM module info).
        #[arg(short = 'f', long)]
        full: bool,
    },
    /// View and analyze system information.
    System(SystemArgs),
    /// Monitor and analyze running processes.
    Processes(ProcessArgs),
    /// Manage and inspect system services.
    Services(ServiceArgs),
    /// Analyze and monitor the network.
    Network(NetworkArgs),
    /// Diagnose and inspect hardware.
    Hardware(HardwareArgs),
    /// Manage and inspect user accounts.
    Users(UserArgs),
    /// Save a full system snapshot to a JSON file.
    Snapshot {
        /// Output file path.
        #[arg(short, long, default_value = "snapshot.json")]
        output: String,
    },
    /// Calculate the SHA-256 hash of a file.
    Hash {
        /// Path to the file to hash.
        path: String,
    },
    /// Run diagnostic tests.
    Diagnose {
        /// Test type: all | cpu | memory | disk | network.
        #[arg(default_value = "all")]
        test_type: String,
    },
    /// Analyze the system for suspicious indicators.
    Analyze {
        /// Analysis type: all | processes | network | startup.
        #[arg(default_value = "all")]
        analysis_type: String,
    },
    /// Generate a system health report.
    Health,
    /// Real-time process/resource monitoring.
    Monitor {
        /// Duration in seconds.
        #[arg(short, long, default_value_t = 10)]
        duration: u64,
        /// Refresh interval in milliseconds.
        #[arg(short, long, default_value_t = 1000)]
        interval: u64,
    },
    /// Start a forensic investigation (collect a labelled snapshot).
    Investigate {
        /// Investigation name/label.
        #[arg(default_value = "investigation")]
        name: String,
    },
    /// List all available commands.
    Features,
}

// ---------------------------------------------------------------------------
// system
// ---------------------------------------------------------------------------

#[derive(Args)]
pub struct SystemArgs {
    #[command(subcommand)]
    pub cmd: SystemCmd,
}

#[derive(Subcommand)]
pub enum SystemCmd {
    /// Show basic system information.
    Info,
    /// Show full operating system information.
    Os,
    /// Show environment variables.
    Environment,
    /// Show system performance counters (CPU/memory/load).
    Performance,
    /// Save system information to a JSON file.
    Export {
        #[arg(short, long, default_value = "system_info.json")]
        output: String,
    },
}

// ---------------------------------------------------------------------------
// processes
// ---------------------------------------------------------------------------

#[derive(Args)]
pub struct ProcessArgs {
    #[command(subcommand)]
    pub cmd: ProcessCmd,
}

#[derive(Subcommand)]
pub enum ProcessCmd {
    /// List running processes.
    List {
        /// Maximum number of processes to show.
        #[arg(short, long, default_value_t = 50)]
        count: usize,
        /// Sort by: cpu | memory | pid | name.
        #[arg(short, long, default_value = "cpu")]
        sort: String,
    },
    /// Show the process tree hierarchy.
    Tree {
        /// Root PID (default: the whole tree).
        #[arg(short, long)]
        pid: Option<u32>,
    },
    /// Show details for a single process.
    Detail {
        #[arg(short, long)]
        pid: u32,
    },
    /// Search processes by name.
    Search {
        #[arg(short, long)]
        pattern: String,
    },
    /// Show the top processes by memory usage.
    Memory {
        #[arg(short, long, default_value_t = 20)]
        top: usize,
    },
    /// Show process I/O statistics.
    Io {
        #[arg(short, long, default_value_t = 20)]
        top: usize,
    },
    /// Terminate a process.
    Kill {
        #[arg(short, long)]
        pid: u32,
        /// Force termination (SIGKILL / TerminateProcess).
        #[arg(short, long)]
        force: bool,
    },
    /// Save the process list to a JSON file.
    Export {
        #[arg(short, long, default_value = "processes.json")]
        output: String,
    },
}

// ---------------------------------------------------------------------------
// services
// ---------------------------------------------------------------------------

#[derive(Args)]
pub struct ServiceArgs {
    #[command(subcommand)]
    pub cmd: ServiceCmd,
}

#[derive(Subcommand)]
pub enum ServiceCmd {
    /// List all services.
    List,
    /// List services filtered by status.
    Status {
        /// running | stopped | failed | all.
        #[arg(default_value = "all")]
        status: String,
    },
    /// Show details for a single service.
    Detail { name: String },
    /// Start a service.
    Start { name: String },
    /// Stop a service.
    Stop { name: String },
    /// Restart a service.
    Restart { name: String },
    /// Save the service list to a JSON file.
    Export {
        #[arg(short, long, default_value = "services.json")]
        output: String,
    },
}

// ---------------------------------------------------------------------------
// network
// ---------------------------------------------------------------------------

#[derive(Args)]
pub struct NetworkArgs {
    #[command(subcommand)]
    pub cmd: NetworkCmd,
}

#[derive(Subcommand)]
pub enum NetworkCmd {
    /// Show network interfaces and traffic counters.
    Interfaces,
    /// Show active connections.
    Connections,
    /// Show listening ports.
    Listening,
    /// Show the routing table.
    Routes,
    /// Show DNS configuration.
    Dns,
    /// Show the ARP / neighbor table.
    Arp,
    /// Test connectivity to a host.
    Test { target: String },
    /// Save network information to a JSON file.
    Export {
        #[arg(short, long, default_value = "network.json")]
        output: String,
    },
}

// ---------------------------------------------------------------------------
// hardware
// ---------------------------------------------------------------------------

#[derive(Args)]
pub struct HardwareArgs {
    #[command(subcommand)]
    pub cmd: HardwareCmd,
}

#[derive(Subcommand)]
pub enum HardwareCmd {
    /// Show CPU information.
    Cpu,
    /// Show memory information.
    Memory,
    /// Show storage devices.
    Storage,
    /// Show temperature sensors.
    Temperature,
    /// Show network adapters.
    Adapters,
    /// Save hardware information to a JSON file.
    Export {
        #[arg(short, long, default_value = "hardware.json")]
        output: String,
    },
}

// ---------------------------------------------------------------------------
// users
// ---------------------------------------------------------------------------

#[derive(Args)]
pub struct UserArgs {
    #[command(subcommand)]
    pub cmd: UserCmd,
}

#[derive(Subcommand)]
pub enum UserCmd {
    /// Show the current user.
    Current,
    /// Show logged-in / active users.
    Active,
    /// List local user accounts.
    List,
    /// Show the groups for a user (or all groups).
    Groups {
        #[arg(short, long)]
        user: Option<String>,
    },
    /// Save user information to a JSON file.
    Export {
        #[arg(short, long, default_value = "users.json")]
        output: String,
    },
}
