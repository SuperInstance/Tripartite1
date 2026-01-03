//! Test 2: Privacy redaction round-trip
//!
//! Tests the complete privacy pipeline:
//! - Input with email, API key, file path
//! - Verify redacted before agents
//! - Verify reinflated in response

use std::collections::HashMap;
use tempfile::TempDir;

use synesis_privacy::{
    redactor::{Redactor, RedactorConfig},
    vault::TokenVault,
    PatternType,
};

/// Test complete privacy round-trip
#[tokio::test]
async fn test_privacy_roundtrip_complete() {
    // Create temporary vault
    let temp_dir = TempDir::new().unwrap();
    let vault_path = temp_dir.path().join("vault.db");
    let vault = TokenVault::new(&vault_path).unwrap();

    // Create redactor
    let mut redactor = Redactor::new(RedactorConfig::default(), vault).unwrap();

    // Input with multiple sensitive items
    let original_input = r#"
My email is john.doe@example.com and my API key is sk-1234567890abcdef.
Please check the file at /home/user/config.json for details.
You can also call me at 555-123-4567.
"#;

    // Step 1: Redact
    let session_id = "test_session_001";
    let redaction_result = redactor.redact(original_input, session_id);

    println!("Original: {}", original_input);
    println!("Redacted: {}", redaction_result.redacted_text);

    // Verify sensitive data is redacted
    assert!(!redaction_result.redacted_text.contains("john.doe@example.com"));
    assert!(!redaction_result.redacted_text.contains("sk-1234567890abcdef"));
    assert!(!redaction_result.redacted_text.contains("/home/user/config.json"));
    assert!(!redaction_result.redacted_text.contains("555-123-4567"));

    // Verify tokens are present
    assert!(redaction_result.redacted_text.contains("[EMAIL_"));
    assert!(redaction_result.redacted_text.contains("[APIKEY_"));
    assert!(redaction_result.redacted_text.contains("[PATH_"));
    assert!(redaction_result.redacted_text.contains("[PHONE_"));

    // Verify statistics
    assert_eq!(redaction_result.stats.patterns_redacted, 4);

    // Step 2: Reinflate
    let reinflated = redactor.reinflate(&redaction_result.redacted_text);

    println!("Reinflated: {}", reinflated);

    // Verify round-trip completeness
    assert_eq!(reinflated.trim(), original_input.trim());

    println!("✓ Privacy round-trip completed successfully");
    println!("  - Patterns detected: {}", redaction_result.stats.patterns_detected);
    println!("  - Tokens created: {}", redaction_result.stats.tokens_created);
}

/// Test email redaction specifically
#[tokio::test]
async fn test_email_redaction() {
    let vault = TokenVault::in_memory().unwrap();
    let mut redactor = Redactor::new(RedactorConfig::default(), vault).unwrap();

    let input = "Contact me at test@example.com for details.";
    let result = redactor.redact(input, "test_session");

    assert!(!result.redacted_text.contains("test@example.com"));
    assert!(result.redacted_text.contains("[EMAIL_"));

    // Verify reinflation
    let reinflated = redactor.reinflate(&result.redacted_text);
    assert_eq!(reinflated, input);

    println!("✓ Email redaction verified");
}

/// Test API key redaction
#[tokio::test]
async fn test_api_key_redaction() {
    let vault = TokenVault::in_memory().unwrap();
    let mut redactor = Redactor::new(RedactorConfig::default(), vault).unwrap();

    let inputs = vec![
        "API_KEY=sk-1234567890abcdefghijklmnop",
        "secret: ghp_1234567890abcdefghijklmnop",
        "token: Bearer abcdefghijklmnopqrstuvwxyz123456",
    ];

    for input in inputs {
        let result = redactor.redact(input, "test_session");

        // Should redact API keys
        assert!(result.redacted_text.contains("[SECRET_") ||
                result.redacted_text.contains("[APIKEY_") ||
                result.redacted_text.contains("[TOKEN_"),
                "Should redact API key in: {}", input);

        // Verify reinflation
        let reinflated = redactor.reinflate(&result.redacted_text);
        assert_eq!(reinflated, input);
    }

    println!("✓ API key redaction verified");
}

/// Test file path redaction
#[tokio::test]
async fn test_file_path_redaction() {
    let vault = TokenVault::in_memory().unwrap();
    let mut redactor = Redactor::new(RedactorConfig::default(), vault).unwrap();

    let input = "Check /home/user/documents/file.txt or C:\\Users\\Admin\\config.ini";
    let result = redactor.redact(input, "test_session");

    assert!(!result.redacted_text.contains("/home/user/documents/file.txt"));
    assert!(!result.redacted_text.contains("C:\\Users\\Admin\\config.ini"));
    assert!(result.redacted_text.contains("[PATH_"));

    // Verify reinflation
    let reinflated = redactor.reinflate(&result.redacted_text);
    assert_eq!(reinflated, input);

    println!("✓ File path redaction verified");
}

