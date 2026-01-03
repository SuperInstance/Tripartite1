//! Model Downloader
//!
//! Downloads models from various sources (HuggingFace, direct URLs)
//! with progress tracking, checksum verification, and resume support.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, info, instrument, warn};

use crate::{ModelError, ModelResult};

/// Download progress callback
pub type ProgressCallback = Arc<dyn Fn(DownloadProgress) + Send + Sync>;

/// Download progress information
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    /// Current bytes downloaded
    pub downloaded: u64,
    /// Total bytes (if known)
    pub total: Option<u64>,
    /// Current download speed in bytes/sec
    pub speed: u64,
    /// Estimated time remaining in seconds
    pub eta: Option<u64>,
    /// Current phase
    pub phase: DownloadPhase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadPhase {
    /// Checking if file exists / getting size
    Checking,
    /// Downloading file
    Downloading,
    /// Verifying checksum
    Verifying,
    /// Extracting (if compressed)
    Extracting,
    /// Complete
    Complete,
}

/// Model source configuration
#[derive(Debug, Clone)]
pub enum ModelSource {
    /// HuggingFace Hub
    HuggingFace {
        repo_id: String,
        filename: String,
        revision: Option<String>,
    },
    /// Direct URL
    Url { url: String, filename: String },
    /// Local file (copy/link)
    Local { path: PathBuf },
}

impl std::fmt::Display for ModelSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelSource::HuggingFace {
                repo_id, filename, ..
            } => {
                write!(f, "{}/{}", repo_id, filename)
            },
            ModelSource::Url { url, .. } => {
                write!(f, "{}", url)
            },
            ModelSource::Local { path } => {
                write!(f, "{}", path.display())
            },
        }
    }
}

/// Model download specification
#[derive(Debug, Clone)]
pub struct DownloadSpec {
    /// Source of the model
    pub source: ModelSource,
    /// Expected SHA256 checksum
    pub sha256: Option<String>,
    /// Expected file size
    pub size_bytes: Option<u64>,
    /// Quantization level
    pub quantization: String,
}

/// Model downloader
pub struct Downloader {
    /// Base directory for models
    models_dir: PathBuf,
    /// HTTP client
    client: reqwest::Client,
    /// HuggingFace token (optional)
    hf_token: Option<String>,
}

impl Downloader {
    /// Create a new downloader
    pub fn new(models_dir: PathBuf) -> Self {
        Self {
            models_dir,
            client: reqwest::Client::builder()
                .user_agent("synesis-models/0.1.0")
                .build()
                .expect("Failed to create HTTP client"),
            hf_token: std::env::var("HF_TOKEN").ok(),
        }
    }

    /// Set HuggingFace token
    pub fn with_hf_token(mut self, token: String) -> Self {
        self.hf_token = Some(token);
        self
    }

    /// Download a model
    #[instrument(skip(self, progress_callback))]
    pub async fn download(
        &self,
        spec: &DownloadSpec,
        progress_callback: Option<ProgressCallback>,
    ) -> ModelResult<PathBuf> {
        info!("Starting model download");

        let dest_path = self.get_dest_path(spec)?;

        // Check if already exists and valid
        if dest_path.exists() {
            if let Some(expected) = &spec.sha256 {
                if self.verify_checksum(&dest_path, expected).await? {
                    info!("Model already exists and checksum matches");
                    if let Some(cb) = &progress_callback {
                        cb(DownloadProgress {
                            downloaded: spec.size_bytes.unwrap_or(0),
                            total: spec.size_bytes,
                            speed: 0,
                            eta: Some(0),
                            phase: DownloadPhase::Complete,
                        });
                    }
                    return Ok(dest_path);
                }
                warn!("Existing file has invalid checksum, re-downloading");
            } else {
                info!("Model already exists (no checksum to verify)");
                return Ok(dest_path);
            }
        }

        // Create parent directories
        if let Some(parent) = dest_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Download based on source
        match &spec.source {
            ModelSource::HuggingFace {
                repo_id,
                filename,
                revision,
            } => {
                self.download_from_huggingface(
                    repo_id,
                    filename,
                    revision.as_deref(),
                    &dest_path,
                    progress_callback.clone(),
                )
                .await?;
            },
            ModelSource::Url { url, .. } => {
                self.download_from_url(url, &dest_path, progress_callback.clone())
                    .await?;
            },
            ModelSource::Local { path } => {
                self.copy_local(path, &dest_path).await?;
            },
        }

        // Verify checksum if provided
        if let Some(expected) = &spec.sha256 {
            if let Some(cb) = &progress_callback {
                cb(DownloadProgress {
                    downloaded: spec.size_bytes.unwrap_or(0),
                    total: spec.size_bytes,
                    speed: 0,
                    eta: None,
                    phase: DownloadPhase::Verifying,
                });
            }

            if !self.verify_checksum(&dest_path, expected).await? {
                tokio::fs::remove_file(&dest_path).await?;
                return Err(ModelError::ChecksumMismatch {
                    model: spec.source.to_string(),
                    expected: expected.clone(),
                    actual: "invalid".to_string(),
                });
            }
        }

        if let Some(cb) = &progress_callback {
            cb(DownloadProgress {
                downloaded: spec.size_bytes.unwrap_or(0),
                total: spec.size_bytes,
                speed: 0,
                eta: Some(0),
                phase: DownloadPhase::Complete,
            });
        }

        Ok(dest_path)
    }

