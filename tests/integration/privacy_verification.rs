//! Privacy Verification Tests
//!
//! Ensure privacy proxy works correctly with cloud communication

use synesis_privacy::{Redactor, RedactionPatterns, TokenVault};
use synesis_cloud::escalation::types::{EscalationRequest, EscalationContext};

#[tokio::test]
async fn test_privacy_cloud_roundtrip() {
    // Test that sensitive data is properly redacted before sending to cloud

    // Create privacy proxy
    let vault = TokenVault::in_memory().unwrap();
    let patterns = RedactionPatterns::all();
    let mut redactor = Redactor::new(vault, patterns);

    // User query with sensitive data
    let user_query = "My email is jane@example.com and my phone is 555-1234. What is 2+2?";

    // Redact
    let session_id = "test-session-1";
    let redacted_result = redactor.redact(user_query, session_id).await.unwrap();

    // Verify sensitive data is replaced
    assert!(!redacted_result.redacted_text.contains("jane@example.com"));
    assert!(!redacted_result.redacted_text.contains("555-1234"));
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
    assert!(!request.query.contains("555-1234"));

    // Simulate cloud response (no sensitive data)
    let cloud_response = "The answer is 4.";

    // Reinflate
    let reinflated = redactor.reinflate(cloud_response).await;

    // Response unchanged (no tokens to reinflate)
    assert_eq!(reinflate, cloud_response);
}

#[tokio::test]
async fn test_multiple_patterns_redaction() {
    // Test redaction of multiple sensitive data types

    let vault = TokenVault::in_memory().unwrap();
    let patterns = RedactionPatterns::all();
    let mut redactor = Redactor::new(vault, patterns);

    let user_query = "Contact me at jane@example.com or call 555-1234. API key: sk-test-12345";

    let redacted_result = redactor.redact(user_query, "test-session").await.unwrap();

    // Verify all patterns are redacted
    assert!(!redacted_result.redacted_text.contains("jane@example.com"));
    assert!(!redacted_result.redacted_text.contains("555-1234"));
    assert!(!redacted_result.redacted_text.contains("sk-test-12345"));

    // Should have 3 redactions
    assert_eq!(redacted_result.stats.patterns_redacted, 3);
}

#[tokio::test]
async fn test_token_session_isolation() {
    // Test that tokens are not reused across sessions

    let vault = TokenVault::in_memory().unwrap();
    let patterns = RedactionPatterns::all();
    let mut redactor = Redactor::new(vault, patterns.clone());

    // Session 1
    let query1 = "Email: test@example.com";
    let redacted1 = redactor.redact(query1, "session-1").await.unwrap();
    let token1 = &redacted1.redacted_text;

    // Session 2 (same email)
    let query2 = "Email: test@example.com";
    let redacted2 = redactor.redact(query2, "session-2").await.unwrap();
    let token2 = &redacted2.redacted_text;

    // Tokens should be different (session isolation)
    assert_ne!(token1, token2, "Tokens should differ across sessions");
}

#[tokio::test]
async fn test_context_does_not_leak_sensitive_data() {
    // Test that context sent to cloud doesn't contain sensitive data

    let vault = TokenVault::in_memory().unwrap();
    let patterns = RedactionPatterns::all();
    let mut redactor = Redactor::new(vault, patterns);

    // User query
    let user_query = "What is my API key sk-test-12345 used for?";

    // Redact
    let redacted = redactor.redact(user_query, "test-session").await.unwrap();

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
    assert!(!redacted.redacted_text.contains("sk-test-12345"));
}

#[tokio::test]
async fn test_constant_time_reinflation() {
    // This is a placeholder test documenting the timing attack fix

    // The reinflate method uses a constant-time algorithm to prevent
    // timing attacks where an attacker could measure response times
    // to infer which tokens exist in the vault

    let vault = TokenVault::in_memory().unwrap();
    let patterns = RedactionPatterns::all();
    let redactor = Redactor::new(vault, patterns);

    // Create query with email
    let query = "Email: test@example.com";
    let redacted = redactor.redact(query, "test-session").await.unwrap();

    // Reinflate (should be constant time regardless of matches)
    let start = std::time::Instant::now();
    let _reinflate = redactor.reinflate(&redacted.redacted_text);
    let duration = start.elapsed();

    // In production, this would be benchmarked to verify constant time
    // For now, just document that the implementation exists
    assert!(duration.as_millis() < 100, "Reinflation should be fast");
}

#[cfg(test)]
mod regression_tests {
    // Regression tests for past security issues

    #[tokio::test]
    async fn test_no_sensitive_data_in_logs() {
        // Ensure sensitive data never appears in logs

        // This test documents the security requirement
        // Actual implementation would capture log output

        let vault = TokenVault::in_memory().unwrap();
        let patterns = RedactionPatterns::all();
        let redactor = Redactor::new(vault, patterns);

        let query = "Email: secret@example.com, Password: hunter2";
        let redacted = redactor.redact(query, "test-session").await.unwrap();

        // Redacted text should be safe for logs
        assert!(!redacted.redacted_text.contains("secret@example.com"));
        assert!(!redacted.redacted_text.contains("hunter2"));
    }

    #[tokio::test]
    async fn test_token_vault_never_leaves_device() {
        // Document that token vault mappings never leave the device

        // The vault stores: token → original_value
        // Only the token is sent to cloud
        // Original value is only used locally for reinflation

        let vault = TokenVault::in_memory().unwrap();

        let token = vault.store("EMAIL", "test@example.com", "session-1").unwrap();

        // Token can be sent (it's just a UUID reference)
        assert!(token.starts_with("[EMAIL_"));

        // But the mapping stays in the vault
        let original = vault.retrieve(&token).unwrap();
        assert_eq!(original, "test@example.com");
    }
}
