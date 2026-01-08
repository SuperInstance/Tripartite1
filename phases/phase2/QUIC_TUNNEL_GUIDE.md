# QUIC Tunnel Implementation Guide

**Version**: 2.0.0
**Crate**: synesis-cloud
**Last Updated**: 2026-01-02

---

## Overview

The QUIC tunnel provides a persistent, low-latency connection between local SuperInstance clients and the Cloudflare edge network. It uses the `quinn` library for QUIC implementation with `rustls` for TLS 1.3.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           LOCAL CLIENT                                   │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │                        CloudTunnel                                 │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐   │  │
│  │  │   quinn     │  │   rustls    │  │    State Machine         │   │  │
│  │  │  Endpoint   │  │   Config    │  │  (Disconnected/Connected │   │  │
│  │  │             │  │  (mTLS)     │  │   /Reconnecting/Failed)  │   │  │
│  │  └──────┬──────┘  └──────┬──────┘  └──────────────────────────┘   │  │
│  │         │                │                                         │  │
│  │         └────────────────┴───────────────────────────────────────┐│  │
│  │                                                                   ││  │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  ││  │
│  │  │ Heartbeat Task  │  │  Reconnect Task │  │  Stream Manager │  ││  │
│  │  │ (30s interval)  │  │ (exp. backoff)  │  │  (bidirectional)│  ││  │
│  │  └─────────────────┘  └─────────────────┘  └─────────────────┘  ││  │
│  └──────────────────────────────────────────────────────────────────┘│  │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                            QUIC/TLS 1.3
                            (UDP port 443)
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                      CLOUDFLARE EDGE                                     │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │                    QUIC Termination                                │  │
│  │  - Certificate validation                                          │  │
│  │  - Connection pooling                                              │  │
│  │  - Load balancing across Workers                                   │  │
│  └───────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Dependencies

```toml
# Cargo.toml
[dependencies]
quinn = "0.10"
rustls = { version = "0.21", features = ["dangerous_configuration"] }
tokio = { version = "1", features = ["full"] }
webpki-roots = "0.25"
rcgen = "0.11"  # For certificate generation
```

---

## Implementation

### 1. TLS Configuration

```rust
// src/tunnel/tls.rs

use rustls::{Certificate, PrivateKey, ClientConfig, RootCertStore};
use std::path::Path;
use std::sync::Arc;

/// Create TLS configuration for mTLS connection
pub fn create_tls_config(
    cert_path: &Path,
    key_path: &Path,
) -> Result<Arc<ClientConfig>, TlsError> {
    // Load device certificate
    let cert_pem = std::fs::read(cert_path)?;
    let certs = rustls_pemfile::certs(&mut cert_pem.as_slice())?
        .into_iter()
        .map(Certificate)
        .collect::<Vec<_>>();
    
    // Load device private key
    let key_pem = std::fs::read(key_path)?;
    let key = rustls_pemfile::private_key(&mut key_pem.as_slice())?
        .ok_or(TlsError::NoPrivateKey)?;
    let key = PrivateKey(key.secret_der().to_vec());
    
    // Build root certificate store with system CAs
    let mut roots = RootCertStore::empty();
    roots.add_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.iter().map(|ta| {
        rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject.as_ref(),
            ta.subject_public_key_info.as_ref(),
            ta.name_constraints.as_ref().map(|nc| nc.as_ref()),
        )
    }));
    
    // Build client config with mTLS
    let config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(roots)
        .with_client_auth_cert(certs, key)?;
    
    Ok(Arc::new(config))
}

/// Generate device certificate (called during `synesis init`)
pub fn generate_device_certificate(
    device_id: &str,
) -> Result<(Certificate, PrivateKey), CertError> {
    use rcgen::{Certificate as RcgenCert, CertificateParams, DnType, KeyPair};
    
    // Generate key pair
    let key_pair = KeyPair::generate(&rcgen::PKCS_ECDSA_P256_SHA256)?;
    
    // Build certificate parameters
    let mut params = CertificateParams::default();
    params.distinguished_name.push(DnType::CommonName, format!("device-{}", device_id));
    params.distinguished_name.push(DnType::OrganizationName, "SuperInstance");
    params.not_before = time::OffsetDateTime::now_utc();
    params.not_after = time::OffsetDateTime::now_utc() + time::Duration::days(365);
    params.key_pair = Some(key_pair);
    
    // Extended key usage for client auth
    params.extended_key_usages = vec![
        rcgen::ExtendedKeyUsagePurpose::ClientAuth,
    ];
    
    let cert = RcgenCert::from_params(params)?;
    
    // Convert to rustls types
    let cert_der = cert.serialize_der()?;
    let key_der = cert.serialize_private_key_der();
    
    Ok((
        Certificate(cert_der),
        PrivateKey(key_der),
    ))
}
```

