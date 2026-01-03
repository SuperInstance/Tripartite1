//! Manifest management commands

use clap::{Parser, Subcommand};
use synesis_models::{HardwareDetector, HardwareManifest};

use crate::config::Config;

/// Manifest management commands
#[derive(Subcommand)]
pub enum ManifestCommands {
    /// Show current hardware manifest
    Show,

    /// Validate a manifest file
    Validate {
        /// Path to manifest file
        path: String,
    },

    /// Install a manifest to the manifests directory
    Install {
        /// Path to manifest file
        path: String,

        /// Name to give the manifest (defaults to filename without extension)
        name: Option<String>,
    },

    /// List available built-in profiles
    List,
}

pub async fn run(cmd: ManifestCommands, _config: &Config) -> anyhow::Result<()> {
    match cmd {
        ManifestCommands::Show => run_show().await,
        ManifestCommands::Validate { path } => run_validate(&path).await,
        ManifestCommands::Install { path, name } => run_install(&path, name.as_deref()).await,
        ManifestCommands::List => run_list().await,
    }
}

async fn run_show() -> anyhow::Result<()> {
    println!("=== Hardware Manifest ===\n");

    // Detect and load manifest
    let manifest = HardwareManifest::detect_and_load()
        .map_err(|e| anyhow::anyhow!("Failed to detect hardware and load manifest: {}", e))?;

    // Detect current hardware
    let hardware = HardwareDetector::detect()
        .map_err(|e| anyhow::anyhow!("Failed to detect hardware: {}", e))?;

    println!("Profile: {}", manifest.name);
    println!("Description: {}", manifest.description);
    println!();

    println!("Requirements:");
    println!("  RAM: {}", format_bytes(manifest.min_ram_bytes));
    if manifest.min_vram_bytes > 0 {
        println!("  VRAM: {}", format_bytes(manifest.min_vram_bytes));
    } else {
        println!("  VRAM: N/A (CPU-only)");
    }
    println!("  Context: {} tokens", manifest.context_size);
    println!("  GPU Layers: {}", manifest.gpu_layers);
    println!();

    println!("Current Hardware:");
    println!(
        "  CPU: {} ({} cores, {} threads)",
        hardware.cpu.name, hardware.cpu.cores, hardware.cpu.threads
    );
    println!(
        "  RAM: {} / {}",
        format_bytes(hardware.ram_available_bytes),
        format_bytes(hardware.ram_bytes)
    );
    if let Some(ref gpu) = hardware.gpu {
        println!("  GPU: {} ({})", gpu.name, format_bytes(gpu.vram_bytes));
    } else {
        println!("  GPU: None detected");
    }
    println!();

    println!("Model Recommendations:");
    print_model_rec("Pathos (Intent)", &manifest.recommendations.pathos);
    print_model_rec("Logos (Logic)", &manifest.recommendations.logos);
    print_model_rec("Ethos (Truth)", &manifest.recommendations.ethos);
    print_model_rec("Embeddings", &manifest.recommendations.embeddings);

    println!();
    println!(
        "Total Download Size: {}",
        format_bytes(manifest.total_download_size())
    );

    Ok(())
}

async fn run_validate(path: &str) -> anyhow::Result<()> {
    println!("Validating manifest: {}\n", path);

    let manifest = HardwareManifest::load(std::path::Path::new(path))
        .map_err(|e| anyhow::anyhow!("Failed to load manifest: {}", e))?;

    println!("✓ Manifest is valid!\n");
    println!("{}", manifest.summary());

    Ok(())
}

async fn run_install(path: &str, name: Option<&str>) -> anyhow::Result<()> {
    println!("Installing manifest: {}\n", path);

    // Load the manifest
    let manifest = HardwareManifest::load(std::path::Path::new(path))
        .map_err(|e| anyhow::anyhow!("Failed to load manifest: {}", e))?;

    // Determine the name
    let install_name = if let Some(name) = name {
        name.to_string()
    } else {
        // Use filename without extension
        std::path::Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("custom")
            .to_string()
    };

    // Install it
    let installed_path = manifest
        .install(&install_name)
        .map_err(|e| anyhow::anyhow!("Failed to install manifest: {}", e))?;

    println!("✓ Manifest installed successfully!");
    println!("  Location: {}", installed_path.display());
    println!("  Name: {}", install_name);
    println!();
    println!("{}", manifest.summary());

    Ok(())
}

async fn run_list() -> anyhow::Result<()> {
    println!("=== Built-in Hardware Profiles ===\n");

    let profiles = synesis_models::manifest::profiles::all();

    for profile in profiles {
        println!("--- {} ---", profile.name);
        println!("  {}", profile.description);
        println!("  RAM: {}+", format_bytes(profile.min_ram_bytes));
        if profile.min_vram_bytes > 0 {
            println!("  VRAM: {}+", format_bytes(profile.min_vram_bytes));
        }
        println!(
            "  Models: pathos={}, logos={}, ethos={}, embeddings={}",
            profile.recommendations.pathos.model,
            profile.recommendations.logos.model,
            profile.recommendations.ethos.model,
            profile.recommendations.embeddings.model,
        );
        println!();
    }

    // Also list custom manifests if they exist
    let manifests_dir = dirs::home_dir().map(|p| p.join(".synesis").join("manifests"));

    if let Some(dir) = manifests_dir {
        if dir.exists() {
            println!("=== Custom Manifests ===\n");

            let entries = std::fs::read_dir(&dir)
                .map_err(|e| anyhow::anyhow!("Failed to read manifests directory: {}", e))?;

            let mut found_custom = false;
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "json") {
                    found_custom = true;
                    let name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown");

                    match HardwareManifest::load(&path) {
                        Ok(manifest) => {
                            println!("--- {} ---", name);
                            println!("  {}", manifest.description);
                            println!("  RAM: {}+", format_bytes(manifest.min_ram_bytes));
                            if manifest.min_vram_bytes > 0 {
                                println!("  VRAM: {}+", format_bytes(manifest.min_vram_bytes));
                            }
                            println!();
                        },
                        Err(e) => {
                            println!("--- {} (INVALID) ---", name);
                            println!("  Error: {}", e);
                            println!();
                        },
                    }
                }
            }

            if !found_custom {
                println!("No custom manifests found.");
                println!();
            }
        }
    }

    Ok(())
}

fn print_model_rec(agent: &str, rec: &synesis_models::ModelRecommendation) {
    println!("  {}:", agent);
    println!("    Model: {}", rec.model);
    println!("    Quantization: {}", rec.quantization);
    println!("    Size: {}", format_bytes(rec.size_bytes));
    println!("    Source: {}/{}", rec.repo_id, rec.filename);
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.1} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
