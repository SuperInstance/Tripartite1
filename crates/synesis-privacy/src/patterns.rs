//! Pattern Detection
//!
//! Defines patterns for detecting sensitive information in text.
//! Includes built-in patterns for common PII types and support for custom patterns.

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::PrivacyResult;

/// Types of patterns we can detect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PatternType {
    /// Email addresses
    Email,
    /// Phone numbers (various formats)
    Phone,
    /// Social Security Numbers
    SSN,
    /// Credit card numbers
    CreditCard,
    /// API keys and tokens
    ApiKey,
    /// IP addresses (v4 and v6)
    IpAddress,
    /// File paths
    FilePath,
    /// URLs with potential sensitive params
    SensitiveUrl,
    /// AWS access keys
    AwsKey,
    /// Generic secrets (passwords in context)
    GenericSecret,
    /// Custom user-defined pattern
    Custom,
}

impl PatternType {
    /// Get the token prefix for this type
    pub fn token_prefix(&self) -> &'static str {
        match self {
            PatternType::Email => "EMAIL",
            PatternType::Phone => "PHONE",
            PatternType::SSN => "SSN",
            PatternType::CreditCard => "CARD",
            PatternType::ApiKey => "APIKEY",
            PatternType::IpAddress => "IP",
            PatternType::FilePath => "PATH",
            PatternType::SensitiveUrl => "URL",
            PatternType::AwsKey => "AWSKEY",
            PatternType::GenericSecret => "SECRET",
            PatternType::Custom => "CUSTOM",
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            PatternType::Email => "Email Address",
            PatternType::Phone => "Phone Number",
            PatternType::SSN => "Social Security Number",
            PatternType::CreditCard => "Credit Card",
            PatternType::ApiKey => "API Key",
            PatternType::IpAddress => "IP Address",
            PatternType::FilePath => "File Path",
            PatternType::SensitiveUrl => "Sensitive URL",
            PatternType::AwsKey => "AWS Access Key",
            PatternType::GenericSecret => "Secret/Password",
            PatternType::Custom => "Custom Pattern",
        }
    }
}

/// A compiled pattern for detection
#[derive(Clone)]
pub struct Pattern {
    /// Pattern type
    pub pattern_type: PatternType,
    /// Pattern name (for custom patterns)
    pub name: String,
    /// Compiled regex
    regex: Regex,
    /// Whether this pattern is enabled
    pub enabled: bool,
    /// Priority (higher = checked first)
    pub priority: u8,
}

impl Pattern {
    /// Create a new pattern
    pub fn new(pattern_type: PatternType, name: &str, regex_str: &str) -> PrivacyResult<Self> {
        let regex =
            Regex::new(regex_str).map_err(|e| crate::PrivacyError::PatternError(e.to_string()))?;

        Ok(Self {
            pattern_type,
            name: name.to_string(),
            regex,
            enabled: true,
            priority: 50,
        })
    }

    /// Create with priority
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Find all matches in text
    pub fn find_matches(&self, text: &str) -> Vec<PatternMatch> {
        if !self.enabled {
            return vec![];
        }

        self.regex
            .find_iter(text)
            .map(|m| PatternMatch {
                pattern_type: self.pattern_type,
                pattern_name: self.name.clone(),
                matched_text: m.as_str().to_string(),
                start: m.start(),
                end: m.end(),
            })
            .collect()
    }

    /// Check if text contains this pattern
    pub fn is_match(&self, text: &str) -> bool {
        self.enabled && self.regex.is_match(text)
    }
}

impl std::fmt::Debug for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pattern")
            .field("pattern_type", &self.pattern_type)
            .field("name", &self.name)
            .field("enabled", &self.enabled)
            .field("priority", &self.priority)
            .finish()
    }
}

/// A match found in text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatch {
    /// Type of pattern matched
    pub pattern_type: PatternType,
    /// Name of the pattern
    pub pattern_name: String,
    /// The matched text
    pub matched_text: String,
    /// Start position in original text
    pub start: usize,
    /// End position in original text
    pub end: usize,
}

