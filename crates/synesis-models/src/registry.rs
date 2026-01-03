//! Model Registry
//!
//! Tracks available and installed models, their metadata, and status.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tracing::{debug, info, instrument};

use crate::{ModelResult, Quantization};

/// Model status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelStatus {
    /// Model is available but not downloaded
    Available,
    /// Model is being downloaded
    Downloading,
    /// Model is downloaded and ready
    Ready,
    /// Model is currently loaded in memory
    Loaded,
    /// Model download/verification failed
    Failed,
}

/// Information about a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Unique model identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Model family (llama, phi, mistral, etc.)
    pub family: String,
    /// Parameter count (e.g., "7B", "8B")
    pub parameters: String,
    /// Available quantizations
    pub quantizations: Vec<QuantizedVariant>,
    /// HuggingFace repository
    pub hf_repo: String,
    /// License
    pub license: String,
    /// Recommended use case
    pub use_case: String,
    /// Context window size
    pub context_length: u32,
    /// Whether this model is recommended
    pub recommended: bool,
}

/// A quantized variant of a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantizedVariant {
    /// Quantization level
    pub quantization: Quantization,
    /// File name
    pub filename: String,
    /// File size in bytes
    pub size_bytes: u64,
    /// SHA256 checksum
    pub sha256: String,
    /// Status
    pub status: ModelStatus,
    /// Local path (if downloaded)
    pub local_path: Option<PathBuf>,
}

/// Model registry
pub struct ModelRegistry {
    /// Base directory for models
    models_dir: PathBuf,
    /// Registry of known models
    models: HashMap<String, ModelInfo>,
    /// Currently loaded model
    #[allow(dead_code)]
    loaded_model: Option<String>,
}

impl ModelRegistry {
    /// Create a new registry
    pub fn new(models_dir: PathBuf) -> Self {
        let mut registry = Self {
            models_dir,
            models: HashMap::new(),
            loaded_model: None,
        };
        registry.load_builtin_models();
        registry
    }

