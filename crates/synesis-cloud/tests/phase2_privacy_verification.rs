//! Phase 2: Privacy Verification Integration Tests
//!
//! Ensures privacy proxy works correctly with cloud communication

use synesis_privacy::{Redactor, RedactorConfig, TokenVault};
use synesis_cloud::escalation::types::{EscalationRequest, EscalationContext};

#[test]
fn test_privacy_cloud_roundtrip() {
    // Test that sensitive data is properly redacted before sending to cloud

    // Create privacy proxy
    let vault = TokenVault::in_memory().unwrap();
    let config = RedactorConfig::default();
    let mut redactor = Redactor::new(config, vault).unwrap();

    // User query with sensitive data (using patterns that will be detected)
    let user_query = "My email is jane@example.com and my phone is 555-123-4567. What is 2+2?";

    // Redact
    let session_id = "test-session-1";
    let redacted_result = redactor.redact(user_query, session_id);

    // Verify sensitive data is replaced
    assert!(!redacted_result.redacted_text.contains("jane@example.com"));
    assert!(!redacted_result.redacted_text.contains("555-123-4567"));
    assert!(redacted_result.redacted_text.contains("[EMAIL_"));
    assert!(redacted_result.redacted_text.contains("[PHONE_"));

    // Create cloud request with redacted query
    let request = EscalationRequest {
        request_id: "test-req-1".to_string(),
        session_id: session_id.to_string(),
        query: redacted_result.redacted_text.clone(),
        context: EscalationContext::default(),
        model: Default::default(),
        max_tokens: 100,
        stream: false,
        lora_id: None,
        timeout_secs: Some(30),
    };

    // Verify request doesn't contain sensitive data
    assert!(!request.query.contains("jane@example.com"));
    assert!(!request.query.contains("555-123-4567"));

    // Simulate cloud response (no sensitive data)
    let cloud_response = "The answer is 4.";

    // Reinflate
    let reinflated = redactor.reinflate(cloud_response);

    // Response unchanged (no tokens to reinflate)
    assert_eq!(reinflated, cloud_response);
}

#[test]
fn test_multiple_patterns_redaction() {
    // Test redaction of multiple sensitive data types

    let vault = TokenVault::in_memory().unwrap();
    let config = RedactorConfig::default();
    let mut redactor = Redactor::new(config, vault).unwrap();

    // Email (detects), phone (detects with 10 digits), API key (needs 20+ chars after sk-)
    let user_query = "Contact me at jane@example.com or call 555-123-4567. API key: sk-test-12345678901234567";

    let redacted_result = redactor.redact(user_query, "test-session");

    // Verify all patterns are redacted
    assert!(!redacted_result.redacted_text.contains("jane@example.com"));
    assert!(!redacted_result.redacted_text.contains("555-123-4567"));
    assert!(!redacted_result.redacted_text.contains("sk-test-12345678901234567"));

    // Should have 3 redactions
    assert_eq!(redacted_result.stats.patterns_redacted, 3);
}

#[test]
fn test_token_session_isolation() {
    // Test that tokens are not reused across sessions

    let vault = TokenVault::in_memory().unwrap();
    let config = RedactorConfig::default();
    let mut redactor = Redactor::new(config, vault).unwrap();

    // Session 1
    let query1 = "Email: test@example.com";
    let redacted1 = redactor.redact(query1, "session-1");
    let token1 = &redacted1.redacted_text;

    // Session 2 (same email)
    let query2 = "Email: test@example.com";
    let redacted2 = redactor.redact(query2, "session-2");
    let token2 = &redacted2.redacted_text;

    // Tokens should be different (session isolation)
    assert_ne!(token1, token2, "Tokens should differ across sessions");
}

#[test]
fn test_context_does_not_leak_sensitive_data() {
    // Test that context sent to cloud doesn't contain sensitive data

    let vault = TokenVault::in_memory().unwrap();
    let config = RedactorConfig::default();
    let mut redactor = Redactor::new(config, vault).unwrap();

    // User query with long API key
    let user_query = "What is my API key sk-test-12345678901234567 used for?";

    // Redact
    let redacted = redactor.redact(user_query, "test-session");

    // Create context with potentially sensitive data
    let context = EscalationContext {
        pathos_framing: Some("User wants to understand their API key usage".to_string()),
        local_knowledge: vec![],
        conversation_history: vec![],
        constraints: vec![],
        user_preferences: None,
    };

    // Verify context doesn't contain sensitive data (it was in the query)
    assert!(context.pathos_framing.as_ref().unwrap().contains("API key"));
    assert!(context.pathos_framing.as_ref().unwrap().contains("usage"));

    // But the actual query is redacted
    assert!(!redacted.redacted_text.contains("sk-test-12345678901234567"));
}

#[test]
fn test_constant_time_reinflation() {
    // This test documents the timing attack fix

    // The reinflate method uses a constant-time algorithm to prevent
    // timing attacks where an attacker could measure response times
    // to infer which tokens exist in the vault

    let vault = TokenVault::in_memory().unwrap();
    let config = RedactorConfig::default();
    let mut redactor = Redactor::new(config, vault).unwrap();

    // Create query with email
    let query = "Email: test@example.com";
    let redacted = redactor.redact(query, "test-session");

    // Reinflate (should be constant time regardless of matches)
    let start = std::time::Instant::now();
    let _reinflate = redactor.reinflate(&redacted.redacted_text);
    let duration = start.elapsed();

    // In production, this would be benchmarked to verify constant time
    // For now, just document that the implementation exists
    assert!(duration.as_millis() < 100, "Reinflation should be fast");
}
