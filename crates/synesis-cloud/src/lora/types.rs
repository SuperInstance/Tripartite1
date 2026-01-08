//! LoRA types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Local LoRA information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalLora {
    /// Unique local identifier
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Base model this LoRA was trained for
    pub base_model: String,

    /// Path to LoRA file
    pub path: PathBuf,

    /// File size in bytes
    pub size_bytes: u64,

    /// SHA256 checksum
    pub checksum: String,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Whether uploaded to cloud
    pub uploaded: bool,

    /// Cloud ID (if uploaded)
    pub cloud_id: Option<String>,

    /// Last upload timestamp
    pub uploaded_at: Option<DateTime<Utc>>,
}

/// Cloud LoRA information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudLora {
    /// Cloud identifier
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Base model
    pub base_model: String,

    /// File size in bytes
    pub size_bytes: u64,

    /// Upload timestamp
    pub uploaded_at: DateTime<Utc>,

    /// Last used timestamp
    pub last_used: Option<DateTime<Utc>>,

    /// Total usage count
    pub usage_count: u64,

    /// Regions where available
    pub regions: Vec<String>,

    /// Status
    pub status: LoraStatus,
}

/// LoRA processing status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoraStatus {
    /// Currently uploading
    Uploading,
    /// Processing and validating
    Processing,
    /// Ready to use
    Ready,
    /// Error during processing
    Error {
        /// Error message
        message: String,
    },
}

/// LoRA upload progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadProgress {
    /// Unique upload identifier
    pub upload_id: String,
    /// Total file size in bytes
    pub total_bytes: u64,
    /// Bytes uploaded so far
    pub uploaded_bytes: u64,
    /// Total number of chunks
    pub chunks_total: u32,
    /// Chunks uploaded so far
    pub chunks_uploaded: u32,
    /// Current upload status
    pub status: UploadStatus,
}

/// Upload status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UploadStatus {
    /// Upload in progress
    InProgress,
    /// Upload completed successfully
    Completed,
    /// Upload failed
    Failed {
        /// Error message
        error: String,
    },
}
