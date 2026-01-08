# Phase 2 Testing Guide

**Version**: 2.0.0
**Last Updated**: 2026-01-02

---

## Testing Overview

Phase 2 testing requires verifying cloud connectivity, billing accuracy, and integration between local and cloud components.

---

## Test Categories

### 1. Unit Tests

Unit tests for individual components in `synesis-cloud` crate.

#### Tunnel Tests

```rust
// tests/tunnel_tests.rs

#[cfg(test)]
mod tunnel_tests {
    use synesis_cloud::tunnel::{CloudTunnel, TunnelConfig, TunnelState};
    
    #[test]
    fn test_tunnel_config_defaults() {
        let config = TunnelConfig::default();
        
        assert_eq!(config.heartbeat_interval, Duration::from_secs(30));
        assert_eq!(config.max_reconnect_attempts, 10);
        assert_eq!(config.connect_timeout, Duration::from_secs(30));
    }
    
    #[test]
    fn test_tunnel_state_transitions() {
        let mut state = TunnelState::Disconnected;
        
        // Disconnected -> Connecting
        state = TunnelState::Connecting;
        assert!(matches!(state, TunnelState::Connecting));
        
        // Connecting -> Connected
        state = TunnelState::Connected {
            since: Instant::now(),
            latency_ms: 50,
        };
        assert!(matches!(state, TunnelState::Connected { .. }));
        
        // Connected -> Reconnecting
        state = TunnelState::Reconnecting {
            attempt: 1,
            last_error: "Connection reset".to_string(),
        };
        assert!(matches!(state, TunnelState::Reconnecting { attempt: 1, .. }));
    }
    
    #[test]
    fn test_tunnel_stats_accumulation() {
        let mut stats = TunnelStats::default();
        
        stats.total_bytes_sent += 1024;
        stats.heartbeats_sent += 1;
        stats.requests_sent += 1;
        stats.requests_succeeded += 1;
        
        assert_eq!(stats.total_bytes_sent, 1024);
        assert_eq!(stats.heartbeats_sent, 1);
        assert_eq!(stats.success_rate(), 1.0);
    }
}
```

#### Escalation Tests

```rust
// tests/escalation_tests.rs

#[cfg(test)]
mod escalation_tests {
    use synesis_cloud::escalation::*;
    
    #[test]
    fn test_escalation_request_validation() {
        let valid_request = EscalationRequest {
            request_id: "req_abc123def456789".to_string(),
            session_id: "sess_xyz".to_string(),
            query: "What is 2+2?".to_string(),
            context: EscalationContext::default(),
            model: CloudModel::Auto,
            max_tokens: 1024,
            stream: false,
            lora_id: None,
            timeout_secs: Some(30),
        };
        
        assert!(valid_request.validate().is_ok());
    }
    
    #[test]
    fn test_invalid_request_id() {
        let request = EscalationRequest {
            request_id: "invalid".to_string(),
            ..Default::default()
        };
        
        assert!(matches!(
            request.validate(),
            Err(ValidationError::InvalidRequestId)
        ));
    }
    
    #[test]
    fn test_query_too_long() {
        let request = EscalationRequest {
            request_id: "req_abc123def456789".to_string(),
            query: "x".repeat(100001),  // Exceeds limit
            ..Default::default()
        };
        
        assert!(matches!(
            request.validate(),
            Err(ValidationError::QueryTooLong)
        ));
    }
    
    #[test]
    fn test_context_serialization() {
        let context = EscalationContext {
            pathos_framing: Some("User wants explanation".to_string()),
            local_knowledge: vec![
                KnowledgeChunk {
                    source: "/docs/readme.md".to_string(),
                    content: "This is a test".to_string(),
                    relevance: 0.95,
                }
            ],
            conversation_history: vec![
                Message {
                    role: "user".to_string(),
                    content: "Hello".to_string(),
                    timestamp: None,
                }
            ],
            constraints: vec!["Keep it brief".to_string()],
            user_preferences: None,
        };
        
        let json = serde_json::to_string(&context).unwrap();
        let deserialized: EscalationContext = serde_json::from_str(&json).unwrap();
        
        assert_eq!(context.pathos_framing, deserialized.pathos_framing);
        assert_eq!(context.local_knowledge.len(), deserialized.local_knowledge.len());
    }
}
```

