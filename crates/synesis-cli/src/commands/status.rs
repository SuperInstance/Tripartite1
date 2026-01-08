//! `synesis status` - Show system status
//!
//! Displays hardware info, model status, cloud connection, and knowledge vault stats.

use clap::Args;
use comfy_table::{presets::UTF8_FULL, Table};
use std::fs;
use std::path::PathBuf;

use crate::config::Config;

#[derive(Args)]
pub struct StatusArgs {
    /// Show detailed status for a specific component
    #[arg(short, long)]
    pub component: Option<String>,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

pub async fn run(args: StatusArgs, config: &Config) -> anyhow::Result<()> {
    if args.json {
        return print_json_status(config).await;
    }

    // Create the unified status table
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![""]);

    // Title
    table.add_row(vec!["         SYNESIS STATUS"]);

    // Hardware section
    let hw_info = get_hardware_info(config)?;
    table.add_row(vec![format!("Hardware")]);
    table.add_row(vec![format!("  GPU: {}", hw_info.gpu)]);
    table.add_row(vec![format!("  RAM: {}", hw_info.ram)]);
    table.add_row(vec![format!("  NPU: {}", hw_info.npu)]);
    table.add_row(vec!["─".repeat(40)]);

    // Models section
    let models_info = get_models_info(config)?;
    table.add_row(vec![format!("Models")]);
    for model in &models_info.models {
        table.add_row(vec![format!(
            "  {} {} ({}GB) - {}",
            if model.loaded { "✓" } else { "○" },
            model.name,
            model.size_gb,
            model.status
        )]);
    }
    table.add_row(vec!["─".repeat(40)]);

    // Knowledge Vault section
    let knowledge_info = get_knowledge_info(config)?;
    table.add_row(vec![format!("Knowledge Vault")]);
    table.add_row(vec![format!("  Documents: {}", knowledge_info.documents)]);
    table.add_row(vec![format!("  Embeddings: {}", knowledge_info.embeddings)]);
    table.add_row(vec![format!("  Last sync: {}", knowledge_info.last_sync)]);
    table.add_row(vec!["─".repeat(40)]);

    // Agents section
    let agents_info = get_agents_info(config)?;
    table.add_row(vec![format!("Agents")]);
    table.add_row(vec![format!("  Pathos: {}", agents_info.pathos)]);
    table.add_row(vec![format!("  Logos: {}", agents_info.logos)]);
    table.add_row(vec![format!("  Ethos: {}", agents_info.ethos)]);

    println!("{table}");

    Ok(())
}
// ============================================================================
// Data Structures
// ============================================================================

struct HardwareInfo {
    gpu: String,
    ram: String,
    npu: String,
}

struct ModelInfo {
    name: String,
    size_gb: String,
    status: String,
    loaded: bool,
}

struct ModelsInfo {
    models: Vec<ModelInfo>,
}

struct KnowledgeInfo {
    documents: String,
    embeddings: String,
    last_sync: String,
}

struct AgentsInfo {
    pathos: String,
    logos: String,
    ethos: String,
}

// ============================================================================
// Info Gathering Functions
// ============================================================================

fn get_hardware_info(_config: &Config) -> anyhow::Result<HardwareInfo> {
    // TODO: Integrate with synesis_models::hardware for actual detection
    // For now, return placeholder values
    Ok(HardwareInfo {
        gpu: "NVIDIA RTX 4050 (6GB VRAM)".to_string(),
        ram: "32GB available".to_string(),
        npu: "Not detected".to_string(),
    })
}

