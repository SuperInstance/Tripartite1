# Phase 2 Implementation Sessions

**Purpose**: Step-by-step prompts for Claude Code agent to implement Phase 2: Cloud Mesh
**Prerequisite**: Phase 1 complete (149/149 tests passing)
**Duration**: 4-6 weeks

---

## Overview

Phase 2 transforms SuperInstance from a local tool into a commercial platform with:
- Cloud escalation via QUIC tunnel
- Automated billing with Stripe
- LoRA hot-swap for cloud inference
- Collaborator system for client handover
- Cloud Synapse (cloud-side consensus engine)

---

## Session 2.1: Cloud Client Crate Setup

### Prompt

```
Create a new crate `synesis-cloud` for cloud client functionality.

This crate will handle:
1. QUIC tunnel connection to Cloudflare
2. Cloud escalation requests
3. Heartbeat/telemetry
4. Pre-warm signaling
5. Response streaming

First, create the crate structure:

1. Add to workspace Cargo.toml:
   - synesis-cloud = { path = "crates/synesis-cloud" }

2. Create crates/synesis-cloud/Cargo.toml with dependencies:
   - quinn (QUIC implementation)
   - rustls (TLS)
   - tokio
   - serde, serde_json
   - anyhow, thiserror
   - tracing
   - prost (protobuf)
   - chrono
   - uuid

3. Create module structure:
   - src/lib.rs (exports)
   - src/tunnel.rs (QUIC tunnel implementation)
   - src/escalation.rs (cloud escalation requests)
   - src/heartbeat.rs (heartbeat telemetry)
   - src/prewarm.rs (pre-warm signaling)
   - src/stream.rs (response streaming)
   - src/proto.rs (protobuf message types)

4. Define error types in src/lib.rs:
   - CloudError::ConnectionFailed
   - CloudError::TunnelClosed
   - CloudError::AuthenticationFailed
   - CloudError::RateLimited
   - CloudError::EscalationFailed
   - CloudError::Timeout

5. Create stub implementations with TODO markers.

Run `cargo build` to verify workspace compiles.
```

### Expected Output
- New crate at `crates/synesis-cloud/`
- All modules compile with stubs
- No warnings in new crate

---

## Session 2.2: QUIC Tunnel Core Implementation

### Prompt

```
Implement the QUIC tunnel in synesis-cloud/src/tunnel.rs.

Requirements:

1. CloudTunnel struct:
   ```rust
   pub struct CloudTunnel {
       endpoint: quinn::Endpoint,
       connection: Option<quinn::Connection>,
       config: TunnelConfig,
       heartbeat_handle: Option<JoinHandle<()>>,
       state: Arc<RwLock<TunnelState>>,
   }
   
   pub struct TunnelConfig {
       pub cloud_url: String,
       pub device_id: String,
       pub cert_path: PathBuf,
       pub key_path: PathBuf,
       pub heartbeat_interval: Duration,
       pub reconnect_delay: Duration,
       pub max_reconnect_attempts: u32,
   }
   
   pub enum TunnelState {
       Disconnected,
       Connecting,
       Connected { since: Instant },
       Reconnecting { attempt: u32 },
       Failed { error: String },
   }
   ```

2. Implement methods:
   - `new(config: TunnelConfig) -> Result<Self>`
   - `connect(&mut self) -> Result<()>` - Establish mTLS connection
   - `disconnect(&mut self) -> Result<()>` - Graceful shutdown
   - `is_connected(&self) -> bool`
   - `state(&self) -> TunnelState`
   - `send(&self, data: &[u8]) -> Result<()>` - Send on unidirectional stream
   - `request(&self, data: &[u8]) -> Result<Vec<u8>>` - Send/receive on bidirectional stream
   - `open_stream(&self) -> Result<(SendStream, RecvStream)>` - For streaming responses

3. Auto-reconnect logic:
   - Spawn background task that monitors connection
   - On disconnect, attempt reconnect with exponential backoff
   - Emit events via channel for state changes

4. mTLS configuration:
   - Load device certificate and key from files
   - Configure rustls with client authentication
   - Verify server certificate against CA bundle

5. Add comprehensive tests:
   - Test connection state transitions
   - Test reconnection logic (mock server)
   - Test graceful shutdown
   - Test concurrent stream handling

Reference the quinn documentation for QUIC specifics.
```

### Expected Output
- Working QUIC tunnel with mTLS
- Auto-reconnect with backoff
- State machine for connection lifecycle
- Tests for all state transitions

---

## Session 2.3: Heartbeat and Telemetry

### Prompt