### 2. QUIC Endpoint

```rust
// src/tunnel/endpoint.rs

use quinn::{Endpoint, ClientConfig as QuinnClientConfig, Connection};
use std::net::SocketAddr;
use std::sync::Arc;

/// Create QUIC endpoint for client connections
pub fn create_endpoint(
    tls_config: Arc<rustls::ClientConfig>,
) -> Result<Endpoint, EndpointError> {
    // Configure QUIC transport
    let mut transport = quinn::TransportConfig::default();
    transport.keep_alive_interval(Some(std::time::Duration::from_secs(10)));
    transport.max_idle_timeout(Some(std::time::Duration::from_secs(60).try_into()?));
    
    // Build QUIC client config
    let mut client_config = QuinnClientConfig::new(Arc::new(tls_config));
    client_config.transport_config(Arc::new(transport));
    
    // Bind to random local port
    let bind_addr: SocketAddr = "0.0.0.0:0".parse()?;
    let mut endpoint = Endpoint::client(bind_addr)?;
    endpoint.set_default_client_config(client_config);
    
    Ok(endpoint)
}

/// Connect to cloud endpoint
pub async fn connect_to_cloud(
    endpoint: &Endpoint,
    cloud_url: &str,
    server_name: &str,
) -> Result<Connection, ConnectionError> {
    let addr: SocketAddr = resolve_dns(cloud_url).await?;
    
    let connection = endpoint
        .connect(addr, server_name)?
        .await?;
    
    tracing::info!(
        "Connected to cloud: addr={}, protocol={:?}",
        addr,
        connection.handshake_data()
            .and_then(|hd| hd.downcast::<quinn::crypto::rustls::HandshakeData>().ok())
            .map(|hd| hd.protocol)
    );
    
    Ok(connection)
}

async fn resolve_dns(url: &str) -> Result<SocketAddr, DnsError> {
    // Parse URL to get host and port
    let parsed = url::Url::parse(url)?;
    let host = parsed.host_str().ok_or(DnsError::NoHost)?;
    let port = parsed.port().unwrap_or(443);
    
    // Resolve DNS
    let addrs = tokio::net::lookup_host(format!("{}:{}", host, port)).await?;
    
    // Prefer IPv4 for now
    addrs
        .filter(|addr| addr.is_ipv4())
        .next()
        .or_else(|| addrs.next())
        .ok_or(DnsError::NoAddresses)
}
```

### 3. Connection State Machine

```rust
// src/tunnel/state.rs

use std::time::Instant;
use tokio::sync::watch;

/// Connection state
#[derive(Debug, Clone)]
pub enum ConnectionState {
    Disconnected,
    Connecting {
        started_at: Instant,
    },
    Connected {
        since: Instant,
        latency_ms: u32,
    },
    Reconnecting {
        attempt: u32,
        last_error: String,
        started_at: Instant,
    },
    Failed {
        error: String,
        at: Instant,
    },
}

impl ConnectionState {
    pub fn is_connected(&self) -> bool {
        matches!(self, ConnectionState::Connected { .. })
    }
    
    pub fn is_healthy(&self) -> bool {
        match self {
            ConnectionState::Connected { latency_ms, .. } => *latency_ms < 500,
            _ => false,
        }
    }
}

/// State machine for connection lifecycle
pub struct ConnectionStateMachine {
    state: watch::Sender<ConnectionState>,
    state_rx: watch::Receiver<ConnectionState>,
}

impl ConnectionStateMachine {
    pub fn new() -> Self {
        let (tx, rx) = watch::channel(ConnectionState::Disconnected);
        Self {
            state: tx,
            state_rx: rx,
        }
    }
    
    pub fn transition(&self, new_state: ConnectionState) {
        let old_state = self.state_rx.borrow().clone();
        
        // Validate transition
        let valid = match (&old_state, &new_state) {
            (ConnectionState::Disconnected, ConnectionState::Connecting { .. }) => true,
            (ConnectionState::Connecting { .. }, ConnectionState::Connected { .. }) => true,
            (ConnectionState::Connecting { .. }, ConnectionState::Failed { .. }) => true,
            (ConnectionState::Connected { .. }, ConnectionState::Reconnecting { .. }) => true,
            (ConnectionState::Connected { .. }, ConnectionState::Disconnected) => true,
            (ConnectionState::Reconnecting { .. }, ConnectionState::Connected { .. }) => true,
            (ConnectionState::Reconnecting { attempt, .. }, ConnectionState::Reconnecting { attempt: new_attempt, .. }) 
                if *new_attempt == attempt + 1 => true,
            (ConnectionState::Reconnecting { .. }, ConnectionState::Failed { .. }) => true,
            (ConnectionState::Failed { .. }, ConnectionState::Connecting { .. }) => true,
            _ => false,
        };
        
        if valid {
            tracing::debug!("State transition: {:?} -> {:?}", old_state, new_state);
            let _ = self.state.send(new_state);
        } else {
            tracing::warn!(
                "Invalid state transition attempted: {:?} -> {:?}",
                old_state,
                new_state
            );
        }
    }
    
    pub fn current(&self) -> ConnectionState {
        self.state_rx.borrow().clone()
    }
    
    pub fn subscribe(&self) -> watch::Receiver<ConnectionState> {
        self.state_rx.clone()
    }
}
```

