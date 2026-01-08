//! Cloud escalation client
//!
//! Client for sending queries to cloud LLMs via QUIC tunnel.
//!
//! ## Request Flow
//!
//! 1. Validate request (query length, token limits, timeout)
//! 2. Serialize request to JSON
//! 3. Send via QUIC tunnel with timeout
//! 4. Deserialize and validate response
//! 5. Verify request ID matches (prevent mixing responses)
//!
//! ## Performance
//!
//! - **Request validation**: O(1) - Simple bounds checking
//! - **Serialization**: O(n) where n = request size
//! - **Network**: Bound by QUIC tunnel latency
//! - **Deserialization**: O(m) where m = response size
//!
//! ## Timeouts
//!
//! Default timeout is 30 seconds. Cloud models may take:
//! - Simple queries: 1-3 seconds
//! - Complex reasoning: 5-15 seconds
//! - Large context: 10-30 seconds

use crate::error::{CloudError, CloudResult};
use crate::escalation::types::{EscalationRequest, EscalationResponse, CloudModel};
use crate::tunnel::tunnel::CloudTunnel;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

// ============================================================================
// CONSTANTS: Escalation Configuration
// ============================================================================

/// Maximum query length in characters
///
/// Prevents excessively large queries that would:
/// - Cause timeouts (models have processing limits)
/// - Incur high costs (charges scale with tokens)
/// - Exceed model context windows
const MAX_QUERY_LENGTH: usize = 100_000;

/// Maximum max_tokens value
///
/// Prevents requests for unreasonably long outputs.
/// Most models have practical limits around 4K-8K output tokens.
const MAX_MAX_TOKENS: u32 = 128_000;

/// Minimum max_tokens value
///
/// Prevents requests that would fail validation.
/// Must be at least 1 to generate any output.
const MIN_MAX_TOKENS: u32 = 1;

/// Maximum timeout in seconds
///
/// 10 minute timeout allows for very long-running complex queries.
/// Most requests should complete within 30 seconds.
const MAX_TIMEOUT_SECS: u64 = 600;

/// Minimum timeout in seconds
///
/// Prevents zero-timeout requests that would fail immediately.
const MIN_TIMEOUT_SECS: u64 = 1;

/// Default request timeout (30 seconds)
///
/// 30 seconds is sufficient for most queries while preventing
/// indefinite hangs on unresponsive cloud endpoints.
///
/// TODO: Use in escalate() when implementing request timeout logic
#[allow(dead_code)]
const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Cloud escalation client
///
/// Sends redacted queries to cloud LLMs and receives responses.
///
/// ## Thread Safety
///
/// This struct is clone-safe. The underlying tunnel uses `Arc` for
/// shared ownership across multiple concurrent requests.
///
/// ## Production Status
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
    ///
    /// * `tunnel` - Connected QUIC tunnel (must already be authenticated)
    /// * `api_key` - API key for cloud authentication (reserved for future use)
    /// * `timeout` - Request timeout (default: 30 seconds)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use synesis_cloud::escalation::client::EscalationClient;
    /// # use synesis_cloud::tunnel::tunnel::CloudTunnel;
    /// # use std::sync::Arc;
    /// # use std::time::Duration;
    /// # let tunnel = Arc::new(CloudTunnel::new(Default::default()).unwrap());
    /// let client = EscalationClient::new(
    ///     tunnel,
    ///     "api-key".to_string(),
    ///     Duration::from_secs(30),
    /// );
    /// ```
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
    ///
    /// Ensures request parameters are within acceptable bounds to prevent:
    /// - Excessive costs (large queries with many tokens)
    /// - Timeouts (queries that take too long to process)
    /// - API errors (invalid parameter ranges)
    ///
    /// # Validation Rules
    ///
    /// - Query: 1 to 100,000 characters
    /// - max_tokens: 1 to 128,000
    /// - timeout_secs: 1 to 600 seconds (if specified)
    ///
    /// # Performance
    ///
    /// O(1) - Simple bounds checking.
    fn validate_request(request: &EscalationRequest) -> CloudResult<()> {
        // Validate query is not empty
        if request.query.is_empty() {
            return Err(CloudError::validation(
                "Query cannot be empty. Please provide a non-empty query string."
            ));
        }

        // Validate query length (prevent excessive costs/timeouts)
        if request.query.len() > MAX_QUERY_LENGTH {
            return Err(CloudError::validation(format!(
                "Query too long ({} chars, max {} chars). Consider splitting into smaller queries.",
                request.query.len(),
                MAX_QUERY_LENGTH
            )));
        }

        // Validate max_tokens (prevent invalid API requests)
        if request.max_tokens < MIN_MAX_TOKENS {
            return Err(CloudError::validation(format!(
                "max_tokens must be at least {} (got {})",
                MIN_MAX_TOKENS,
                request.max_tokens
            )));
        }

        if request.max_tokens > MAX_MAX_TOKENS {
            return Err(CloudError::validation(format!(
                "max_tokens too large (got {}, max {})",
                request.max_tokens,
                MAX_MAX_TOKENS
            )));
        }

        // Validate timeout range (if specified)
        if let Some(timeout) = request.timeout_secs {
            let timeout_u64 = timeout as u64;
            if !(MIN_TIMEOUT_SECS..=MAX_TIMEOUT_SECS).contains(&timeout_u64) {
                return Err(CloudError::validation(format!(
                    "timeout_secs must be between {} and {} seconds (got {})",
                    MIN_TIMEOUT_SECS,
                    MAX_TIMEOUT_SECS,
                    timeout
                )));
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