```
Implement heartbeat telemetry in synesis-cloud/src/heartbeat.rs.

The heartbeat system sends periodic updates to cloud with device vitals.

1. Create HeartbeatService:
   ```rust
   pub struct HeartbeatService {
       tunnel: Arc<CloudTunnel>,
       interval: Duration,
       shutdown: broadcast::Sender<()>,
       vitals_collector: Arc<dyn VitalsCollector>,
   }
   
   pub trait VitalsCollector: Send + Sync {
       fn collect(&self) -> DeviceVitals;
   }
   
   pub struct DeviceVitals {
       pub device_id: String,
       pub timestamp: i64,
       pub cpu_usage: f32,
       pub memory_usage: f32,
       pub gpu_usage: Option<f32>,
       pub gpu_temp: Option<f32>,
       pub disk_usage: f32,
       pub active_sessions: u32,
       pub pending_queries: u32,
   }
   ```

2. Implement heartbeat loop:
   - Collect vitals every N seconds (configurable)
   - Serialize to protobuf (see proto.rs)
   - Send via tunnel unidirectional stream
   - Handle send failures gracefully (log, don't crash)
   - Respect shutdown signal

3. Create default VitalsCollector using synesis-models hardware module:
   ```rust
   pub struct DefaultVitalsCollector {
       hardware: Arc<HardwareManifest>,
   }
   
   impl VitalsCollector for DefaultVitalsCollector {
       fn collect(&self) -> DeviceVitals {
           // Use sysinfo for CPU/RAM
           // Use hardware manifest for GPU
           // Query active session count from shared state
       }
   }
   ```

4. Add pre-warm signaling (when GPU is hot):
   ```rust
   impl HeartbeatService {
       pub fn check_prewarm_condition(&self, vitals: &DeviceVitals) -> bool {
           vitals.gpu_usage.map(|u| u > 0.8).unwrap_or(false)
               || vitals.gpu_temp.map(|t| t > 80.0).unwrap_or(false)
       }
       
       pub async fn send_prewarm_signal(&self, project_id: &str, lora_id: Option<&str>) {
           // Send prewarm message to cloud
       }
   }
   ```

5. Tests:
   - Test vitals collection
   - Test heartbeat timing
   - Test graceful shutdown
   - Test prewarm condition detection
```

### Expected Output
- HeartbeatService with configurable interval
- DeviceVitals collection from hardware
- Pre-warm signaling when GPU is stressed
- Clean shutdown handling

---

## Session 2.4: Cloud Escalation Client

### Prompt

```
Implement cloud escalation in synesis-cloud/src/escalation.rs.

This is the main interface for sending queries to cloud when local processing is insufficient.

1. Create EscalationClient:
   ```rust
   pub struct EscalationClient {
       tunnel: Arc<CloudTunnel>,
       config: EscalationConfig,
       metrics: Arc<EscalationMetrics>,
   }
   
   pub struct EscalationConfig {
       pub timeout: Duration,
       pub max_retries: u32,
       pub retry_delay: Duration,
       pub default_model: CloudModel,
   }
   
   pub enum CloudModel {
       Claude3Sonnet,
       Claude3Opus,
       Gpt4Turbo,
       Auto,  // Let cloud decide based on query complexity
   }
   
   pub struct EscalationRequest {
       pub request_id: String,
       pub query: String,  // Already redacted by privacy proxy
       pub context: EscalationContext,
       pub model: CloudModel,
       pub max_tokens: u32,
       pub stream: bool,
   }
   
   pub struct EscalationContext {
       pub pathos_framing: Option<String>,
       pub local_knowledge: Vec<String>,  // Relevant chunks from RAG
       pub conversation_history: Vec<Message>,
       pub constraints: Vec<String>,  // From Ethos
   }
   
   pub struct EscalationResponse {
       pub request_id: String,
       pub content: String,
       pub model_used: String,
       pub tokens_used: TokenUsage,
       pub cost_cents: u32,
       pub latency_ms: u64,
   }
   ```

2. Implement methods:
   ```rust
   impl EscalationClient {
       pub async fn escalate(&self, request: EscalationRequest) -> Result<EscalationResponse>
       
       pub async fn escalate_streaming(
           &self, 
           request: EscalationRequest
       ) -> Result<impl Stream<Item = Result<String>>>
       
       pub fn metrics(&self) -> &EscalationMetrics
   }
   ```

3. Request/response flow:
   - Generate request_id (UUID)
   - Serialize request to protobuf
   - Send via bidirectional stream
   - Await response with timeout
   - Deserialize and validate response
   - Update metrics

4. Retry logic:
   - Retry on transient failures (timeout, connection reset)
   - Don't retry on auth failures or rate limits
   - Exponential backoff between retries

5. Metrics tracking:
   ```rust
   pub struct EscalationMetrics {
       pub total_requests: AtomicU64,
       pub successful_requests: AtomicU64,
       pub failed_requests: AtomicU64,
       pub total_tokens: AtomicU64,
       pub total_cost_cents: AtomicU64,
       pub latency_histogram: Mutex<Vec<u64>>,
   }
   ```

6. Integration with Council:
   - Add `cloud_client: Option<Arc<EscalationClient>>` to Council
   - Modify routing.rs to use escalation client when decision is Cloud
   - Reinflate tokens in response using privacy proxy

7. Tests:
   - Test request serialization
   - Test timeout handling
   - Test retry logic
   - Test metrics accumulation
```

### Expected Output
- Complete EscalationClient implementation
- Streaming support for long responses
- Retry logic with backoff
- Metrics collection
- Integration point with Council

---

## Session 2.5: Protobuf Message Definitions

### Prompt

