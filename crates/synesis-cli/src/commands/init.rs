//! `synesis init` - Initialize SuperInstance
//!
//! Downloads required models, creates configuration, sets up knowledge vault.

use clap::Args;
use dialoguer::{theme::ColorfulTheme, Confirm};
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::sync::Arc;
use tokio::signal::ctrl_c;

use crate::config::{save_config, Config};
use synesis_knowledge::vault::KnowledgeVault;
use synesis_models::downloader::{
    known_models, DownloadPhase, DownloadProgress as ModelDownloadProgress,
    Downloader as ModelDownloader,
};
use synesis_models::hardware::HardwareDetector;

#[derive(Args)]
pub struct InitArgs {
    /// Skip model downloads (config only)
    #[arg(long)]
    pub skip_models: bool,

    /// Force re-initialization
    #[arg(short, long)]
    pub force: bool,

    /// Specify hardware profile
    #[arg(long)]
    pub profile: Option<String>,

    /// Auto-confirm all prompts (non-interactive)
    #[arg(long)]
    pub yes: bool,
}

pub async fn run(args: InitArgs, _config: &Config) -> anyhow::Result<()> {
    print_banner();

    // Step 1: Detect hardware
    print_step(1, "Detecting hardware...");
    let hardware = HardwareDetector::detect()?;
    println!("  {} {}", "CPU:".dimmed(), hardware.cpu.name);
    println!(
        "  {} {} cores / {} threads",
        "Cores:".dimmed(),
        hardware.cpu.cores,
        hardware.cpu.threads
    );
    println!(
        "  {} {} ({} available)",
        "RAM:".dimmed(),
        format_bytes(hardware.ram_bytes),
        format_bytes(hardware.ram_available_bytes)
    );
    if let Some(gpu) = &hardware.gpu {
        println!(
            "  {} {} ({})",
            "GPU:".dimmed(),
            gpu.name,
            format_bytes(gpu.vram_bytes)
        );
    } else {
        println!("  {} No GPU detected (CPU-only inference)", "GPU:".dimmed());
    }
    println!(
        "  {} {}",
        "Platform:".dimmed(),
        format!("{} {}", hardware.platform.os, hardware.platform.arch)
    );
    println!();

    // Check minimum requirements
    if !hardware.meets_minimum_requirements() {
        println!(
            "{} {}",
            "⚠️  Warning:".yellow().bold(),
            "System may not meet minimum requirements (8GB RAM, 10GB disk)"
        );
        println!();
    }

    // Step 2: Create directories
    print_step(2, "Creating directories...");
    let base_dir = create_directories().await?;
    println!("  {} {}", "Created:".dimmed(), base_dir.display());
    println!();

    // Step 3: Recommend models based on hardware
    let recommended_models = recommend_models(&hardware);
    print_step(3, "Model recommendations:");

    let total_size_gb: u64 = recommended_models
        .iter()
        .filter_map(|(_, spec)| spec.size_bytes)
        .sum::<u64>()
        / (1024 * 1024 * 1024);

    println!(
        "  {} {} models (~{} GB total):",
        "Recommended:".dimmed(),
        recommended_models.len(),
        total_size_gb
    );
    for (name, spec) in &recommended_models {
        let size = spec
            .size_bytes
            .map(|b| format_bytes(b))
            .unwrap_or_else(|| "unknown".to_string());
        println!("    • {} ({})", name.cyan(), size.dimmed());
    }
    println!();

    // Step 4: Confirm download
    if !args.skip_models {
        if !args.yes {
            let should_download = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(format!(
                    "Download {} models (~{} GB)?",
                    recommended_models.len(),
                    total_size_gb
                ))
                .default(true)
                .interact()?;

            if !should_download {
                println!("  {} Skipping model downloads", "Skipped:".yellow());
                println!();
            }
        }

        print_step(4, "Downloading models...");
        download_models(&recommended_models, &base_dir, args.force).await?;
        println!();
    } else {
        print_step(4, "Skipping model downloads (--skip-models)");
        println!();
    }

    // Step 5: Initialize knowledge vault
    print_step(5, "Initializing knowledge vault...");
    init_knowledge_vault(&base_dir).await?;
    println!(
        "  {} {}",
        "Created:".dimmed(),
        base_dir.join("knowledge/knowledge.db").display()
    );
    println!();

    // Step 6: Write configuration
    print_step(6, "Writing configuration...");
    write_config(&hardware, &base_dir).await?;
    println!(
        "  {} {}",
        "Created:".dimmed(),
        base_dir.join("config.toml").display()
    );
    println!();

    // Success message
    println!("{}", "✅ Initialization complete!".green().bold());
    println!();
    println!("Next steps:");
    println!("  {} - Ask a question", "synesis ask \"Hello!\"".cyan());
    println!("  {} - Check system status", "synesis status".cyan());
    println!(
        "  {} - Add documents to knowledge",
        "synesis knowledge add <path>".cyan()
    );
    println!("  {} - View configuration", "synesis config".cyan());
    println!();

    Ok(())
}

