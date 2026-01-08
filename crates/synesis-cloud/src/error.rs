//! Error types for synesis-cloud

use std::io;
use thiserror::Error;

/// Result type for synesis-cloud operations
pub type CloudResult<T> = Result<T, CloudError>;

/// Errors that can occur in synesis-cloud operations
#[derive(Error, Debug)]
pub enum CloudError {
    /// Tunnel connection errors
    #[error("Tunnel connection error: {0}")]
    TunnelConnection(String),

    /// TLS/configuration errors
    #[error("TLS error: {0}")]
    Tls(String),

    /// Certificate errors
    #[error("Certificate error: {0}")]
    Certificate(String),

    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// Timeout errors
    #[error("Operation timed out after {0:?}")]
    Timeout(std::time::Duration),

    /// API errors
    #[error("API error: {0}")]
    Api(String),

    /// Billing errors
    #[error("Billing error: {0}")]
    Billing(String),

    /// Authentication errors
    #[error("Authentication error: {0}")]
    Auth(String),

    /// Rate limit errors
    #[error("Rate limit exceeded, retry after {0}s")]
    RateLimit(u32),

    /// Validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    /// Not connected error
    #[error("Not connected to cloud")]
    NotConnected,

    /// Generic error with message
    #[error("{0}")]
    Other(String),

    /// Telemetry errors
    #[error("Telemetry error: {0}")]
    Telemetry(String),
}

impl CloudError {
    /// Create a tunnel connection error
    pub fn tunnel_connection(msg: impl Into<String>) -> Self {
        Self::TunnelConnection(msg.into())
    }

    /// Create a TLS error
    pub fn tls(msg: impl Into<String>) -> Self {
        Self::Tls(msg.into())
    }

    /// Create a certificate error
    pub fn certificate(msg: impl Into<String>) -> Self {
        Self::Certificate(msg.into())
    }

    /// Create an API error
    pub fn api(msg: impl Into<String>) -> Self {
        Self::Api(msg.into())
    }

    /// Create a billing error
    pub fn billing(msg: impl Into<String>) -> Self {
        Self::Billing(msg.into())
    }

    /// Create an authentication error
    pub fn auth(msg: impl Into<String>) -> Self {
        Self::Auth(msg.into())
    }

    /// Create a validation error
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    /// Create a generic error
    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }

    /// Create a telemetry error
    pub fn telemetry(msg: impl Into<String>) -> Self {
        Self::Telemetry(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = CloudError::tunnel_connection("connection failed");
        assert!(matches!(err, CloudError::TunnelConnection(_)));
    }

    #[test]
    fn test_error_display() {
        let err = CloudError::RateLimit(60);
        assert_eq!(err.to_string(), "Rate limit exceeded, retry after 60s");
    }

    #[test]
    fn test_result_type() {
        fn returns_result() -> CloudResult<()> {
            Ok(())
        }

        assert!(returns_result().is_ok());
    }
}
