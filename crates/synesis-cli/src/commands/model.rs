//! `synesis model` - Model management commands

use clap::Subcommand;
use comfy_table::{presets::UTF8_FULL, Table};
use owo_colors::OwoColorize;
use std::path::PathBuf;

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
    /// Model name (e.g., bge-micro, phi-3-mini, llama-3.2-3b)
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

pub async fn run(cmd: ModelCommands, config: &Config) -> anyhow::Result<()> {
    match cmd {
        ModelCommands::List(args) => list_models(args).await,
        ModelCommands::Download(args) => download_model(args, config).await,
        ModelCommands::Remove(args) => remove_model(args, config).await,
        ModelCommands::Info(args) => show_model_info(args).await,
        ModelCommands::Verify(args) => verify_model(args, config).await,
    }
}

/// Model metadata
struct ModelInfo {
    name: &'static str,
    display_name: &'static str,
    description: &'static str,
    size_mb: u64,
    url: &'static str,
    file_name: &'static str,
    model_type: ModelType,
}

enum ModelType {
    Embedding,
    Llm,
}

impl ModelInfo {
    const fn new(
        name: &'static str,
        display_name: &'static str,
        description: &'static str,
        size_mb: u64,
        url: &'static str,
        file_name: &'static str,
        model_type: ModelType,
    ) -> Self {
        Self {
            name,
            display_name,
            description,
            size_mb,
            url,
            file_name,
            model_type,
        }
    }
}

/// Available models registry
const MODELS: &[ModelInfo] = &[
    ModelInfo::new(
        "bge-micro",
        "BGE-Micro-v2",
        "Embedding model (384 dimensions)",
        48,
        "https://huggingface.co/BAAI/bge-m3/resolve/main/bge-micro-v2.gguf",
        "bge-micro-v2.gguf",
        ModelType::Embedding,
    ),
    ModelInfo::new(
        "phi-3-mini",
        "Phi-3 Mini",
        "Pathos agent (intent understanding)",
        2100,
        "https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf/resolve/main/Phi-3-mini-4k-instruct-q4.gguf",
        "Phi-3-mini-4k-instruct-q4.gguf",
        ModelType::Llm,
    ),
    ModelInfo::new(
        "llama-3.2-3b",
        "Llama 3.2 3B",
        "Logos agent (lightweight reasoning)",
        2000,
        "https://huggingface.co/hugging-quants/Llama-3.2-3B-Instruct-Q4_K_M-GGUF/resolve/main/llama-3.2-3b-instruct-q4_k_m.gguf",
        "llama-3.2-3b-instruct-q4_k_m.gguf",
        ModelType::Llm,
    ),
    ModelInfo::new(
        "llama-3.2-8b",
        "Llama 3.2 8B",
        "Logos agent (recommended reasoning)",
        4700,
        "https://huggingface.co/hugging-quants/Llama-3.2-8B-Instruct-Q4_K_M-GGUF/resolve/main/llama-3.2-8b-instruct-q4_k_m.gguf",
        "llama-3.2-8b-instruct-q4_k_m.gguf",
        ModelType::Llm,
    ),
];

async fn list_models(args: ListArgs) -> anyhow::Result<()> {
    println!("{}", "Available Models".bold());
    println!();

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["Model", "Type", "Size", "Status", "Use Case"]);

    let models_dir = get_models_dir()?;
    let has_filter = args.installed || args.available;

    for model in MODELS {
        let model_path = models_dir.join(model.file_name);
        let installed = model_path.exists();

        // Apply filters
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

        let model_type = match model.model_type {
            ModelType::Embedding => "Embedding".cyan().to_string(),
            ModelType::Llm => "LLM".yellow().to_string(),
        };

        table.add_row(vec![
            model.display_name,
            &model_type,
            &format!("{} MB", model.size_mb),
            &status,
            model.description,
        ]);
    }

    if !has_filter || table.row_iter().count() > 0 {
        println!("{table}");
    }

    Ok(())
}

async fn download_model(args: DownloadArgs, _config: &Config) -> anyhow::Result<()> {
    // Find model in registry
    let model_info = MODELS
        .iter()
        .find(|m| m.name == args.model)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Unknown model: {}. Available models: {}",
                args.model,
                MODELS.iter().map(|m| m.name).collect::<Vec<_>>().join(", ")
            )
        })?;

    println!("{} {}", "Downloading:".bold(), model_info.display_name);
    println!(
        "{} {}",
        "Type:".dimmed(),
        match model_info.model_type {
            ModelType::Embedding => "Embedding Model",
            ModelType::Llm => "Large Language Model",
        }
    );
    println!("{} {} MB", "Size:".dimmed(), model_info.size_mb);
    println!();
    println!("{} {}", "Source URL:".dimmed(), model_info.url);
    println!();

    println!("{}", "Download instructions:".bold());
    println!();
    println!("To download this model, run:");
    println!(
        "  curl -L {} -o ~/.synesis/models/{}",
        model_info.url, model_info.file_name
    );
    println!();
    println!("Or use wget:");
    println!(
        "  wget {} -O ~/.synesis/models/{}",
        model_info.url, model_info.file_name
    );
    println!();
    println!("{} Models directory: ~/.synesis/models/", "Note:".yellow());
    println!();

    // If it's the embedding model, add special note
    if matches!(model_info.model_type, ModelType::Embedding) {
        println!(
            "{}",
            "Note: This embedding model will be used for semantic search.".dimmed()
        );
    }

    Ok(())
}

