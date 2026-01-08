# Phase 2 Cloud Mesh - Detailed Implementation Roadmap

**Version**: 2.0.0
**Start Date**: 2026-01-02
**Target Completion**: 2026-01-16 (2 weeks)
**Status**: In Progress

---

## Executive Summary

Phase 2 transforms SuperInstance from a **local-only tool** into a **commercial platform** with cloud connectivity, enabling:

1. **Cloud Escalation** - Offload queries to cloud when local is insufficient
2. **Billing Infrastructure** - Stripe integration with 3% (managed) or 30% (BYOK) markup
3. **LoRA Hot-Swap** - Upload and use local LoRAs in cloud inference
4. **Collaborator System** - Share projects with guests or handover to clients
5. **Real-time Streaming** - Low-latency streaming responses via QUIC

**Implementation Approach**: 12 detailed sessions, each building on the previous, from local Rust client → QUIC tunnel → Cloudflare Workers.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           LOCAL CLIENT                                   │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │                        synesis-cloud Crate                          │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐   │  │
│  │  │ CloudTunnel │  │   Billing   │  │  Escalation Client      │   │  │
│  │  │  (QUIC)     │  │   Ledger    │  │  (with privacy proxy)    │   │  │
│  │  └──────┬──────┘  └──────┬──────┘  └───────────�─────────────────┘   │  │
│  │         │                │                                        │  │
│  │         └────────────────┴───────────────────────────────────────┐│  │
│  │                                                                   ││  │
│  │  CLI Commands: synesis cloud login|status|ask|push|invite         ││  │
│  └───────────────────────────────────────────────────────────────────┘│  │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                            QUIC/TLS 1.3 (mTLS)
                            UDP port 443
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                      CLOUDFLARE EDGE                                     │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │                    cloud/ (TypeScript)                             │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐   │  │
│  │  │ QUIC Server │  │   Billing   │  │  Escalation Handler     │   │  │
│  │  │ (Terminator)│  │  DurableObj │  │  (Workers AI)            │   │  │
│  │  └──────┬──────┘  └──────┬──────┘  └───────────┬─────────────────┘   │  │
│  │         │                │                      │                   │  │
│  │         └────────────────┴──────────────────────┴───────────────────┘│  │
│  │                                                                   ││  │
│  │  Integrations: Stripe, R2 Storage, Vectorize, Workers AI          ││  │
│  └───────────────────────────────────────────────────────────────────┘│  │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Session Breakdown

### **Session 2.0: Planning & Setup** ✅ COMPLETE
**Status**: Documentation studied, roadmap created
**Deliverables**:
- ✅ All Phase 2 documents reviewed
- ✅ Detailed roadmap created
- ✅ Task list initialized

---

### **Session 2.1: synesis-cloud Crate Setup**
**Priority**: HIGH - Foundation for all cloud functionality
**Effort**: 2-3 hours
**Dependencies**: None

**Objectives**:
1. Create `crates/synesis-cloud/` directory structure
2. Add dependencies to workspace Cargo.toml
3. Define module structure and types
4. Set up basic error handling
5. Add placeholder tests

**Files to Create**:
```
crates/synesis-cloud/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── error.rs
│   ├── tunnel/
│   │   ├── mod.rs
│   │   └── types.rs
│   ├── escalation/
│   │   ├── mod.rs
│   │   └── types.rs
│   ├── billing/
│   │   ├── mod.rs
│   │   └── types.rs
│   ├── lora/
│   │   ├── mod.rs
│   │   └── types.rs
│   └── telemetry/
│       ├── mod.rs
│       └── types.rs
└── tests/
    └── common.rs
```

**Dependencies**:
```toml
[dependencies]
quinn = "0.10"
rustls = { version = "0.21", features = ["dangerous_configuration"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1.0"
tracing = "0.1"
uuid = { version = "1", features = ["v4", "serde"] }
```

**Acceptance Criteria**:
- [ ] Crate compiles successfully
- [ ] All modules exist with placeholder implementations
- [ ] Error types defined
- [ ] Basic tests pass
- [ ] Documentation added

---

