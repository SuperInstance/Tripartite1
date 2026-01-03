//! Hardware Manifests
//!
//! Defines hardware profiles and model recommendations based on
//! detected hardware capabilities.

use crate::{HardwareDetector, HardwareInfo, ModelError, ModelResult, Quantization};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{debug, info, instrument};

/// Hardware manifest - maps hardware capabilities to model recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareManifest {
    /// Profile name
    pub name: String,
    /// Description
    pub description: String,
    /// Minimum RAM required (bytes)
    pub min_ram_bytes: u64,
    /// Minimum VRAM required (bytes) - 0 for CPU-only
    pub min_vram_bytes: u64,
    /// Recommended models for each agent
    pub recommendations: AgentRecommendations,
    /// GPU layers to offload (0 for CPU-only)
    pub gpu_layers: u32,
    /// Context size
    pub context_size: u32,
}

/// Model recommendations for each agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRecommendations {
    pub pathos: ModelRecommendation,
    pub logos: ModelRecommendation,
    pub ethos: ModelRecommendation,
    pub embeddings: ModelRecommendation,
}

/// A specific model recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRecommendation {
    /// Model name/ID
    pub model: String,
    /// Quantization level
    pub quantization: Quantization,
    /// HuggingFace repo ID
    pub repo_id: String,
    /// Filename to download
    pub filename: String,
    /// Expected size in bytes
    pub size_bytes: u64,
    /// SHA256 checksum (optional)
    pub sha256: Option<String>,
}

impl HardwareManifest {
    /// Check if hardware meets this manifest's requirements
    pub fn is_compatible(&self, hardware: &HardwareInfo) -> bool {
        if hardware.ram_bytes < self.min_ram_bytes {
            return false;
        }

        if self.min_vram_bytes > 0 {
            let total_vram: u64 = hardware.gpu.as_ref().map_or(0, |g| g.vram_bytes);
            if total_vram < self.min_vram_bytes {
                return false;
            }
        }

        true
    }

    /// Get total download size for all models
    pub fn total_download_size(&self) -> u64 {
        self.recommendations.pathos.size_bytes
            + self.recommendations.logos.size_bytes
            + self.recommendations.ethos.size_bytes
            + self.recommendations.embeddings.size_bytes
    }

    /// Load manifest from a JSON file
    #[instrument]
    pub fn load(path: &Path) -> ModelResult<Self> {
        debug!("Loading manifest from: {}", path.display());

        let content = std::fs::read_to_string(path)
            .map_err(|e| ModelError::InvalidPath(format!("Failed to read manifest: {}", e)))?;

        let manifest: HardwareManifest = serde_json::from_str(&content)
            .map_err(|e| ModelError::Internal(format!("Failed to parse manifest JSON: {}", e)))?;

        // Validate the loaded manifest
        manifest.validate()?;

        Ok(manifest)
    }

    /// Validate manifest structure and values
    pub fn validate(&self) -> ModelResult<()> {
        // Check required fields
        if self.name.is_empty() {
            return Err(ModelError::Internal(
                "Manifest name cannot be empty".to_string(),
            ));
        }

        if self.min_ram_bytes == 0 {
            return Err(ModelError::Internal(
                "min_ram_bytes must be > 0".to_string(),
            ));
        }

        // Validate model recommendations
        self.validate_recommendation(&self.recommendations.pathos, "pathos")?;
        self.validate_recommendation(&self.recommendations.logos, "logos")?;
        self.validate_recommendation(&self.recommendations.ethos, "ethos")?;
        self.validate_recommendation(&self.recommendations.embeddings, "embeddings")?;

        // Validate context size
        if self.context_size == 0 || self.context_size > 131072 {
            return Err(ModelError::Internal(
                "context_size must be between 1 and 131072".to_string(),
            ));
        }

        debug!("Manifest '{}' validated successfully", self.name);
        Ok(())
    }

