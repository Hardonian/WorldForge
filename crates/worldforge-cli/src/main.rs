//! World Forge CLI — the command-line interface for the World Forge simulation platform.

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;

mod commands;
mod dashboard_server;

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
    #[arg(long = "format", default_value = "text", global = true)]
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
        #[arg(long)]
        ticks: Option<u64>,
        /// Random seed
        #[arg(long)]
        seed: Option<u64>,
    },

    /// Repeat a world with the same seed and verify deterministic results
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

    /// Export simulation data (JSON with tick-by-tick snapshots)
    Export {
        /// Path to the world directory
        path: PathBuf,
        /// Number of ticks
        #[arg(long)]
        ticks: Option<u64>,
        /// Random seed
        #[arg(long)]
        seed: Option<u64>,
        /// Output file path (stdout if omitted)
        #[arg(long = "output")]
        output_path: Option<PathBuf>,
    },

    /// Run performance benchmarks
    Benchmark {
        /// Path to the world directory
        path: PathBuf,
        /// Number of ticks per run
        #[arg(long, default_value = "1000")]
        ticks: u64,
        /// Number of repetitions
        #[arg(long, default_value = "5")]
        reps: u32,
    },

    /// Run Monte Carlo risk analysis and systemic bottleneck diagnosis
    Analyze {
        /// Path to the world directory
        path: PathBuf,
        /// Number of ticks per run
        #[arg(long, default_value = "500")]
        ticks: u64,
        /// Starting random seed
        #[arg(long, default_value = "42")]
        seed: u64,
        /// Number of Monte Carlo runs
        #[arg(long, default_value = "20")]
        runs: usize,
    },

    /// Serve the browser dashboard backed by the real simulation engine
    Dashboard {
        /// Local address to bind
        #[arg(long, default_value = "127.0.0.1:8787")]
        bind: String,
        /// Directory containing example worlds exposed by the dashboard
        #[arg(long, default_value = "examples")]
        worlds_dir: PathBuf,
        /// Directory used for durable local play saves
        #[arg(long, default_value = ".worldforge/saves")]
        saves_dir: PathBuf,
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
    /// Validate a world directory
    Validate { path: PathBuf },
    /// Build a .world package from a directory
    Build {
        /// Path to the world directory
        path: PathBuf,
        /// Output file path
        #[arg(long)]
        #[arg(long = "output")]
        output_path: Option<PathBuf>,
    },
    /// Inspect a .world package
    Inspect {
        /// Path to the .world file
        file: PathBuf,
    },
    /// Print a package or world-directory content fingerprint
    Fingerprint { path: PathBuf },
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
    /// Re-run the recorded world and compare the resulting state
    Run { file: PathBuf },
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
        Command::Export {
            path,
            ticks,
            seed,
            output_path,
        } => commands::export(&path, ticks, seed, output_path.as_deref()),
        Command::Benchmark { path, ticks, reps } => commands::benchmark(&path, ticks, reps),
        Command::Analyze {
            path,
            ticks,
            seed,
            runs,
        } => commands::analyze(&path, ticks, seed, runs, &cli.output),
        Command::Dashboard {
            bind,
            worlds_dir,
            saves_dir,
        } => dashboard_server::serve(&bind, &worlds_dir, &saves_dir),
        Command::Package { action } => match action {
            PackageAction::Validate { path } => commands::validate(&path),
            PackageAction::Build { path, output_path } => {
                commands::package_build(&path, output_path.as_deref())
            }
            PackageAction::Inspect { file } => commands::package_inspect(&file),
            PackageAction::Fingerprint { path } => commands::package_fingerprint(&path),
        },
        Command::Replay { action } => match action {
            ReplayAction::Inspect { file } => commands::replay_inspect(&file),
            ReplayAction::Verify { file } => commands::replay_verify(&file),
            ReplayAction::Run { file } => commands::replay_run(&file),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_output_does_not_conflict_with_global_format() {
        let cli = Cli::try_parse_from([
            "worldforge",
            "--format",
            "json",
            "export",
            "examples/minimal-world",
            "--output",
            "run.json",
        ])
        .unwrap();
        assert!(matches!(cli.output, OutputFormat::Json));
        assert!(matches!(
            cli.command,
            Command::Export {
                output_path: Some(_),
                ..
            }
        ));
    }
}
