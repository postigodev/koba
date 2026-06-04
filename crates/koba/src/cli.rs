use clap::{Parser, Subcommand};

use crate::{commands, hooks::HooksCommand, run_checks::Stage};

#[derive(Debug, Parser)]
#[command(
    name = "koba",
    version,
    about = "Local-first Git workflow configurator",
    long_about = "Koba scans and configures repository workflow infrastructure such as commit conventions, hooks, CI checks, PR templates, and repo hygiene."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Create a starter koba.yml for the current repository.
    Init {
        /// Write the proposed koba.yml to the current directory.
        #[arg(long)]
        apply: bool,
    },
    /// Inspect workflow infrastructure and report what Koba finds.
    Scan,
    /// Diagnose workflow issues and unsafe assumptions.
    Doctor,
    /// Run a named workflow check.
    Run {
        /// Stage to run.
        stage: Stage,
        /// Print checks without executing them.
        #[arg(long)]
        dry_run: bool,
    },
    /// Inspect or plan hook installation.
    Hooks {
        #[command(subcommand)]
        command: HooksCommand,
    },
    /// Suggest a commit command from staged changes.
    SuggestCommit,
    /// Inspect or prepare pull request workflow assets.
    Pr,
}

pub fn run() -> Result<(), String> {
    let cli = Cli::parse();

    match cli.command {
        Command::Init { apply } => commands::init(apply),
        Command::Scan => commands::scan(),
        Command::Doctor => commands::doctor(),
        Command::Run { stage, dry_run } => commands::run(stage, dry_run),
        Command::Hooks { command } => match command {
            HooksCommand::Install {
                adapter,
                dry_run,
                apply,
            } => commands::hooks_install(adapter, dry_run, apply),
        },
        Command::SuggestCommit => commands::suggest_commit(),
        Command::Pr => commands::pr(),
    }
}