    /// Get destination path for a model
    fn get_dest_path(&self, spec: &DownloadSpec) -> ModelResult<PathBuf> {
        let filename = match &spec.source {
            ModelSource::HuggingFace { filename, .. } => filename.clone(),
            ModelSource::Url { filename, .. } => filename.clone(),
            ModelSource::Local { path } => path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .ok_or_else(|| ModelError::InvalidPath("No filename".to_string()))?,
        };

        Ok(self.models_dir.join(&filename))
    }

    /// Download from HuggingFace Hub
    async fn download_from_huggingface(
        &self,
        repo_id: &str,
        filename: &str,
        revision: Option<&str>,
        dest: &Path,
        progress_callback: Option<ProgressCallback>,
    ) -> ModelResult<()> {
        let revision = revision.unwrap_or("main");
        let url = format!(
            "https://huggingface.co/{}/resolve/{}/{}",
            repo_id, revision, filename
        );

        debug!("Downloading from HuggingFace: {}", url);
        self.download_from_url(&url, dest, progress_callback).await
    }

    /// Download from a URL
    async fn download_from_url(
        &self,
        url: &str,
        dest: &Path,
        progress_callback: Option<ProgressCallback>,
    ) -> ModelResult<()> {
        let mut request = self.client.get(url);

        // Add HuggingFace token if available and it's a HF URL
        if url.contains("huggingface.co") {
            if let Some(token) = &self.hf_token {
                request = request.header("Authorization", format!("Bearer {}", token));
            }
        }

        // Check for partial download (resume support)
        let temp_path = dest.with_extension("part");
        let resume_from = if temp_path.exists() {
            let metadata = tokio::fs::metadata(&temp_path).await?;
            let size = metadata.len();
            if size > 0 {
                request = request.header("Range", format!("bytes={}-", size));
                Some(size)
            } else {
                None
            }
        } else {
            None
        };

        let response = request.send().await?;

        if !response.status().is_success() && response.status().as_u16() != 206 {
            return Err(ModelError::DownloadFailed(format!(
                "HTTP {}: {}",
                response.status(),
                response.status().canonical_reason().unwrap_or("Unknown")
            )));
        }

        let total_size = response
            .content_length()
            .map(|cl| resume_from.unwrap_or(0) + cl);

        // Open file for writing
        let mut file = if resume_from.is_some() {
            tokio::fs::OpenOptions::new()
                .append(true)
                .open(&temp_path)
                .await?
        } else {
            tokio::fs::File::create(&temp_path).await?
        };

        let mut downloaded = resume_from.unwrap_or(0);
        let mut last_progress = std::time::Instant::now();
        let mut last_downloaded = downloaded;

        // Stream the response
        use futures_util::StreamExt;
        use tokio::io::AsyncWriteExt;

        let mut stream = response.bytes_stream();
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;

            // Update progress every 100ms
            let now = std::time::Instant::now();
            if now.duration_since(last_progress).as_millis() >= 100 {
                let elapsed = now.duration_since(last_progress).as_secs_f64();
                let bytes_since = downloaded - last_downloaded;
                let speed = (bytes_since as f64 / elapsed) as u64;

                let eta = total_size.map(|total| {
                    if speed > 0 {
                        (total - downloaded) / speed
                    } else {
                        0
                    }
                });

                if let Some(cb) = &progress_callback {
                    cb(DownloadProgress {
                        downloaded,
                        total: total_size,
                        speed,
                        eta,
                        phase: DownloadPhase::Downloading,
                    });
                }

                last_progress = now;
                last_downloaded = downloaded;
            }
        }

        file.flush().await?;
        drop(file);

        // Move temp file to final destination
        tokio::fs::rename(&temp_path, dest).await?;