### **Session 2.2: QUIC Tunnel Core Implementation**
**Priority**: CRITICAL - Core connectivity
**Effort**: 4-6 hours
**Dependencies**: Session 2.1

**Objectives**:
1. Implement TLS 1.3 configuration with mTLS
2. Create QUIC endpoint
3. Implement connection state machine
4. Add auto-reconnection with exponential backoff
5. Unit tests for tunnel functionality

**Key Files**:
- `src/tunnel/tls.rs` - TLS configuration, certificate loading
- `src/tunnel/endpoint.rs` - QUIC endpoint creation
- `src/tunnel/state.rs` - Connection state machine
- `src/tunnel/mod.rs` - Main CloudTunnel struct

**Implementation Details**:
```rust
pub struct CloudTunnel {
    config: TunnelConfig,
    endpoint: quinn::Endpoint,
    connection: Arc<RwLock<Option<quinn::Connection>>>,
    state_machine: ConnectionStateMachine,
    heartbeat_service: HeartbeatService,
    stats: Arc<RwLock<TunnelStats>>,
}

pub enum TunnelState {
    Disconnected,
    Connecting { started_at: Instant },
    Connected { since: Instant, latency_ms: u32 },
    Reconnecting { attempt: u32, last_error: String },
    Failed { error: String, at: Instant },
}
```

**Acceptance Criteria**:
- [ ] TLS 1.3 enforced with mTLS
- [ ] QUIC endpoint can connect to server
- [ ] State machine handles all transitions correctly
- [ ] Auto-reconnection works with exponential backoff
- [ ] Connection stats tracked accurately
- [ ] All tests passing

---

### **Session 2.3: Heartbeat & Telemetry System**
**Priority**: HIGH - Required for connection health
**Effort**: 3-4 hours
**Dependencies**: Session 2.2

**Objectives**:
1. Implement heartbeat service (30s interval)
2. Collect device vitals (CPU, memory, GPU)
3. Send pre-warm signals when GPU stressed
4. Handle server acknowledgments
5. Add telemetry tests

**Key Files**:
- `src/telemetry/vitals.rs` - Device vitals collection
- `src/tunnel/heartbeat.rs` - Heartbeat service implementation

**Implementation Details**:
```rust
pub struct HeartbeatService {
    interval: Duration,
    sequence: AtomicU64,
    connection: Arc<RwLock<Option<quinn::Connection>>>,
    shutdown: broadcast::Sender<()>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeviceVitals {
    pub device_id: String,
    pub timestamp: DateTime<Utc>,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub gpu_usage: Option<f32>,
    pub gpu_temp_celsius: Option<f32>,
    pub active_sessions: u32,
    pub loaded_model: Option<String>,
}
```

**Acceptance Criteria**:
- [ ] Heartbeats sent every 30 seconds
- [ ] Device vitals collected accurately
- [ ] Prewarm signals sent when GPU > 80%
- [ ] Server ACKs tracked
- [ ] All tests passing

---

### **Session 2.4: Cloud Escalation Client**
**Priority**: HIGH - Core cloud query functionality
**Effort**: 4-5 hours
**Dependencies**: Session 2.2, Session 2.3

**Objectives**:
1. Implement escalation request/response types
2. Create EscalationClient with privacy integration
3. Add context passing (Pathos framing, local knowledge)
4. Implement model selection (Auto/Sonnet/Opus)
5. Add basic error handling

**Key Files**:
- `src/escalation/types.rs` - Request/response types
- `src/escalation/client.rs` - EscalationClient implementation
- `src/escalation/context.rs` - EscalationContext builder

**Implementation Details**:
```rust
pub struct EscalationClient {
    tunnel: Arc<CloudTunnel>,
    api_key: String,
    timeout: Duration,
}

pub struct EscalationRequest {
    pub request_id: String,
    pub session_id: String,
    pub query: String,  // Already redacted
    pub context: EscalationContext,
    pub model: CloudModel,
    pub max_tokens: u32,
    pub stream: bool,
    pub lora_id: Option<String>,
}

pub enum CloudModel {
    Auto,
    ClaudeSonnet,
    ClaudeOpus,
    Gpt4Turbo,
}
```

