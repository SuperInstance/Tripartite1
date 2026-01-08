//! Token Vault
//!
//! Secure local storage for original values that have been redacted.
//! Maps `[EMAIL_0001]` back to user@example.com
//! Never leaves the local machine - this is the core of privacy protection.

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::{params, Connection};
use tracing::{debug, info, instrument};

use crate::{PrivacyError, PrivacyResult};

/// The token vault for session-based token storage
pub struct TokenVault {
    conn: Arc<Mutex<Connection>>,
    /// Track counters per category for token generation (global, not per-session)
    counters: Arc<Mutex<HashMap<String, u32>>>,
}

impl TokenVault {
    /// Create a new vault with a database file
    pub fn new<P: AsRef<Path>>(db_path: P) -> PrivacyResult<Self> {
        let conn = Connection::open(db_path)?;

        // Create the tokens table as per Session 12 spec
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tokens (
                id INTEGER PRIMARY KEY,
                token TEXT UNIQUE NOT NULL,
                category TEXT NOT NULL,
                original TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                session_id TEXT NOT NULL
            )",
            [],
        )?;

        // Create index on session_id for efficient cleanup
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_session_id ON tokens(session_id)",
            [],
        )?;

        // Create index on token for efficient lookups
        conn.execute("CREATE INDEX IF NOT EXISTS idx_token ON tokens(token)", [])?;

        info!("Token vault initialized at database path");

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            counters: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Create an in-memory vault (for testing)
    pub fn in_memory() -> PrivacyResult<Self> {
        let conn = Connection::open_in_memory()?;

        conn.execute(
            "CREATE TABLE tokens (
                id INTEGER PRIMARY KEY,
                token TEXT UNIQUE NOT NULL,
                category TEXT NOT NULL,
                original TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                session_id TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute("CREATE INDEX idx_session_id ON tokens(session_id)", [])?;

        conn.execute("CREATE INDEX idx_token ON tokens(token)", [])?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            counters: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Store a value and return its token
    ///
    /// Token format: `[CATEGORY_NNNN]` where NNNN is zero-padded counter per category (global)
    ///
    /// # Arguments
    /// * `category` - The category of sensitive data (e.g., "EMAIL", "PHONE")
    /// * `original` - The original sensitive value
    /// * `session_id` - The session identifier for grouping tokens
    ///
    /// # Returns
    /// The token that can be used to retrieve the original value later
    #[instrument(skip(self, original), fields(category, session_id))]
    pub fn store(&self, category: &str, original: &str, session_id: &str) -> PrivacyResult<String> {
        // Validate category
        if category.is_empty() {
            return Err(PrivacyError::Internal("Category cannot be empty".to_string()));
        }

        if category.len() > 50 {
            return Err(PrivacyError::Internal("Category too long (max 50 chars)".to_string()));
        }

        if !category.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(PrivacyError::Internal(
                "Category must contain only alphanumeric characters and underscores".to_string()
            ));
        }

        // Validate session_id
        if session_id.is_empty() {
            return Err(PrivacyError::Internal("Session ID cannot be empty".to_string()));
        }

        if session_id.len() > 255 {
            return Err(PrivacyError::Internal("Session ID too long (max 255 chars)".to_string()));
        }

        let conn = self
            .conn
            .lock()
            .map_err(|e| PrivacyError::Internal(format!("Lock poisoned: {}", e)))?;

        // Get and increment global counter for this category
        let mut counters = self
            .counters
            .lock()
            .map_err(|e| PrivacyError::Internal(format!("Lock poisoned: {}", e)))?;
        let counter = counters.entry(category.to_string()).or_insert(0);

        // Check for overflow before incrementing
        if *counter == u32::MAX {
            return Err(PrivacyError::Internal(
                format!("Token counter overflow for category '{}'. This indicates an excessive number of redactions.", category)
            ));
        }

        *counter += 1;
        let token_number = *counter;
        drop(counters);

        // Generate token: [CATEGORY_NNNN]
        let token = format!("[{}_{:04}]", category, token_number);

        debug!(%token, %category, %session_id, "Storing token in vault");

        // Insert into database
        conn.execute(
            "INSERT INTO tokens (token, category, original, session_id)
             VALUES (?1, ?2, ?3, ?4)",
            params![token, category, original, session_id],
        )?;

        Ok(token)
    }

    /// Retrieve the original value for a token
    ///
    /// # Arguments
    /// * `token` - The token to look up (e.g., "`[EMAIL_0001]`")
    ///
    /// # Returns
    /// * Some(original_value) if token exists
    /// * None if token not found
    #[instrument(skip(self), fields(token))]
    pub fn retrieve(&self, token: &str) -> Option<String> {
        let conn = match self.conn.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                tracing::error!(
                    %token,
                    "Mutex is poisoned, attempting to recover from retrieve"
                );
                // Attempt to recover by taking the lock
                poisoned.into_inner()
            }
        };

        debug!(%token, "Retrieving token from vault");

        match conn.query_row(
            "SELECT original FROM tokens WHERE token = ?1",
            params![token],
            |row| row.get::<_, String>(0),
        ) {
            Ok(original) => {
                debug!(%token, "Token found in vault");
                Some(original)
            },
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                debug!(%token, "Token not found in vault");
                None
            },
            Err(e) => {
                tracing::error!(%token, error = %e, "Error retrieving token from vault");
                None
            },
        }
    }

    /// Clear all tokens for a specific session
    ///
    /// # Arguments
    /// * `session_id` - The session to clear
    ///
    /// # Returns
    /// Ok(()) on success, Error on failure
    #[instrument(skip(self), fields(session_id))]
    pub fn clear_session(&self, session_id: &str) -> PrivacyResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| PrivacyError::Internal(format!("Lock poisoned: {}", e)))?;

        let deleted = conn.execute(
            "DELETE FROM tokens WHERE session_id = ?1",
            params![session_id],
        )?;

        info!(%session_id, deleted, "Cleared session tokens from vault");
        Ok(())
    }

    /// Get statistics about tokens in a session
    #[instrument(skip(self), fields(session_id))]
    pub fn session_stats(&self, session_id: &str) -> PrivacyResult<SessionStats> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| PrivacyError::Internal(format!("Lock poisoned: {}", e)))?;

        let total: i64 = conn.query_row(
            "SELECT COUNT(*) FROM tokens WHERE session_id = ?1",
            params![session_id],
            |row| row.get(0),
        )?;

        let mut by_category: HashMap<String, i64> = HashMap::new();
        let mut stmt = conn.prepare(
            "SELECT category, COUNT(*) FROM tokens WHERE session_id = ?1 GROUP BY category",
        )?;

        let rows = stmt.query_map(params![session_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;

        for row in rows {
            let (category, count) = row?;
            by_category.insert(category, count);
        }

        Ok(SessionStats {
            total_tokens: total as usize,
            by_category,
        })
    }
}

