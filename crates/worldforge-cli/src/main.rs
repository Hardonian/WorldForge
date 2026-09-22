//! World Forge CLI — the command-line interface for the World Forge simulation platform.

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;

mod commands;

#[derive(Parser)]
#[command(
    name = "worldforge",
    about = "World Forge — deterministic, moddable simulation runtime",
    version,
    long_about = "A deterministic, moddable simulation runtime where entire worlds are packages.\n\n\
                   Run simulations headlessly, verify replays, build world packages, and test\n\
                   determinism across seeds."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Output format (text or json)
    #[arg(long, default_value = "text", global = true)]
    output: OutputFormat,
}

#[derive(Clone, Debug, clap::ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Subcommand)]
enum Command {
    /// Check environment and runtime health
    Doctor,

    /// Validate a world directory
    Validate {
        /// Path to the world directory
        path: PathBuf,
    },

    /// Run a simulation
    Run {
        /// Path to the world directory
        path: PathBuf,
        /// Number of ticks to simulate
        #[arg(long, default_value = "1000")]
        ticks: u64,
        /// Random seed
        #[arg(long, default_value = "42")]
        seed: u64,
    },

    /// Test a world across multiple seeds for determinism
    TestWorld {
        /// Path to the world directory
        path: PathBuf,
        /// Number of runs to execute
        #[arg(long, default_value = "10")]
        runs: u32,
        /// Number of ticks per run
        #[arg(long, default_value = "1000")]
        ticks: u64,
    },

    /// Package operations
    Package {
        #[command(subcommand)]
        action: PackageAction,
    },

    /// Replay operations
    Replay {
        #[command(subcommand)]
        action: ReplayAction,
    },
}

#[derive(Subcommand)]
enum PackageAction {
    /// Build a .world package from a directory
    Build {
        /// Path to the world directory
        path: PathBuf,
        /// Output file path
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Inspect a .world package
    Inspect {
        /// Path to the .world file
        file: PathBuf,
    },
}

#[derive(Subcommand)]
enum ReplayAction {
    /// Inspect a replay file
    Inspect {
        /// Path to the replay file
        file: PathBuf,
    },
    /// Verify a replay file's integrity
    Verify {
        /// Path to the replay file
        file: PathBuf,
    },
}

fn main() -> ExitCode {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .with_target(false)
        .init();

    let cli = Cli::parse();

    let result = match cli.command {
        Command::Doctor => commands::doctor(),
        Command::Validate { path } => commands::validate(&path),
        Command::Run { path, ticks, seed } => commands::run(&path, ticks, seed, &cli.output),
        Command::TestWorld { path, runs, ticks } => commands::test_world(&path, runs, ticks),
        Command::Package { action } => match action {
            PackageAction::Build { path, output } => commands::package_build(&path, output.as_deref()),
            PackageAction::Inspect { file } => commands::package_inspect(&file),
        },
        Command::Replay { action } => match action {
            ReplayAction::Inspect { file } => commands::replay_inspect(&file),
            ReplayAction::Verify { file } => commands::replay_verify(&file),
        },
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {}", e);
            ExitCode::FAILURE
        }
    }
}
