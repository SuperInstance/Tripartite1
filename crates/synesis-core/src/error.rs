//! # Unified Error Handling for SuperInstance
//!
//! This module provides a comprehensive error type that covers all error scenarios
//! across the SuperInstance AI system. All library crates use this unified error type
//! for consistent error handling and user-friendly error messages.

/// Unified error type for all SuperInstance operations
///
/// This error type covers all error scenarios across:
/// - Database operations (SQLite)
/// - Model loading and inference
/// - Configuration parsing and validation
/// - Network connectivity (cloud)
/// - Privacy operations (redaction, tokens)
/// - Knowledge operations (indexing, retrieval)
/// - Consensus operations (agent failures, timeouts)
/// - File I/O operations
///
/// # Example
///
/// ```rust
/// use synesis_core::{SynesisError, SynesisResult};
///
/// fn load_model(path: &str) -> SynesisResult<()> {
///     if std::path::Path::new(path).exists() {
///         Ok(())
///     } else {
///         Err(SynesisError::ModelNotFound(path.to_string()))
///     }
/// }
/// ```
#[derive(Debug, thiserror::Error)]
pub enum SynesisError {
    // ==================== Database Errors ====================

    /// Failed to connect to database
    #[error("Database connection failed: {0}")]
    DatabaseConnection(String),

    /// Query execution failed
    #[error("Database query failed: {0}")]
    DatabaseQuery(String),

    /// Generic SQLite error (from rusqlite)
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    // ==================== Model Errors ====================

    /// Model file not found
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    /// Failed to load model
    #[error("Model load failed: {0}")]
    ModelLoadFailed(String),

    /// Model inference failed
    #[error("Model inference failed: {0}")]
    ModelInferenceFailed(String),

    /// Model not available (not loaded)
    #[error("Model not available: {0}")]
    ModelUnavailable(String),

    /// Checksum mismatch during download
    #[error("Checksum mismatch for {model}: expected {expected}, got {actual}")]
    ChecksumMismatch {
        model: String,
        expected: String,
        actual: String,
    },

    /// Insufficient hardware resources
    #[error("Insufficient resources: {0}")]
    InsufficientResources(String),

    // ==================== Configuration Errors ====================

    /// Failed to parse configuration
    #[error("Config parse error: {0}")]
    ConfigParse(String),

    /// Configuration validation failed
    #[error("Config validation failed: {0}")]
    ConfigValidation(String),

    /// Invalid configuration value
    #[error("Invalid config value: {0}")]
    InvalidConfigValue(String),

    // ==================== Network Errors ====================

    /// Network connection failed
    #[error("Network connection failed: {0}")]
    NetworkConnection(String),

    /// Network operation timed out
    #[error("Network timeout: {0}")]
    NetworkTimeout(String),

    /// HTTP request failed
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    // ==================== Privacy Errors ====================

    /// Pattern compilation failed (regex)
    #[error("Pattern compilation failed: {0}")]
    PatternError(String),

    /// Redaction operation failed
    #[error("Redaction failed: {0}")]
    RedactionFailed(String),

    /// Token vault operation failed
    #[error("Token vault error: {0}")]
    TokenVaultError(String),

    /// Token not found in vault (for reinflation)
    #[error("Token not found: {0}")]
    TokenNotFound(String),

    // ==================== Knowledge Errors ====================

    /// Document not found in knowledge vault
    #[error("Document not found: {0}")]
    DocumentNotFound(String),

    /// Document indexing failed
    #[error("Indexing failed: {0}")]
    IndexingFailed(String),

    /// Knowledge retrieval failed
    #[error("Retrieval failed: {0}")]
    RetrievalFailed(String),

    /// Embedding generation failed
    #[error("Embedding error: {0}")]
    EmbeddingError(String),

    /// Invalid document format
    #[error("Invalid document format: {0}")]
    InvalidDocumentFormat(String),

    /// File watcher error
    #[error("File watch error: {0}")]
    WatchError(String),

    // ==================== Consensus Errors ====================

    /// Consensus not reached after max rounds
    #[error("Consensus not reached after {rounds} rounds")]
    NoConsensus { rounds: u8 },

    /// Ethos agent vetoed the response
    #[error("Ethos veto: {reason}")]
    EthosVeto { reason: String },

    /// Agent operation failed
    #[error("Agent error: {0}")]
    AgentError(String),

    /// Agent operation timed out
    #[error("Agent timeout: {0}")]
    AgentTimeout(String),

    /// Consensus engine error
    #[error("Consensus engine error: {0}")]
    ConsensusError(String),

