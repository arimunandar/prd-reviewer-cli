mod commands;
mod config;
mod figma;
mod jira;
mod templates;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "prd-reviewer")]
#[command(about = "PRD Reviewer CLI — fetch and review Product Requirement Documents for automation-readiness")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a project with Claude Code agent/skill files
    Init {
        /// Target directory (defaults to current directory)
        #[arg(short, long, default_value = ".")]
        path: String,
    },
    /// Update prd-reviewer to the latest version from git
    Update,
    /// Figma design inspection (PRD design references)
    Figma {
        #[command(subcommand)]
        command: commands::figma::FigmaCommands,
    },
    /// Jira & Confluence Wiki operations (fetch PRDs)
    Jira {
        #[command(subcommand)]
        command: commands::jira::JiraCommands,
    },
    /// PRD tools: fetch, rules, workflow, and template
    Prd {
        #[command(subcommand)]
        command: commands::prd::PrdCommands,
    },
    /// Show current version
    Version,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path } => commands::init::run(&path),
        Commands::Update => commands::update::run(),
        Commands::Figma { command } => commands::figma::run(command),
        Commands::Jira { command } => commands::jira::run(command),
        Commands::Prd { command } => commands::prd::run(command),
        Commands::Version => println!("prd-reviewer {}", env!("CARGO_PKG_VERSION")),
    }
}
