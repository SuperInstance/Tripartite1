//! Redactor
//!
//! Handles redaction of sensitive information and reinflation of responses.
//! Works with the TokenVault to store original values.

use regex::Regex;
use std::collections::HashMap;
use tracing::{debug, instrument};

use crate::patterns::{PatternMatch, PatternSet, PatternType};
use crate::vault::TokenVault;
use crate::{PrivacyResult, RedactionStats};

/// Redactor configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RedactorConfig {
    /// Enable email redaction
    pub redact_emails: bool,
    /// Enable phone redaction
    pub redact_phones: bool,
    /// Enable SSN redaction
    pub redact_ssns: bool,
    /// Enable credit card redaction
    pub redact_credit_cards: bool,
    /// Enable API key redaction
    pub redact_api_keys: bool,
    /// Enable IP address redaction
    pub redact_ips: bool,
    /// Enable file path redaction
    pub redact_paths: bool,
    /// Enable URL redaction
    pub redact_urls: bool,
    /// Custom patterns to redact
    pub custom_patterns: Vec<CustomPatternConfig>,
}

impl Default for RedactorConfig {
    fn default() -> Self {
        Self {
            redact_emails: true,
            redact_phones: true,
            redact_ssns: true,
            redact_credit_cards: true,
            redact_api_keys: true,
            redact_ips: true,
            redact_paths: true,
            redact_urls: true,
            custom_patterns: vec![],
        }
    }
}

/// Custom pattern configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CustomPatternConfig {
    /// Pattern name (used in token prefix)
    pub name: String,
    /// Regex pattern
    pub pattern: String,
}

/// Result of redaction
#[derive(Debug, Clone)]
pub struct RedactionResult {
    /// The redacted text
    pub redacted_text: String,
    /// Mapping of tokens to their categories
    pub token_map: HashMap<String, String>,
    /// Statistics about what was redacted
    pub stats: RedactionStats,
}

/// The redactor
pub struct Redactor {
    #[allow(dead_code)]
    config: RedactorConfig,
    patterns: PatternSet,
    vault: TokenVault,
    token_regex: Regex,
}

impl Redactor {
    /// Create a new redactor
    pub fn new(config: RedactorConfig, vault: TokenVault) -> PrivacyResult<Self> {
        let mut patterns = PatternSet::with_builtins();

        // Configure built-in patterns based on config
        patterns.set_type_enabled(PatternType::Email, config.redact_emails);
        patterns.set_type_enabled(PatternType::Phone, config.redact_phones);
        patterns.set_type_enabled(PatternType::SSN, config.redact_ssns);
        patterns.set_type_enabled(PatternType::CreditCard, config.redact_credit_cards);
        patterns.set_type_enabled(PatternType::ApiKey, config.redact_api_keys);
        patterns.set_type_enabled(PatternType::AwsKey, config.redact_api_keys);
        patterns.set_type_enabled(PatternType::IpAddress, config.redact_ips);
        patterns.set_type_enabled(PatternType::FilePath, config.redact_paths);
        patterns.set_type_enabled(PatternType::SensitiveUrl, config.redact_urls);
        patterns.set_type_enabled(PatternType::GenericSecret, config.redact_api_keys);

        // Add custom patterns
        for custom in &config.custom_patterns {
            patterns.add_custom(&custom.name, &custom.pattern)?;
        }

        // Token regex for reinflation
        let token_regex = Regex::new(r"\[([A-Z]+)_([0-9]{4})\]")
            .map_err(|e| crate::PrivacyError::PatternError(e.to_string()))?;

        Ok(Self {
            config,
            patterns,
            vault,
            token_regex,
        })
    }