### 4. Heartbeat System

```rust
// src/tunnel/heartbeat.rs

use tokio::sync::mpsc;
use std::time::Duration;

/// Heartbeat message sent to cloud
#[derive(Debug, Clone, serde::Serialize)]
pub struct Heartbeat {
    pub device_id: String,
    pub timestamp: i64,
    pub sequence: u64,
    pub vitals: DeviceVitals,
}

/// Heartbeat service
pub struct HeartbeatService {
    interval: Duration,
    sequence: AtomicU64,
    connection: Arc<RwLock<Option<quinn::Connection>>>,
    shutdown: broadcast::Sender<()>,
}

impl HeartbeatService {
    pub fn new(interval: Duration) -> Self {
        let (shutdown, _) = broadcast::channel(1);
        Self {
            interval,
            sequence: AtomicU64::new(0),
            connection: Arc::new(RwLock::new(None)),
            shutdown,
        }
    }
    
    pub fn set_connection(&self, conn: quinn::Connection) {
        *self.connection.write().unwrap() = Some(conn);
    }
    
    pub fn spawn(&self) -> tokio::task::JoinHandle<()> {
        let interval = self.interval;
        let sequence = Arc::new(self.sequence.clone());
        let connection = self.connection.clone();
        let mut shutdown = self.shutdown.subscribe();
        
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            
            loop {
                tokio::select! {
                    _ = interval_timer.tick() => {
                        if let Some(conn) = connection.read().unwrap().as_ref() {
                            let seq = sequence.fetch_add(1, Ordering::SeqCst);
                            if let Err(e) = Self::send_heartbeat(conn, seq).await {
                                tracing::warn!("Heartbeat failed: {}", e);
                            }
                        }
                    }
                    _ = shutdown.recv() => {
                        tracing::info!("Heartbeat service shutting down");
                        break;
                    }
                }
            }
        })
    }
    
    async fn send_heartbeat(
        conn: &quinn::Connection,
        sequence: u64,
    ) -> Result<(), HeartbeatError> {
        // Collect device vitals
        let vitals = DeviceVitals::collect().await;
        
        let heartbeat = Heartbeat {
            device_id: get_device_id(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            sequence,
            vitals,
        };
        
        // Serialize to protobuf/JSON
        let data = serde_json::to_vec(&heartbeat)?;
        
        // Send on unidirectional stream
        let mut send = conn.open_uni().await?;
        send.write_all(&[0x01])?;  // Message type: Heartbeat
        send.write_all(&(data.len() as u32).to_be_bytes())?;
        send.write_all(&data)?;
        send.finish().await?;
        
        tracing::trace!("Heartbeat sent: seq={}", sequence);
        
        Ok(())
    }
    
    pub fn shutdown(&self) {
        let _ = self.shutdown.send(());
    }
}
```

### 5. Auto-Reconnection