    // ==================== File I/O Errors ====================

    /// File not found
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Invalid path
    #[error("Invalid path: {0}")]
    InvalidPath(String),

    /// Generic I/O error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    // ==================== Generic Errors ====================

    /// Generic internal error
    #[error("Internal error: {0}")]
    Internal(String),

    /// Uncategorized error
    #[error("Error: {0}")]
    Other(String),
}

/// Result type alias for SynesisError
///
/// # Example
///
/// ```rust
/// use synesis_core::SynesisResult;
///
/// fn do_something() -> SynesisResult<()> {
///     Ok(())
/// }
/// ```
pub type Result<T> = std::result::Result<T, SynesisError>;

// ==================== From Implementations ====================

impl From<serde_json::Error> for SynesisError {
    fn from(err: serde_json::Error) -> Self {
        SynesisError::ConfigParse(format!("JSON parsing failed: {}", err))
    }
}

impl From<tokio::task::JoinError> for SynesisError {
    fn from(err: tokio::task::JoinError) -> Self {
        SynesisError::AgentError(format!("Task join failed: {}", err))
    }
}

// Note: Conversion from crate-specific error types (KnowledgeError, PrivacyError, ModelError)
// to SynesisError are implemented here to avoid circular dependencies.

impl From<synesis_knowledge::KnowledgeError> for SynesisError {
    fn from(err: synesis_knowledge::KnowledgeError) -> Self {
        match err {
            synesis_knowledge::KnowledgeError::NotFound(msg) => {
                SynesisError::DocumentNotFound(msg)
            }
            synesis_knowledge::KnowledgeError::InvalidFormat(msg) => {
                SynesisError::InvalidDocumentFormat(msg)
            }
            synesis_knowledge::KnowledgeError::EmbeddingError(msg) => {
                SynesisError::EmbeddingError(msg)
            }
            synesis_knowledge::KnowledgeError::DatabaseError(msg) => {
                SynesisError::DatabaseQuery(msg)
            }
            synesis_knowledge::KnowledgeError::IoError(err) => {
                SynesisError::IoError(err)
            }
            synesis_knowledge::KnowledgeError::SqliteError(err) => {
                SynesisError::Sqlite(err)
            }
            synesis_knowledge::KnowledgeError::WatchError(msg) => {
                SynesisError::WatchError(msg)
            }
            synesis_knowledge::KnowledgeError::Internal(msg) => {
                SynesisError::Internal(msg)
            }
        }
    }
}

impl From<synesis_privacy::PrivacyError> for SynesisError {
    fn from(err: synesis_privacy::PrivacyError) -> Self {
        match err {
            synesis_privacy::PrivacyError::PatternError(msg) => {
                SynesisError::PatternError(msg)
            }
            synesis_privacy::PrivacyError::VaultError(msg) => {
                SynesisError::TokenVaultError(msg)
            }
            synesis_privacy::PrivacyError::TokenNotFound(msg) => {
                SynesisError::TokenNotFound(msg)
            }
            synesis_privacy::PrivacyError::DatabaseError(err) => {
                SynesisError::Sqlite(err)
            }
            synesis_privacy::PrivacyError::Internal(msg) => {
                SynesisError::Internal(msg)
            }
        }
    }
}

impl From<synesis_models::ModelError> for SynesisError {
    fn from(err: synesis_models::ModelError) -> Self {
        match err {
            synesis_models::ModelError::NotFound(msg) => {
                SynesisError::ModelNotFound(msg)
            }
            synesis_models::ModelError::NotLoaded(msg) => {
                SynesisError::ModelUnavailable(msg)
            }
            synesis_models::ModelError::InvalidPath(msg) => {
                SynesisError::InvalidPath(msg)
            }
            synesis_models::ModelError::DownloadFailed(msg) => {
                SynesisError::NetworkConnection(msg)
            }
            synesis_models::ModelError::ChecksumMismatch { model, expected, actual } => {
                SynesisError::ChecksumMismatch { model, expected, actual }
            }
            synesis_models::ModelError::InsufficientResources(msg) => {
                SynesisError::InsufficientResources(msg)
            }
            synesis_models::ModelError::InferenceError(msg) => {
                SynesisError::ModelInferenceFailed(msg)
            }
            synesis_models::ModelError::HardwareError(msg) => {
                SynesisError::Internal(format!("Hardware error: {}", msg))
            }
            synesis_models::ModelError::IoError(err) => {
                SynesisError::IoError(err)
            }
            synesis_models::ModelError::HttpError(err) => {
                SynesisError::HttpError(err)
            }
            synesis_models::ModelError::Internal(msg) => {
                SynesisError::Internal(msg)
            }
        }
    }
}