    /// Load built-in model definitions
    fn load_builtin_models(&mut self) {
        // Phi-3 Mini (Pathos agent)
        self.models.insert(
            "phi-3-mini".to_string(),
            ModelInfo {
                id: "phi-3-mini".to_string(),
                name: "Phi-3 Mini".to_string(),
                family: "phi".to_string(),
                parameters: "3.8B".to_string(),
                quantizations: vec![QuantizedVariant {
                    quantization: Quantization::Q4,
                    filename: "phi-3-mini-4k-instruct-q4_k_m.gguf".to_string(),
                    size_bytes: 2_200_000_000,
                    sha256: "placeholder".to_string(),
                    status: ModelStatus::Available,
                    local_path: None,
                }],
                hf_repo: "microsoft/Phi-3-mini-4k-instruct-gguf".to_string(),
                license: "MIT".to_string(),
                use_case: "Intent extraction, classification".to_string(),
                context_length: 4096,
                recommended: true,
            },
        );

        // Llama 3.2 3B
        self.models.insert(
            "llama-3.2-3b".to_string(),
            ModelInfo {
                id: "llama-3.2-3b".to_string(),
                name: "Llama 3.2 3B".to_string(),
                family: "llama".to_string(),
                parameters: "3B".to_string(),
                quantizations: vec![QuantizedVariant {
                    quantization: Quantization::Q4,
                    filename: "llama-3.2-3b-instruct-q4_k_m.gguf".to_string(),
                    size_bytes: 2_000_000_000,
                    sha256: "placeholder".to_string(),
                    status: ModelStatus::Available,
                    local_path: None,
                }],
                hf_repo: "meta-llama/Llama-3.2-3B-Instruct-GGUF".to_string(),
                license: "Llama 3.2 Community".to_string(),
                use_case: "Reasoning (lightweight)".to_string(),
                context_length: 8192,
                recommended: false,
            },
        );

        // Llama 3.2 8B (Logos agent)
        self.models.insert(
            "llama-3.2-8b".to_string(),
            ModelInfo {
                id: "llama-3.2-8b".to_string(),
                name: "Llama 3.2 8B".to_string(),
                family: "llama".to_string(),
                parameters: "8B".to_string(),
                quantizations: vec![
                    QuantizedVariant {
                        quantization: Quantization::Q4,
                        filename: "llama-3.2-8b-instruct-q4_k_m.gguf".to_string(),
                        size_bytes: 4_700_000_000,
                        sha256: "placeholder".to_string(),
                        status: ModelStatus::Available,
                        local_path: None,
                    },
                    QuantizedVariant {
                        quantization: Quantization::Q8,
                        filename: "llama-3.2-8b-instruct-q8_0.gguf".to_string(),
                        size_bytes: 8_500_000_000,
                        sha256: "placeholder".to_string(),
                        status: ModelStatus::Available,
                        local_path: None,
                    },
                ],
                hf_repo: "meta-llama/Llama-3.2-8B-Instruct-GGUF".to_string(),
                license: "Llama 3.2 Community".to_string(),
                use_case: "Reasoning, RAG synthesis".to_string(),
                context_length: 8192,
                recommended: true,
            },
        );

        // Mistral 7B Instruct (Ethos agent)
        self.models.insert(
            "mistral-7b-instruct".to_string(),
            ModelInfo {
                id: "mistral-7b-instruct".to_string(),
                name: "Mistral 7B Instruct".to_string(),
                family: "mistral".to_string(),
                parameters: "7B".to_string(),
                quantizations: vec![QuantizedVariant {
                    quantization: Quantization::Q4,
                    filename: "mistral-7b-instruct-v0.3-q4_k_m.gguf".to_string(),
                    size_bytes: 4_100_000_000,
                    sha256: "placeholder".to_string(),
                    status: ModelStatus::Available,
                    local_path: None,
                }],
                hf_repo: "mistralai/Mistral-7B-Instruct-v0.3-GGUF".to_string(),
                license: "Apache 2.0".to_string(),
                use_case: "Verification, safety checks".to_string(),
                context_length: 32768,
                recommended: true,
            },
        );

        // BGE Micro (Embeddings)
        self.models.insert(
            "bge-micro-v1.5".to_string(),
            ModelInfo {
                id: "bge-micro-v1.5".to_string(),
                name: "BGE Micro v1.5".to_string(),
                family: "bge".to_string(),
                parameters: "22M".to_string(),
                quantizations: vec![QuantizedVariant {
                    quantization: Quantization::F16,
                    filename: "bge-micro-v1.5.gguf".to_string(),
                    size_bytes: 48_000_000,
                    sha256: "placeholder".to_string(),
                    status: ModelStatus::Available,
                    local_path: None,
                }],
                hf_repo: "BAAI/bge-micro-v1.5".to_string(),
                license: "MIT".to_string(),
                use_case: "Embeddings for RAG".to_string(),
                context_length: 512,
                recommended: true,
            },
        );

        // Qwen 2.5 7B (Alternative)
        self.models.insert(
            "qwen2.5-7b".to_string(),
            ModelInfo {
                id: "qwen2.5-7b".to_string(),
                name: "Qwen 2.5 7B".to_string(),
                family: "qwen".to_string(),
                parameters: "7B".to_string(),
                quantizations: vec![QuantizedVariant {
                    quantization: Quantization::Q4,
                    filename: "qwen2.5-7b-instruct-q4_k_m.gguf".to_string(),
                    size_bytes: 4_500_000_000,
                    sha256: "placeholder".to_string(),
                    status: ModelStatus::Available,
                    local_path: None,
                }],
                hf_repo: "Qwen/Qwen2.5-7B-Instruct-GGUF".to_string(),
                license: "Apache 2.0".to_string(),
                use_case: "Alternative reasoning model".to_string(),
                context_length: 32768,
                recommended: false,
            },
        );
    }

    /// Get all models
    pub fn list(&self) -> Vec<&ModelInfo> {
        self.models.values().collect()
    }

    /// Get installed models
    pub fn list_installed(&self) -> Vec<&ModelInfo> {
        self.models
            .values()
            .filter(|m| {
                m.quantizations
                    .iter()
                    .any(|q| matches!(q.status, ModelStatus::Ready | ModelStatus::Loaded))
            })
            .collect()
    }

    /// Get model by ID
    pub fn get(&self, id: &str) -> Option<&ModelInfo> {
        self.models.get(id)
    }

