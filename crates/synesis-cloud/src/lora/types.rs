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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoraStatus {
    Uploading,
    Processing,
    Ready,
    Error { message: String },
}

/// LoRA upload progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadProgress {
    pub upload_id: String,
    pub total_bytes: u64,
    pub uploaded_bytes: u64,
    pub chunks_total: u32,
    pub chunks_uploaded: u32,
    pub status: UploadStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UploadStatus {
    InProgress,
    Completed,
    Failed { error: String },
}
