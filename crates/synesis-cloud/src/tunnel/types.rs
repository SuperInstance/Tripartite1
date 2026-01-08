//! Tunnel types and configuration

use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Configuration for QUIC tunnel connection
#[derive(Debug, Clone)]
pub struct TunnelConfig {
    /// Cloud endpoint URL (e.g., "https://tunnel.superinstance.ai:443")
    pub cloud_url: String,

    /// Unique device identifier (generated on init)
    pub device_id: String,

    /// Path to device certificate (PEM format)
    pub cert_path: PathBuf,

    /// Path to device private key (PEM format)
    pub key_path: PathBuf,

    /// Interval between heartbeat messages
    pub heartbeat_interval: Duration,

    /// Delay before reconnection attempt
    pub reconnect_delay: Duration,

    /// Maximum reconnection attempts before giving up
    pub max_reconnect_attempts: u32,

    /// Connection timeout
    pub connect_timeout: Duration,

    /// Read timeout for responses
    pub read_timeout: Duration,
}

impl Default for TunnelConfig {
    fn default() -> Self {
        Self {
            cloud_url: "https://tunnel.superinstance.ai:443".to_string(),
            device_id: String::new(),
            cert_path: PathBuf::new(),
            key_path: PathBuf::new(),
            heartbeat_interval: Duration::from_secs(30),
            reconnect_delay: Duration::from_secs(5),
            max_reconnect_attempts: 10,
            connect_timeout: Duration::from_secs(30),
            read_timeout: Duration::from_secs(60),
        }
    }
}

/// Current state of the tunnel connection
#[derive(Debug, Clone, PartialEq)]
pub enum TunnelState {
    /// Not connected to cloud
    Disconnected,

    /// Attempting to establish connection
    Connecting {
        since: Instant,
    },

    /// Connected and healthy
    Connected {
        since: Instant,
        latency_ms: u32,
    },

    /// Connection lost, attempting to reconnect
    Reconnecting {
        attempt: u32,
        last_error: String,
    },

    /// Connection failed permanently
    Failed {
        error: String,
        at: Instant,
    },
}

impl TunnelState {
    /// Check if currently connected
    pub fn is_connected(&self) -> bool {
        matches!(self, TunnelState::Connected { .. })
    }

    /// Check if connection is healthy (connected and low latency)
    pub fn is_healthy(&self) -> bool {
        match self {
            TunnelState::Connected { latency_ms, .. } => *latency_ms < 500,
            _ => false,
        }
    }
}

/// Statistics for tunnel connection
#[derive(Debug, Clone, Default)]
pub struct TunnelStats {
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
    pub heartbeats_sent: u64,
    pub heartbeats_acked: u64,
    pub requests_sent: u64,
    pub requests_succeeded: u64,
    pub requests_failed: u64,
    pub reconnections: u32,
    pub avg_latency_ms: u32,
}

impl TunnelStats {
    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        if self.requests_sent == 0 {
            return 1.0;
        }
        self.requests_succeeded as f64 / self.requests_sent as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tunnel_config_defaults() {
        let config = TunnelConfig::default();
        assert_eq!(config.heartbeat_interval, Duration::from_secs(30));
        assert_eq!(config.max_reconnect_attempts, 10);
        assert_eq!(config.connect_timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_tunnel_state_transitions() {
        let mut state = TunnelState::Disconnected;

        // Disconnected -> Connecting
        state = TunnelState::Connecting {
            since: Instant::now(),
        };
        assert!(matches!(state, TunnelState::Connecting { .. }));

        // Connecting -> Connected
        state = TunnelState::Connected {
            since: Instant::now(),
            latency_ms: 50,
        };
        assert!(matches!(state, TunnelState::Connected { .. }));
        assert!(state.is_connected());
        assert!(state.is_healthy());
    }

    #[test]
    fn test_tunnel_stats_accumulation() {
        let mut stats = TunnelStats::default();

        stats.total_bytes_sent += 1024;
        stats.heartbeats_sent += 1;
        stats.requests_sent += 1;
        stats.requests_succeeded += 1;

        assert_eq!(stats.total_bytes_sent, 1024);
        assert_eq!(stats.heartbeats_sent, 1);
        assert_eq!(stats.success_rate(), 1.0);
    }
}