#### Billing Tests

```rust
// tests/billing_tests.rs

#[cfg(test)]
mod billing_tests {
    use synesis_cloud::billing::*;
    
    #[test]
    fn test_managed_tier_markup() {
        let tier = BillingTier::Managed { markup_percent: 3.0 };
        
        let base_cost = 100;  // cents
        let final_cost = tier.apply_markup(base_cost);
        
        assert_eq!(final_cost, 103);  // 3% markup
    }
    
    #[test]
    fn test_byok_tier_markup() {
        let tier = BillingTier::Byok {
            licensing_percent: 30.0,
            anthropic_key: None,
            openai_key: None,
        };
        
        let base_cost = 100;
        let final_cost = tier.apply_markup(base_cost);
        
        assert_eq!(final_cost, 130);  // 30% licensing
    }
    
    #[test]
    fn test_credit_application() {
        let mut ledger = LocalLedger {
            unbilled_cents: 0,
            knowledge_credits_cents: 50,
            last_sync: Instant::now(),
            pending_events: vec![],
        };
        
        let charge = 100;
        let (net_charge, credits_used) = ledger.apply_credits(charge);
        
        assert_eq!(credits_used, 50);
        assert_eq!(net_charge, 50);
        assert_eq!(ledger.knowledge_credits_cents, 0);
    }
    
    #[test]
    fn test_usage_event_creation() {
        let event = UsageEvent::new(
            "req_123".to_string(),
            100,  // tokens_in
            200,  // tokens_out
            "claude-3-5-sonnet".to_string(),
            10,   // cost_basis_cents
        );
        
        assert!(!event.id.is_empty());
        assert_eq!(event.tokens_in, 100);
        assert_eq!(event.tokens_out, 200);
    }
}
```

### 2. Integration Tests

Integration tests verify component interactions.

#### Mock Server

```rust
// tests/integration/mock_server.rs

use std::net::SocketAddr;
use tokio::sync::{broadcast, Mutex};
use std::sync::Arc;

pub struct MockCloudServer {
    addr: SocketAddr,
    shutdown_tx: broadcast::Sender<()>,
    received_requests: Arc<Mutex<Vec<EscalationRequest>>>,
    billing_events: Arc<Mutex<Vec<UsageEvent>>>,
    config: MockConfig,
}

pub struct MockConfig {
    pub response_delay_ms: u64,
    pub fail_rate: f32,
    pub model_responses: HashMap<String, String>,
}

impl MockCloudServer {
    pub async fn start() -> Self {
        Self::start_with_config(MockConfig::default()).await
    }
    
    pub async fn start_with_config(config: MockConfig) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        let received_requests = Arc::new(Mutex::new(Vec::new()));
        let billing_events = Arc::new(Mutex::new(Vec::new()));
        
        // Start QUIC server
        let addr = start_quic_server(
            shutdown_tx.subscribe(),
            received_requests.clone(),
            billing_events.clone(),
            config.clone(),
        ).await;
        
        Self {
            addr,
            shutdown_tx,
            received_requests,
            billing_events,
            config,
        }
    }
    
    pub fn url(&self) -> String {
        format!("https://localhost:{}", self.addr.port())
    }
    
    pub async fn received_requests(&self) -> Vec<EscalationRequest> {
        self.received_requests.lock().await.clone()
    }
    
    pub async fn billing_events(&self) -> Vec<UsageEvent> {
        self.billing_events.lock().await.clone()
    }
    
    pub async fn disconnect_clients(&self) {
        // Force disconnect all connected clients
    }
    
    pub async fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
    }
}
```

#### Full Flow Tests