    /// Validate a single model recommendation
    fn validate_recommendation(
        &self,
        rec: &ModelRecommendation,
        agent_name: &str,
    ) -> ModelResult<()> {
        if rec.model.is_empty() {
            return Err(ModelError::Internal(format!(
                "{} model name cannot be empty",
                agent_name
            )));
        }

        if rec.repo_id.is_empty() {
            return Err(ModelError::Internal(format!(
                "{} repo_id cannot be empty",
                agent_name
            )));
        }

        if rec.filename.is_empty() {
            return Err(ModelError::Internal(format!(
                "{} filename cannot be empty",
                agent_name
            )));
        }

        // Size can be 0 for shared models (like ethos reusing pathos)
        // But if it's not 0, it should be reasonable
        if rec.size_bytes > 0 && rec.size_bytes < 1024 {
            return Err(ModelError::Internal(format!(
                "{} size_bytes seems too small (< 1KB)",
                agent_name
            )));
        }

        Ok(())
    }

    /// Detect hardware and load appropriate manifest
    #[instrument]
    pub fn detect_and_load() -> ModelResult<Self> {
        info!("Detecting hardware and selecting appropriate manifest...");

        // Detect hardware
        let hardware = HardwareDetector::detect()?;

        // Get manifests directory
        let manifests_dir = Self::manifests_dir()?;

        // If manifests directory exists, try to find a matching manifest
        if manifests_dir.exists() {
            debug!("Searching for manifests in: {}", manifests_dir.display());

            // Try to find a specific manifest for this hardware
            if let Some(manifest) = Self::find_matching_manifest(&manifests_dir, &hardware)? {
                info!("Found matching manifest: {}", manifest.name);
                return Ok(manifest);
            }
        } else {
            debug!("Manifests directory does not exist, using built-in profiles");
        }

        // Fallback to built-in profiles
        info!("No matching manifest found, using built-in profile selection");
        let profile = profiles::select_for_hardware(&hardware);
        info!("Selected profile: {}", profile.name);
        Ok(profile)
    }

    /// Get the manifests directory path
    fn manifests_dir() -> ModelResult<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| ModelError::Internal("Could not find home directory".to_string()))?;

        Ok(home.join(".synesis").join("manifests"))
    }

    /// Find a manifest that matches the detected hardware
    fn find_matching_manifest(dir: &Path, hardware: &HardwareInfo) -> ModelResult<Option<Self>> {
        let entries = std::fs::read_dir(dir).map_err(|e| {
            ModelError::Internal(format!("Failed to read manifests directory: {}", e))
        })?;

        // Try to find manifests with specific hardware identifiers
        let manifests: Vec<_> = entries
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "json"))
            .collect();

        debug!("Found {} manifest files", manifests.len());

        for entry in manifests {
            let path = entry.path();

            // Try to load this manifest
            match Self::load(&path) {
                Ok(manifest) => {
                    debug!("Checking manifest: {}", manifest.name);

                    // Check if it's compatible
                    if manifest.is_compatible(hardware) {
                        return Ok(Some(manifest));
                    }
                },
                Err(e) => {
                    debug!("Failed to load manifest {:?}: {}", path, e);
                    continue;
                },
            }
        }

        Ok(None)
    }

    /// Save manifest to file
    pub fn save(&self, path: &Path) -> ModelResult<()> {
        // Validate before saving
        self.validate()?;

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| ModelError::Internal(format!("Failed to create directory: {}", e)))?;
        }

        // Serialize and save
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| ModelError::Internal(format!("Failed to serialize manifest: {}", e)))?;

        std::fs::write(path, json)
            .map_err(|e| ModelError::Internal(format!("Failed to write manifest: {}", e)))?;

        info!("Saved manifest to: {}", path.display());
        Ok(())
    }

    /// Install this manifest to the manifests directory
    pub fn install(&self, name: &str) -> ModelResult<PathBuf> {
        let manifests_dir = Self::manifests_dir()?;

        // Create directory if needed
        std::fs::create_dir_all(&manifests_dir).map_err(|e| {
            ModelError::Internal(format!("Failed to create manifests directory: {}", e))
        })?;

        let manifest_path = manifests_dir.join(format!("{}.json", name));

        self.save(&manifest_path)?;

        info!(
            "Installed manifest '{}' to: {}",
            name,
            manifest_path.display()
        );
        Ok(manifest_path)
    }

    /// Get a summary of the manifest
    pub fn summary(&self) -> String {
        format!(
            "Profile: {}\n  {}\n  RAM: {}+, VRAM: {}+\n  Models: {}, {}, {}, {}",
            self.name,
            self.description,
            format_bytes(self.min_ram_bytes),
            if self.min_vram_bytes > 0 {
                format_bytes(self.min_vram_bytes)
            } else {
                "N/A".to_string()
            },
            self.recommendations.pathos.model,
            self.recommendations.logos.model,
            self.recommendations.ethos.model,
            self.recommendations.embeddings.model,
        )
    }
}