async fn remove_model(args: RemoveArgs, _config: &Config) -> anyhow::Result<()> {
    // Find model in registry
    let model_info = MODELS
        .iter()
        .find(|m| m.name == args.model)
        .ok_or_else(|| anyhow::anyhow!("Unknown model: {}", args.model))?;

    let models_dir = get_models_dir()?;
    let model_path = models_dir.join(model_info.file_name);

    if !model_path.exists() {
        println!(
            "{} Model {} is not installed",
            "Note:".yellow(),
            args.model.cyan()
        );
        return Ok(());
    }

    if !args.force {
        println!(
            "{} Remove model '{}'? [y/N] ",
            "Warning:".yellow().bold(),
            args.model.cyan()
        );
        // TODO: Read confirmation from stdin
        // For now, require explicit --force flag
        println!("Use --force to confirm removal");
        return Ok(());
    }

    println!("Removing {}...", args.model);
    std::fs::remove_file(&model_path)?;
    println!("{} {} removed", "✓".green(), args.model.cyan());

    Ok(())
}

async fn show_model_info(args: InfoArgs) -> anyhow::Result<()> {
    // Find model in registry
    let model_info = MODELS
        .iter()
        .find(|m| m.name == args.model)
        .ok_or_else(|| anyhow::anyhow!("Unknown model: {}", args.model))?;

    println!("{}", format!("Model: {}", model_info.display_name).bold());
    println!();

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);

    table.add_row(vec!["Name", model_info.display_name]);
    table.add_row(vec![
        "Type",
        match model_info.model_type {
            ModelType::Embedding => "Embedding Model",
            ModelType::Llm => "Large Language Model",
        },
    ]);
    table.add_row(vec!["Description", model_info.description]);
    table.add_row(vec!["Size", &format!("{} MB", model_info.size_mb)]);
    table.add_row(vec!["Source URL", model_info.url]);

    let models_dir = get_models_dir()?;
    let model_path = models_dir.join(model_info.file_name);

    if model_path.exists() {
        let metadata = std::fs::metadata(&model_path)?;
        table.add_row(vec!["Status", &"✓ Installed".green().to_string()]);
        table.add_row(vec![
            "File Size",
            &format!("{} MB", metadata.len() / 1024 / 1024),
        ]);
        table.add_row(vec!["Path", &model_path.display().to_string()]);
    } else {
        table.add_row(vec!["Status", &"○ Not installed".dimmed().to_string()]);
    }

    println!("{table}");
    Ok(())
}

async fn verify_model(args: VerifyArgs, _config: &Config) -> anyhow::Result<()> {
    if args.model == "all" {
        println!("Verifying all installed models...");
        println!();

        let models_dir = get_models_dir()?;
        let mut verified_count = 0;
        let mut failed_count = 0;

        for model in MODELS {
            let model_path = models_dir.join(model.file_name);
            if model_path.exists() {
                match verify_model_file(&model_path) {
                    Ok(_) => {
                        println!("{} {} - OK", "✓".green(), model.display_name);
                        verified_count += 1;
                    },
                    Err(e) => {
                        println!("{} {} - FAILED: {}", "✗".red(), model.display_name, e);
                        failed_count += 1;
                    },
                }
            }
        }

        println!();
        println!("Verified: {}, Failed: {}", verified_count, failed_count);
    } else {
        // Find model in registry
        let model_info = MODELS
            .iter()
            .find(|m| m.name == args.model)
            .ok_or_else(|| anyhow::anyhow!("Unknown model: {}", args.model))?;

        let models_dir = get_models_dir()?;
        let model_path = models_dir.join(model_info.file_name);

        if !model_path.exists() {
            println!(
                "{} Model {} is not installed",
                "Note:".yellow(),
                args.model.cyan()
            );
            return Ok(());
        }

        println!("Verifying {}...", model_info.display_name);
        verify_model_file(&model_path)?;
        println!("{} Model verified", "✓".green());
    }

    Ok(())
}

/// Verify a model file
fn verify_model_file(path: &PathBuf) -> anyhow::Result<()> {
    let metadata = std::fs::metadata(path)?;

    // Check file size (must be at least 1KB)
    if metadata.len() < 1024 {
        anyhow::bail!("File too small ({} bytes)", metadata.len());
    }

    // TODO: Add SHA256 checksum verification
    // For now, just check that file exists and has reasonable size

    Ok(())
}

/// Get the models directory
fn get_models_dir() -> anyhow::Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    Ok(home.join(".synesis").join("models"))
}