```rust
// tests/integration/cloud_flow_tests.rs

use synesis_cloud::{CloudTunnel, EscalationClient, BillingClient};
use synesis_privacy::{Redactor, TokenVault};
use crate::mock_server::MockCloudServer;

#[tokio::test]
async fn test_full_escalation_with_privacy() {
    // Start mock server
    let server = MockCloudServer::start().await;
    
    // Create tunnel
    let mut tunnel = CloudTunnel::new(TunnelConfig {
        cloud_url: server.url(),
        device_id: "test-device".to_string(),
        ..Default::default()
    }).unwrap();
    
    tunnel.connect().await.unwrap();
    
    // Create privacy proxy
    let vault = TokenVault::in_memory().unwrap();
    let redactor = Redactor::new(vault, Default::default());
    
    // Create escalation client
    let client = EscalationClient::new(
        Arc::new(tunnel),
        Default::default(),
    );
    
    // Query with sensitive data
    let query = "Contact john@example.com about the API key sk-abc123";
    
    // Redact
    let redacted = redactor.redact(query).await.unwrap();
    assert!(!redacted.contains("john@example.com"));
    assert!(!redacted.contains("sk-abc123"));
    
    // Escalate
    let request = EscalationRequest {
        request_id: generate_request_id(),
        query: redacted,
        ..Default::default()
    };
    
    let response = client.escalate(request).await.unwrap();
    
    // Verify server never saw sensitive data
    let received = server.received_requests().await;
    for req in received {
        assert!(!req.query.contains("john@example.com"));
        assert!(!req.query.contains("sk-abc123"));
    }
    
    server.shutdown().await;
}

#[tokio::test]
async fn test_billing_accuracy() {
    let server = MockCloudServer::start().await;
    let tunnel = Arc::new(create_test_tunnel(&server).await);
    
    let billing = BillingClient::new(
        tunnel.clone(),
        "test-user".to_string(),
        BillingTier::Managed { markup_percent: 3.0 },
    );
    
    // Record usage events
    for i in 0..10 {
        billing.record_usage(UsageEvent {
            request_id: format!("req_{}", i),
            tokens_in: 100,
            tokens_out: 200,
            model: "claude-3-5-sonnet".to_string(),
            cost_basis_cents: 10,
            ..Default::default()
        }).await.unwrap();
    }
    
    // Verify totals
    let summary = billing.current_usage();
    assert_eq!(summary.total_requests, 10);
    assert_eq!(summary.total_tokens_in, 1000);
    assert_eq!(summary.total_tokens_out, 2000);
    
    // With 3% markup: 10 * 10 * 1.03 = 103 cents
    assert_eq!(summary.total_cents, 103);
    
    server.shutdown().await;
}

#[tokio::test]
async fn test_streaming_response() {
    let server = MockCloudServer::start_with_config(MockConfig {
        model_responses: hashmap! {
            "default".to_string() => "One two three four five".to_string()
        },
        ..Default::default()
    }).await;
    
    let tunnel = Arc::new(create_test_tunnel(&server).await);
    let client = EscalationClient::new(tunnel, Default::default());
    
    let request = EscalationRequest {
        request_id: generate_request_id(),
        query: "Count to 5".to_string(),
        stream: true,
        ..Default::default()
    };
    
    let mut response = client.escalate_streaming(request).await.unwrap();
    
    let mut chunks = Vec::new();
    while let Some(chunk) = response.next().await {
        if let StreamChunk::Content { text } = chunk.unwrap() {
            chunks.push(text);
        }
    }
    
    let full_response = chunks.join("");
    assert!(full_response.contains("One"));
    assert!(full_response.contains("five"));
    
    server.shutdown().await;
}

#[tokio::test]
async fn test_tunnel_reconnection() {
    let server = MockCloudServer::start().await;
    
    let mut tunnel = CloudTunnel::new(TunnelConfig {
        cloud_url: server.url(),
        reconnect_delay: Duration::from_millis(100),
        max_reconnect_attempts: 3,
        ..Default::default()
    }).unwrap();
    
    tunnel.connect().await.unwrap();
    assert!(tunnel.is_connected());
    
    // Simulate disconnect
    server.disconnect_clients().await;
    
    // Wait for reconnection
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // Should have reconnected
    assert!(tunnel.is_connected());
    assert!(tunnel.stats().reconnections >= 1);
    
    server.shutdown().await;
}
```

### 3. Cloud Worker Tests

Tests for Cloudflare Workers using Miniflare.