        Ok(())
    }

    /// Copy a local file
    async fn copy_local(&self, src: &Path, dest: &Path) -> ModelResult<()> {
        tokio::fs::copy(src, dest).await?;
        Ok(())
    }

    /// Verify SHA256 checksum
    async fn verify_checksum(&self, path: &Path, expected: &str) -> ModelResult<bool> {
        use sha2::{Digest, Sha256};

        let mut file = tokio::fs::File::open(path).await?;
        let mut hasher = Sha256::new();

        let mut buffer = vec![0u8; 1024 * 1024]; // 1MB buffer
        loop {
            use tokio::io::AsyncReadExt;
            let n = file.read(&mut buffer).await?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }

        let result = hex::encode(hasher.finalize());
        Ok(result == expected.to_lowercase())
    }

    /// List downloaded models
    pub async fn list_downloaded(&self) -> ModelResult<Vec<PathBuf>> {
        let mut models = Vec::new();

        let mut entries = tokio::fs::read_dir(&self.models_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "gguf" || ext == "bin" || ext == "safetensors" {
                        models.push(path);
                    }
                }
            }
        }

        Ok(models)
    }

    /// Delete a downloaded model
    pub async fn delete(&self, model_name: &str) -> ModelResult<()> {
        let path = self.models_dir.join(model_name);
        if path.exists() {
            tokio::fs::remove_file(&path).await?;
            info!("Deleted model: {}", model_name);
        }
        Ok(())
    }

    /// Get the size of a downloaded model
    pub async fn get_size(&self, model_name: &str) -> ModelResult<u64> {
        let path = self.models_dir.join(model_name);
        let metadata = tokio::fs::metadata(&path).await?;
        Ok(metadata.len())
    }
}

/// Known model specifications
pub mod known_models {
    use super::*;

    /// Phi-3 Mini (Pathos agent)
    pub fn phi3_mini_q4() -> DownloadSpec {
        DownloadSpec {
            source: ModelSource::HuggingFace {
                repo_id: "microsoft/Phi-3-mini-4k-instruct-gguf".to_string(),
                filename: "Phi-3-mini-4k-instruct-q4.gguf".to_string(),
                revision: None,
            },
            sha256: None,                    // TODO: Add actual checksum
            size_bytes: Some(2_200_000_000), // ~2.2GB
            quantization: "q4".to_string(),
        }
    }

    /// Llama 3.2 8B (Logos agent)
    pub fn llama_3_2_8b_q4() -> DownloadSpec {
        DownloadSpec {
            source: ModelSource::HuggingFace {
                repo_id: "bartowski/Meta-Llama-3.2-8B-Instruct-GGUF".to_string(),
                filename: "Meta-Llama-3.2-8B-Instruct-Q4_K_M.gguf".to_string(),
                revision: None,
            },
            sha256: None,
            size_bytes: Some(4_700_000_000), // ~4.7GB
            quantization: "q4_k_m".to_string(),
        }
    }

    /// Mistral 7B Instruct (Ethos agent)
    pub fn mistral_7b_instruct_q4() -> DownloadSpec {
        DownloadSpec {
            source: ModelSource::HuggingFace {
                repo_id: "TheBloke/Mistral-7B-Instruct-v0.2-GGUF".to_string(),
                filename: "mistral-7b-instruct-v0.2.Q4_K_M.gguf".to_string(),
                revision: None,
            },
            sha256: None,
            size_bytes: Some(4_100_000_000), // ~4.1GB
            quantization: "q4_k_m".to_string(),
        }
    }

    /// BGE Micro (embeddings)
    pub fn bge_micro() -> DownloadSpec {
        DownloadSpec {
            source: ModelSource::HuggingFace {
                repo_id: "TaylorAI/bge-micro-v2".to_string(),
                filename: "model.safetensors".to_string(),
                revision: None,
            },
            sha256: None,
            size_bytes: Some(50_000_000), // ~50MB
            quantization: "f32".to_string(),
        }
    }

    /// Get all recommended models
    pub fn recommended_models() -> Vec<(&'static str, DownloadSpec)> {
        vec![
            ("phi-3-mini", phi3_mini_q4()),
            ("llama-3.2-8b", llama_3_2_8b_q4()),
            ("mistral-7b-instruct", mistral_7b_instruct_q4()),
            ("bge-micro", bge_micro()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_spec_creation() {
        let spec = known_models::phi3_mini_q4();
        assert_eq!(spec.quantization, "q4");
    }

    #[tokio::test]
    async fn test_dest_path_calculation() {
        let downloader = Downloader::new(PathBuf::from("/tmp/models"));
        let spec = known_models::phi3_mini_q4();
        let path = downloader.get_dest_path(&spec).unwrap();
        assert!(path.to_string_lossy().contains("Phi-3-mini"));
    }
}