**Integration with Privacy Proxy**:
```rust
// In CLI layer
let redacted = privacy_proxy.redact(&user_query).await?;
let request = EscalationRequest {
    query: redacted,
    ...
};
let response = escalation_client.escalate(request).await?;
let restored = privacy_proxy.reinflate(&response.content).await?;
```

**Acceptance Criteria**:
- [ ] EscalationClient can send requests
- [ ] Privacy proxy integration works
- [ ] Context passed correctly
- [ ] Model selection works
- [ ] All tests passing

---

### **Session 2.5: Message Protocol Definition**
**Priority**: HIGH - Protocol for all tunnel communication
**Effort**: 2-3 hours
**Dependencies**: Session 2.2

**Objectives**:
1. Define message types (Heartbeat, Escalation, StreamChunk, Error)
2. Create frame format (type + length + payload)
3. Implement serialization/deserialization
4. Add protocol tests

**Key Files**:
- `src/protocol/mod.rs` - Protocol definitions
- `src/protocol/messages.rs` - Message types
- `src/protocol/frame.rs` - Frame format

**Message Types**:
```rust
pub enum TunnelMessage {
    Heartbeat(Heartbeat),
    HeartbeatAck(HeartbeatAck),
    EscalationRequest(EscalationRequest),
    EscalationResponse(EscalationResponse),
    StreamChunk(StreamChunk),
    PrewarmSignal(PrewarmSignal),
    Error(ProtocolError),
}

// Frame format
// +------+----------------+------------------------+
// | Type | Length (4B, BE) | Payload (protobuf/json) |
// +------+----------------+------------------------+
```

**Acceptance Criteria**:
- [ ] All message types defined
- [ ] Frame format implemented
- [ ] Serialization/deserialization works
- [ ] Protocol tests passing

---

### **Session 2.6: Billing Client Implementation**
**Priority**: HIGH - Revenue infrastructure
**Effort**: 3-4 hours
**Dependencies**: Session 2.2

**Objectives**:
1. Implement local billing ledger
2. Create UsageEvent type
3. Add markup calculation (3% or 30%)
4. Implement credit tracking
5. Add balance sync with cloud

**Key Files**:
- `src/billing/ledger.rs` - Local billing ledger
- `src/billing/types.rs` - Billing types
- `src/billing/client.rs` - Billing REST API client

**Implementation Details**:
```rust
pub struct LocalLedger {
    pub unbilled_cents: u64,
    pub knowledge_credits_cents: u64,
    pub credit_ceiling_cents: u64,
    pub tier: BillingTier,
    pub pending_events: Vec<UsageEvent>,
}

pub enum BillingTier {
    Free { monthly_limit_cents: u32 },
    Managed { markup_percent: f32 },  // 3%
    Byok { licensing_percent: f32 },  // 30%
}

pub struct UsageEvent {
    pub id: String,
    pub request_id: String,
    pub timestamp: DateTime<Utc>,
    pub tokens_in: u32,
    pub tokens_out: u32,
    pub model: String,
    pub cost_basis_cents: u32,
    pub final_charge_cents: u32,
}
```

**Acceptance Criteria**:
- [ ] Local ledger tracks usage
- [ ] Markup calculated correctly (3% or 30%)
- [ ] Credits applied properly
- [ ] Balance sync works
- [ ] All tests passing

---

### **Session 2.7: Cloud Workers Durable Objects**
**Priority**: CRITICAL - Cloud-side processing
**Effort**: 6-8 hours
**Dependencies**: Session 2.6

**Objectives**:
1. Set up Cloudflare Workers project
2. Implement BillingLedger Durable Object
3. Implement SessionState Durable Object
4. Add REST API handlers
5. Deploy to staging for testing

**Key Files**:
```
cloud/
├── wrangler.toml
├── package.json
├── src/
│   ├── index.ts
│   ├── billing-ledger.ts
│   ├── session-state.ts
│   ├── escalation.ts
│   └── types.ts
└── schema/
    └── 001_initial.sql  (D1 database schema)
```