/// Format bytes as human-readable string
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

/// Predefined hardware profiles
pub mod profiles {
    use super::*;

    /// Minimal profile - CPU-only with smallest models
    pub fn minimal() -> HardwareManifest {
        HardwareManifest {
            name: "minimal".to_string(),
            description: "CPU-only with smallest models (8GB RAM minimum)".to_string(),
            min_ram_bytes: 8 * 1024 * 1024 * 1024, // 8GB
            min_vram_bytes: 0,
            recommendations: AgentRecommendations {
                pathos: ModelRecommendation {
                    model: "phi-3-mini".to_string(),
                    quantization: Quantization::Q4,
                    repo_id: "microsoft/Phi-3-mini-4k-instruct-gguf".to_string(),
                    filename: "Phi-3-mini-4k-instruct-q4.gguf".to_string(),
                    size_bytes: 2_200_000_000,
                    sha256: None,
                },
                logos: ModelRecommendation {
                    model: "llama-3.2-3b".to_string(),
                    quantization: Quantization::Q4,
                    repo_id: "bartowski/Llama-3.2-3B-Instruct-GGUF".to_string(),
                    filename: "Llama-3.2-3B-Instruct-Q4_K_M.gguf".to_string(),
                    size_bytes: 2_000_000_000,
                    sha256: None,
                },
                ethos: ModelRecommendation {
                    model: "phi-3-mini".to_string(), // Reuse Pathos model
                    quantization: Quantization::Q4,
                    repo_id: "microsoft/Phi-3-mini-4k-instruct-gguf".to_string(),
                    filename: "Phi-3-mini-4k-instruct-q4.gguf".to_string(),
                    size_bytes: 0, // Already counted
                    sha256: None,
                },
                embeddings: ModelRecommendation {
                    model: "bge-micro".to_string(),
                    quantization: Quantization::F16,
                    repo_id: "TaylorAI/bge-micro-v2".to_string(),
                    filename: "model.safetensors".to_string(),
                    size_bytes: 50_000_000,
                    sha256: None,
                },
            },
            gpu_layers: 0,
            context_size: 2048,
        }
    }

    /// Standard profile - Entry-level GPU or good CPU
    pub fn standard() -> HardwareManifest {
        HardwareManifest {
            name: "standard".to_string(),
            description: "Entry-level GPU (4GB VRAM) or 16GB RAM".to_string(),
            min_ram_bytes: 16 * 1024 * 1024 * 1024, // 16GB
            min_vram_bytes: 4 * 1024 * 1024 * 1024, // 4GB
            recommendations: AgentRecommendations {
                pathos: ModelRecommendation {
                    model: "phi-3-mini".to_string(),
                    quantization: Quantization::Q4,
                    repo_id: "microsoft/Phi-3-mini-4k-instruct-gguf".to_string(),
                    filename: "Phi-3-mini-4k-instruct-q4.gguf".to_string(),
                    size_bytes: 2_200_000_000,
                    sha256: None,
                },
                logos: ModelRecommendation {
                    model: "llama-3.2-8b".to_string(),
                    quantization: Quantization::Q4,
                    repo_id: "bartowski/Meta-Llama-3.2-8B-Instruct-GGUF".to_string(),
                    filename: "Meta-Llama-3.2-8B-Instruct-Q4_K_M.gguf".to_string(),
                    size_bytes: 4_700_000_000,
                    sha256: None,
                },
                ethos: ModelRecommendation {
                    model: "mistral-7b-instruct".to_string(),
                    quantization: Quantization::Q4,
                    repo_id: "TheBloke/Mistral-7B-Instruct-v0.2-GGUF".to_string(),
                    filename: "mistral-7b-instruct-v0.2.Q4_K_M.gguf".to_string(),
                    size_bytes: 4_100_000_000,
                    sha256: None,
                },
                embeddings: ModelRecommendation {
                    model: "bge-small".to_string(),
                    quantization: Quantization::F16,
                    repo_id: "BAAI/bge-small-en-v1.5".to_string(),
                    filename: "model.safetensors".to_string(),
                    size_bytes: 130_000_000,
                    sha256: None,
                },
            },
            gpu_layers: 20,
            context_size: 4096,
        }
    }

