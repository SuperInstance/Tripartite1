//! Heartbeat service for keeping tunnel alive

use crate::error::{CloudError, CloudResult};
use crate::telemetry::collect_device_vitals;
use crate::telemetry::types::{DeviceVitals, Heartbeat, HeartbeatAck, ServerStatus};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::sync::RwLock;

/// Pre-warm signal sent when GPU is under stress
#[derive(Debug, Clone)]
pub struct PrewarmSignal {
    /// Device identifier
    pub device_id: String,
    /// When the signal was generated
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Current GPU utilization (0.0 - 1.0)
    pub gpu_usage: f32,
    /// Current GPU temperature in Celsius (if available)
    pub gpu_temp: Option<f32>,
    /// Human-readable reason for pre-warm
    pub reason: String,
}

/// Heartbeat service configuration
#[derive(Debug, Clone)]
pub struct HeartbeatConfig {
    /// Interval between heartbeat messages
    pub interval: Duration,
    /// Timeout for heartbeat acknowledgments
    pub timeout: Duration,
    /// GPU usage threshold (0.0 - 1.0) for sending pre-warm signals
    pub gpu_prewarm_threshold: f32,
    /// Device identifier
    pub device_id: String,
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(10),
            gpu_prewarm_threshold: 0.8, // 80%
            device_id: "device-unknown".to_string(),
        }
    }
}

/// Callback for pre-warm signals
pub type PrewarmCallback = Arc<dyn Fn(PrewarmSignal) + Send + Sync>;

/// Heartbeat service
///
/// Sends periodic heartbeats to the cloud server with real device vitals
pub struct HeartbeatService {
    config: HeartbeatConfig,
    sequence: Arc<AtomicU64>,
    connection: Arc<RwLock<Option<quinn::Connection>>>,
    shutdown: broadcast::Sender<()>,
    prewarm_callback: Arc<RwLock<Option<PrewarmCallback>>>,
    last_prewarm: Arc<RwLock<Option<chrono::DateTime<chrono::Utc>>>>,
}

impl HeartbeatService {
    /// Create a new heartbeat service
    pub fn new(config: HeartbeatConfig) -> Self {
        let (shutdown, _) = broadcast::channel(1);
        Self {
            config,
            sequence: Arc::new(AtomicU64::new(0)),
            connection: Arc::new(RwLock::new(None)),
            shutdown,
            prewarm_callback: Arc::new(RwLock::new(None)),
            last_prewarm: Arc::new(RwLock::new(None)),
        }
    }

    /// Set the active connection
    pub async fn set_connection(&self, conn: quinn::Connection) {
        let mut connection = self.connection.write().await;
        *connection = Some(conn);
    }

    /// Clear the connection (on disconnect)
    pub async fn clear_connection(&self) {
        let mut connection = self.connection.write().await;
        *connection = None;
    }

    /// Set callback for pre-warm signals
    pub async fn set_prewarm_callback(&self, callback: PrewarmCallback) {
        let mut prewarm_cb = self.prewarm_callback.write().await;
        *prewarm_cb = Some(callback);
    }