```rust
// src/tunnel/reconnect.rs

use std::time::Duration;
use tokio::time::sleep;

/// Reconnection configuration
pub struct ReconnectConfig {
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub max_attempts: u32,
    pub backoff_multiplier: f32,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            max_attempts: 10,
            backoff_multiplier: 2.0,
        }
    }
}

/// Reconnection manager
pub struct ReconnectManager {
    config: ReconnectConfig,
    current_delay: Duration,
    attempts: u32,
}

impl ReconnectManager {
    pub fn new(config: ReconnectConfig) -> Self {
        Self {
            current_delay: config.initial_delay,
            attempts: 0,
            config,
        }
    }
    
    /// Wait for next reconnection attempt
    pub async fn wait_for_retry(&mut self) -> bool {
        if self.attempts >= self.config.max_attempts {
            tracing::error!(
                "Max reconnection attempts ({}) reached",
                self.config.max_attempts
            );
            return false;
        }
        
        self.attempts += 1;
        
        tracing::info!(
            "Reconnection attempt {}/{} in {:?}",
            self.attempts,
            self.config.max_attempts,
            self.current_delay
        );
        
        sleep(self.current_delay).await;
        
        // Exponential backoff
        self.current_delay = std::cmp::min(
            Duration::from_secs_f32(
                self.current_delay.as_secs_f32() * self.config.backoff_multiplier
            ),
            self.config.max_delay,
        );
        
        true
    }
    
    /// Reset after successful connection
    pub fn reset(&mut self) {
        self.attempts = 0;
        self.current_delay = self.config.initial_delay;
    }
    
    pub fn attempts(&self) -> u32 {
        self.attempts
    }
}

/// Spawn reconnection task
pub fn spawn_reconnect_task(
    tunnel: Arc<CloudTunnel>,
    config: ReconnectConfig,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut manager = ReconnectManager::new(config);
        let mut state_rx = tunnel.state_machine.subscribe();
        
        loop {
            // Wait for disconnection
            state_rx.changed().await.ok();
            
            let state = state_rx.borrow().clone();
            
            match state {
                ConnectionState::Connected { .. } => {
                    manager.reset();
                }
                ConnectionState::Reconnecting { .. } | ConnectionState::Failed { .. } => {
                    if manager.wait_for_retry().await {
                        // Attempt reconnection
                        match tunnel.connect_internal().await {
                            Ok(_) => {
                                tracing::info!("Reconnection successful");
                                manager.reset();
                            }
                            Err(e) => {
                                tracing::warn!("Reconnection failed: {}", e);
                            }
                        }
                    } else {
                        // Max attempts reached, transition to failed
                        tunnel.state_machine.transition(ConnectionState::Failed {
                            error: "Max reconnection attempts exceeded".to_string(),
                            at: Instant::now(),
                        });
                    }
                }
                _ => {}
            }
        }
    })
}
```

### 6. Main CloudTunnel Implementation