/// Built-in pattern definitions
pub struct BuiltinPatterns;

impl BuiltinPatterns {
    /// Get all built-in patterns
    ///
    /// Patterns are returned with priority ordering to ensure proper detection:
    /// - Private keys (priority 100): Highest security, very specific format
    /// - SSNs (priority 95): High security, specific format
    /// - API keys/secrets (priority 85-93): Various service keys
    /// - Credit cards (priority 90): High security, specific format
    /// - URLs with tokens (priority 75): Before emails to avoid double-detection
    /// - Emails (priority 80): Common pattern
    /// - Phones (priority 65-70): Various formats
    /// - IP addresses (priority 60): Network identifiers
    /// - File paths (priority 50): System-specific paths
    /// - Generic secrets (priority 85): Passwords in context
    pub fn all() -> Vec<Pattern> {
        let mut patterns: Vec<Pattern> = [
            // Priority 100: Private keys (most critical)
            Self::private_key(),
            // Priority 95: SSNs (critical PII)
            Self::ssn(),
            // Priority 90-94: Credentials and keys
            Self::credit_card(),
            Self::api_key_github(),
            Self::api_key_sk(),
            Self::aws_access_key(),
            Self::aws_secret_key(),
            Self::github_token(),
            Self::slack_token(),
            // Priority 80-89: API keys and secrets
            Self::api_key_generic(),
            Self::password_in_context(),
            Self::email(),
            // Priority 70-79: URLs with tokens
            Self::url_with_token(),
            // Priority 65-70: Phone numbers
            Self::phone_us(),
            Self::phone_international(),
            // Priority 60: IP addresses
            Self::ipv4(),
            Self::ipv6(),
            // Priority 50: File paths
            Self::file_path_unix(),
            Self::file_path_windows(),
        ]
        .into_iter()
        .flatten()
        .collect();

        // Sort by priority (highest first)
        patterns.sort_by(|a, b| b.priority.cmp(&a.priority));
        patterns
    }