```typescript
// cloud/__tests__/billing-ledger.test.ts

import { unstable_dev } from 'wrangler';
import { describe, it, expect, beforeAll, afterAll } from 'vitest';

describe('BillingLedger Durable Object', () => {
    let worker: any;
    
    beforeAll(async () => {
        worker = await unstable_dev('src/index.ts', {
            experimental: { disableExperimentalWarning: true },
        });
    });
    
    afterAll(async () => {
        await worker.stop();
    });
    
    it('should record usage and calculate markup', async () => {
        const response = await worker.fetch('/billing/record', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': 'Bearer test_key',
            },
            body: JSON.stringify({
                requestId: 'req_123',
                tokensIn: 100,
                tokensOut: 200,
                model: 'claude-3-5-sonnet',
                costBasisCents: 10,
            }),
        });
        
        const data = await response.json();
        
        expect(response.status).toBe(200);
        expect(data.charged).toBe(11);  // 10 * 1.03 = 10.3 -> 11
        expect(data.balance).toBeGreaterThan(0);
    });
    
    it('should apply knowledge credits', async () => {
        // First, add credits
        await worker.fetch('/billing/add-credits', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': 'Bearer test_key',
            },
            body: JSON.stringify({ cents: 50 }),
        });
        
        // Then record usage
        const response = await worker.fetch('/billing/record', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': 'Bearer test_key',
            },
            body: JSON.stringify({
                requestId: 'req_456',
                tokensIn: 100,
                tokensOut: 200,
                model: 'claude-3-5-sonnet',
                costBasisCents: 100,
            }),
        });
        
        const data = await response.json();
        
        // Credits should be applied: 100 * 1.03 = 103, minus 50 credits = 53
        expect(data.charged).toBe(53);
        expect(data.credits).toBe(0);
    });
    
    it('should enforce credit ceiling', async () => {
        // Record large usage
        const response = await worker.fetch('/billing/record', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': 'Bearer test_key',
            },
            body: JSON.stringify({
                requestId: 'req_789',
                tokensIn: 10000,
                tokensOut: 20000,
                model: 'claude-3-5-sonnet',
                costBasisCents: 15000,  // $150 - exceeds $100 ceiling
            }),
        });
        
        expect(response.status).toBe(402);
        
        const data = await response.json();
        expect(data.error).toBe('credit_ceiling_exceeded');
    });
});

// cloud/__tests__/collaborator.test.ts

describe('Collaborator System', () => {
    it('should create invite with quota', async () => {
        const response = await worker.fetch('/projects/proj_123/invites', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': 'Bearer test_key',
            },
            body: JSON.stringify({
                role: 'viewer',
                quotaCents: 500,
                expiresHours: 48,
            }),
        });
        
        const data = await response.json();
        
        expect(response.status).toBe(200);
        expect(data.token).toMatch(/^inv_/);
        expect(data.url).toContain('/join/');
        expect(data.quotaCents).toBe(500);
    });
    
    it('should enforce guest quota', async () => {
        // Accept invite first
        const acceptResponse = await worker.fetch('/join/inv_test123', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ email: 'guest@example.com' }),
        });
        
        const session = await acceptResponse.json();
        
        // Use up quota
        for (let i = 0; i < 5; i++) {
            await worker.fetch('/escalate', {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer guest_${session.id}`,
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    requestId: `req_${i}`,
                    query: 'Test query',
                }),
            });
        }
        
        // Next request should fail
        const response = await worker.fetch('/escalate', {
            method: 'POST',
            headers: {
                'Authorization': `Bearer guest_${session.id}`,
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                requestId: 'req_final',
                query: 'Test query',
            }),
        });
        
        expect(response.status).toBe(402);
    });
});
```

### 4. End-to-End Tests

Full system tests from CLI to cloud and back.

```bash
#!/bin/bash
# tests/e2e/test_full_flow.sh

set -e

echo "=== Phase 2 E2E Test Suite ==="

# Setup
export SYNESIS_TEST_MODE=1
export SYNESIS_CLOUD_URL="https://staging.superinstance.ai"

# 1. Initialize system
echo "Test 1: Initialize system"
synesis init --test
[ $? -eq 0 ] || { echo "FAIL: init"; exit 1; }

# 2. Login to cloud
echo "Test 2: Cloud login"
synesis cloud login --api-key "$TEST_API_KEY"
[ $? -eq 0 ] || { echo "FAIL: cloud login"; exit 1; }

# 3. Check cloud status
echo "Test 3: Cloud status"
synesis cloud status | grep -q "Connected"
[ $? -eq 0 ] || { echo "FAIL: cloud status"; exit 1; }

