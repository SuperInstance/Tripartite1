//! Telemetry types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Device vitals sent with each heartbeat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceVitals {
    /// Device identifier
    pub device_id: String,

    /// Timestamp of collection
    pub timestamp: DateTime<Utc>,

    /// CPU usage percentage (0.0 - 1.0)
    pub cpu_usage: f32,

    /// Memory usage percentage (0.0 - 1.0)
    pub memory_usage: f32,

    /// GPU usage percentage (0.0 - 1.0), if GPU present
    pub gpu_usage: Option<f32>,

    /// GPU temperature in Celsius, if GPU present
    pub gpu_temp_celsius: Option<f32>,

    /// GPU VRAM usage percentage (0.0 - 1.0), if GPU present
    pub gpu_vram_usage: Option<f32>,

    /// Disk usage percentage (0.0 - 1.0)
    pub disk_usage: f32,

    /// Number of active sessions
    pub active_sessions: u32,

    /// Number of queries in processing queue
    pub pending_queries: u32,

    /// Current model loaded (if any)
    pub loaded_model: Option<String>,
}

/// Heartbeat message sent to cloud
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heartbeat {
    pub device_id: String,
    pub timestamp: i64,
    pub sequence: u64,
    pub vitals: DeviceVitals,
}

/// Heartbeat acknowledgment from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatAck {
    /// Server timestamp
    pub server_time: DateTime<Utc>,

    /// Round-trip latency in milliseconds
    pub latency_ms: u32,

    /// Any pending messages for client
    pub pending_messages: u32,

    /// Server status
    pub server_status: ServerStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerStatus {
    Healthy,
    Degraded { reason: String },
    Maintenance { until: DateTime<Utc> },
}