```
Create protobuf message definitions for cloud communication.

1. Create proto/cloud.proto:
   ```protobuf
   syntax = "proto3";
   package superinstance.cloud;
   
   // Heartbeat messages
   message Heartbeat {
       string device_id = 1;
       int64 timestamp = 2;
       DeviceVitals vitals = 3;
   }
   
   message DeviceVitals {
       float cpu_usage = 1;
       float memory_usage = 2;
       optional float gpu_usage = 3;
       optional float gpu_temp = 4;
       float disk_usage = 5;
       uint32 active_sessions = 6;
       uint32 pending_queries = 7;
   }
   
   message PrewarmSignal {
       string project_id = 1;
       optional string lora_id = 2;
       int64 timestamp = 3;
   }
   
   // Escalation messages
   message EscalationRequest {
       string request_id = 1;
       string user_id = 2;
       string session_id = 3;
       string query = 4;
       EscalationContext context = 5;
       CloudModel model = 6;
       uint32 max_tokens = 7;
       bool stream = 8;
   }
   
   message EscalationContext {
       optional string pathos_framing = 1;
       repeated string local_knowledge = 2;
       repeated Message conversation_history = 3;
       repeated string constraints = 4;
   }
   
   message Message {
       string role = 1;  // "user" or "assistant"
       string content = 2;
   }
   
   enum CloudModel {
       CLOUD_MODEL_UNSPECIFIED = 0;
       CLOUD_MODEL_CLAUDE_SONNET = 1;
       CLOUD_MODEL_CLAUDE_OPUS = 2;
       CLOUD_MODEL_GPT4_TURBO = 3;
       CLOUD_MODEL_AUTO = 4;
   }
   
   message EscalationResponse {
       string request_id = 1;
       string content = 2;
       string model_used = 3;
       TokenUsage tokens_used = 4;
       uint32 cost_cents = 5;
       uint64 latency_ms = 6;
   }
   
   message TokenUsage {
       uint32 prompt = 1;
       uint32 completion = 2;
   }
   
   // Streaming chunk
   message StreamChunk {
       string request_id = 1;
       oneof payload {
           string content = 2;
           StreamEnd end = 3;
           StreamError error = 4;
       }
   }
   
   message StreamEnd {
       TokenUsage tokens_used = 1;
       uint32 cost_cents = 2;
   }
   
   message StreamError {
       string code = 1;
       string message = 2;
   }
   ```

2. Configure prost build:
   - Add build.rs to synesis-cloud
   - Generate Rust types from .proto files
   - Include generated code in proto.rs module

3. Create manual Rust equivalents as fallback (if protobuf adds too much complexity):
   ```rust
   // src/messages.rs
   use serde::{Serialize, Deserialize};
   
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct Heartbeat { ... }
   ```

4. Add serialization tests:
   - Round-trip serialize/deserialize
   - Compatibility with TypeScript cloud workers
   - Size optimization verification
```

### Expected Output
- Proto definitions or Rust message types
- Serialization working both directions
- Compatible with existing cloud workers

---

## Session 2.6: Billing Client Integration

### Prompt

```
Create billing client in synesis-cloud for tracking usage and costs.

1. Create src/billing.rs:
   ```rust
   pub struct BillingClient {
       tunnel: Arc<CloudTunnel>,
       user_id: String,
       tier: BillingTier,
       local_ledger: Arc<Mutex<LocalLedger>>,
   }
   
   pub enum BillingTier {
       Free { monthly_limit_cents: u32 },
       Managed { markup_percent: f32 },  // 3%
       Byok { licensing_percent: f32 },   // 30%
   }
   
   pub struct LocalLedger {
       pub unbilled_cents: u32,
       pub knowledge_credits_cents: u32,
       pub last_sync: Instant,
       pub pending_events: Vec<UsageEvent>,
   }
   
   pub struct UsageEvent {
       pub id: String,
       pub timestamp: i64,
       pub tokens_in: u32,
       pub tokens_out: u32,
       pub model: String,
       pub cost_basis_cents: u32,  // Before markup
       pub final_cost_cents: u32,  // After markup
   }
   ```

2. Implement methods:
   ```rust
   impl BillingClient {
       pub async fn record_usage(&self, event: UsageEvent) -> Result<()>
       pub async fn sync_with_cloud(&self) -> Result<()>
       pub async fn get_balance(&self) -> Result<Balance>
       pub async fn add_credits(&self, cents: u32) -> Result<()>
       pub fn current_usage(&self) -> UsageSummary
   }
   ```

3. Local-first billing:
   - Record usage locally first (immediate)
   - Batch sync to cloud every N seconds or M cents
   - Handle offline gracefully (queue events)
   - Prevent overspend with local ceiling check

4. Credit system integration:
   - Track knowledge credits locally
   - Apply credits before calculating charge
   - Sync credit balance with cloud

5. CLI commands for billing:
   ```bash
   synesis cloud balance     # Show current balance
   synesis cloud usage       # Show usage breakdown
   synesis cloud topup $10   # Add credits (opens Stripe checkout)
   ```

6. Tests:
   - Test usage recording
   - Test markup calculations
   - Test credit application
   - Test offline queueing
   - Test sync batching
```

### Expected Output
- BillingClient with local-first recording
- Sync mechanism with cloud
- Credit system working
- CLI commands integrated

---

## Session 2.7: Cloud Worker Durable Objects

### Prompt

