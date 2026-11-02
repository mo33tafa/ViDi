//! vidi - Cross-platform system investigation & diagnostics CLI.
//!
//! A command-line tool for inspecting Linux and Windows systems.

mod cli;
mod commands;
mod output;

use clap::Parser;
use cli::{Cli, Command};

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Get { what, target, full } => commands::get::run(what, target, full, cli.json),
        Command::System(c) => commands::system::run(c, cli.json),
        Command::Processes(c) => commands::processes::run(c, cli.json),
        Command::Services(c) => commands::services::run(c, cli.json),
        Command::Network(c) => commands::network::run(c, cli.json),
        Command::Hardware(c) => commands::hardware::run(c, cli.json),
        Command::Users(c) => commands::users::run(c, cli.json),
        Command::Snapshot { output } => commands::tools::snapshot(output, cli.json),
        Command::Hash { path } => commands::tools::hash(path, cli.json),
        Command::Diagnose { test_type } => commands::tools::diagnose(test_type, cli.json),
        Command::Analyze { analysis_type } => commands::tools::analyze(analysis_type, cli.json),
        Command::Health => commands::tools::health(cli.json),
        Command::Monitor { duration, interval } => commands::tools::monitor(duration, interval),
        Command::Investigate { name } => commands::tools::investigate(name, cli.json),
        Command::Features => {
            commands::features::print();
            Ok(())
        }
    };

    if let Err(e) = result {
        output::error(&format!("{e:#}"));
        std::process::exit(1);
    }
}
