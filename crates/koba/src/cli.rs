use clap::{Parser, Subcommand};

use crate::{commands, github::GithubCommand, hooks::HooksCommand, run_checks::Stage};

#[derive(Debug, Parser)]
#[command(
    name = "koba",
    version,
    about = "Local-first commit planning and workflow tooling",
    long_about = "Koba analyzes working-tree changes into coherent commit plans and supports local checks, hooks, PR templates, and repository workflow diagnostics."
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
    /// Review the current working tree and suggest commit groups/checks.
    Changes,
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
    /// Preview or generate GitHub workflow infrastructure.
    Github {
        #[command(subcommand)]
        command: GithubCommand,
    },
    /// Suggest a commit command from working tree changes.
    SuggestCommit,
    /// Inspect or prepare pull request workflow assets.
    Pr {
        /// Preview without writing files.
        #[arg(long)]
        dry_run: bool,
        /// Write the PR body draft to .koba/pr-body.md.
        #[arg(long)]
        apply: bool,
    },
}

pub fn run() -> Result<(), String> {
    let cli = Cli::parse();

    match cli.command {
        Command::Init { apply } => commands::init(apply),
        Command::Scan => commands::scan(),
        Command::Doctor => commands::doctor(),
        Command::Changes => commands::changes(),
        Command::Run { stage, dry_run } => commands::run(stage, dry_run),
        Command::Hooks { command } => match command {
            HooksCommand::Install {
                adapter,
                dry_run,
                apply,
            } => commands::hooks_install(adapter, dry_run, apply),
        },
        Command::Github { command } => commands::github(command),
        Command::SuggestCommit => commands::suggest_commit(),
        Command::Pr { dry_run, apply } => commands::pr(dry_run, apply),
    }
}
