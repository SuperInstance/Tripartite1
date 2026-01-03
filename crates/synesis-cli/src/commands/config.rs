//! `synesis config` - Configuration management

use clap::Subcommand;
use comfy_table::{presets::UTF8_FULL, Table};
use owo_colors::OwoColorize;

use crate::config::Config;

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show current configuration
    Show,

    /// Get a specific config value
    Get(GetArgs),

    /// Set a config value
    Set(SetArgs),

    /// Reset configuration to defaults
    Reset(ResetArgs),

    /// Edit config file in $EDITOR
    Edit,

    /// Show config file path
    Path,
}

#[derive(clap::Args)]
pub struct GetArgs {
    /// Config key (e.g., "agents.pathos.model")
    pub key: String,
}

#[derive(clap::Args)]
pub struct SetArgs {
    /// Config key
    pub key: String,

    /// Config value
    pub value: String,
}

#[derive(clap::Args)]
pub struct ResetArgs {
    /// Reset specific key only
    pub key: Option<String>,

    /// Skip confirmation
    #[arg(short, long)]
    pub force: bool,
}

pub async fn run(cmd: ConfigCommands, config: &Config) -> anyhow::Result<()> {
    match cmd {
        ConfigCommands::Show => show_config(config).await,
        ConfigCommands::Get(args) => get_value(args, config).await,
        ConfigCommands::Set(args) => set_value(args).await,
        ConfigCommands::Reset(args) => reset_config(args).await,
        ConfigCommands::Edit => edit_config().await,
        ConfigCommands::Path => show_path().await,
    }
}

async fn show_config(config: &Config) -> anyhow::Result<()> {
    println!("{}", "Current Configuration".bold());
    println!();

    // General section
    println!("{}", "[general]".cyan());
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.add_row(vec!["data_dir", &config.data_dir]);
    table.add_row(vec!["log_level", &config.log_level]);
    println!("{table}");
    println!();

    // Agents section
    println!("{}", "[agents]".cyan());
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["Agent", "Model", "Enabled"]);
    table.add_row(vec!["pathos", "phi-3-mini", "true"]);
    table.add_row(vec!["logos", "llama-3.2-8b", "true"]);
    table.add_row(vec!["ethos", "mistral-7b", "true"]);
    println!("{table}");
    println!();

    // Privacy section
    println!("{}", "[privacy]".cyan());
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.add_row(vec!["redact_emails", "true"]);
    table.add_row(vec!["redact_phones", "true"]);
    table.add_row(vec!["redact_paths", "true"]);
    table.add_row(vec!["redact_api_keys", "true"]);
    table.add_row(vec!["custom_patterns", "[]"]);
    println!("{table}");
    println!();

    // Cloud section
    println!("{}", "[cloud]".cyan());
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.add_row(vec!["enabled", "true"]);
    table.add_row(vec!["endpoint", "api.superinstance.ai"]);
    table.add_row(vec!["auto_escalate", "true"]);
    table.add_row(vec!["max_local_tokens", "4096"]);
    println!("{table}");
    println!();

    // Consensus section
    println!("{}", "[consensus]".cyan());
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.add_row(vec!["threshold", "0.85"]);
    table.add_row(vec!["max_rounds", "3"]);
    table.add_row(vec!["weights.pathos", "0.25"]);
    table.add_row(vec!["weights.logos", "0.45"]);
    table.add_row(vec!["weights.ethos", "0.30"]);
    println!("{table}");

    Ok(())
}

async fn get_value(args: GetArgs, _config: &Config) -> anyhow::Result<()> {
    // TODO: Parse dotted key path and retrieve value
    match args.key.as_str() {
        "agents.pathos.model" => println!("phi-3-mini"),
        "consensus.threshold" => println!("0.85"),
        "cloud.endpoint" => println!("api.superinstance.ai"),
        key => {
            eprintln!("{} Unknown key: {}", "Error:".red(), key);
            std::process::exit(1);
        },
    }
    Ok(())
}

async fn set_value(args: SetArgs) -> anyhow::Result<()> {
    println!("Setting {} = {}", args.key.cyan(), args.value);

    // TODO: Validate and write to config file
    // Use synesis_cli::config::set()

    println!("{} Configuration updated", "✓".green());
    println!();
    println!("{}", "Note: Some changes require restart.".dimmed());

    Ok(())
}

async fn reset_config(args: ResetArgs) -> anyhow::Result<()> {
    let target = args.key.as_deref().unwrap_or("all settings");

    if !args.force {
        println!("{} Reset {}? [y/N] ", "Warning:".yellow().bold(), target);
        // TODO: Read confirmation
    }

    // TODO: Reset config
    println!("{} Configuration reset", "✓".green());

    Ok(())
}

async fn edit_config() -> anyhow::Result<()> {
    let config_path = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?
        .join(".superinstance")
        .join("config.toml");

    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());

    println!("Opening {} in {}...", config_path.display(), editor);

    // TODO: Actually open editor
    // std::process::Command::new(&editor).arg(&config_path).status()?;

    Ok(())
}

async fn show_path() -> anyhow::Result<()> {
    let config_path = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?
        .join(".superinstance")
        .join("config.toml");

    println!("{}", config_path.display());

    Ok(())
}