    /// Performance profile - Mid-range GPU
    pub fn performance() -> HardwareManifest {
        HardwareManifest {
            name: "performance".to_string(),
            description: "Mid-range GPU (8GB VRAM)".to_string(),
            min_ram_bytes: 32 * 1024 * 1024 * 1024, // 32GB
            min_vram_bytes: 8 * 1024 * 1024 * 1024, // 8GB
            recommendations: AgentRecommendations {
                pathos: ModelRecommendation {
                    model: "phi-3-medium".to_string(),
                    quantization: Quantization::Q4,
                    repo_id: "microsoft/Phi-3-medium-4k-instruct-gguf".to_string(),
                    filename: "Phi-3-medium-4k-instruct-q4.gguf".to_string(),
                    size_bytes: 8_000_000_000,
                    sha256: None,
                },
                logos: ModelRecommendation {
                    model: "llama-3.1-8b".to_string(),
                    quantization: Quantization::Q5,
                    repo_id: "bartowski/Meta-Llama-3.1-8B-Instruct-GGUF".to_string(),
                    filename: "Meta-Llama-3.1-8B-Instruct-Q5_K_M.gguf".to_string(),
                    size_bytes: 5_700_000_000,
                    sha256: None,
                },
                ethos: ModelRecommendation {
                    model: "mistral-7b-instruct".to_string(),
                    quantization: Quantization::Q5,
                    repo_id: "TheBloke/Mistral-7B-Instruct-v0.2-GGUF".to_string(),
                    filename: "mistral-7b-instruct-v0.2.Q5_K_M.gguf".to_string(),
                    size_bytes: 5_100_000_000,
                    sha256: None,
                },
                embeddings: ModelRecommendation {
                    model: "bge-base".to_string(),
                    quantization: Quantization::F16,
                    repo_id: "BAAI/bge-base-en-v1.5".to_string(),
                    filename: "model.safetensors".to_string(),
                    size_bytes: 440_000_000,
                    sha256: None,
                },
            },
            gpu_layers: 35,
            context_size: 8192,
        }
    }