    /// Email pattern
    pub fn email() -> Option<Pattern> {
        Pattern::new(
            PatternType::Email,
            "email",
            r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}",
        )
        .ok()
        .map(|p| p.with_priority(80))
    }

    /// US phone number
    pub fn phone_us() -> Option<Pattern> {
        Pattern::new(
            PatternType::Phone,
            "phone_us",
            r"(?:\+1[-.\s]?)?\(?[0-9]{3}\)?[-.\s]?[0-9]{3}[-.\s]?[0-9]{4}",
        )
        .ok()
        .map(|p| p.with_priority(70))
    }

    /// International phone number
    pub fn phone_international() -> Option<Pattern> {
        Pattern::new(PatternType::Phone, "phone_intl", r"\+[1-9][0-9]{1,14}")
            .ok()
            .map(|p| p.with_priority(65))
    }

    /// Social Security Number
    pub fn ssn() -> Option<Pattern> {
        Pattern::new(
            PatternType::SSN,
            "ssn",
            r"\b[0-9]{3}[-\s]?[0-9]{2}[-\s]?[0-9]{4}\b",
        )
        .ok()
        .map(|p| p.with_priority(95))
    }

    /// Credit card number (Luhn-valid patterns)
    pub fn credit_card() -> Option<Pattern> {
        Pattern::new(
            PatternType::CreditCard,
            "credit_card",
            r"\b(?:4[0-9]{12}(?:[0-9]{3})?|5[1-5][0-9]{14}|3[47][0-9]{13}|6(?:011|5[0-9]{2})[0-9]{12})\b",
        )
        .ok()
        .map(|p| p.with_priority(90))
    }

    /// Generic API key pattern
    pub fn api_key_generic() -> Option<Pattern> {
        Pattern::new(
            PatternType::ApiKey,
            "api_key",
            r#"(?i)(?:api[_-]?key|apikey|api[_-]?token)[=:\s]+['\''"]?([a-zA-Z0-9_-]{20,})['\''"]?"#,
        )
        .ok()
        .map(|p| p.with_priority(85))
    }

    /// AWS Access Key ID
    pub fn aws_access_key() -> Option<Pattern> {
        Pattern::new(
            PatternType::AwsKey,
            "aws_access_key",
            r"(?:A3T[A-Z0-9]|AKIA|AGPA|AIDA|AROA|AIPA|ANPA|ANVA|ASIA)[A-Z0-9]{16}",
        )
        .ok()
        .map(|p| p.with_priority(90))
    }

    /// AWS Secret Key
    pub fn aws_secret_key() -> Option<Pattern> {
        Pattern::new(
            PatternType::AwsKey,
            "aws_secret_key",
            r#"(?i)(?:aws[_-]?secret[_-]?(?:access[_-]?)?key)[=:\s]+['\''"]?([A-Za-z0-9/+=]{40})['\''"]?"#,
        )
        .ok()
        .map(|p| p.with_priority(90))
    }

    /// IPv4 address
    pub fn ipv4() -> Option<Pattern> {
        Pattern::new(
            PatternType::IpAddress,
            "ipv4",
            r"\b(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\b",
        )
        .ok()
        .map(|p| p.with_priority(60))
    }

    /// IPv6 address
    pub fn ipv6() -> Option<Pattern> {
        Pattern::new(
            PatternType::IpAddress,
            "ipv6",
            r"(?i)(?:[0-9a-f]{1,4}:){7}[0-9a-f]{1,4}|(?:[0-9a-f]{1,4}:){1,7}:|(?:[0-9a-f]{1,4}:){1,6}:[0-9a-f]{1,4}|(?:[0-9a-f]{1,4}:){1,5}(?::[0-9a-f]{1,4}){1,2}|(?:[0-9a-f]{1,4}:){1,4}(?::[0-9a-f]{1,4}){1,3}|(?:[0-9a-f]{1,4}:){1,3}(?::[0-9a-f]{1,4}){1,4}|(?:[0-9a-f]{1,4}:){1,2}(?::[0-9a-f]{1,4}){1,5}|[0-9a-f]{1,4}:(?::[0-9a-f]{1,4}){1,6}|:(?::[0-9a-f]{1,4}){1,7}|::",
        )
        .ok()
        .map(|p| p.with_priority(60))
    }

    /// Unix file path
    pub fn file_path_unix() -> Option<Pattern> {
        Pattern::new(
            PatternType::FilePath,
            "path_unix",
            r"(?:/(?:home|users|var|etc|opt|usr)/[a-zA-Z0-9._/-]+)",
        )
        .ok()
        .map(|p| p.with_priority(50))
    }

    /// Windows file path
    pub fn file_path_windows() -> Option<Pattern> {
        Pattern::new(
            PatternType::FilePath,
            "path_windows",
            r#"(?i)(?:[a-z]:\\(?:[^\\/:*?"<>|\r\n]+\\)*[^\\/:*?"<>|\r\n]*)"#,
        )
        .ok()
        .map(|p| p.with_priority(50))
    }

    /// URL with token/key parameter
    pub fn url_with_token() -> Option<Pattern> {
        Pattern::new(
            PatternType::SensitiveUrl,
            "url_token",
            r"(?i)https?://[^\s]+[?&](?:token|key|api_key|apikey|secret|password|auth)=[^\s&]+",
        )
        .ok()
        .map(|p| p.with_priority(75))
    }

    /// Password in context
    pub fn password_in_context() -> Option<Pattern> {
        Pattern::new(
            PatternType::GenericSecret,
            "password",
            r#"(?i)(?:password|passwd|pwd)[=:\s]+['\''"]?([^\s'\''"]{4,})['\''"]?"#,
        )
        .ok()
        .map(|p| p.with_priority(85))
    }

    /// Private key (PEM format)
    pub fn private_key() -> Option<Pattern> {
        Pattern::new(
            PatternType::GenericSecret,
            "private_key",
            r#"-----BEGIN[A-Z\s]*PRIVATE KEY-----"#,
        )
        .ok()
        .map(|p| p.with_priority(100))
    }

    /// GitHub personal access token
    pub fn github_token() -> Option<Pattern> {
        Pattern::new(
            PatternType::ApiKey,
            "github_token",
            r#"(?i)github[_-]?token[=:\s]+['"]?([a-zA-Z0-9_-]{36,})['"]?"#,
        )
        .ok()
        .map(|p| p.with_priority(90))
    }

    /// GitHub token format (ghp_*, gho_*, ghu_*, ghs_*, ghr_*)
    pub fn api_key_github() -> Option<Pattern> {
        Pattern::new(
            PatternType::ApiKey,
            "github_api_key",
            r"(?i)(?:ghp_|gho_|ghu_|ghs_|ghr_)[a-zA-Z0-9]{36}",
        )
        .ok()
        .map(|p| p.with_priority(92))
    }

    /// Slack token
    pub fn slack_token() -> Option<Pattern> {
        Pattern::new(
            PatternType::ApiKey,
            "slack_token",
            r#"(?i)slack[_-]?token[=:\s]+['"]?([a-zA-Z0-9_-]{24,})['"]?"#,
        )
        .ok()
        .map(|p| p.with_priority(90))
    }

    /// API key with sk- prefix (Stripe, OpenAI, etc.)
    pub fn api_key_sk() -> Option<Pattern> {
        Pattern::new(
            PatternType::ApiKey,
            "api_key_sk",
            r#"\bsk[_-][a-zA-Z0-9_-]{20,}\b"#,
        )
        .ok()
        .map(|p| p.with_priority(93))
    }
}