```
Enhance Cloudflare Workers with Durable Objects for stateful operations.

This session focuses on the cloud-side implementation.

1. Create cloud/durable-objects/billing-ledger.ts:
   ```typescript
   export class BillingLedger extends DurableObject {
       private state: DurableObjectState;
       private ledger: LedgerState;
       
       constructor(state: DurableObjectState, env: Env) {
           super(state, env);
           this.state = state;
           this.ledger = {
               userId: '',
               tier: 'managed',
               unbilledCents: 0,
               knowledgeCredits: 0,
               creditCeiling: 10000, // $100 default
               stripeCustomerId: null,
               usageHistory: [],
           };
       }
       
       async fetch(request: Request): Promise<Response> {
           // Load state
           const stored = await this.state.storage.get<LedgerState>('ledger');
           if (stored) this.ledger = stored;
           
           const url = new URL(request.url);
           
           switch (url.pathname) {
               case '/record':
                   return this.handleRecord(request);
               case '/balance':
                   return this.handleBalance();
               case '/sync':
                   return this.handleSync(request);
               case '/add-credits':
                   return this.handleAddCredits(request);
               case '/set-tier':
                   return this.handleSetTier(request);
               default:
                   return new Response('Not Found', { status: 404 });
           }
       }
       
       private async handleRecord(request: Request): Promise<Response> {
           const event: UsageEvent = await request.json();
           
           // Calculate charge with markup
           const markup = this.ledger.tier === 'managed' ? 1.03 : 1.30;
           let charge = Math.ceil(event.costBasisCents * markup);
           
           // Apply knowledge credits
           if (this.ledger.knowledgeCredits > 0) {
               const creditDeduction = Math.min(this.ledger.knowledgeCredits, charge);
               this.ledger.knowledgeCredits -= creditDeduction;
               charge -= creditDeduction;
           }
           
           // Update balance
           this.ledger.unbilledCents += charge;
           this.ledger.usageHistory.push({
               ...event,
               finalCharge: charge,
               timestamp: Date.now(),
           });
           
           // Persist
           await this.state.storage.put('ledger', this.ledger);
           
           // Check ceiling
           if (this.ledger.unbilledCents >= this.ledger.creditCeiling) {
               return jsonResponse({
                   error: 'credit_ceiling_exceeded',
                   action: 'suspend_cloud',
               }, 402);
           }
           
           // Flush to Stripe if threshold reached ($5)
           if (this.ledger.unbilledCents >= 500) {
               await this.flushToStripe();
           }
           
           return jsonResponse({
               charged: charge,
               balance: this.ledger.unbilledCents,
               credits: this.ledger.knowledgeCredits,
           });
       }
       
       private async flushToStripe(): Promise<void> {
           if (!this.ledger.stripeCustomerId) return;
           
           // Create Stripe meter event
           // See Stripe metered billing docs
       }
   }
   ```

2. Create cloud/durable-objects/session-state.ts:
   ```typescript
   export class SessionState extends DurableObject {
       // Stores conversation history and context for a session
       // Enables cloud consensus to maintain context
       
       private sessions: Map<string, SessionData>;
       
       async handleGetSession(sessionId: string): Promise<SessionData>
       async handleUpdateSession(sessionId: string, data: Partial<SessionData>): Promise<void>
       async handleClearSession(sessionId: string): Promise<void>
   }
   ```

3. Create cloud/durable-objects/cloud-synapse.ts:
   ```typescript
   export class CloudSynapse extends DurableObject {
       // Cloud-side consensus engine
       // Mirrors local tripartite council
       
       async handleConsensus(request: ConsensusRequest): Promise<ConsensusResponse>
       private async runPathos(prompt: string): Promise<PathosResult>
       private async runLogos(manifest: Manifest, loraId?: string): Promise<LogosResult>
       private async runEthos(solution: string, manifest: Manifest): Promise<EthosResult>
       private calculateScore(pathos: PathosResult, logos: LogosResult, ethos: EthosResult): number
   }
   ```

4. Update cloud/wrangler.toml:
   ```toml
   [[durable_objects.bindings]]
   name = "BILLING_LEDGER"
   class_name = "BillingLedger"
   
   [[durable_objects.bindings]]
   name = "SESSION_STATE"
   class_name = "SessionState"
   
   [[durable_objects.bindings]]
   name = "CLOUD_SYNAPSE"
   class_name = "CloudSynapse"
   
   [[migrations]]
   tag = "v1"
   new_classes = ["BillingLedger", "SessionState", "CloudSynapse"]
   ```

5. Tests:
   - Unit tests for each Durable Object
   - Integration tests with miniflare
```

### Expected Output
- Three Durable Object classes
- Wrangler configuration updated
- Basic tests passing

---

## Session 2.8: LoRA Upload and Hot-Swap

### Prompt

```
Implement LoRA upload and cloud-side hot-swap.

1. Add to synesis-cloud/src/lora.rs:
   ```rust
   pub struct LoraManager {
       tunnel: Arc<CloudTunnel>,
       local_loras: Arc<Mutex<HashMap<String, LoraInfo>>>,
   }
   
   pub struct LoraInfo {
       pub id: String,
       pub name: String,
       pub base_model: String,
       pub path: PathBuf,
       pub size_bytes: u64,
       pub uploaded: bool,
       pub cloud_id: Option<String>,
   }
   
   impl LoraManager {
       pub async fn discover_local_loras(&self) -> Result<Vec<LoraInfo>>
       pub async fn upload(&self, lora_id: &str) -> Result<String>  // Returns cloud_id
       pub async fn list_cloud_loras(&self) -> Result<Vec<CloudLoraInfo>>
       pub async fn delete_cloud_lora(&self, cloud_id: &str) -> Result<()>
   }
   ```

2. Upload flow:
   - Read LoRA file from local path
   - Calculate checksum
   - Compress if beneficial
   - Stream upload to cloud via tunnel
   - Receive cloud_id on success
   - Update local registry

3. CLI command `synesis push`:
   ```rust
   // crates/synesis-cli/src/commands/push.rs
   
   pub async fn push_lora(
       lora_name: Option<String>,
       force: bool,
   ) -> Result<()> {
       let manager = get_lora_manager()?;
       
       // Select LoRA
       let lora = if let Some(name) = lora_name {
           manager.get_by_name(&name)?
       } else {
           // Interactive selection
           select_lora_interactive(&manager)?
       };
       
       // Check if already uploaded
       if lora.uploaded && !force {
           println!("LoRA already uploaded. Use --force to re-upload.");
           return Ok(());
       }
       
       // Upload with progress
       println!("Uploading {} ({})...", lora.name, format_bytes(lora.size_bytes));
       
       let pb = ProgressBar::new(lora.size_bytes);
       let cloud_id = manager.upload_with_progress(&lora.id, |bytes| {
           pb.set_position(bytes);
       }).await?;
       
       pb.finish_with_message("Upload complete!");
       
       println!("✓ LoRA available in cloud: {}", cloud_id);
       
       Ok(())
   }
   ```

4. Cloud-side LoRA loading (TypeScript):
   ```typescript
   // cloud/workers/lora-manager/index.ts
   
   export async function loadLora(loraId: string, env: Env): Promise<void> {
       // Check cache first
       const cached = await env.LORA_CACHE.get(loraId);
       if (cached) {
           return;  // Already loaded
       }
       
       // Load from R2
       const loraObject = await env.LORA_BUCKET.get(loraId);
       if (!loraObject) {
           throw new Error(`LoRA not found: ${loraId}`);
       }
       
       // Decompress and load into Workers AI runtime
       const loraData = await loraObject.arrayBuffer();
       
       // Cache for future requests
       await env.LORA_CACHE.put(loraId, loraData, {
           expirationTtl: 3600,  // 1 hour
       });
   }
   
   export async function runInferenceWithLora(
       request: InferenceRequest,
       loraId: string,
       env: Env
   ): Promise<InferenceResponse> {
       await loadLora(loraId, env);
       
       // Run inference with LoRA adapter
       const response = await env.AI.run(
           request.baseModel,
           {
               prompt: request.prompt,
               lora: loraId,
               max_tokens: request.maxTokens,
           }
       );
       
       return response;
   }
   ```

5. Tests:
   - Test local LoRA discovery
   - Test upload chunking
   - Test cloud-side loading
   - Test inference with LoRA
```

