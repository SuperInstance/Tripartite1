//! SuperInstance AI - Command Line Interface
//!
//! The main entry point for the `synesis` command.

use clap::{Parser, Subcommand};
use tracing_subscriber::{fmt, EnvFilter};

mod commands;
mod config;
mod display;

/// SuperInstance AI - Local-first AI with intelligent cloud escalation
#[derive(Parser)]
#[command(name = "synesis")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Configuration file path
    #[arg(short, long, global = true)]
    config: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize SuperInstance (download models, create config)
    Init(commands::init::InitArgs),

    /// Ask a question (main interaction)
    Ask(commands::ask::AskArgs),

    /// Show system status
    Status(commands::status::StatusArgs),

    /// Display metrics and performance data
    #[command(subcommand)]
    Metrics(commands::metrics::MetricsCommands),

    /// Manage hardware manifests
    #[command(subcommand)]
    Manifest(commands::manifest::ManifestCommands),

    /// Manage local models
    #[command(subcommand)]
    Model(commands::model::ModelCommands),

    /// Manage knowledge vault
    #[command(subcommand)]
    Knowledge(commands::knowledge::KnowledgeCommands),

    /// Manage cloud connection
    #[command(subcommand)]
    Cloud(commands::cloud::CloudCommands),

    /// Configuration management
    #[command(subcommand)]
    Config(commands::config::ConfigCommands),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    fmt().with_env_filter(filter).with_target(false).init();

    let cli = Cli::parse();

    // Set verbose mode
    if cli.verbose {
        tracing::info!("Verbose mode enabled");
    }

    // Load configuration
    let config = config::load_config(cli.config.as_deref())?;

    // Dispatch to command handlers
    match cli.command {
        Commands::Init(args) => commands::init::run(args, &config).await,
        Commands::Ask(args) => commands::ask::run(args, &config).await,
        Commands::Status(args) => commands::status::run(args, &config).await,
        Commands::Metrics(cmd) => commands::metrics::run(cmd).await,
        Commands::Manifest(cmd) => commands::manifest::run(cmd, &config).await,
        Commands::Model(cmd) => commands::model::run(cmd, &config).await,
        Commands::Knowledge(cmd) => commands::knowledge::run(cmd, &config).await,
        Commands::Cloud(cmd) => commands::cloud::run(cmd, &config).await,
        Commands::Config(cmd) => commands::config::run(cmd, &config).await,
    }
}