/// Pattern set for batch detection
#[derive(Debug)]
pub struct PatternSet {
    patterns: Vec<Pattern>,
}

impl PatternSet {
    /// Create a new empty pattern set
    pub fn new() -> Self {
        Self { patterns: vec![] }
    }

    /// Create with built-in patterns
    pub fn with_builtins() -> Self {
        Self {
            patterns: BuiltinPatterns::all(),
        }
    }

    /// Add a pattern
    pub fn add(&mut self, pattern: Pattern) {
        self.patterns.push(pattern);
        // Sort by priority (highest first)
        self.patterns.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Add a custom pattern
    pub fn add_custom(&mut self, name: &str, regex: &str) -> PrivacyResult<()> {
        let pattern = Pattern::new(PatternType::Custom, name, regex)?;
        self.add(pattern);
        Ok(())
    }

    /// Enable/disable a pattern by name
    pub fn set_enabled(&mut self, name: &str, enabled: bool) {
        for pattern in &mut self.patterns {
            if pattern.name == name {
                pattern.enabled = enabled;
            }
        }
    }

    /// Enable/disable patterns by type
    pub fn set_type_enabled(&mut self, pattern_type: PatternType, enabled: bool) {
        for pattern in &mut self.patterns {
            if pattern.pattern_type == pattern_type {
                pattern.enabled = enabled;
            }
        }
    }

    /// Find all matches in text
    pub fn find_all_matches(&self, text: &str) -> Vec<PatternMatch> {
        let mut all_matches = vec![];

        for pattern in &self.patterns {
            all_matches.extend(pattern.find_matches(text));
        }

        // Sort by position
        all_matches.sort_by_key(|m| m.start);

        // Remove overlapping matches (keep higher priority)
        let mut filtered = vec![];
        let mut last_end = 0;

        for m in all_matches {
            if m.start >= last_end {
                last_end = m.end;
                filtered.push(m);
            }
        }

        filtered
    }

    /// Check if text contains any pattern
    pub fn contains_sensitive(&self, text: &str) -> bool {
        self.patterns.iter().any(|p| p.is_match(text))
    }

    /// Get pattern count
    pub fn len(&self) -> usize {
        self.patterns.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }
}

impl Default for PatternSet {
    fn default() -> Self {
        Self::with_builtins()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_detection() {
        let pattern = BuiltinPatterns::email().unwrap();

        assert!(pattern.is_match("contact me at test@example.com please"));
        assert!(!pattern.is_match("no email here"));

        let matches = pattern.find_matches("email me at foo@bar.com or baz@qux.org");
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_phone_detection() {
        let pattern = BuiltinPatterns::phone_us().unwrap();

        assert!(pattern.is_match("Call me at 555-123-4567"));
        assert!(pattern.is_match("Phone: (555) 123-4567"));
        assert!(pattern.is_match("+1 555 123 4567"));
    }

    #[test]
    fn test_ssn_detection() {
        let pattern = BuiltinPatterns::ssn().unwrap();

        assert!(pattern.is_match("SSN: 123-45-6789"));
        assert!(pattern.is_match("123 45 6789"));
    }

    #[test]
    fn test_api_key_detection() {
        let pattern = BuiltinPatterns::api_key_generic().unwrap();

        assert!(pattern.is_match("API_KEY=sk_test_1234567890abcdefghij"));
        assert!(pattern.is_match("api-key: abcdefghijklmnopqrstuvwxyz"));
    }

    #[test]
    fn test_aws_key_detection() {
        let pattern = BuiltinPatterns::aws_access_key().unwrap();

        assert!(pattern.is_match("AKIAIOSFODNN7EXAMPLE"));
    }

    #[test]
    fn test_pattern_set() {
        let set = PatternSet::with_builtins();

        let text = "Contact john@example.com or call 555-123-4567";
        let matches = set.find_all_matches(text);

        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].pattern_type, PatternType::Email);
        assert_eq!(matches[1].pattern_type, PatternType::Phone);
    }