### Expected Output
- LoraManager in synesis-cloud
- `synesis push` CLI command
- Cloud-side LoRA loading
- Integration tests

---

## Session 2.9: Collaborator System

### Prompt

```
Implement the collaborator system for client handover.

1. Create synesis-cloud/src/collaborator.rs:
   ```rust
   pub struct CollaboratorClient {
       tunnel: Arc<CloudTunnel>,
       user_id: String,
   }
   
   pub struct InviteRequest {
       pub project_id: String,
       pub role: CollaboratorRole,
       pub quota_cents: u32,
       pub expires_hours: u32,
   }
   
   pub enum CollaboratorRole {
       Viewer,    // Can watch agent interactions
       Commenter, // Can add feedback
       Editor,    // Can modify prompts
   }
   
   pub struct Invite {
       pub token: String,
       pub url: String,
       pub expires_at: DateTime<Utc>,
   }
   
   impl CollaboratorClient {
       pub async fn create_invite(&self, request: InviteRequest) -> Result<Invite>
       pub async fn list_invites(&self, project_id: &str) -> Result<Vec<Invite>>
       pub async fn revoke_invite(&self, token: &str) -> Result<()>
       pub async fn list_collaborators(&self, project_id: &str) -> Result<Vec<Collaborator>>
       pub async fn remove_collaborator(&self, project_id: &str, user_id: &str) -> Result<()>
   }
   ```

2. Create handover flow:
   ```rust
   pub struct HandoverClient {
       tunnel: Arc<CloudTunnel>,
   }
   
   pub struct HandoverRequest {
       pub project_id: String,
       pub to_email: String,
       pub include_loras: bool,
       pub include_knowledge: bool,
   }
   
   pub struct HandoverStatus {
       pub token: String,
       pub status: HandoverState,
       pub created_at: DateTime<Utc>,
       pub completed_at: Option<DateTime<Utc>>,
   }
   
   pub enum HandoverState {
       Pending,
       EmailSent,
       Accepted,
       Completed,
       Expired,
       Cancelled,
   }
   
   impl HandoverClient {
       pub async fn initiate(&self, request: HandoverRequest) -> Result<String>  // Token
       pub async fn status(&self, token: &str) -> Result<HandoverStatus>
       pub async fn cancel(&self, token: &str) -> Result<()>
   }
   ```

3. CLI commands:
   ```bash
   synesis invite create --project myproject --role viewer --quota 5.00
   synesis invite list --project myproject
   synesis invite revoke <token>
   
   synesis handover start --project myproject --to client@example.com
   synesis handover status <token>
   synesis handover cancel <token>
   ```

4. Cloud-side implementation:
   ```typescript
   // cloud/workers/collaborator/index.ts
   
   export async function createInvite(
       userId: string,
       request: InviteRequest,
       env: Env
   ): Promise<Invite> {
       const token = crypto.randomUUID();
       
       const invite: InviteRecord = {
           token,
           projectId: request.projectId,
           hostUserId: userId,
           role: request.role,
           guestQuota: request.quotaCents,
           expiresAt: Date.now() + request.expiresHours * 3600000,
           createdAt: Date.now(),
       };
       
       await env.INVITE_KV.put(token, JSON.stringify(invite), {
           expirationTtl: request.expiresHours * 3600,
       });
       
       return {
           token,
           url: `https://app.superinstance.ai/join/${token}`,
           expiresAt: new Date(invite.expiresAt).toISOString(),
       };
   }
   ```

5. Tests:
   - Test invite creation/revocation
   - Test collaborator role permissions
   - Test handover flow
   - Test quota enforcement
```

### Expected Output
- CollaboratorClient and HandoverClient
- CLI commands for invite/handover
- Cloud-side worker implementation
- E2E test for invite flow

---

## Session 2.10: Response Streaming

### Prompt