fn print_banner() {
    println!(
        "{}",
        "╔═══════════════════════════════════════════════════════╗".bold()
    );
    println!(
        "{}",
        "║           SuperInstance AI - Initialization          ║".bold()
    );
    println!(
        "{}",
        "╚═══════════════════════════════════════════════════════╝".bold()
    );
    println!();
}

fn print_step(num: u8, msg: &str) {
    println!("{} {}", format!("[{}/6]", num).dimmed(), msg.bold());
}

fn recommend_models(
    hardware: &synesis_models::hardware::HardwareInfo,
) -> Vec<(&'static str, synesis_models::downloader::DownloadSpec)> {
    let vram_gb = hardware
        .gpu
        .as_ref()
        .map(|g| g.vram_bytes / (1024 * 1024 * 1024))
        .unwrap_or(0);

    // Always include phi-3-mini (Pathos agent)
    let mut models = vec![("phi-3-mini", known_models::phi3_mini_q4())];

    // Add models based on VRAM
    if vram_gb >= 8 || hardware.ram_bytes >= 16 * 1024 * 1024 * 1024 {
        // Good GPU or lots of RAM: add Llama 3.2 8B
        models.push(("llama-3.2-8b", known_models::llama_3_2_8b_q4()));
    }

    if vram_gb >= 12 || hardware.ram_bytes >= 32 * 1024 * 1024 * 1024 {
        // Excellent GPU or lots of RAM: add Mistral 7B
        models.push((
            "mistral-7b-instruct",
            known_models::mistral_7b_instruct_q4(),
        ));
    }

    // Always add embeddings model (small)
    models.push(("bge-micro", known_models::bge_micro()));

    models
}

async fn create_directories() -> anyhow::Result<std::path::PathBuf> {
    let base = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?
        .join(".superinstance");

    tokio::fs::create_dir_all(&base).await?;
    tokio::fs::create_dir_all(base.join("models")).await?;
    tokio::fs::create_dir_all(base.join("knowledge")).await?;
    tokio::fs::create_dir_all(base.join("cache")).await?;
    tokio::fs::create_dir_all(base.join("logs")).await?;

    Ok(base)
}

