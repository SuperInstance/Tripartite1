//! Integration tests for synesis-cloud
//!
//! End-to-end tests for cloud connectivity, escalation, billing, and LoRA management

// =============================================================================
// Tunnel State Machine Tests
// =============================================================================

#[test]
fn test_tunnel_state_initialization() {
    let sm = synesis_cloud::tunnel::state::ConnectionStateMachine::new();

    assert!(matches!(sm.current(), synesis_cloud::tunnel::TunnelState::Disconnected));
}

#[test]
fn test_tunnel_state_transitions() {
    use synesis_cloud::tunnel::TunnelState;

    let sm = synesis_cloud::tunnel::state::ConnectionStateMachine::new();

    // Disconnected → Connecting
    sm.transition(TunnelState::Connecting {
        since: std::time::Instant::now(),
    });
    assert!(matches!(sm.current(), TunnelState::Connecting { .. }));

    // Connecting → Connected
    sm.transition(TunnelState::Connected {
        since: std::time::Instant::now(),
        latency_ms: 0,
    });
    assert!(matches!(sm.current(), TunnelState::Connected { .. }));

    // Connected → Disconnected
    sm.transition(TunnelState::Disconnected);
    assert!(matches!(sm.current(), TunnelState::Disconnected));
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[test]
fn test_error_display() {
    use synesis_cloud::CloudError;

    let err = CloudError::tunnel_connection("Connection failed");
    let display = format!("{}", err);
    assert!(display.contains("Connection failed"));
}

#[test]
fn test_error_from_io() {
    use std::io;
    use synesis_cloud::CloudError;

    let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let cloud_err: CloudError = io_err.into();

    assert!(matches!(cloud_err, CloudError::Io(_)));
}

// =============================================================================
// Billing Tests
// =============================================================================

#[test]
fn test_billing_calculation() {
    use synesis_cloud::billing::client::BillingClient;

    let client = BillingClient::new("test-key".to_string(), synesis_cloud::billing::types::BillingTier::Free {
        monthly_limit_cents: 1000,
    });

    let cost = client.calculate_cost("claude-sonnet", 10_000, 5_000).unwrap();

    assert_eq!(cost.base_cost_cents, 11);
    assert_eq!(cost.markup_cents, 0);
    assert_eq!(cost.final_charge_cents, 0);
}

// =============================================================================
// Local Ledger Tests
// =============================================================================

#[test]
fn test_local_ledger_default() {
    use synesis_cloud::billing::types::LocalLedger;

    let ledger = LocalLedger::default();

    assert_eq!(ledger.unbilled_cents, 0);
    assert_eq!(ledger.knowledge_credits_cents, 0);
}

// =============================================================================
// Escalation Context Builder Tests
// =============================================================================

#[test]
fn test_escalation_context_builder() {
    use synesis_cloud::escalation::context::EscalationContextBuilder;
    use synesis_cloud::escalation::types::KnowledgeChunk;

    let context = EscalationContextBuilder::new()
        .pathos_framing("Test framing")
        .add_knowledge(KnowledgeChunk {
            content: "Test content".to_string(),
            source: "test.txt".to_string(),
            relevance: 0.9,
        })
        .user("Test question")
        .build();

    assert_eq!(context.pathos_framing, Some("Test framing".to_string()));
    assert_eq!(context.local_knowledge.len(), 1);
    assert_eq!(context.conversation_history.len(), 1);
}

// =============================================================================
// LoRA Hot-Swap Tests
// =============================================================================

#[tokio::test]
async fn test_lora_hotswap_load_unload() {
    use synesis_cloud::lora::upload::LoraHotSwap;

    let manager = LoraHotSwap::new();

    manager.load("lora-1").await.unwrap();
    assert!(manager.is_loaded("lora-1").await);

    manager.unload("lora-1").await.unwrap();
    assert!(!manager.is_loaded("lora-1").await);
}

#[tokio::test]
async fn test_lora_hotswap_duplicate_error() {
    use synesis_cloud::lora::upload::LoraHotSwap;

    let manager = LoraHotSwap::new();

    manager.load("lora-1").await.unwrap();
    let result = manager.load("lora-1").await;
    assert!(result.is_err());
}

// =============================================================================
// Collaborator System Tests
// =============================================================================

#[test]
fn test_collaborator_create_invite() {
    use synesis_cloud::collaborator::{CollaboratorClient, InviteRequest, CollaboratorRole};

    let client = CollaboratorClient::new();

    let request = InviteRequest {
        project_id: "proj-123".to_string(),
        role: CollaboratorRole::Editor,
        quota_cents: 5000,
        expires_hours: 48,
    };

    let invite = client.create_invite(request).unwrap();

    assert!(!invite.token.is_empty());
    assert!(invite.url.contains(&invite.token));
    assert_eq!(invite.role, CollaboratorRole::Editor);
    assert!(!invite.accepted);
}

#[test]
fn test_collaborator_initiate_handover() {
    use synesis_cloud::collaborator::{CollaboratorClient, HandoverRequest};

    let client = CollaboratorClient::new();

    let request = HandoverRequest {
        project_id: "proj-123".to_string(),
        to_email: "newowner@example.com".to_string(),
        include_loras: true,
        include_knowledge: true,
        message: Some("Please take over".to_string()),
    };

    let handover = client.initiate_handover(request).unwrap();

    assert_eq!(handover.to_email, "newowner@example.com");
    assert!(!handover.token.is_empty());
}

// =============================================================================
// Streaming Tests
// =============================================================================

#[tokio::test]
async fn test_streaming_response_collect() {
    use synesis_cloud::streaming::{StreamChunk, StreamingResponse};
    use tokio::sync::mpsc;

    let (tx, rx) = mpsc::channel(100);

    tokio::spawn(async move {
        let _ = tx.send(StreamChunk {
            content: "Hello".to_string(),
            sequence: 0,
            is_final: false,
        }).await;

        let _ = tx.send(StreamChunk {
            content: " World".to_string(),
            sequence: 1,
            is_final: true,
        }).await;
    });

    let stream = StreamingResponse::new(rx);
    let collected = stream.collect().await.unwrap();

    assert_eq!(collected, "Hello World");
}

// =============================================================================
// Device Vitals Collection Tests
// =============================================================================

#[test]
fn test_device_vitals_collection() {
    use synesis_cloud::telemetry::vitals::collect_device_vitals;

    let vitals = collect_device_vitals("test-device".to_string());

    assert_eq!(vitals.device_id, "test-device");
    assert!(!vitals.timestamp.to_rfc3339().is_empty());
    assert!(vitals.cpu_usage >= 0.0);
    assert!(vitals.memory_usage >= 0.0);
}