```
Implement real-time response streaming from cloud to local.

1. Create synesis-cloud/src/stream.rs:
   ```rust
   use futures::Stream;
   use tokio_stream::StreamExt;
   
   pub struct StreamingResponse {
       request_id: String,
       stream: Pin<Box<dyn Stream<Item = Result<StreamChunk>> + Send>>,
   }
   
   pub enum StreamChunk {
       Content(String),
       Metadata(ChunkMetadata),
       End(StreamEnd),
       Error(StreamError),
   }
   
   pub struct ChunkMetadata {
       pub agent: String,  // "pathos", "logos", "ethos"
       pub phase: String,  // "thinking", "responding"
   }
   
   pub struct StreamEnd {
       pub tokens_used: TokenUsage,
       pub cost_cents: u32,
       pub total_latency_ms: u64,
   }
   
   impl StreamingResponse {
       pub async fn next(&mut self) -> Option<Result<StreamChunk>>
       pub fn request_id(&self) -> &str
   }
   ```

2. Implement streaming escalation:
   ```rust
   impl EscalationClient {
       pub async fn escalate_streaming(
           &self,
           request: EscalationRequest,
       ) -> Result<StreamingResponse> {
           let stream = self.tunnel.open_stream().await?;
           
           // Send request
           stream.send(request.encode()?).await?;
           
           // Create stream transformer
           let chunk_stream = stream.recv_stream()
               .map(|bytes| {
                   StreamChunk::decode(&bytes)
               });
           
           Ok(StreamingResponse {
               request_id: request.request_id,
               stream: Box::pin(chunk_stream),
           })
       }
   }
   ```

3. CLI streaming output:
   ```rust
   // In ask.rs
   if request.stream {
       let mut response = client.escalate_streaming(request).await?;
       
       let mut display = StreamingDisplay::new();
       
       while let Some(chunk) = response.next().await {
           match chunk? {
               StreamChunk::Content(text) => {
                   display.print_chunk(&text);
               }
               StreamChunk::Metadata(meta) => {
                   display.show_agent_indicator(&meta.agent);
               }
               StreamChunk::End(end) => {
                   display.finish();
                   println!("\n{}", format_usage(&end));
               }
               StreamChunk::Error(err) => {
                   return Err(anyhow!("Stream error: {}", err.message));
               }
           }
       }
   }
   ```

4. Cloud-side streaming (Server-Sent Events):
   ```typescript
   export async function handleStreamingEscalation(
       request: Request,
       env: Env
   ): Promise<Response> {
       const { readable, writable } = new TransformStream();
       const writer = writable.getWriter();
       const encoder = new TextEncoder();
       
       // Start streaming in background
       (async () => {
           try {
               // Run Pathos
               await sendEvent(writer, encoder, { agent: 'pathos', phase: 'thinking' });
               const pathosResult = await runPathos(request);
               await sendEvent(writer, encoder, { content: `[Pathos: ${pathosResult.intent}]` });
               
               // Run Logos
               await sendEvent(writer, encoder, { agent: 'logos', phase: 'thinking' });
               for await (const chunk of streamLogos(pathosResult)) {
                   await sendEvent(writer, encoder, { content: chunk });
               }
               
               // Run Ethos
               await sendEvent(writer, encoder, { agent: 'ethos', phase: 'verifying' });
               const ethosResult = await runEthos(...);
               
               // Send end event
               await sendEvent(writer, encoder, {
                   end: {
                       tokensUsed: { prompt: 100, completion: 200 },
                       costCents: 5,
                   }
               });
           } finally {
               await writer.close();
           }
       })();
       
       return new Response(readable, {
           headers: {
               'Content-Type': 'text/event-stream',
               'Cache-Control': 'no-cache',
           },
       });
   }
   ```

5. Tests:
   - Test stream creation
   - Test chunk parsing
   - Test error handling mid-stream
   - Test backpressure
```

### Expected Output
- StreamingResponse with async iteration
- CLI streaming display
- Cloud-side SSE implementation
- Proper error handling

---

## Session 2.11: Integration Testing

### Prompt