// ==================== Error Display Enhancement ====================

impl SynesisError {
    /// Get a user-friendly error message with suggestions
    pub fn with_context(&self) -> String {
        match self {
            SynesisError::ModelNotFound(model) => {
                format!(
                    "Model '{}' not found.\n  → Run 'synesis model list' to see available models\n  → Run 'synesis model download {}' to download it",
                    model, model
                )
            }
            SynesisError::FileNotFound(path) => {
                format!(
                    "File not found: {}.\n  → Please check the file path\n  → Ensure the file exists\n  → Run 'synesis init' if this is your first time",
                    path
                )
            }
            SynesisError::PermissionDenied(path) => {
                format!(
                    "Permission denied: {}.\n  → Please check file permissions\n  → Try running with appropriate permissions\n  → Check file ownership: ls -l {}",
                    path, path
                )
            }
            SynesisError::NetworkConnection(msg) => {
                format!(
                    "Network error: {}.\n  → Please check your internet connection\n  → Try 'synesis cloud ping' to test connectivity\n  → Check firewall settings if problem persists",
                    msg
                )
            }
            SynesisError::NetworkTimeout(msg) => {
                format!(
                    "Network timeout: {}.\n  → Request took too long to complete\n  → Try again later or check network speed\n  → Consider using a smaller model",
                    msg
                )
            }
            SynesisError::InsufficientResources(msg) => {
                format!(
                    "Insufficient system resources: {}.\n  → Consider using a smaller model\n  → Close other applications\n  → Check available memory: free -h",
                    msg
                )
            }
            SynesisError::ConfigParse(msg) => {
                format!(
                    "Configuration parse error: {}.\n  → Check config file syntax\n  → Run 'synesis config edit' to fix\n  → Reset with 'synesis config reset'",
                    msg
                )
            }
            SynesisError::ConfigValidation(msg) => {
                format!(
                    "Configuration validation failed: {}.\n  → Invalid configuration value\n  → Run 'synesis config get' to review\n  → Edit with 'synesis config edit'",
                    msg
                )
            }
            SynesisError::NoConsensus { rounds } => {
                format!(
                    "Consensus not reached after {} rounds.\n  → Agents could not agree on a response\n  → Try rephrasing your query\n  → Use --cloud to escalate to more powerful models\n  → Adjust threshold: export SYNESIS_CONSENSUS_THRESHOLD=0.75",
                    rounds
                )
            }
            SynesisError::EthosVeto { reason } => {
                format!(
                    "Response vetoed by Ethos agent: {}.\n  → Response was deemed unsafe or inaccurate\n  → Try rephrasing your request\n  → Check for factual errors\n  → Ensure request follows safety guidelines",
                    reason
                )
            }
            SynesisError::DocumentNotFound(doc) => {
                format!(
                    "Document not found: {}.\n  → Document may not be indexed\n  → Run 'synesis knowledge list' to see indexed documents\n  → Index with 'synesis knowledge index <path>'",
                    doc
                )
            }
            SynesisError::IndexingFailed(msg) => {
                format!(
                    "Document indexing failed: {}.\n  → Check document format\n  → Ensure file is readable\n  → Supported formats: PDF, TXT, MD, RS, PY, JS, TS",
                    msg
                )
            }
            SynesisError::EmbeddingError(msg) => {
                format!(
                    "Embedding generation failed: {}.\n  → Model may not be loaded\n  → Check model availability\n  → Ensure sufficient system resources",
                    msg
                )
            }
            SynesisError::TokenNotFound(token) => {
                format!(
                    "Token not found: {}.\n  → Session may have expired\n  → Tokens are cleared after each session\n  → This is normal and expected for privacy",
                    token
                )
            }
            _ => self.to_string(),
        }
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            SynesisError::NetworkTimeout(_)
                | SynesisError::NetworkConnection(_)
                | SynesisError::AgentTimeout(_)
        )
    }

    /// Check if this error is a user error (vs. system error)
    pub fn is_user_error(&self) -> bool {
        matches!(
            self,
            SynesisError::FileNotFound(_)
                | SynesisError::InvalidPath(_)
                | SynesisError::PermissionDenied(_)
                | SynesisError::ConfigValidation(_)
                | SynesisError::InvalidConfigValue(_)
        )
    }

    /// Get suggested recovery commands for this error
    pub fn recovery_commands(&self) -> Vec<&'static str> {
        match self {
            SynesisError::ModelNotFound(_) => vec!["synesis model list", "synesis model download <model>"],
            SynesisError::NetworkConnection(_) => vec!["synesis cloud ping", "ping -c 3 api.superinstance.ai"],
            SynesisError::ConfigValidation(_) => vec!["synesis config edit", "synesis config reset"],
            SynesisError::DocumentNotFound(_) => vec!["synesis knowledge list", "synesis knowledge index <path>"],
            SynesisError::NoConsensus { .. } => vec![
                "export SYNESIS_CONSENSUS_THRESHOLD=0.75",
                "synesis ask --cloud \"<query>\"",
            ],
            _ => vec![],
        }
    }
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = SynesisError::ModelNotFound("test-model".to_string());
        assert!(err.to_string().contains("test-model"));
    }

    #[test]
    fn test_error_with_context() {
        let err = SynesisError::ModelNotFound("test-model".to_string());
        let context = err.with_context();
        assert!(context.contains("test-model"));
        assert!(context.contains("synesis model list"));
    }

    #[test]
    fn test_retryable_errors() {
        assert!(SynesisError::NetworkTimeout("test".to_string()).is_retryable());
        assert!(SynesisError::NetworkConnection("test".to_string()).is_retryable());
        assert!(!SynesisError::ModelNotFound("test".to_string()).is_retryable());
    }

    #[test]
    fn test_user_errors() {
        assert!(SynesisError::FileNotFound("test".to_string()).is_user_error());
        assert!(SynesisError::InvalidPath("test".to_string()).is_user_error());
        assert!(!SynesisError::Internal("test".to_string()).is_user_error());
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let syn_err: SynesisError = io_err.into();
        assert!(matches!(syn_err, SynesisError::IoError(_)));
    }

    #[test]
    fn test_from_sqlite_error() {
        let sql_err = rusqlite::Error::QueryReturnedNoRows;
        let syn_err: SynesisError = sql_err.into();
        assert!(matches!(syn_err, SynesisError::Sqlite(_)));
    }

    #[test]
    fn test_result_alias() {
        fn returns_ok() -> Result<()> {
            Ok(())
        }

        fn returns_err() -> Result<()> {
            Err(SynesisError::Internal("test".to_string()))
        }

        assert!(returns_ok().is_ok());
        assert!(returns_err().is_err());
    }

    #[test]
    fn test_checksum_mismatch() {
        let err = SynesisError::ChecksumMismatch {
            model: "test-model".to_string(),
            expected: "abc123".to_string(),
            actual: "def456".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("test-model"));
        assert!(msg.contains("abc123"));
        assert!(msg.contains("def456"));
    }

    #[test]
    fn test_no_consensus() {
        let err = SynesisError::NoConsensus { rounds: 3 };
        let msg = err.to_string();
        assert!(msg.contains("3"));
        assert!(msg.contains("Consensus")); // Capital C
    }

    #[test]
    fn test_ethos_veto() {
        let err = SynesisError::EthosVeto {
            reason: "Unsafe response detected".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Unsafe response detected"));
        assert!(msg.contains("veto"));
    }

    #[test]
    fn test_enhanced_error_context() {
        let err = SynesisError::ModelNotFound("test-model".to_string());
        let context = err.with_context();
        assert!(context.contains("test-model"));
        assert!(context.contains("synesis model list"));
        assert!(context.contains("synesis model download"));
    }

    #[test]
    fn test_recovery_commands() {
        let err = SynesisError::ModelNotFound("test-model".to_string());
        let commands = err.recovery_commands();
        assert!(commands.contains(&"synesis model list"));
        assert!(commands.contains(&"synesis model download <model>"));
    }

    #[test]
    fn test_no_consensus_context() {
        let err = SynesisError::NoConsensus { rounds: 3 };
        let context = err.with_context();
        assert!(context.contains("3"));
        assert!(context.contains("rephrasing"));
        assert!(context.contains("--cloud"));
        assert!(context.contains("SYNESIS_CONSENSUS_THRESHOLD"));
    }

    #[test]
    fn test_network_error_context() {
        let err = SynesisError::NetworkConnection("Connection refused".to_string());
        let context = err.with_context();
        assert!(context.contains("Connection refused"));
        assert!(context.contains("synesis cloud ping"));
    }

    #[test]
    fn test_ethos_veto_context() {
        let err = SynesisError::EthosVeto {
            reason: "Factual inaccuracies detected".to_string(),
        };
        let context = err.with_context();
        assert!(context.contains("Factual inaccuracies"));
        assert!(context.contains("unsafe"));
        assert!(context.contains("safety"));
    }
}