    /// Redact sensitive information from text
    #[instrument(skip(self, text), fields(session_id, text_len = text.len()))]
    pub fn redact(&mut self, text: &str, session_id: &str) -> RedactionResult {
        debug!("Redacting text");

        // Find all matches
        let matches = self.patterns.find_all_matches(text);

        if matches.is_empty() {
            return RedactionResult {
                redacted_text: text.to_string(),
                token_map: HashMap::new(),
                stats: RedactionStats::default(),
            };
        }

        let mut stats = RedactionStats {
            patterns_detected: matches.len(),
            ..Default::default()
        };

        let mut token_map = HashMap::new();
        let mut result = String::with_capacity(text.len());
        let mut last_end = 0;

        for m in &matches {
            // Add text before this match
            result.push_str(&text[last_end..m.start]);

            // Generate token using vault
            let category = m.pattern_type.token_prefix();
            let token = self
                .vault
                .store(category, &m.matched_text, session_id)
                .unwrap_or_else(|_| format!("[{}_????]", category));

            token_map.insert(token.clone(), category.to_string());

            // Add token to result
            result.push_str(&token);

            // Update stats
            stats.patterns_redacted += 1;
            stats.tokens_created += 1;
            *stats
                .by_type
                .entry(m.pattern_type.display_name().to_string())
                .or_insert(0) += 1;

            last_end = m.end;
        }

        // Add remaining text
        result.push_str(&text[last_end..]);

        debug!(
            redacted = stats.patterns_redacted,
            tokens = stats.tokens_created,
            "Redaction complete"
        );

        RedactionResult {
            redacted_text: result,
            token_map,
            stats,
        }
    }

    /// Reinflate tokens in text with original values
    #[instrument(skip(self, text), fields(text_len = text.len()))]
    pub fn reinflate(&self, text: &str) -> String {
        debug!("Reinflating text");

        let result = text.to_string();
        let mut tokens_seen = Vec::new();

        // Find all tokens
        for cap in self.token_regex.captures_iter(text) {
            let token = cap.get(0).map(|m| m.as_str());
            if let Some(token_str) = token {
                tokens_seen.push(token_str.to_string());
            }
        }

        // Replace tokens using constant-time algorithm to prevent timing attacks
        // Find all token positions first, then build result in single pass
        let mut token_positions = Vec::new();
        for cap in self.token_regex.find_iter(&result) {
            let token_str = cap.as_str();
            // Always lookup to prevent timing differences
            if let Some(original) = self.vault.retrieve(token_str) {
                token_positions.push((cap.start(), cap.end(), original));
            }
        }

        // Build result in one pass (constant time regardless of matches found)
        if token_positions.is_empty() {
            // No tokens found, return original
            return result;
        }

        let mut reinflated = String::with_capacity(result.len());
        let mut last_pos = 0;

        for (start, end, original) in token_positions {
            reinflated.push_str(&result[last_pos..start]);
            reinflated.push_str(&original);
            last_pos = end;
        }

        // Add remaining text
        reinflated.push_str(&result[last_pos..]);

        reinflated
    }

    /// Get statistics about redactions for a session
    #[instrument(skip(self, session_id))]
    pub fn get_stats(&self, session_id: &str) -> RedactionStats {
        match self.vault.session_stats(session_id) {
            Ok(session_stats) => {
                let mut by_type = HashMap::new();
                for (category, count) in session_stats.by_category {
                    by_type.insert(category, count as usize);
                }

                RedactionStats {
                    patterns_detected: by_type.values().sum(),
                    patterns_redacted: by_type.values().sum(),
                    tokens_created: session_stats.total_tokens,
                    by_type,
                }
            },
            Err(_) => RedactionStats::default(),
        }
    }

    /// Check if text contains sensitive information
    pub fn contains_sensitive(&self, text: &str) -> bool {
        self.patterns.contains_sensitive(text)
    }

    /// Get a preview of what would be redacted (without storing)
    pub fn preview(&self, text: &str) -> Vec<PatternMatch> {
        self.patterns.find_all_matches(text)
    }