```
Create comprehensive integration tests for Phase 2 cloud functionality.

1. Create tests/integration/cloud_tests.rs:
   ```rust
   use synesis_cloud::{CloudTunnel, EscalationClient, BillingClient};
   use synesis_privacy::Redactor;
   use synesis_core::Council;
   
   mod test_helpers;
   use test_helpers::{MockCloudServer, TestConfig};
   
   #[tokio::test]
   async fn test_full_escalation_flow() {
       // Start mock cloud server
       let server = MockCloudServer::start().await;
       
       // Create tunnel
       let tunnel = CloudTunnel::new(TunnelConfig {
           cloud_url: server.url(),
           ..Default::default()
       }).unwrap();
       
       tunnel.connect().await.unwrap();
       assert!(tunnel.is_connected());
       
       // Create escalation client
       let client = EscalationClient::new(Arc::new(tunnel), Default::default());
       
       // Send escalation request
       let request = EscalationRequest {
           request_id: "test-123".to_string(),
           query: "What is 2+2?".to_string(),
           ..Default::default()
       };
       
       let response = client.escalate(request).await.unwrap();
       
       assert_eq!(response.request_id, "test-123");
       assert!(!response.content.is_empty());
       assert!(response.cost_cents > 0);
   }
   
   #[tokio::test]
   async fn test_privacy_proxy_with_escalation() {
       let server = MockCloudServer::start().await;
       let tunnel = CloudTunnel::new_test(&server).await;
       
       // Create redactor
       let vault = TokenVault::in_memory().unwrap();
       let redactor = Redactor::new(vault, Default::default());
       
       // Query with sensitive data
       let query = "My email is test@example.com and my API key is sk-abc123";
       
       // Redact
       let redacted = redactor.redact(query).await.unwrap();
       assert!(!redacted.contains("test@example.com"));
       assert!(!redacted.contains("sk-abc123"));
       
       // Escalate with redacted query
       let request = EscalationRequest {
           query: redacted,
           ..Default::default()
       };
       
       let response = client.escalate(request).await.unwrap();
       
       // Reinflate response
       let reinflated = redactor.reinflate(&response.content).await.unwrap();
       
       // Verify no tokens leaked to server
       assert!(!server.received_queries().contains("test@example.com"));
   }
   
   #[tokio::test]
   async fn test_billing_integration() {
       let server = MockCloudServer::start().await;
       let tunnel = Arc::new(CloudTunnel::new_test(&server).await);
       
       let billing = BillingClient::new(
           tunnel.clone(),
           "test-user".to_string(),
           BillingTier::Managed { markup_percent: 0.03 },
       );
       
       // Record some usage
       billing.record_usage(UsageEvent {
           tokens_in: 100,
           tokens_out: 200,
           cost_basis_cents: 10,
           ..Default::default()
       }).await.unwrap();
       
       // Check local ledger
       assert_eq!(billing.current_usage().total_cents, 11);  // 10 * 1.03 = 10.3 → 11
       
       // Sync with cloud
       billing.sync_with_cloud().await.unwrap();
       
       // Verify server received usage
       assert!(server.billing_events().len() > 0);
   }
   
   #[tokio::test]
   async fn test_streaming_response() {
       let server = MockCloudServer::start_streaming().await;
       let tunnel = Arc::new(CloudTunnel::new_test(&server).await);
       let client = EscalationClient::new(tunnel, Default::default());
       
       let request = EscalationRequest {
           query: "Count to 5".to_string(),
           stream: true,
           ..Default::default()
       };
       
       let mut response = client.escalate_streaming(request).await.unwrap();
       
       let mut chunks = Vec::new();
       while let Some(chunk) = response.next().await {
           if let StreamChunk::Content(text) = chunk.unwrap() {
               chunks.push(text);
           }
       }
       
       assert!(chunks.len() >= 5);
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
       server.disconnect_client().await;
       
       // Wait for reconnect
       tokio::time::sleep(Duration::from_millis(500)).await;
       
       // Should be reconnected
       assert!(tunnel.is_connected());
   }
   ```

2. Create test helpers in tests/integration/test_helpers.rs:
   ```rust
   pub struct MockCloudServer {
       addr: SocketAddr,
       shutdown_tx: broadcast::Sender<()>,
       received_queries: Arc<Mutex<Vec<String>>>,
       billing_events: Arc<Mutex<Vec<UsageEvent>>>,
   }
   
   impl MockCloudServer {
       pub async fn start() -> Self {
           // Start QUIC server with mock responses
       }
       
       pub fn url(&self) -> String
       pub fn received_queries(&self) -> Vec<String>
       pub fn billing_events(&self) -> Vec<UsageEvent>
       pub async fn disconnect_client(&self)
       pub async fn shutdown(&self)
   }
   ```

3. Run tests:
   ```bash
   cargo test --test integration -- --test-threads=1
   ```
```

### Expected Output
- Comprehensive integration tests
- Mock cloud server for testing
- All tests passing
- CI-ready test suite

---

## Session 2.12: CLI Cloud Commands Completion

### Prompt