    #[test]
    fn test_custom_pattern() {
        let mut set = PatternSet::new();
        set.add_custom("employee_id", r"EMP-[0-9]{6}").unwrap();

        let matches = set.find_all_matches("Employee EMP-123456 started today");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].matched_text, "EMP-123456");
    }

    // Tests for newly added patterns

    #[test]
    fn test_private_key_detection() {
        let pattern = BuiltinPatterns::private_key().unwrap();

        // RSA private key
        assert!(pattern.is_match("-----BEGIN RSA PRIVATE KEY-----"));
        // EC private key
        assert!(pattern.is_match("-----BEGIN EC PRIVATE KEY-----"));
        // OpenSSH private key
        assert!(pattern.is_match("-----BEGIN OPENSSH PRIVATE KEY-----"));
        // Generic private key
        assert!(pattern.is_match("-----BEGIN PRIVATE KEY-----"));

        assert!(!pattern.is_match("-----BEGIN PUBLIC KEY-----"));
    }

    #[test]
    fn test_github_token_detection() {
        let pattern = BuiltinPatterns::api_key_github().unwrap();

        // GitHub personal access token (ghp_) - 36 chars after prefix
        assert!(pattern.is_match("ghp_1234567890abcdefghijklmnopqrstuvwxyz123456"));
        // GitHub OAuth token (gho_)
        assert!(pattern.is_match("gho_1234567890abcdefghijklmnopqrstuvwxyz123456"));
        // GitHub user token (ghu_)
        assert!(pattern.is_match("ghu_1234567890abcdefghijklmnopqrstuvwxyz123456"));
        // GitHub server token (ghs_)
        assert!(pattern.is_match("ghs_1234567890abcdefghijklmnopqrstuvwxyz123456"));
        // GitHub refresh token (ghr_)
        assert!(pattern.is_match("ghr_1234567890abcdefghijklmnopqrstuvwxyz123456"));

        // Not a GitHub token (too short)
        assert!(!pattern.is_match("ghp_tooshort"));
    }

    #[test]
    fn test_slack_token_detection() {
        let pattern = BuiltinPatterns::slack_token().unwrap();

        assert!(pattern
            .is_match("slack_token=xoxb-123456789012-1234567890123-AbCdEfGhIjKlMnOpQrStUvWx"));
        assert!(pattern.is_match("SLACK-TOKEN: xoxp-123456789012-1234567890123-1234567890123-12345678901234567890123456789012"));
    }

    #[test]
    fn test_sk_api_key_detection() {
        let pattern = BuiltinPatterns::api_key_sk().unwrap();

        // Stripe key
        assert!(pattern.is_match("sk_test_1234567890abcdefghijklmnopqrst"));
        // OpenAI key
        assert!(pattern.is_match("sk-1234567890abcdefghijklmnopqrst"));

        // Too short
        assert!(!pattern.is_match("sk-short"));
    }

    #[test]
    fn test_credit_card_detection() {
        let pattern = BuiltinPatterns::credit_card().unwrap();

        // Visa
        assert!(pattern.is_match("4111111111111111"));
        // MasterCard
        assert!(pattern.is_match("5555555555554444"));
        // American Express
        assert!(pattern.is_match("378282246310005"));
        // Discover
        assert!(pattern.is_match("6011111111111117"));
    }

    #[test]
    fn test_ipv6_detection() {
        let pattern = BuiltinPatterns::ipv6().unwrap();

        assert!(pattern.is_match("2001:0db8:85a3:0000:0000:8a2e:0370:7334"));
        assert!(pattern.is_match("2001:db8:85a3::8a2e:370:7334"));
    }

    #[test]
    fn test_priority_ordering() {
        let set = PatternSet::with_builtins();

        // Verify patterns are sorted by priority
        let priorities: Vec<u8> = set.patterns.iter().map(|p| p.priority).collect();

        // Check that priorities are in descending order
        for i in 1..priorities.len() {
            assert!(
                priorities[i - 1] >= priorities[i],
                "Patterns not sorted by priority"
            );
        }
    }

    #[test]
    fn test_comprehensive_redaction_scenario() {
        let set = PatternSet::with_builtins();

        let text = r#"
            Contact: john@example.com
            Phone: 555-123-4567
            SSN: 123-45-6789
            API Key: sk_test_1234567890abcdefghijklmnopqrst
            GitHub: ghp_1234567890abcdefghijklmnopqrstuvwxyz123456
            AWS Key: AKIAIOSFODNN7EXAMPLE
            File: /home/user/secrets.txt
            Private Key:
            -----BEGIN RSA PRIVATE KEY-----
            Card: 4111111111111111
            IP: 192.168.1.1
            URL: https://api.example.com?token=abc123def456
        "#;

        let matches = set.find_all_matches(text);

        // Should detect multiple patterns
        assert!(
            matches.len() >= 8,
            "Expected at least 8 matches, got {}",
            matches.len()
        );

        // Check that specific patterns were found
        let pattern_types: Vec<PatternType> = matches.iter().map(|m| m.pattern_type).collect();

        assert!(pattern_types.contains(&PatternType::Email));
        assert!(pattern_types.contains(&PatternType::Phone));
        assert!(pattern_types.contains(&PatternType::SSN));
        assert!(pattern_types.contains(&PatternType::ApiKey));
        assert!(pattern_types.contains(&PatternType::AwsKey));
        assert!(pattern_types.contains(&PatternType::FilePath));
        assert!(pattern_types.contains(&PatternType::CreditCard));
        assert!(pattern_types.contains(&PatternType::IpAddress));
    }

    #[test]
    fn test_pattern_enable_disable() {
        let mut set = PatternSet::with_builtins();

        // Initially, email detection should work
        assert!(set.contains_sensitive("test@example.com"));

        // Disable email detection
        set.set_type_enabled(PatternType::Email, false);

        // Now email should not be detected
        assert!(!set.contains_sensitive("test@example.com"));

        // But other patterns should still work
        assert!(set.contains_sensitive("555-123-4567"));
    }

    #[test]
    fn test_overlapping_matches() {
        let set = PatternSet::with_builtins();

        // Text that might have overlapping patterns
        let text = "Email user@host.com and API key sk-12345678901234567890";

        let matches = set.find_all_matches(text);

        // Should not have overlapping matches
        for i in 1..matches.len() {
            assert!(
                matches[i].start >= matches[i - 1].end,
                "Found overlapping matches: {:?} and {:?}",
                matches[i - 1],
                matches[i]
            );
        }
    }
}