async fn download_models(
    models: &[(&'static str, synesis_models::downloader::DownloadSpec)],
    base_dir: &std::path::Path,
    _force: bool,
) -> anyhow::Result<()> {
    let models_dir = base_dir.join("models");
    let downloader = std::sync::Arc::new(ModelDownloader::new(models_dir.clone()));

    // Set up Ctrl+C handler
    let ctrl_c = tokio::spawn(async move {
        ctrl_c().await.ok();
    });

    let mut download_futures = Vec::new();

    for (name, spec) in models {
        let name = name.to_string();
        let name_for_callback = name.clone();
        let spec = spec.clone();

        // Create progress bar for this model
        let total_size = spec.size_bytes.unwrap_or(100);
        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(&format!(
                    "  {} [{{bar:40.cyan/blue}}] {{bytes}}/{{total_bytes}} ({{eta}})",
                    name
                ))?
                .progress_chars("█▓░"),
        );
        pb.set_message(format!("Downloading {}", name));

        let pb_clone = pb.clone();

        let progress_callback =
            Arc::new(
                move |progress: ModelDownloadProgress| match progress.phase {
                    DownloadPhase::Checking => {
                        pb_clone.set_message(format!("Checking {}", name_for_callback));
                    },
                    DownloadPhase::Downloading => {
                        if let Some(total) = progress.total {
                            pb_clone.set_length(total);
                        }
                        pb_clone.set_position(progress.downloaded);
                    },
                    DownloadPhase::Verifying => {
                        pb_clone.set_message(format!("Verifying {}", name_for_callback));
                    },
                    DownloadPhase::Complete => {
                        pb_clone.finish_with_message(format!("{} ✓", name_for_callback.cyan()));
                    },
                    _ => {},
                },
            );

        let downloader_ref = downloader.clone();
        let spec_clone = spec.clone();

        download_futures.push(async move {
            tokio::task::spawn_blocking(move || {
                let rt = tokio::runtime::Handle::current();
                rt.block_on(async {
                    if let Err(e) = downloader_ref
                        .download(&spec_clone, Some(progress_callback))
                        .await
                    {
                        eprintln!("  {} {}: {}", "Error:".red(), name, e);
                    }
                })
            })
            .await
            .ok();
        });
    }

    // Wait for all downloads to complete or Ctrl+C
    tokio::select! {
        _ = async { futures::future::join_all(download_futures).await } => {},
        _ = ctrl_c => {
            println!();
            println!("  {} Downloads cancelled by user", "Interrupted:".yellow());
            return Ok(());
        }
    }

    Ok(())
}

async fn init_knowledge_vault(base_dir: &std::path::Path) -> anyhow::Result<()> {
    let db_path = base_dir.join("knowledge/knowledge.db");

    // Use blocking task for SQLite operations
    tokio::task::spawn_blocking(move || {
        KnowledgeVault::open(&db_path, 384) // 384 dimensions for bge-micro
            .map_err(|e| anyhow::anyhow!("Failed to initialize knowledge vault: {}", e))?;
        Ok::<(), anyhow::Error>(())
    })
    .await??;

    Ok(())
}

async fn write_config(
    hardware: &synesis_models::hardware::HardwareInfo,
    base_dir: &std::path::Path,
) -> anyhow::Result<()> {
    let mut config = Config::default();
    config.data_dir = base_dir.to_string_lossy().to_string();

    // Configure models based on hardware
    let vram_gb = hardware
        .gpu
        .as_ref()
        .map(|g| g.vram_bytes / (1024 * 1024 * 1024))
        .unwrap_or(0);

    // Always enable Pathos with phi-3-mini
    config.agents.pathos.model = "phi-3-mini".to_string();
    config.agents.pathos.enabled = true;

    // Enable Logos if we have enough resources
    if vram_gb >= 8 || hardware.ram_bytes >= 16 * 1024 * 1024 * 1024 {
        config.agents.logos.model = "llama-3.2-8b".to_string();
        config.agents.logos.enabled = true;
    } else {
        config.agents.logos.enabled = false;
    }

    // Enable Ethos if we have excellent resources
    if vram_gb >= 12 || hardware.ram_bytes >= 32 * 1024 * 1024 * 1024 {
        config.agents.ethos.model = "mistral-7b-instruct".to_string();
        config.agents.ethos.enabled = true;
    } else {
        config.agents.ethos.enabled = false;
    }

    // Save config
    save_config(
        &config,
        Some(&base_dir.join("config.toml").to_string_lossy()),
    )?;

    Ok(())
}

fn format_bytes(bytes: u64) -> String {
    const GB: u64 = 1024 * 1024 * 1024;
    const MB: u64 = 1024 * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(16 * 1024 * 1024 * 1024), "16.0 GB");
        assert_eq!(format_bytes(512 * 1024 * 1024), "512.0 MB");
    }
}
