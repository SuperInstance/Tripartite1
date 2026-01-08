//! QUIC endpoint for client connections

use crate::error::{CloudError, CloudResult};
use crate::tunnel::tls::create_tls_config;
use quinn::{Endpoint, Connection};
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

/// Create QUIC endpoint for client connections
///
/// # Arguments
/// * `cert_path` - Path to device certificate
/// * `key_path` - Path to device private key
///
/// # Returns
/// * Configured QUIC endpoint
pub fn create_endpoint(
    cert_path: &Path,
    key_path: &Path,
) -> CloudResult<Endpoint> {
    // Create TLS config
    let tls_config = create_tls_config(cert_path, key_path)?;

    // Configure QUIC transport
    let mut transport = quinn::TransportConfig::default();
    transport.keep_alive_interval(Some(Duration::from_secs(10)));
    transport.max_idle_timeout(Some(Duration::from_secs(60).try_into()
        .map_err(|e| CloudError::other(format!("Invalid duration: {}", e)))?));

    // Build QUIC client config
    let mut client_config = quinn::ClientConfig::new(tls_config);
    client_config.transport_config(Arc::new(transport));

    // Bind to random local port
    let bind_addr: SocketAddr = "0.0.0.0:0".parse()
        .map_err(|e| CloudError::other(format!("Invalid bind address: {}", e)))?;
    let mut endpoint = Endpoint::client(bind_addr)
        .map_err(|e| CloudError::tunnel_connection(format!("Failed to create endpoint: {}", e)))?;
    endpoint.set_default_client_config(client_config);

    Ok(endpoint)
}

/// Connect to cloud endpoint
///
/// # Arguments
/// * `endpoint` - QUIC endpoint
/// * `cloud_url` - Cloud server URL
/// * `server_name` - Server name for TLS verification
///
/// # Returns
/// * Connected QUIC connection
pub async fn connect_to_cloud(
    endpoint: &Endpoint,
    cloud_url: &str,
    server_name: &str,
) -> CloudResult<Connection> {
    let addr = resolve_dns(cloud_url).await?;

    let connection = endpoint
        .connect(addr, server_name)
        .map_err(|e| CloudError::tunnel_connection(format!("Failed to connect: {}", e)))?
        .await
        .map_err(|e| CloudError::tunnel_connection(format!("Connection failed: {}", e)))?;

    tracing::info!(
        "Connected to cloud: addr={}, server_name={}",
        addr,
        server_name
    );

    Ok(connection)
}

/// Resolve DNS hostname to SocketAddr
///
/// # Arguments
/// * `url` - Cloud URL (e.g., `<https://tunnel.superinstance.ai:443>`)
///
/// # Returns
/// * Resolved SocketAddr
async fn resolve_dns(url: &str) -> CloudResult<SocketAddr> {
    let parsed = url::Url::parse(url)
        .map_err(|e| CloudError::tunnel_connection(format!("Invalid URL: {}", e)))?;

    let host = parsed.host_str()
        .ok_or_else(|| CloudError::tunnel_connection("No host in URL"))?;

    let port = parsed.port().unwrap_or(443);

    // Resolve DNS
    let mut addrs = tokio::net::lookup_host(format!("{}:{}", host, port))
        .await
        .map_err(|e| CloudError::tunnel_connection(format!("DNS resolution failed: {}", e)))?;

    // Prefer IPv4 for now
    addrs
        .find(|addr| addr.is_ipv4())
        .or_else(|| addrs.next())
        .ok_or_else(|| CloudError::tunnel_connection("No addresses found"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_dns() {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        let result = runtime.block_on(async {
            resolve_dns("https://example.com:443").await
        });

        // Should resolve example.com
        assert!(result.is_ok());
        let addr = result.unwrap();
        assert!(addr.port() == 443 || addr.is_ipv4());
    }

    #[test]
    fn test_invalid_url() {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        let result = runtime.block_on(async {
            resolve_dns("not-a-url").await
        });

        assert!(result.is_err());
    }
}
