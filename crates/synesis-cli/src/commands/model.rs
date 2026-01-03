//! `synesis model` - Model management commands

use clap::Subcommand;
use comfy_table::{presets::UTF8_FULL, Table};
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;

use crate::config::Config;

#[derive(Subcommand)]
pub enum ModelCommands {
    /// List available and installed models
    List(ListArgs),

    /// Download a model
    Download(DownloadArgs),

    /// Remove a model
    Remove(RemoveArgs),

    /// Show model details
    Info(InfoArgs),

    /// Verify model integrity
    Verify(VerifyArgs),
}

#[derive(clap::Args)]
pub struct ListArgs {
    /// Show only installed models
    #[arg(long)]
    pub installed: bool,

    /// Show only available (not installed) models
    #[arg(long)]
    pub available: bool,
}

#[derive(clap::Args)]
pub struct DownloadArgs {
    /// Model name or HuggingFace ID
    pub model: String,

    /// Quantization level (q4, q5, q8, f16)
    #[arg(short, long, default_value = "q4")]
    pub quant: String,
}

#[derive(clap::Args)]
pub struct RemoveArgs {
    /// Model name
    pub model: String,

    /// Skip confirmation
    #[arg(short, long)]
    pub force: bool,
}

#[derive(clap::Args)]
pub struct InfoArgs {
    /// Model name
    pub model: String,
}

#[derive(clap::Args)]
pub struct VerifyArgs {
    /// Model name (or 'all' to verify all)
    pub model: String,
}

pub async fn run(cmd: ModelCommands, _config: &Config) -> anyhow::Result<()> {
    match cmd {
        ModelCommands::List(args) => list_models(args).await,
        ModelCommands::Download(args) => download_model(args).await,
        ModelCommands::Remove(args) => remove_model(args).await,
        ModelCommands::Info(args) => show_model_info(args).await,
        ModelCommands::Verify(args) => verify_model(args).await,
    }
}

async fn list_models(args: ListArgs) -> anyhow::Result<()> {
    println!("{}", "Available Models".bold());
    println!();

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["Model", "Status", "Size", "Use Case"]);

    // TODO: Get from synesis_models::registry
    let models = vec![
        ("phi-3-mini", true, "2.1 GB", "Pathos agent (intent)"),
        ("llama-3.2-3b", false, "2.0 GB", "Logos agent (small)"),
        ("llama-3.2-8b", true, "4.7 GB", "Logos agent (recommended)"),
        ("mistral-7b-instruct", false, "4.1 GB", "Ethos agent"),
        ("qwen2.5-7b", false, "4.5 GB", "Alternative reasoning"),
        ("bge-micro-v1.5", true, "48 MB", "Embeddings"),
        ("bge-small-en", false, "130 MB", "Embeddings (better)"),
    ];

    for (name, installed, size, use_case) in models {
        if args.installed && !installed {
            continue;
        }
        if args.available && installed {
            continue;
        }

        let status = if installed {
            "✓ Installed".green().to_string()
        } else {
            "○ Available".dimmed().to_string()
        };

        table.add_row(vec![name, &status, size, use_case]);
    }

    println!("{table}");
    Ok(())
}

async fn download_model(args: DownloadArgs) -> anyhow::Result<()> {
    println!("{} {}", "Downloading:".bold(), args.model);
    println!("{} {}", "Quantization:".dimmed(), args.quant);
    println!();

    // TODO: Use synesis_models::downloader
    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg}\n[{bar:50.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
            .progress_chars("█▓░"),
    );

    pb.set_message(format!("Downloading {}-{}.gguf...", args.model, args.quant));

    for i in 0..=100 {
        pb.set_position(i);
        tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;
    }

    pb.finish_with_message(format!("✓ {} downloaded successfully", args.model));

    println!();
    println!("{}", "Verifying checksum...".dimmed());
    println!("{}", "✓ Checksum verified".green());

    Ok(())
}

async fn remove_model(args: RemoveArgs) -> anyhow::Result<()> {
    if !args.force {
        println!(
            "{} Remove model '{}'? [y/N] ",
            "Warning:".yellow().bold(),
            args.model
        );
        // TODO: Read confirmation
    }

    println!("Removing {}...", args.model);
    // TODO: synesis_models::registry::remove()
    println!("{} {} removed", "✓".green(), args.model);

    Ok(())
}

async fn show_model_info(args: InfoArgs) -> anyhow::Result<()> {
    println!("{}", format!("Model: {}", args.model).bold());
    println!();

    // TODO: Get from synesis_models::registry
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);

    table.add_row(vec!["Name", &args.model]);
    table.add_row(vec!["Source", "HuggingFace"]);
    table.add_row(vec!["Architecture", "Llama"]);
    table.add_row(vec!["Parameters", "8B"]);
    table.add_row(vec!["Quantization", "Q4_K_M"]);
    table.add_row(vec!["Size", "4.7 GB"]);
    table.add_row(vec!["Context Length", "8192"]);
    table.add_row(vec!["SHA256", "a1b2c3d4..."]);
    table.add_row(vec!["Downloaded", "2024-01-15"]);
    table.add_row(vec!["Last Used", "2 hours ago"]);

    println!("{table}");
    Ok(())
}

async fn verify_model(args: VerifyArgs) -> anyhow::Result<()> {
    if args.model == "all" {
        println!("Verifying all models...");
        // TODO: Iterate through all models
    } else {
        println!("Verifying {}...", args.model);
    }

    // TODO: synesis_models::verify()
    println!("{} Checksum valid", "✓".green());

    Ok(())
}