/// Test multiple instances of same pattern type
#[tokio::test]
async fn test_multiple_same_type() {
    let vault = TokenVault::in_memory().unwrap();
    let mut redactor = Redactor::new(RedactorConfig::default(), vault).unwrap();

    let input = "Email me at test1@example.com or test2@example.com or admin@example.org";
    let result = redactor.redact(input, "test_session");

    // All three emails should be redacted
    assert!(!result.redacted_text.contains("test1@example.com"));
    assert!(!result.redacted_text.contains("test2@example.com"));
    assert!(!result.redacted_text.contains("admin@example.org"));

    // Should have three different tokens
    let tokens: Vec<&str> = result.redacted_text
        .split_whitespace()
        .filter(|s| s.contains("[EMAIL_"))
        .collect();

    assert_eq!(tokens.len(), 3);

    // Verify each token is unique
    let mut unique_tokens = std::collections::HashSet::new();
    for token in tokens {
        unique_tokens.insert(token);
    }
    assert_eq!(unique_tokens.len(), 3);

    // Verify reinflation
    let reinflated = redactor.reinflate(&result.redacted_text);
    assert_eq!(reinflated, input);

    println!("✓ Multiple same-type redaction verified");
}

/// Test sensitive data detection
#[test]
fn test_sensitive_detection() {
    let vault = TokenVault::in_memory().unwrap();
    let redactor = Redactor::new(RedactorConfig::default(), vault).unwrap();

    // Should detect sensitive data
    assert!(redactor.contains_sensitive("email@example.com"));
    assert!(redactor.contains_sensitive("555-123-4567"));
    assert!(redactor.contains_sensitive("/home/user/file.txt"));
    assert!(redactor.contains_sensitive("sk-1234567890abcdef"));

    // Should not detect in clean text
    assert!(!redactor.contains_sensitive("Hello, world!"));
    assert!(!redactor.contains_sensitive("No secrets here"));

    println!("✓ Sensitive data detection verified");
}

/// Test vault statistics
#[tokio::test]
async fn test_vault_statistics() {
    let vault = TokenVault::in_memory().unwrap();
    let mut redactor = Redactor::new(RedactorConfig::default(), vault).unwrap();

    // Redact multiple items
    let session = "test_session_stats";
    let input1 = "Email: test1@example.com";
    let input2 = "Email: test2@example.com, Phone: 555-123-4567";

    redactor.redact(input1, session);
    redactor.redact(input2, session);

    // Get statistics
    let stats = redactor.get_stats(session);
    assert!(stats.patterns_redacted >= 3); // At least 2 emails + 1 phone

    println!("✓ Vault statistics verified");
    println!("  - Patterns redacted: {}", stats.patterns_redacted);
    println!("  - By type: {:?}", stats.by_type);
}

/// Test privacy with no sensitive data
#[tokio::test]
async fn test_no_sensitive_data() {
    let vault = TokenVault::in_memory().unwrap();
    let mut redactor = Redactor::new(RedactorConfig::default(), vault).unwrap();

    let clean_input = "Hello! How can I help you today?";
    let result = redactor.redact(clean_input, "test_session");

    // Should remain unchanged
    assert_eq!(result.redacted_text, clean_input);
    assert_eq!(result.stats.patterns_redacted, 0);
    assert!(result.token_map.is_empty());

    println!("✓ Clean text handling verified");
}

/// Test session cleanup
#[tokio::test]
async fn test_session_cleanup() {
    let vault = TokenVault::in_memory().unwrap();
    let mut redactor = Redactor::new(RedactorConfig::default(), vault).unwrap();

    let session1 = "session_1";
    let session2 = "session_2";

    // Add tokens to both sessions
    redactor.redact("Email: test1@example.com", session1);
    redactor.redact("Email: test2@example.com", session2);

    // Clear session 1
    redactor.clear_session(session1).unwrap();

    // Verify session 1 stats are empty
    let stats1 = redactor.get_stats(session1);
    assert_eq!(stats1.patterns_redacted, 0);

    // Verify session 2 still has data
    let stats2 = redactor.get_stats(session2);
    assert!(stats2.patterns_redacted > 0);

    println!("✓ Session cleanup verified");
}