# 4. Send escalation request
echo "Test 4: Escalation"
RESPONSE=$(synesis ask "What is 2+2?" --cloud --json)
echo "$RESPONSE" | grep -q '"content"'
[ $? -eq 0 ] || { echo "FAIL: escalation"; exit 1; }

# 5. Check billing
echo "Test 5: Billing recorded"
synesis cloud balance | grep -q "Unbilled"
[ $? -eq 0 ] || { echo "FAIL: billing"; exit 1; }

# 6. Upload LoRA
echo "Test 6: LoRA upload"
synesis push --path ./test-lora/ --name test-lora
[ $? -eq 0 ] || { echo "FAIL: lora upload"; exit 1; }

# 7. List cloud LoRAs
echo "Test 7: List LoRAs"
synesis model list --cloud | grep -q "test-lora"
[ $? -eq 0 ] || { echo "FAIL: list loras"; exit 1; }

# 8. Create invite
echo "Test 8: Create invite"
INVITE=$(synesis invite create --project testproj --role viewer --quota 5.00 --json)
echo "$INVITE" | grep -q '"url"'
[ $? -eq 0 ] || { echo "FAIL: create invite"; exit 1; }

# 9. Streaming response
echo "Test 9: Streaming"
synesis ask "Count to 5 slowly" --cloud --stream | grep -q "5"
[ $? -eq 0 ] || { echo "FAIL: streaming"; exit 1; }

# 10. Privacy verification
echo "Test 10: Privacy"
# This should NOT leak the email
synesis ask "Contact test@example.com" --cloud --verbose 2>&1 | grep -v "test@example.com" | grep -q "[EMAIL"
[ $? -eq 0 ] || { echo "FAIL: privacy"; exit 1; }

echo "=== All E2E Tests Passed ==="
```

### 5. Performance Tests

```rust
// benches/cloud_benchmarks.rs

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use synesis_cloud::{EscalationClient, EscalationRequest};

fn benchmark_escalation_latency(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let client = runtime.block_on(setup_client());
    
    let mut group = c.benchmark_group("escalation");
    
    for query_len in [100, 500, 1000, 5000].iter() {
        group.bench_with_input(
            BenchmarkId::new("latency", query_len),
            query_len,
            |b, &len| {
                let query = "x".repeat(len);
                b.to_async(&runtime).iter(|| async {
                    let request = EscalationRequest {
                        query: query.clone(),
                        ..Default::default()
                    };
                    client.escalate(request).await
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_serialization(c: &mut Criterion) {
    let context = create_large_context();
    
    c.bench_function("serialize_context", |b| {
        b.iter(|| serde_json::to_vec(&context))
    });
    
    let serialized = serde_json::to_vec(&context).unwrap();
    
    c.bench_function("deserialize_context", |b| {
        b.iter(|| serde_json::from_slice::<EscalationContext>(&serialized))
    });
}

criterion_group!(benches, benchmark_escalation_latency, benchmark_serialization);
criterion_main!(benches);
```

---

## Test Commands

```bash
# Run all tests
cargo test --workspace

# Run cloud-specific tests
cargo test --package synesis-cloud

# Run integration tests
cargo test --test integration

# Run with coverage
cargo llvm-cov --workspace

# Run benchmarks
cargo bench

# Run E2E tests
./tests/e2e/test_full_flow.sh

# Run cloud worker tests
cd cloud && npm test
```

---

## CI Configuration

```yaml
# .github/workflows/phase2-tests.yml

name: Phase 2 Tests

on:
  push:
    branches: [main, phase-2]
  pull_request:
    branches: [main]

jobs:
  rust-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-action@stable
      
      - name: Run unit tests
        run: cargo test --workspace
      
      - name: Run integration tests
        run: cargo test --test integration
      
      - name: Run benchmarks
        run: cargo bench --no-run
  
  worker-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
      
      - name: Install dependencies
        run: cd cloud && npm ci
      
      - name: Run worker tests
        run: cd cloud && npm test
  
  e2e-tests:
    runs-on: ubuntu-latest
    needs: [rust-tests, worker-tests]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-action@stable
      
      - name: Build
        run: cargo build --release
      
      - name: Run E2E tests
        env:
          TEST_API_KEY: ${{ secrets.TEST_API_KEY }}
        run: ./tests/e2e/test_full_flow.sh
```

---

*Document Version: 2.0.0*
*Last Updated: 2026-01-02*