**Implementation Details**:
```typescript
// cloud/src/billing-ledger.ts
export class BillingLedger extends DurableObject {
  private state: LedgerState;
  private env: Env;

  async fetch(request: Request) {
    const url = new URL(request.url);

    if (url.pathname === '/record') {
      return this.recordUsage(await request.json());
    }

    if (url.pathname === '/balance') {
      return this.getBalance();
    }

    return new Response('Not found', { status: 404 });
  }

  private async recordUsage(event: UsageEvent): Promise<Response> {
    // Apply markup
    const markup = this.getMarkup(this.state.tier);
    const finalCharge = Math.ceil(event.costBasisCents * (1 + markup));

    // Check credit ceiling
    if (this.state.unbilledCents + finalCharge > this.state.creditCeiling) {
      return new Response('Credit ceiling exceeded', { status: 402 });
    }

    // Apply credits
    const creditsUsed = Math.min(this.state.knowledgeCredits, finalCharge);
    const netCharge = finalCharge - creditsUsed;

    this.state.unbilledCents += netCharge;
    this.state.knowledgeCredits -= creditsUsed;
    this.state.usageHistory.push(event);

    await this.state.storage.put('ledger', this.state);

    return Response.json({
      charged: netCharge,
      balance: this.state.unbilledCents,
      credits: this.state.knowledgeCredits,
    });
  }

  private getMarkup(tier: BillingTier): number {
    switch (tier.type) {
      case 'managed': return tier.markupPercent / 100;
      case 'byok': return tier.licensingPercent / 100;
      case 'free': return 0;
    }
  }
}
```

**Acceptance Criteria**:
- [ ] Durable Objects deploy successfully
- [ ] Billing ledger records usage
- [ ] Session state persists conversations
- [ ] REST API handlers work
- [ ] Integration tests pass

---

### **Session 2.8: LoRA Upload & Hot-Swap**
**Priority**: MEDIUM - Advanced feature
**Effort**: 4-5 hours
**Dependencies**: Session 2.7

**Objectives**:
1. Implement LoRA upload flow (chunked)
2. Store LoRAs in R2
3. Implement cloud hot-swap
4. Add LoRA listing/deletion
5. Add CLI commands

**Key Files**:
- `src/lora/upload.rs` - Chunked upload implementation
- `src/lora/client.rs` - LoRA REST API client
- `crates/synesis-cli/src/commands/cloud.rs` - CLI commands

**Implementation Details**:
```rust
pub async fn upload_lora(
    client: &LoraClient,
    lora_path: &Path,
    name: &str,
    base_model: &str,
) -> Result<CloudLora> {
    // 1. Calculate checksum
    let checksum = calculate_sha256(lora_path).await?;
    let size = fs::metadata(lora_path)?.len();

    // 2. Initiate upload
    let upload = client.initiate_upload(name, base_model, size, checksum).await?;

    // 3. Upload in chunks
    const CHUNK_SIZE: usize = 5 * 1024 * 1024;  // 5MB
    let file = File::open(lora_path)?;
    let mut reader = BufReader::new(file);
    let mut chunk_index = 0;

    loop {
        let mut chunk = vec![0u8; CHUNK_SIZE];
        let n = reader.read(&mut chunk)?;
        if n == 0 { break; }
        chunk.truncate(n);

        client.upload_chunk(&upload.id, chunk_index, &chunk).await?;
        chunk_index += 1;
    }

    // 4. Complete upload
    let cloud_lora = client.complete_upload(&upload.id).await?;

    Ok(cloud_lora)
}
```

**Acceptance Criteria**:
- [ ] LoRAs upload in chunks
- [ ] Stored in R2 successfully
- [ ] Cloud can load and use LoRAs
- [ ] CLI commands work
- [ ] All tests passing

---

### **Session 2.9: Collaborator System**
**Priority**: MEDIUM - Multi-user feature
**Effort**: 4-5 hours
**Dependencies**: Session 2.7

**Objectives**:
1. Implement invite creation
2. Add guest quota enforcement
3. Implement project handover
4. Add role-based access control
5. Add CLI commands