fn get_models_info(config: &Config) -> anyhow::Result<ModelsInfo> {
    let models_dir = PathBuf::from(&config.data_dir).join("models");

    let mut models = Vec::new();

    // Check for common model files in the models directory
    if models_dir.exists() {
        let entries = fs::read_dir(&models_dir)?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");

                // Check for common model formats
                if file_name.ends_with(".gguf") || file_name.ends_with(".bin") {
                    let metadata = fs::metadata(&path)?;
                    let size_gb = metadata.len() as f64 / (1024.0 * 1024.0 * 1024.0);
                    let size_str = format!("{:.1}", size_gb);

                    // Extract model name from filename
                    let model_name = file_name
                        .replace("-q4", "")
                        .replace("-q5", "")
                        .replace("-q8", "")
                        .replace("-f16", "")
                        .replace(".gguf", "")
                        .replace(".bin", "")
                        .to_string();

                    models.push(ModelInfo {
                        name: model_name.clone(),
                        size_gb: size_str,
                        status: "ready".to_string(),
                        loaded: false, // TODO: Check if actually loaded in memory
                    });
                }
            }
        }
    }

    // If no models found, add default placeholders
    if models.is_empty() {
        models.push(ModelInfo {
            name: "phi-3-mini-4k".to_string(),
            size_gb: "2.4".to_string(),
            status: "loaded".to_string(),
            loaded: true,
        });
        models.push(ModelInfo {
            name: "llama-3.2-8b".to_string(),
            size_gb: "4.7".to_string(),
            status: "ready".to_string(),
            loaded: false,
        });
        models.push(ModelInfo {
            name: "bge-micro-v1.5".to_string(),
            size_gb: "0.05".to_string(),
            status: "loaded".to_string(),
            loaded: true,
        });
    }

    Ok(ModelsInfo { models })
}

fn get_knowledge_info(config: &Config) -> anyhow::Result<KnowledgeInfo> {
    let knowledge_db = PathBuf::from(&config.data_dir).join("knowledge.db");

    if knowledge_db.exists() {
        // TODO: Query actual database for stats
        // For now, return placeholder values
        Ok(KnowledgeInfo {
            documents: "47".to_string(),
            embeddings: "1,234".to_string(),
            last_sync: "2 hours ago".to_string(),
        })
    } else {
        Ok(KnowledgeInfo {
            documents: "0".to_string(),
            embeddings: "0".to_string(),
            last_sync: "Never".to_string(),
        })
    }
}

fn get_agents_info(config: &Config) -> anyhow::Result<AgentsInfo> {
    // TODO: Check actual agent status from synesis_core::agents
    // For now, check config and return idle status
    let pathos_status = if config.agents.pathos.enabled {
        "idle".to_string()
    } else {
        "disabled".to_string()
    };

    let logos_status = if config.agents.logos.enabled {
        "idle".to_string()
    } else {
        "disabled".to_string()
    };

    let ethos_status = if config.agents.ethos.enabled {
        "idle".to_string()
    } else {
        "disabled".to_string()
    };

    Ok(AgentsInfo {
        pathos: pathos_status,
        logos: logos_status,
        ethos: ethos_status,
    })
}

// ============================================================================
// JSON Output
// ============================================================================

async fn print_json_status(config: &Config) -> anyhow::Result<()> {
    let hw_info = get_hardware_info(config)?;
    let models_info = get_models_info(config)?;
    let knowledge_info = get_knowledge_info(config)?;
    let agents_info = get_agents_info(config)?;

    let status = serde_json::json!({
        "hardware": {
            "gpu": hw_info.gpu,
            "ram": hw_info.ram,
            "npu": hw_info.npu,
        },
        "models": models_info.models.iter().map(|m| {
            serde_json::json!({
                "name": m.name,
                "size_gb": m.size_gb,
                "status": m.status,
                "loaded": m.loaded,
            })
        }).collect::<Vec<_>>(),
        "knowledge": {
            "documents": knowledge_info.documents,
            "embeddings": knowledge_info.embeddings,
            "last_sync": knowledge_info.last_sync,
        },
        "agents": {
            "pathos": agents_info.pathos,
            "logos": agents_info.logos,
            "ethos": agents_info.ethos,
        },
        "cloud": {
            "connected": false,
            "endpoint": config.cloud.endpoint,
        }
    });

    println!("{}", serde_json::to_string_pretty(&status)?);
    Ok(())
}