/// Statistics about tokens in a session
#[derive(Debug, Clone)]
pub struct SessionStats {
    pub total_tokens: usize,
    pub by_category: HashMap<String, i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_and_retrieve() {
        let vault = TokenVault::in_memory().unwrap();

        let token = vault
            .store("EMAIL", "test@example.com", "session1")
            .unwrap();
        assert!(token.contains("EMAIL"));
        assert_eq!(token, "[EMAIL_0001]");

        let retrieved = vault.retrieve(&token);
        assert_eq!(retrieved, Some("test@example.com".to_string()));
    }

    #[test]
    fn test_retrieve_nonexistent() {
        let vault = TokenVault::in_memory().unwrap();

        let result = vault.retrieve("[EMAIL_FAKE]");
        assert!(result.is_none());
    }

    #[test]
    fn test_counter_increments() {
        let vault = TokenVault::in_memory().unwrap();

        let token1 = vault
            .store("EMAIL", "test1@example.com", "session1")
            .unwrap();
        let token2 = vault
            .store("EMAIL", "test2@example.com", "session1")
            .unwrap();

        assert_eq!(token1, "[EMAIL_0001]");
        assert_eq!(token2, "[EMAIL_0002]");
    }

    #[test]
    fn test_different_categories_separate_counters() {
        let vault = TokenVault::in_memory().unwrap();

        let email_token = vault
            .store("EMAIL", "test@example.com", "session1")
            .unwrap();
        let phone_token = vault.store("PHONE", "555-1234", "session1").unwrap();

        assert_eq!(email_token, "[EMAIL_0001]");
        assert_eq!(phone_token, "[PHONE_0001]");
    }

    #[test]
    fn test_different_sessions_separate_counters() {
        let vault = TokenVault::in_memory().unwrap();

        let token1 = vault
            .store("EMAIL", "test1@example.com", "session1")
            .unwrap();
        let token2 = vault
            .store("EMAIL", "test2@example.com", "session2")
            .unwrap();

        assert_eq!(token1, "[EMAIL_0001]");
        assert_eq!(token2, "[EMAIL_0002]"); // Global counter, so second token gets 0002
    }

    #[test]
    fn test_clear_session() {
        let vault = TokenVault::in_memory().unwrap();

        vault
            .store("EMAIL", "test1@example.com", "session1")
            .unwrap();
        vault
            .store("EMAIL", "test2@example.com", "session1")
            .unwrap();
        vault
            .store("EMAIL", "test3@example.com", "session2")
            .unwrap();

        vault.clear_session("session1").unwrap();

        assert_eq!(vault.retrieve("[EMAIL_0001]"), None);
        assert_eq!(vault.retrieve("[EMAIL_0002]"), None);
        // session2's token should still be accessible
        assert_eq!(
            vault.retrieve("[EMAIL_0003]"),
            Some("test3@example.com".to_string())
        );
    }

    #[test]
    fn test_session_stats() {
        let vault = TokenVault::in_memory().unwrap();

        for i in 0..3 {
            vault
                .store("EMAIL", &format!("test{}@example.com", i), "session1")
                .unwrap();
        }
        for i in 0..2 {
            vault
                .store("PHONE", &format!("555-{:04}", i), "session1")
                .unwrap();
        }

        let stats = vault.session_stats("session1").unwrap();
        assert_eq!(stats.total_tokens, 5);
        assert_eq!(stats.by_category.get("EMAIL"), Some(&3));
        assert_eq!(stats.by_category.get("PHONE"), Some(&2));
    }
}