```rust
// src/tunnel/mod.rs

mod tls;
mod endpoint;
mod state;
mod heartbeat;
mod reconnect;

pub use state::ConnectionState;

use std::sync::Arc;
use tokio::sync::RwLock;

/// Main cloud tunnel struct
pub struct CloudTunnel {
    config: TunnelConfig,
    endpoint: quinn::Endpoint,
    connection: Arc<RwLock<Option<quinn::Connection>>>,
    state_machine: ConnectionStateMachine,
    heartbeat_service: HeartbeatService,
    stats: Arc<RwLock<TunnelStats>>,
}

impl CloudTunnel {
    /// Create new tunnel instance
    pub fn new(config: TunnelConfig) -> Result<Self, TunnelError> {
        // Create TLS config
        let tls_config = tls::create_tls_config(&config.cert_path, &config.key_path)?;
        
        // Create QUIC endpoint
        let endpoint = endpoint::create_endpoint(tls_config)?;
        
        // Create heartbeat service
        let heartbeat_service = HeartbeatService::new(config.heartbeat_interval);
        
        Ok(Self {
            config,
            endpoint,
            connection: Arc::new(RwLock::new(None)),
            state_machine: ConnectionStateMachine::new(),
            heartbeat_service,
            stats: Arc::new(RwLock::new(TunnelStats::default())),
        })
    }
    
    /// Connect to cloud
    pub async fn connect(&mut self) -> Result<(), TunnelError> {
        self.state_machine.transition(ConnectionState::Connecting {
            started_at: Instant::now(),
        });
        
        match self.connect_internal().await {
            Ok(()) => {
                // Start heartbeat
                self.heartbeat_service.spawn();
                
                // Start reconnection monitor
                reconnect::spawn_reconnect_task(
                    Arc::new(self.clone()),
                    ReconnectConfig::default(),
                );
                
                Ok(())
            }
            Err(e) => {
                self.state_machine.transition(ConnectionState::Failed {
                    error: e.to_string(),
                    at: Instant::now(),
                });
                Err(e)
            }
        }
    }
    
    async fn connect_internal(&self) -> Result<(), TunnelError> {
        let conn = endpoint::connect_to_cloud(
            &self.endpoint,
            &self.config.cloud_url,
            "superinstance.ai",
        ).await?;
        
        // Store connection
        *self.connection.write().await = Some(conn.clone());
        
        // Update heartbeat service
        self.heartbeat_service.set_connection(conn);
        
        // Transition to connected
        self.state_machine.transition(ConnectionState::Connected {
            since: Instant::now(),
            latency_ms: 0,  // Will be updated by first heartbeat
        });
        
        Ok(())
    }
    
    /// Disconnect from cloud
    pub async fn disconnect(&mut self) -> Result<(), TunnelError> {
        self.heartbeat_service.shutdown();
        
        if let Some(conn) = self.connection.write().await.take() {
            conn.close(0u32.into(), b"client disconnect");
        }
        
        self.state_machine.transition(ConnectionState::Disconnected);
        
        Ok(())
    }
    
    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.state_machine.current().is_connected()
    }
    
    /// Get current state
    pub fn state(&self) -> ConnectionState {
        self.state_machine.current()
    }
    
    /// Send request and receive response (bidirectional stream)
    pub async fn request(&self, data: &[u8]) -> Result<Vec<u8>, TunnelError> {
        let conn = self.connection.read().await
            .as_ref()
            .ok_or(TunnelError::NotConnected)?
            .clone();
        
        let (mut send, mut recv) = conn.open_bi().await?;
        
        // Send request
        send.write_all(data).await?;
        send.finish().await?;
        
        // Receive response
        let response = recv.read_to_end(MAX_RESPONSE_SIZE).await?;
        
        // Update stats
        let mut stats = self.stats.write().await;
        stats.total_bytes_sent += data.len() as u64;
        stats.total_bytes_received += response.len() as u64;
        stats.requests_sent += 1;
        stats.requests_succeeded += 1;
        
        Ok(response)
    }
    
    /// Open stream for streaming responses
    pub async fn open_stream(&self) -> Result<(quinn::SendStream, quinn::RecvStream), TunnelError> {
        let conn = self.connection.read().await
            .as_ref()
            .ok_or(TunnelError::NotConnected)?
            .clone();
        
        Ok(conn.open_bi().await?)
    }
    
    /// Get tunnel statistics
    pub async fn stats(&self) -> TunnelStats {
        self.stats.read().await.clone()
    }
}
```

---

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_state_machine_transitions() {
        let sm = ConnectionStateMachine::new();
        
        // Initial state
        assert!(matches!(sm.current(), ConnectionState::Disconnected));
        
        // Valid: Disconnected -> Connecting
        sm.transition(ConnectionState::Connecting { started_at: Instant::now() });
        assert!(matches!(sm.current(), ConnectionState::Connecting { .. }));
        
        // Valid: Connecting -> Connected
        sm.transition(ConnectionState::Connected { since: Instant::now(), latency_ms: 50 });
        assert!(sm.current().is_connected());
    }
    
    #[tokio::test]
    async fn test_reconnect_backoff() {
        let config = ReconnectConfig {
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            max_attempts: 5,
            backoff_multiplier: 2.0,
        };
        
        let mut manager = ReconnectManager::new(config);
        
        // First retry: 10ms
        let start = Instant::now();
        assert!(manager.wait_for_retry().await);
        assert!(start.elapsed() >= Duration::from_millis(10));
        
        // Second retry: 20ms
        let start = Instant::now();
        assert!(manager.wait_for_retry().await);
        assert!(start.elapsed() >= Duration::from_millis(20));
        
        // Continue until max
        for _ in 0..3 {
            manager.wait_for_retry().await;
        }
        
        // Should return false after max attempts
        assert!(!manager.wait_for_retry().await);
    }
}
```

---

## Performance Considerations

1. **Connection Pooling**: Reuse connections for multiple requests
2. **Stream Multiplexing**: Use QUIC's native multiplexing for concurrent requests
3. **Heartbeat Coalescing**: Batch vitals collection to reduce overhead
4. **Zero-Copy Transfers**: Use `bytes` crate for efficient buffer management

---

*Document Version: 2.0.0*
*Last Updated: 2026-01-02*
