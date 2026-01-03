//! SuperInstance Privacy - Redaction & Token Vault
//!
//! This crate handles all privacy-related functionality:
//! - Pattern detection (emails, phones, API keys, etc.)
//! - Redaction with reversible tokens
//! - Secure token vault for storing original values
//! - Reinflation of responses
//!
//! # Privacy Flow
//!
//! ```text
//! User Input → Detect Patterns → Redact → [TOKEN_XXXX] → Cloud
//!                                              ↓
//!                               Store in Vault (local only)
//!                                              ↓
//! Response ← Reinflate ← [TOKEN_XXXX] ← Cloud Response
//! ```

pub mod patterns;
pub mod redactor;
pub mod vault;

// Re-exports
pub use patterns::{Pattern, PatternMatch, PatternSet, PatternType};
pub use redactor::{RedactionResult, Redactor, RedactorConfig};
pub use vault::{SessionStats, TokenVault};

/// Privacy error types
#[derive(Debug, thiserror::Error)]
pub enum PrivacyError {
    #[error("Pattern compilation failed: {0}")]
    PatternError(String),

    #[error("Vault error: {0}")]
    VaultError(String),

    #[error("Token not found: {0}")]
    TokenNotFound(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type for privacy operations
pub type PrivacyResult<T> = Result<T, PrivacyError>;

/// Statistics about redaction
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RedactionStats {
    /// Total patterns detected
    pub patterns_detected: usize,
    /// Patterns redacted
    pub patterns_redacted: usize,
    /// Tokens created
    pub tokens_created: usize,
    /// By pattern type
    pub by_type: std::collections::HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = PrivacyError::TokenNotFound("TOKEN_1234".to_string());
        assert!(err.to_string().contains("TOKEN_1234"));
    }
}
