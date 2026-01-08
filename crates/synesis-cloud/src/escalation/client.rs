//! Cloud escalation client
//!
//! Client for sending queries to cloud LLMs via QUIC tunnel.

use crate::error::{CloudError, CloudResult};
use crate::escalation::types::{EscalationRequest, EscalationResponse, CloudModel};
use crate::tunnel::tunnel::CloudTunnel;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

/// Cloud escalation client
///
/// Sends redacted queries to cloud LLMs and receives responses
///
/// # Production Status
///
/// The `api_key` field is reserved for future authentication implementation.
pub struct EscalationClient {
    tunnel: Arc<CloudTunnel>,
    #[allow(dead_code)]
    api_key: String,
    timeout: Duration,
    default_model: CloudModel,
}

impl EscalationClient {
    /// Create a new escalation client
    ///
    /// # Arguments
    /// * `tunnel` - Connected QUIC tunnel
    /// * `api_key` - API key for authentication
    /// * `timeout` - Request timeout
    pub fn new(
        tunnel: Arc<CloudTunnel>,
        api_key: String,
        timeout: Duration,
    ) -> Self {
        Self {
            tunnel,
            api_key,
            timeout,
            default_model: CloudModel::Auto,
        }
    }

    /// Set default model
    pub fn with_default_model(mut self, model: CloudModel) -> Self {
        self.default_model = model;
        self
    }

    /// Escalate a query to cloud
    ///
    /// # Arguments
    /// * `request` - Escalation request (query should already be redacted)
    ///
    /// # Returns
    /// * Escalation response from cloud
    ///
    /// # Errors
    /// * Tunnel connection error
    /// * Timeout error
    /// * Cloud API error
    pub async fn escalate(&self, mut request: EscalationRequest) -> CloudResult<EscalationResponse> {
        // Ensure request_id is set
        if request.request_id.is_empty() {
            request.request_id = Uuid::new_v4().to_string();
        }

        // Use default model if Auto
        if request.model == CloudModel::Auto {
            request.model = self.default_model;
        }

        // Validate request
        Self::validate_request(&request)?;

        // Serialize request
        let payload = serde_json::to_vec(&request)
            .map_err(CloudError::Serialization)?;

        // Send via tunnel
        let response_data = tokio::time::timeout(
            self.timeout,
            self.tunnel.request(&payload)
        )
        .await
        .map_err(|_| CloudError::Timeout(self.timeout))?
        .map_err(|e| CloudError::tunnel_connection(format!("Escalation failed: {}", e)))?;

        // Parse response
        let response: EscalationResponse = serde_json::from_slice(&response_data)
            .map_err(CloudError::Serialization)?;

        // Verify request_id matches
        if response.request_id != request.request_id {
            return Err(CloudError::validation(format!(
                "Request ID mismatch: expected {}, got {}",
                request.request_id, response.request_id
            )));
        }

        tracing::info!(
            "Escalation completed: model={}, tokens={}, cost={}¢, latency={}ms",
            response.model_used,
            response.tokens_used.total(),
            response.cost_cents,
            response.latency_ms
        );

        Ok(response)
    }

    /// Escalate with streaming
    ///
    /// TODO: Implement streaming in Session 2.10
    pub async fn escalate_stream(
        &self,
        _request: EscalationRequest,
    ) -> CloudResult<tokio::sync::mpsc::Receiver<String>> {
        Err(CloudError::other("Streaming not yet implemented - see Session 2.10"))
    }

    /// Validate escalation request
    fn validate_request(request: &EscalationRequest) -> CloudResult<()> {
        if request.query.is_empty() {
            return Err(CloudError::validation("Query cannot be empty"));
        }

        if request.query.len() > 100_000 {
            return Err(CloudError::validation("Query too long (max 100k characters)"));
        }

        if request.max_tokens == 0 {
            return Err(CloudError::validation("max_tokens must be > 0"));
        }

        if request.max_tokens > 128_000 {
            return Err(CloudError::validation("max_tokens too large (max 128k)"));
        }

        if let Some(timeout) = request.timeout_secs {
            if timeout == 0 || timeout > 600 {
                return Err(CloudError::validation("timeout_secs must be 1-600"));
            }
        }

        Ok(())
    }

    /// Get client statistics
    pub fn stats(&self) -> ClientStats {
        ClientStats {
            default_model: self.default_model,
            timeout: self.timeout,
        }
    }
}

/// Client statistics
#[derive(Debug, Clone)]
pub struct ClientStats {
    /// Default cloud model for requests
    pub default_model: CloudModel,
    /// Request timeout duration
    pub timeout: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn make_test_config() -> crate::tunnel::types::TunnelConfig {
        crate::tunnel::types::TunnelConfig {
            cert_path: "/tmp/test-cert.pem".into(),
            key_path: "/tmp/test-key.pem".into(),
            ..Default::default()
        }
    }

    #[test]
    fn test_client_creation() {
        let tunnel = Arc::new(crate::tunnel::tunnel::CloudTunnel::new(
            make_test_config()
        ).unwrap());

        let client = EscalationClient::new(
            tunnel,
            "test-key".to_string(),
            Duration::from_secs(30),
        );

        assert_eq!(client.api_key, "test-key");
        assert_eq!(client.timeout, Duration::from_secs(30));
        assert_eq!(client.default_model, CloudModel::Auto);
    }

    #[test]
    fn test_client_with_default_model() {
        let tunnel = Arc::new(crate::tunnel::tunnel::CloudTunnel::new(
            make_test_config()
        ).unwrap());

        let client = EscalationClient::new(
            tunnel.clone(),
            "test-key".to_string(),
            Duration::from_secs(30),
        ).with_default_model(CloudModel::ClaudeOpus);

        assert_eq!(client.default_model, CloudModel::ClaudeOpus);
    }

    #[test]
    fn test_validate_request_empty_query() {
        let request = EscalationRequest {
            query: String::new(),
            ..Default::default()
        };

        assert!(EscalationClient::validate_request(&request).is_err());
    }

    #[test]
    fn test_validate_query_too_long() {
        let request = EscalationRequest {
            query: "x".repeat(100_001),
            ..Default::default()
        };

        assert!(EscalationClient::validate_request(&request).is_err());
    }

    #[test]
    fn test_validate_max_tokens() {
        let request = EscalationRequest {
            query: "test".to_string(),
            max_tokens: 0,
            ..Default::default()
        };

        assert!(EscalationClient::validate_request(&request).is_err());
    }

    #[test]
    fn test_validate_valid_request() {
        let request = EscalationRequest {
            query: "What is 2+2?".to_string(),
            max_tokens: 100,
            ..Default::default()
        };

        assert!(EscalationClient::validate_request(&request).is_ok());
    }
}