    /// Get mutable model by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut ModelInfo> {
        self.models.get_mut(id)
    }

    /// Check if a model is installed
    pub fn is_installed(&self, id: &str, quant: Quantization) -> bool {
        self.models.get(id).is_some_and(|m| {
            m.quantizations.iter().any(|q| {
                q.quantization == quant
                    && matches!(q.status, ModelStatus::Ready | ModelStatus::Loaded)
            })
        })
    }

    /// Get the local path for a model
    pub fn get_path(&self, id: &str, quant: Quantization) -> Option<PathBuf> {
        self.models.get(id).and_then(|m| {
            m.quantizations
                .iter()
                .find(|q| q.quantization == quant)
                .and_then(|q| q.local_path.clone())
        })
    }

    /// Update model status
    #[instrument(skip(self))]
    pub fn set_status(&mut self, id: &str, quant: Quantization, status: ModelStatus) {
        if let Some(model) = self.models.get_mut(id) {
            if let Some(variant) = model
                .quantizations
                .iter_mut()
                .find(|q| q.quantization == quant)
            {
                debug!("Setting {} {} status to {:?}", id, quant, status);
                variant.status = status;
            }
        }
    }

    /// Set local path for a downloaded model
    pub fn set_local_path(&mut self, id: &str, quant: Quantization, path: PathBuf) {
        if let Some(model) = self.models.get_mut(id) {
            if let Some(variant) = model
                .quantizations
                .iter_mut()
                .find(|q| q.quantization == quant)
            {
                variant.local_path = Some(path);
                variant.status = ModelStatus::Ready;
            }
        }
    }

    /// Get recommended models for each agent
    pub fn get_recommended(&self) -> RecommendedModels {
        RecommendedModels {
            pathos: "phi-3-mini".to_string(),
            logos: "llama-3.2-8b".to_string(),
            ethos: "mistral-7b-instruct".to_string(),
            embeddings: "bge-micro-v1.5".to_string(),
        }
    }

    /// Calculate total size of recommended models
    pub fn recommended_download_size(&self, quant: Quantization) -> u64 {
        let recommended = self.get_recommended();
        let mut total = 0u64;

        for id in [
            &recommended.pathos,
            &recommended.logos,
            &recommended.ethos,
            &recommended.embeddings,
        ] {
            if let Some(model) = self.models.get(id) {
                if let Some(variant) = model.quantizations.iter().find(|q| q.quantization == quant)
                {
                    total += variant.size_bytes;
                } else if let Some(variant) = model.quantizations.first() {
                    total += variant.size_bytes;
                }
            }
        }

        total
    }

    /// Scan local directory for existing models
    #[instrument(skip(self))]
    pub fn scan_local(&mut self) -> ModelResult<usize> {
        info!("Scanning {} for existing models", self.models_dir.display());

        if !self.models_dir.exists() {
            return Ok(0);
        }

        let mut found = 0;

        for entry in std::fs::read_dir(&self.models_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "gguf") {
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    // Try to match with known models
                    for model in self.models.values_mut() {
                        for variant in &mut model.quantizations {
                            if variant.filename == filename {
                                variant.local_path = Some(path.clone());
                                variant.status = ModelStatus::Ready;
                                found += 1;
                                info!(
                                    "Found existing model: {} at {}",
                                    variant.filename,
                                    path.display()
                                );
                            }
                        }
                    }
                }
            }
        }

        Ok(found)
    }
}

/// Recommended models for the tripartite council
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedModels {
    pub pathos: String,
    pub logos: String,
    pub ethos: String,
    pub embeddings: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_registry_creation() {
        let dir = tempdir().unwrap();
        let registry = ModelRegistry::new(dir.path().to_path_buf());

        assert!(!registry.models.is_empty());
        assert!(registry.get("phi-3-mini").is_some());
        assert!(registry.get("llama-3.2-8b").is_some());
    }

    #[test]
    fn test_recommended_models() {
        let dir = tempdir().unwrap();
        let registry = ModelRegistry::new(dir.path().to_path_buf());

        let recommended = registry.get_recommended();
        assert_eq!(recommended.pathos, "phi-3-mini");
        assert_eq!(recommended.logos, "llama-3.2-8b");
    }

    #[test]
    fn test_status_update() {
        let dir = tempdir().unwrap();
        let mut registry = ModelRegistry::new(dir.path().to_path_buf());

        registry.set_status("phi-3-mini", Quantization::Q4, ModelStatus::Downloading);

        let model = registry.get("phi-3-mini").unwrap();
        assert_eq!(model.quantizations[0].status, ModelStatus::Downloading);
    }
}