    /// Ultra profile - High-end GPU
    pub fn ultra() -> HardwareManifest {
        HardwareManifest {
            name: "ultra".to_string(),
            description: "High-end GPU (16GB+ VRAM)".to_string(),
            min_ram_bytes: 64 * 1024 * 1024 * 1024,  // 64GB
            min_vram_bytes: 16 * 1024 * 1024 * 1024, // 16GB
            recommendations: AgentRecommendations {
                pathos: ModelRecommendation {
                    model: "phi-3-medium".to_string(),
                    quantization: Quantization::Q8,
                    repo_id: "microsoft/Phi-3-medium-4k-instruct-gguf".to_string(),
                    filename: "Phi-3-medium-4k-instruct-q8.gguf".to_string(),
                    size_bytes: 15_000_000_000,
                    sha256: None,
                },
                logos: ModelRecommendation {
                    model: "llama-3.1-70b".to_string(),
                    quantization: Quantization::Q4,
                    repo_id: "bartowski/Meta-Llama-3.1-70B-Instruct-GGUF".to_string(),
                    filename: "Meta-Llama-3.1-70B-Instruct-Q4_K_M.gguf".to_string(),
                    size_bytes: 40_000_000_000,
                    sha256: None,
                },
                ethos: ModelRecommendation {
                    model: "mixtral-8x7b".to_string(),
                    quantization: Quantization::Q4,
                    repo_id: "TheBloke/Mixtral-8x7B-Instruct-v0.1-GGUF".to_string(),
                    filename: "mixtral-8x7b-instruct-v0.1.Q4_K_M.gguf".to_string(),
                    size_bytes: 26_000_000_000,
                    sha256: None,
                },
                embeddings: ModelRecommendation {
                    model: "bge-large".to_string(),
                    quantization: Quantization::F16,
                    repo_id: "BAAI/bge-large-en-v1.5".to_string(),
                    filename: "model.safetensors".to_string(),
                    size_bytes: 1_340_000_000,
                    sha256: None,
                },
            },
            gpu_layers: 80,
            context_size: 16384,
        }
    }

    /// Jetson Orin Nano profile - Edge device
    pub fn jetson_orin_nano() -> HardwareManifest {
        HardwareManifest {
            name: "jetson-orin-nano".to_string(),
            description: "NVIDIA Jetson Orin Nano (8GB unified memory)".to_string(),
            min_ram_bytes: 8 * 1024 * 1024 * 1024, // 8GB unified
            min_vram_bytes: 0,                     // Unified memory
            recommendations: AgentRecommendations {
                pathos: ModelRecommendation {
                    model: "phi-3-mini".to_string(),
                    quantization: Quantization::Q4,
                    repo_id: "microsoft/Phi-3-mini-4k-instruct-gguf".to_string(),
                    filename: "Phi-3-mini-4k-instruct-q4.gguf".to_string(),
                    size_bytes: 2_200_000_000,
                    sha256: None,
                },
                logos: ModelRecommendation {
                    model: "llama-3.2-3b".to_string(),
                    quantization: Quantization::Q4,
                    repo_id: "bartowski/Llama-3.2-3B-Instruct-GGUF".to_string(),
                    filename: "Llama-3.2-3B-Instruct-Q4_K_M.gguf".to_string(),
                    size_bytes: 2_000_000_000,
                    sha256: None,
                },
                ethos: ModelRecommendation {
                    model: "phi-3-mini".to_string(),
                    quantization: Quantization::Q4,
                    repo_id: "microsoft/Phi-3-mini-4k-instruct-gguf".to_string(),
                    filename: "Phi-3-mini-4k-instruct-q4.gguf".to_string(),
                    size_bytes: 0,
                    sha256: None,
                },
                embeddings: ModelRecommendation {
                    model: "bge-micro".to_string(),
                    quantization: Quantization::F16,
                    repo_id: "TaylorAI/bge-micro-v2".to_string(),
                    filename: "model.safetensors".to_string(),
                    size_bytes: 50_000_000,
                    sha256: None,
                },
            },
            gpu_layers: 99, // Full GPU offload for Jetson
            context_size: 2048,
        }
    }

    /// Get all profiles
    pub fn all() -> Vec<HardwareManifest> {
        vec![
            minimal(),
            standard(),
            performance(),
            ultra(),
            jetson_orin_nano(),
        ]
    }

    /// Select best profile for given hardware
    pub fn select_for_hardware(hardware: &HardwareInfo) -> HardwareManifest {
        // Check from highest to lowest
        let profiles = [ultra(), performance(), standard(), minimal()];

        for profile in profiles {
            if profile.is_compatible(hardware) {
                return profile;
            }
        }

        // Fallback to minimal
        minimal()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_selection() {
        // This would need actual HardwareInfo construction
        let profiles = profiles::all();
        assert_eq!(profiles.len(), 5);
    }

    #[test]
    fn test_download_size_calculation() {
        let profile = profiles::standard();
        let size = profile.total_download_size();
        assert!(size > 0);
    }
}