    /// Spawn the heartbeat task
    ///
    /// Returns a JoinHandle that can be used to wait for shutdown
    pub fn spawn(&self) -> tokio::task::JoinHandle<()> {
        let interval = self.config.interval;
        let sequence = self.sequence.clone();
        let connection_lock = self.connection.clone();
        let device_id = self.config.device_id.clone();
        let gpu_threshold = self.config.gpu_prewarm_threshold;
        let prewarm_callback = self.prewarm_callback.clone();
        let last_prewarm = self.last_prewarm.clone();
        let mut shutdown_rx = self.shutdown.subscribe();

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                tokio::select! {
                    _ = interval_timer.tick() => {
                        let conn_opt = connection_lock.read().await;
                        if let Some(conn) = conn_opt.as_ref() {
                            let seq = sequence.fetch_add(1, Ordering::SeqCst);

                            // Collect real device vitals
                            let vitals = collect_device_vitals(device_id.clone());

                            // Check for GPU stress and send pre-warm signal
                            if let Some(gpu_usage) = vitals.gpu_usage {
                                if gpu_usage > gpu_threshold {
                                    // Rate limit pre-warm signals (max once per minute)
                                    let should_send = {
                                        let last = last_prewarm.read().await;
                                        match *last {
                                            Some(last_time) => {
                                                let elapsed = chrono::Utc::now().signed_duration_since(last_time);
                                                elapsed.num_seconds() > 60
                                            }
                                            None => true,
                                        }
                                    };

                                    if should_send {
                                        tracing::warn!(
                                            "GPU stress detected: {:.1}% usage, sending pre-warm signal",
                                            gpu_usage * 100.0
                                        );

                                        let signal = PrewarmSignal {
                                            device_id: vitals.device_id.clone(),
                                            timestamp: vitals.timestamp,
                                            gpu_usage,
                                            gpu_temp: vitals.gpu_temp_celsius,
                                            reason: format!("GPU usage ({:.1}%) exceeds threshold ({:.1}%)",
                                                gpu_usage * 100.0,
                                                gpu_threshold * 100.0),
                                        };

                                        // Trigger callback if set
                                        let callback = prewarm_callback.read().await;
                                        if let Some(ref cb) = *callback {
                                            cb(signal.clone());
                                        }

                                        // Update last pre-warm time
                                        let mut last = last_prewarm.write().await;
                                        *last = Some(vitals.timestamp);
                                    }
                                }
                            }

                            // Send heartbeat
                            if let Err(e) = Self::send_heartbeat(conn, seq, vitals).await {
                                tracing::warn!("Heartbeat failed: {}", e);
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        tracing::info!("Heartbeat service shutting down");
                        break;
                    }
                }
            }
        })
    }

    /// Send a single heartbeat with real vitals
    async fn send_heartbeat(
        conn: &quinn::Connection,
        sequence: u64,
        vitals: DeviceVitals,
    ) -> CloudResult<HeartbeatAck> {
        let heartbeat = Heartbeat {
            device_id: vitals.device_id.clone(),
            timestamp: vitals.timestamp.timestamp_millis(),
            sequence,
            vitals,
        };

        // Serialize
        let data = serde_json::to_vec(&heartbeat)
            .map_err(CloudError::Serialization)?;

        // Send on unidirectional stream
        let mut send = conn.open_uni().await
            .map_err(|e| CloudError::tunnel_connection(format!("Failed to open stream: {}", e)))?;

        // Message type: Heartbeat (0x01)
        send.write_all(&[0x01]).await
            .map_err(|e| CloudError::tunnel_connection(format!("Failed to write type: {}", e)))?;

        // Length (4 bytes big-endian)
        let len = data.len() as u32;
        send.write_all(&len.to_be_bytes()).await
            .map_err(|e| CloudError::tunnel_connection(format!("Failed to write length: {}", e)))?;

        // Payload
        send.write_all(&data).await
            .map_err(|e| CloudError::tunnel_connection(format!("Failed to write payload: {}", e)))?;

        send.finish().await
            .map_err(|e| CloudError::tunnel_connection(format!("Failed to finish stream: {}", e)))?;

        tracing::trace!("Heartbeat sent: seq={}", sequence);

        // TODO: Wait for actual ACK from server
        // For now, return a mock ACK
        // In production, we would:
        // 1. Open a bidirectional stream
        // 2. Send heartbeat
        // 3. Wait for ACK response
        // 4. Parse ACK and return it
        Ok(HeartbeatAck {
            server_time: chrono::Utc::now(),
            latency_ms: 50,
            pending_messages: 0,
            server_status: ServerStatus::Healthy,
        })
    }

    /// Shutdown the heartbeat service
    pub fn shutdown(&self) {
        let _ = self.shutdown.send(());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_heartbeat_config_defaults() {
        let config = HeartbeatConfig::default();
        assert_eq!(config.interval, Duration::from_secs(30));
        assert_eq!(config.timeout, Duration::from_secs(10));
        assert_eq!(config.gpu_prewarm_threshold, 0.8);
    }

    #[test]
    fn test_heartbeat_service_creation() {
        let service = HeartbeatService::new(HeartbeatConfig::default());
        assert_eq!(service.sequence.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_prewarm_signal_creation() {
        let signal = PrewarmSignal {
            device_id: "test-device".to_string(),
            timestamp: chrono::Utc::now(),
            gpu_usage: 0.85,
            gpu_temp: Some(75.0),
            reason: "GPU stress".to_string(),
        };

        assert_eq!(signal.device_id, "test-device");
        assert_eq!(signal.gpu_usage, 0.85);
        assert_eq!(signal.gpu_temp, Some(75.0));
        assert_eq!(signal.reason, "GPU stress");
    }

    #[tokio::test]
    async fn test_prewarm_callback() {
        let service = HeartbeatService::new(HeartbeatConfig::default());

        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();

        let callback: PrewarmCallback = Arc::new(move |_signal| {
            called_clone.store(true, Ordering::SeqCst);
        });

        service.set_prewarm_callback(callback).await;

        // Simulate pre-warm signal
        let prewarm_cb = service.prewarm_callback.read().await;
        if let Some(ref cb) = *prewarm_cb {
            let signal = PrewarmSignal {
                device_id: "test".to_string(),
                timestamp: chrono::Utc::now(),
                gpu_usage: 0.9,
                gpu_temp: Some(80.0),
                reason: "Test".to_string(),
            };
            cb(signal);
        }

        // Give it a moment
        tokio::time::sleep(Duration::from_millis(10)).await;

        assert!(called.load(Ordering::SeqCst));
    }
}