```
Complete all CLI commands for Phase 2 cloud functionality.

1. Update synesis-cli/src/commands/cloud.rs:
   ```rust
   pub async fn handle_cloud_command(cmd: CloudCommand) -> Result<()> {
       match cmd {
           CloudCommand::Login { api_key } => handle_login(api_key).await,
           CloudCommand::Logout => handle_logout().await,
           CloudCommand::Status => handle_cloud_status().await,
           CloudCommand::Balance => handle_balance().await,
           CloudCommand::Usage { days } => handle_usage(days).await,
           CloudCommand::Topup { amount } => handle_topup(amount).await,
           CloudCommand::Tier { tier } => handle_set_tier(tier).await,
       }
   }
   
   async fn handle_login(api_key: Option<String>) -> Result<()> {
       let api_key = if let Some(key) = api_key {
           key
       } else {
           // Interactive login via device code flow
           println!("Opening browser for authentication...");
           let code = request_device_code().await?;
           println!("Enter code: {}", code.user_code);
           open::that(&code.verification_url)?;
           poll_for_token(&code).await?
       };
       
       // Validate API key
       let user_info = validate_api_key(&api_key).await?;
       
       // Save to config
       save_api_key(&api_key)?;
       
       println!("✓ Logged in as {}", user_info.email);
       Ok(())
   }
   
   async fn handle_balance() -> Result<()> {
       let client = get_billing_client()?;
       let balance = client.get_balance().await?;
       
       println!("Current Balance");
       println!("  Unbilled:  ${:.2}", balance.unbilled_cents as f64 / 100.0);
       println!("  Credits:   ${:.2}", balance.credits_cents as f64 / 100.0);
       println!("  Ceiling:   ${:.2}", balance.ceiling_cents as f64 / 100.0);
       println!("  Tier:      {}", balance.tier);
       
       Ok(())
   }
   
   async fn handle_usage(days: u32) -> Result<()> {
       let client = get_billing_client()?;
       let usage = client.get_usage_history(days).await?;
       
       let mut table = Table::new();
       table.set_header(vec!["Date", "Requests", "Tokens", "Cost"]);
       
       for day in usage.daily {
           table.add_row(vec![
               day.date,
               day.requests.to_string(),
               format!("{}/{}", day.tokens_in, day.tokens_out),
               format!("${:.2}", day.cost_cents as f64 / 100.0),
           ]);
       }
       
       println!("{}", table);
       println!("\nTotal: ${:.2}", usage.total_cents as f64 / 100.0);
       
       Ok(())
   }
   
   async fn handle_topup(amount: f64) -> Result<()> {
       let cents = (amount * 100.0) as u32;
       
       println!("Creating checkout session for ${:.2}...", amount);
       
       let client = get_billing_client()?;
       let checkout_url = client.create_checkout(cents).await?;
       
       println!("Opening payment page...");
       open::that(&checkout_url)?;
       
       // Poll for completion
       let pb = ProgressBar::new_spinner();
       pb.set_message("Waiting for payment...");
       
       loop {
           tokio::time::sleep(Duration::from_secs(2)).await;
           
           let status = client.check_payment_status().await?;
           if status.completed {
               pb.finish_with_message("Payment complete!");
               println!("✓ Added ${:.2} to your account", amount);
               break;
           }
       }
       
       Ok(())
   }
   ```

2. Add push command for LoRA:
   ```rust
   // commands/push.rs
   pub async fn handle_push(
       name: Option<String>,
       path: Option<PathBuf>,
       force: bool,
   ) -> Result<()> {
       let manager = get_lora_manager()?;
       
       let lora = if let Some(p) = path {
           manager.load_from_path(&p)?
       } else if let Some(n) = name {
           manager.get_by_name(&n)?
       } else {
           select_lora_interactive(&manager)?
       };
       
       if lora.uploaded && !force {
           println!("LoRA '{}' already uploaded (cloud_id: {})", 
                    lora.name, lora.cloud_id.unwrap());
           println!("Use --force to re-upload");
           return Ok(());
       }
       
       println!("Preparing LoRA for upload...");
       println!("  Name: {}", lora.name);
       println!("  Base model: {}", lora.base_model);
       println!("  Size: {}", format_bytes(lora.size_bytes));
       
       let pb = ProgressBar::new(lora.size_bytes);
       pb.set_style(ProgressStyle::default_bar()
           .template("{wide_bar} {bytes}/{total_bytes} ({eta})")?);
       
       let cloud_id = manager.upload_with_progress(&lora.id, |bytes| {
           pb.set_position(bytes);
       }).await?;
       
       pb.finish_and_clear();
       
       println!("✓ Upload complete!");
       println!("  Cloud ID: {}", cloud_id);
       println!("\nYour LoRA is now available for cloud inference.");
       
       Ok(())
   }
   ```

3. Add invite/handover commands:
   ```rust
   // commands/invite.rs
   pub async fn handle_invite(cmd: InviteCommand) -> Result<()> {
       match cmd {
           InviteCommand::Create { project, role, quota, expires } => {
               let client = get_collaborator_client()?;
               let invite = client.create_invite(InviteRequest {
                   project_id: project,
                   role: role.parse()?,
                   quota_cents: (quota * 100.0) as u32,
                   expires_hours: expires,
               }).await?;
               
               println!("✓ Invite created!");
               println!("  URL: {}", invite.url);
               println!("  Expires: {}", invite.expires_at);
               println!("\nShare this link with your collaborator.");
           }
           InviteCommand::List { project } => { /* ... */ }
           InviteCommand::Revoke { token } => { /* ... */ }
       }
       Ok(())
   }
   ```

4. Update main.rs with new commands:
   ```rust
   #[derive(Subcommand)]
   enum Commands {
       // Existing...
       
       /// Cloud connection and billing
       Cloud {
           #[command(subcommand)]
           command: CloudCommand,
       },
       
       /// Push LoRA to cloud
       Push {
           #[arg(short, long)]
           name: Option<String>,
           #[arg(short, long)]
           path: Option<PathBuf>,
           #[arg(short, long)]
           force: bool,
       },
       
       /// Manage collaborator invites
       Invite {
           #[command(subcommand)]
           command: InviteCommand,
       },
       
       /// Initiate project handover
       Handover {
           #[command(subcommand)]
           command: HandoverCommand,
       },
   }
   ```

5. Test all commands:
   ```bash
   synesis cloud login
   synesis cloud status
   synesis cloud balance
   synesis cloud usage --days 7
   synesis push --name my-lora
   synesis invite create --project myproject --role viewer
   synesis handover start --project myproject --to client@example.com
   ```
```

### Expected Output
- All cloud CLI commands implemented
- Interactive flows working
- Progress indicators for uploads
- Integration with cloud client

---

## Phase 2 Completion Checklist

After completing all sessions:

### Functional Requirements
- [ ] QUIC tunnel connects and maintains connection
- [ ] Auto-reconnect works on disconnect
- [ ] Heartbeat sends vitals every 30 seconds
- [ ] Pre-warm signaling on high GPU load
- [ ] Escalation requests complete successfully
- [ ] Streaming responses display in real-time
- [ ] Billing records all usage
- [ ] Credits applied before charges
- [ ] LoRA upload works with progress
- [ ] Collaborator invites work end-to-end
- [ ] Handover flow completes

### Technical Requirements
- [ ] All tests passing (200+ tests)
- [ ] Zero compiler warnings
- [ ] <50ms tunnel latency
- [ ] <200ms LoRA hot-swap
- [ ] Cloud workers deployed

### Business Requirements
- [ ] First paying customer
- [ ] Stripe integration working
- [ ] Usage tracking accurate

---

*Document Version: 1.0*
*Created: 2026-01-02*
*For: Claude Code Agent*
