//! Main CloudTunnel implementation

use super::r#types::{TunnelConfig, TunnelState, TunnelStats};
use super::state::ConnectionStateMachine;
use super::heartbeat::{HeartbeatService, HeartbeatConfig};
use super::endpoint::{create_endpoint, connect_to_cloud};
use crate::error::{CloudError, CloudResult};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Main cloud tunnel struct
///
/// Provides a persistent QUIC tunnel with mTLS authentication,
/// automatic heartbeat, and reconnection logic.
pub struct CloudTunnel {
    config: TunnelConfig,
    endpoint: Option<quinn::Endpoint>,
    connection: Arc<RwLock<Option<quinn::Connection>>>,
    state_machine: ConnectionStateMachine,
    heartbeat_service: Option<HeartbeatService>,
    stats: Arc<RwLock<TunnelStats>>,
}

impl CloudTunnel {
    /// Create a new tunnel instance
    ///
    /// # Arguments
    /// * `config` - Tunnel configuration
    ///
    /// # Returns
    /// * Configured tunnel (not yet connected)
    pub fn new(config: TunnelConfig) -> CloudResult<Self> {
        if config.cert_path.as_path().as_os_str().is_empty() {
            return Err(CloudError::validation("Certificate path is required"));
        }
        if config.key_path.as_path().as_os_str().is_empty() {
            return Err(CloudError::validation("Key path is required"));
        }

        Ok(Self {
            config,
            endpoint: None,
            connection: Arc::new(RwLock::new(None)),
            state_machine: ConnectionStateMachine::new(),
            heartbeat_service: None,
            stats: Arc::new(RwLock::new(TunnelStats::default())),
        })
    }

    /// Connect to cloud
    ///
    /// # Arguments
    /// * `cloud_url` - Override cloud URL (optional)
    ///
    /// # Returns
    /// * Ok(()) if connection successful
    /// * Err if connection fails
    pub async fn connect(&mut self) -> CloudResult<()> {
        self.state_machine.transition(TunnelState::Connecting {
            since: Instant::now(),
        });

        match self.connect_internal().await {
            Ok(()) => {
                // Start heartbeat
                if let Some(ref heartbeat_service) = self.heartbeat_service {
                    heartbeat_service.spawn();
                }

                // Start reconnection monitor
                // TODO: Implement in next iteration

                tracing::info!("Tunnel connected successfully");
                Ok(())
            }
            Err(e) => {
                self.state_machine.transition(TunnelState::Failed {
                    error: e.to_string(),
                    at: Instant::now(),
                });
                Err(e)
            }
        }
    }

    /// Internal connection logic
    async fn connect_internal(&mut self) -> CloudResult<()> {
        // Create endpoint if not exists
        if self.endpoint.is_none() {
            self.endpoint = Some(create_endpoint(
                &self.config.cert_path,
                &self.config.key_path,
            )?);
        }

        // Connect to cloud
        let cloud_url = self.config.cloud_url.clone();
        let server_name = extract_server_name(&cloud_url)?;

        let conn = connect_to_cloud(
            self.endpoint.as_ref().unwrap(),
            &cloud_url,
            &server_name,
        ).await?;

        // Store connection
        *self.connection.write().await = Some(conn.clone());

        // Create and start heartbeat service
        let heartbeat_service = HeartbeatService::new(HeartbeatConfig::default());
        heartbeat_service.set_connection(conn.clone()).await;
        self.heartbeat_service = Some(heartbeat_service);

        // Transition to connected
        self.state_machine.transition(TunnelState::Connected {
            since: Instant::now(),
            latency_ms: 0, // Will be updated by first heartbeat
        });

        Ok(())
    }

    /// Disconnect from cloud
    pub async fn disconnect(&mut self) -> CloudResult<()> {
        if let Some(ref heartbeat_service) = self.heartbeat_service {
            heartbeat_service.shutdown();
            heartbeat_service.clear_connection().await;
        }

        if let Some(ref conn) = self.connection.write().await.take() {
            conn.close(0u32.into(), b"client disconnect");
        }

        self.state_machine.transition(TunnelState::Disconnected);

        tracing::info!("Tunnel disconnected");
        Ok(())
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.state_machine.current().is_connected()
    }

    /// Get current state
    pub fn state(&self) -> TunnelState {
        self.state_machine.current()
    }

    /// Get tunnel statistics
    pub async fn stats(&self) -> TunnelStats {
        self.stats.read().await.clone()
    }

    /// Send request and receive response (bidirectional stream)
    pub async fn request(&self, data: &[u8]) -> CloudResult<Vec<u8>> {
        let conn = self.connection.read().await
            .as_ref()
            .ok_or(CloudError::NotConnected)?
            .clone();

        let (mut send, mut recv) = conn.open_bi().await
            .map_err(|e| CloudError::tunnel_connection(format!("Failed to open stream: {}", e)))?;

        // Send request
        send.write_all(data).await
            .map_err(|e| CloudError::tunnel_connection(format!("Failed to write: {}", e)))?;
        send.finish().await
            .map_err(|e| CloudError::tunnel_connection(format!("Failed to finish: {}", e)))?;

        // Receive response
        let response = recv.read_to_end(10 * 1024 * 1024).await // 10MB max
            .map_err(|e| CloudError::tunnel_connection(format!("Failed to read: {}", e)))?;

        // Update stats
        let mut stats = self.stats.write().await;
        stats.total_bytes_sent += data.len() as u64;
        stats.total_bytes_received += response.len() as u64;
        stats.requests_sent += 1;
        stats.requests_succeeded += 1;

        Ok(response)
    }
}

/// Extract server name from URL
fn extract_server_name(url: &str) -> CloudResult<String> {
    let parsed = url::Url::parse(url)
        .map_err(|e| CloudError::validation(format!("Invalid URL: {}", e)))?;

    parsed.host_str()
        .map(|s| s.to_string())
        .ok_or_else(|| CloudError::validation("No host in URL"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_extract_server_name() {
        assert_eq!(
            extract_server_name("https://tunnel.superinstance.ai:443").unwrap(),
            "tunnel.superinstance.ai"
        );
        assert_eq!(
            extract_server_name("https://api.example.com").unwrap(),
            "api.example.com"
        );
    }

    #[test]
    fn test_tunnel_creation() {
        let config = TunnelConfig {
            cert_path: "/tmp/cert.pem".into(),
            key_path: "/tmp/key.pem".into(),
            ..Default::default()
        };

        let tunnel = CloudTunnel::new(config);
        assert!(tunnel.is_ok());
        let tunnel = tunnel.unwrap();

        assert!(!tunnel.is_connected());
        assert!(matches!(tunnel.state(), TunnelState::Disconnected));
    }

    #[test]
    fn test_tunnel_validation() {
        let config = TunnelConfig {
            cert_path: PathBuf::new(), // Empty
            key_path: PathBuf::new(), // Empty
            ..Default::default()
        };

        let result = CloudTunnel::new(config);
        assert!(result.is_err());
    }
}