**Key Files**:
- `src/collaborator/invite.rs` - Invite management
- `src/collaborator/handover.rs` - Project handover
- `crates/synesis-cli/src/commands/collaborate.rs` - CLI commands

**Implementation Details**:
```rust
pub struct Invite {
    pub token: String,
    pub url: String,
    pub role: CollaboratorRole,
    pub quota_cents: u32,
    pub expires_at: DateTime<Utc>,
}

pub enum CollaboratorRole {
    Viewer,    // Can view sessions
    Commenter, // Can add comments
    Editor,    // Can modify prompts
}

pub async fn create_invite(
    client: &CollaboratorClient,
    project_id: &str,
    role: CollaboratorRole,
    quota_cents: u32,
    expires_hours: u32,
) -> Result<Invite> {
    client.post("/invites", json!({
        "project_id": project_id,
        "role": role,
        "quota_cents": quota_cents,
        "expires_hours": expires_hours,
    })).await
}
```

**Acceptance Criteria**:
- [ ] Invites can be created
- [ ] Guest quota enforced
- [ ] Handover flow works
- [ ] RBAC enforced
- [ ] CLI commands work

---

### **Session 2.10: Response Streaming**
**Priority**: HIGH - UX improvement
**Effort**: 3-4 hours
**Dependencies**: Session 2.4

**Objectives**:
1. Implement streaming client
2. Add StreamChunk parsing
3. Integrate with CLI for real-time output
4. Add error handling
5. Add streaming tests

**Key Files**:
- `src/escalation/streaming.rs` - Streaming client
- `src/protocol/stream.rs` - StreamChunk types

**Implementation Details**:
```rust
pub struct EscalationStream {
    recv: quinn::RecvStream,
    buffer: BytesMut,
}

impl Stream for EscalationStream {
    type Item = Result<StreamChunk>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<StreamChunk>>> {
        // Read frame (type + length + payload)
        let frame = ready!(self.read_frame(cx)?)?;
        let chunk = StreamChunk::from_frame(&frame)?;
        Poll::Ready(Some(Ok(chunk)))
    }
}

pub enum StreamChunk {
    Content { text: String },
    Metadata { agent: String, phase: String },
    End { tokens_used: TokenUsage, cost_cents: u32 },
    Error { code: String, message: String },
}
```

**Acceptance Criteria**:
- [ ] Streaming works end-to-end
- [ ] Content chunks received in order
- [ ] Metadata events work
- [ ] Errors handled gracefully
- [ ] All tests passing

---

### **Session 2.11: Integration Testing**
**Priority**: HIGH - Quality assurance
**Effort**: 4-5 hours
**Dependencies**: Session 2.10

**Objectives**:
1. Create mock cloud server
2. Add full-flow integration tests
3. Add privacy verification tests
4. Add billing accuracy tests
5. Add tunnel resilience tests

**Key Files**:
- `tests/integration/mock_server.rs` - Mock QUIC server
- `tests/integration/full_flow_tests.rs` - End-to-end tests
- `tests/integration/privacy_tests.rs` - Privacy verification

**Implementation Details**:
```rust
#[tokio::test]
async fn test_full_escalation_with_privacy() {
    // Start mock server
    let server = MockCloudServer::start().await;

    // Create tunnel
    let mut tunnel = CloudTunnel::new(server.config()).unwrap();
    tunnel.connect().await.unwrap();

    // Create privacy proxy
    let vault = TokenVault::in_memory().unwrap();
    let redactor = Redactor::new(vault, Default::default());

    // Query with sensitive data
    let query = "Contact john@example.com about API key sk-abc123";
    let redacted = redactor.redact(query).await.unwrap();

    // Escalate
    let response = client.escalate(EscalationRequest {
        query: redacted,
        ...
    }).await.unwrap();

    // Verify server never saw sensitive data
    let received = server.received_requests().await;
    for req in received {
        assert!(!req.query.contains("john@example.com"));
        assert!(!req.query.contains("sk-abc123"));
    }
}
```

**Acceptance Criteria**:
- [ ] All integration tests pass
- [ ] Privacy verified in tests
- [ ] Billing accuracy verified
- [ ] Tunnel resilience tested

