//! SuperInstance Models - Model Management & Inference
//!
//! This crate handles:
//! - Hardware detection (CPU, GPU, RAM)
//! - Model downloads from HuggingFace
//! - Model registry and versioning
//! - Inference execution (via llama.cpp bindings)
//! - Hardware manifests for optimal model selection

pub mod downloader;
pub mod hardware;
pub mod inference;
pub mod manifest;
pub mod registry;

// Re-exports
pub use downloader::{DownloadProgress, Downloader as ModelDownloader};
pub use hardware::{GpuInfo, HardwareDetector, HardwareInfo};
pub use inference::{InferenceRequest, InferenceResponse, ModelInstance, ModelPool};
pub use manifest::{HardwareManifest, ModelRecommendation};
pub use registry::{ModelInfo, ModelRegistry, ModelStatus};

/// Result type for model operations
pub type ModelResult<T> = std::result::Result<T, ModelError>;

/// Model error types
#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("Model not found: {0}")]
    NotFound(String),

    #[error("Model not loaded: {0}")]
    NotLoaded(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Download failed: {0}")]
    DownloadFailed(String),

    #[error("Checksum mismatch for {model}: expected {expected}, got {actual}")]
    ChecksumMismatch {
        model: String,
        expected: String,
        actual: String,
    },

    #[error("Insufficient resources: {0}")]
    InsufficientResources(String),

    #[error("Inference error: {0}")]
    InferenceError(String),

    #[error("Hardware detection failed: {0}")]
    HardwareError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Quantization levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Quantization {
    /// 4-bit quantization (smallest, fastest)
    Q4,
    /// 5-bit quantization
    Q5,
    /// 8-bit quantization
    Q8,
    /// 16-bit float (largest, most accurate)
    F16,
}

impl Quantization {
    /// Get file suffix for this quantization
    pub fn suffix(&self) -> &'static str {
        match self {
            Quantization::Q4 => "q4_k_m",
            Quantization::Q5 => "q5_k_m",
            Quantization::Q8 => "q8_0",
            Quantization::F16 => "f16",
        }
    }

    /// Approximate size multiplier relative to F16
    pub fn size_multiplier(&self) -> f32 {
        match self {
            Quantization::Q4 => 0.25,
            Quantization::Q5 => 0.31,
            Quantization::Q8 => 0.5,
            Quantization::F16 => 1.0,
        }
    }
}

impl std::fmt::Display for Quantization {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.suffix())
    }
}

impl std::str::FromStr for Quantization {
    type Err = ModelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "q4" | "q4_k_m" => Ok(Quantization::Q4),
            "q5" | "q5_k_m" => Ok(Quantization::Q5),
            "q8" | "q8_0" => Ok(Quantization::Q8),
            "f16" => Ok(Quantization::F16),
            _ => Err(ModelError::Internal(format!("Unknown quantization: {}", s))),
        }
    }
}