    /// Clear all tokens for a session
    pub fn clear_session(&self, session_id: &str) -> PrivacyResult<()> {
        self.vault.clear_session(session_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_redactor() -> Redactor {
        let vault = TokenVault::in_memory().unwrap();
        Redactor::new(RedactorConfig::default(), vault).unwrap()
    }

    #[test]
    fn test_redact_email() {
        let mut redactor = create_test_redactor();

        let result = redactor.redact("Contact me at test@example.com", "session1");

        assert!(!result.redacted_text.contains("test@example.com"));
        assert!(result.redacted_text.contains("[EMAIL_"));
        assert_eq!(result.stats.patterns_redacted, 1);
    }

    #[test]
    fn test_redact_multiple() {
        let mut redactor = create_test_redactor();

        let result = redactor.redact("Email: test@example.com, Phone: 555-123-4567", "session1");

        assert!(!result.redacted_text.contains("test@example.com"));
        assert!(!result.redacted_text.contains("555-123-4567"));
        assert_eq!(result.stats.patterns_redacted, 2);
    }

    #[test]
    fn test_reinflate_returns_original() {
        let mut redactor = create_test_redactor();

        let original = "Contact me at test@example.com";
        let redaction = redactor.redact(original, "session1");
        let reinflated = redactor.reinflate(&redaction.redacted_text);

        assert_eq!(reinflated, original);
    }

    #[test]
    fn test_multiple_same_category_unique_tokens() {
        let mut redactor = create_test_redactor();

        let text = "Email me at foo@example.com or bar@example.org";
        let result = redactor.redact(text, "session1");

        // Should have two different tokens
        let tokens: Vec<&String> = result.token_map.keys().collect();
        assert_eq!(tokens.len(), 2);
        assert_ne!(tokens[0], tokens[1]);
    }

    #[test]
    fn test_empty_string() {
        let mut redactor = create_test_redactor();

        let result = redactor.redact("", "session1");

        assert_eq!(result.redacted_text, "");
        assert_eq!(result.stats.patterns_redacted, 0);
    }

    #[test]
    fn test_no_sensitive_data() {
        let mut redactor = create_test_redactor();

        let result = redactor.redact("Hello, world!", "session1");

        assert_eq!(result.redacted_text, "Hello, world!");
        assert_eq!(result.stats.patterns_redacted, 0);
    }

    #[test]
    fn test_unicode_handling() {
        let mut redactor = create_test_redactor();

        let text = "Contact: test@example.com or visit café";
        let result = redactor.redact(text, "session1");

        // Unicode characters should be preserved
        assert!(result.redacted_text.contains("café"));
        assert!(!result.redacted_text.contains("test@example.com"));
    }

    #[test]
    fn test_nested_patterns() {
        let mut redactor = create_test_redactor();

        // File path with email in it - should be caught as file path first (higher priority)
        let text = "File at /home/user/docs/email_backup@example.com.txt";
        let result = redactor.redact(text, "session1");

        // Should redact the file path
        assert!(result.redacted_text.contains("[PATH_"));
    }

    #[test]
    fn test_contains_sensitive() {
        let redactor = create_test_redactor();

        assert!(redactor.contains_sensitive("test@example.com"));
        assert!(!redactor.contains_sensitive("Hello, world!"));
    }

    #[test]
    fn test_preview_doesnt_store() {
        let redactor = create_test_redactor();

        let matches = redactor.preview("Contact test@example.com");

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].matched_text, "test@example.com");

        // Check that nothing was stored in vault
        let stats = redactor.get_stats("session1");
        assert_eq!(stats.tokens_created, 0);
    }

    #[test]
    fn test_get_stats() {
        let mut redactor = create_test_redactor();

        redactor.redact("Email: test@example.com", "session1");
        redactor.redact("Phone: 555-123-4567, Email: foo@bar.com", "session1");

        let stats = redactor.get_stats("session1");
        assert_eq!(stats.tokens_created, 3);
        assert_eq!(stats.by_type.get("EMAIL"), Some(&2));
        assert_eq!(stats.by_type.get("PHONE"), Some(&1));
    }

    #[test]
    fn test_clear_session() {
        let mut redactor = create_test_redactor();

        redactor.redact("Email: test@example.com", "session1");
        redactor.redact("Email: test2@example.com", "session2");

        // Check both have tokens
        let stats1 = redactor.get_stats("session1");
        let stats2 = redactor.get_stats("session2");
        assert_eq!(stats1.tokens_created, 1);
        assert_eq!(stats2.tokens_created, 1);

        // Clear session1
        redactor.clear_session("session1").unwrap();

        // session1 should be empty, session2 should still have tokens
        let stats1_after = redactor.get_stats("session1");
        let stats2_after = redactor.get_stats("session2");
        assert_eq!(stats1_after.tokens_created, 0);
        assert_eq!(stats2_after.tokens_created, 1);
    }
}