---

### **Session 2.12: CLI Commands Completion**
**Priority**: HIGH - User-facing functionality
**Effort**: 3-4 hours
**Dependencies**: Session 2.11

**Objectives**:
1. Complete all `synesis cloud` commands
2. Add `synesis ask --cloud` flag
3. Add `synesis push` for LoRA upload
4. Add `synesis invite` for collaborators
5. Add comprehensive help text

**CLI Commands**:
```bash
# Cloud connection
synesis cloud login <api-key>
synesis cloud status
synesis cloud logout

# Cloud escalation
synesis ask "query" --cloud
synesis ask "query" --cloud --stream
synesis ask "query" --cloud --model claude-opus

# LoRA management
synesis model list --cloud
synesis push <path> --name <name> --base-model <model>

# Collaborators
synesis invite create --project <id> --role viewer --quota 5.00
synesis invite list
synesis invite revoke <token>

# Billing
synesis cloud balance
synesis cloud usage --days 30
```

**Acceptance Criteria**:
- [ ] All CLI commands implemented
- [ ] Help text comprehensive
- [ ] Error messages user-friendly
- [ ] All commands tested

---

## Testing Strategy

### Unit Tests
- Each module has comprehensive unit tests
- Target: >80% code coverage
- Run: `cargo test --package synesis-cloud`

### Integration Tests
- Mock cloud server for realistic testing
- Full flow tests (CLI → cloud → CLI)
- Privacy verification tests
- Billing accuracy tests
- Run: `cargo test --test integration`

### Cloud Worker Tests
- Durable Object unit tests
- API endpoint tests
- Run: `cd cloud && npm test`

### End-to-End Tests
- Full CLI workflows with real cloud (staging)
- Run: `./tests/e2e/test_full_flow.sh`

---

## Security Considerations

### Must Implement
1. ✅ TLS 1.3 only (no TLS 1.2 fallback)
2. ✅ mTLS for device authentication
3. ✅ API keys stored in system keychain
4. ✅ Privacy proxy enforced before cloud
5. ✅ Rate limiting enforced
6. ✅ Credit ceiling enforced server-side
7. ✅ Input validation on all requests
8. ✅ SQL injection prevention (parameterized queries)

### Must Verify
- [ ] No secrets in code
- [ ] All TLS certificates valid
- [ ] Certificate pinning implemented
- [ ] No sensitive data in logs
- [ ] Cross-tenant isolation tested
- [ ] RBAC enforced correctly

---

## Performance Targets

| Metric | Target |
|--------|--------|
| QUIC handshake | <500ms |
| Cloud escalation (non-streaming) | <2s |
| Streaming latency (first byte) | <500ms |
| Heartbeat overhead | <1% CPU |
| Tunnel reconnection | <5s |
| Billing sync | <500ms |

---

## Milestones

| Milestone | Sessions | Target Date |
|-----------|----------|-------------|
| **M1: Foundation** | 2.1-2.3 | Day 2 |
| **M2: Core Cloud** | 2.4-2.6 | Day 4 |
| **M3: Cloud Workers** | 2.7-2.8 | Day 7 |
| **M4: Advanced Features** | 2.9-2.10 | Day 9 |
| **M5: Polish & Test** | 2.11-2.12 | Day 11 |
| **M6: Production Deploy** | All sessions | Day 14 |

---

## Current Status

**Session**: 2.0 (Planning) - ✅ COMPLETE
**Next**: Session 2.1 - synesis-cloud Crate Setup
**Blocked**: None
**Cloudflare Access**: Available for testing

---

## Success Criteria

Phase 2 is complete when:
- [ ] All 12 sessions implemented
- [ ] All tests passing (>80% coverage)
- [ ] Cloud deployed to staging
- [ ] End-to-end flow tested (CLI → cloud → CLI)
- [ ] Privacy verified (no sensitive data leaked)
- [ ] Billing accurate (to the cent)
- [ ] Documentation complete
- [ ] Performance targets met

---

**Roadmap Version**: 2.0.0
**Last Updated**: 2026-01-02
**Status**: Ready to begin Session 2.1
