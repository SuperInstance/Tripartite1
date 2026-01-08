# Phase 2 Data Models

**Version**: 2.0.0
**Last Updated**: 2026-01-02

This document defines all data models used in Phase 2 Cloud Mesh implementation.

---

## Rust Data Models (synesis-cloud)

### Connection & Tunnel

```rust
//! Connection and tunnel state models

use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Configuration for QUIC tunnel connection
#[derive(Debug, Clone)]
pub struct TunnelConfig {
    /// Cloud endpoint URL (e.g., "https://tunnel.superinstance.ai:443")
    pub cloud_url: String,
    
    /// Unique device identifier (generated on init)
    pub device_id: String,
    
    /// Path to device certificate (PEM format)
    pub cert_path: PathBuf,
    
    /// Path to device private key (PEM format)
    pub key_path: PathBuf,
    
    /// Interval between heartbeat messages
    pub heartbeat_interval: Duration,
    
    /// Delay before reconnection attempt
    pub reconnect_delay: Duration,
    
    /// Maximum reconnection attempts before giving up
    pub max_reconnect_attempts: u32,
    
    /// Connection timeout
    pub connect_timeout: Duration,
    
    /// Read timeout for responses
    pub read_timeout: Duration,
}

impl Default for TunnelConfig {
    fn default() -> Self {
        Self {
            cloud_url: "https://tunnel.superinstance.ai:443".to_string(),
            device_id: String::new(),
            cert_path: PathBuf::new(),
            key_path: PathBuf::new(),
            heartbeat_interval: Duration::from_secs(30),
            reconnect_delay: Duration::from_secs(5),
            max_reconnect_attempts: 10,
            connect_timeout: Duration::from_secs(30),
            read_timeout: Duration::from_secs(60),
        }
    }
}

/// Current state of the tunnel connection
#[derive(Debug, Clone, PartialEq)]
pub enum TunnelState {
    /// Not connected to cloud
    Disconnected,
    
    /// Attempting to establish connection
    Connecting,
    
    /// Connected and healthy
    Connected {
        since: Instant,
        latency_ms: u32,
    },
    
    /// Connection lost, attempting to reconnect
    Reconnecting {
        attempt: u32,
        last_error: String,
    },
    
    /// Connection failed permanently
    Failed {
        error: String,
        at: Instant,
    },
}

/// Statistics for tunnel connection
#[derive(Debug, Clone, Default)]
pub struct TunnelStats {
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
    pub heartbeats_sent: u64,
    pub heartbeats_acked: u64,
    pub requests_sent: u64,
    pub requests_succeeded: u64,
    pub requests_failed: u64,
    pub reconnections: u32,
    pub avg_latency_ms: u32,
}
```

### Heartbeat & Telemetry

```rust
//! Heartbeat and device telemetry models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Device vitals sent with each heartbeat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceVitals {
    /// Device identifier
    pub device_id: String,
    
    /// Timestamp of collection
    pub timestamp: DateTime<Utc>,
    
    /// CPU usage percentage (0.0 - 1.0)
    pub cpu_usage: f32,
    
    /// Memory usage percentage (0.0 - 1.0)
    pub memory_usage: f32,
    
    /// GPU usage percentage (0.0 - 1.0), if GPU present
    pub gpu_usage: Option<f32>,
    
    /// GPU temperature in Celsius, if GPU present
    pub gpu_temp_celsius: Option<f32>,
    
    /// GPU VRAM usage percentage (0.0 - 1.0), if GPU present
    pub gpu_vram_usage: Option<f32>,
    
    /// Disk usage percentage (0.0 - 1.0)
    pub disk_usage: f32,
    
    /// Number of active sessions
    pub active_sessions: u32,
    
    /// Number of queries in processing queue
    pub pending_queries: u32,
    
    /// Current model loaded (if any)
    pub loaded_model: Option<String>,
}

/// Pre-warm signal sent when GPU is stressed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrewarmSignal {
    /// Project that may need cloud assistance
    pub project_id: String,
    
    /// LoRA to pre-load in cloud (if applicable)
    pub lora_id: Option<String>,
    
    /// Estimated time until escalation (seconds)
    pub estimated_escalation_secs: Option<u32>,
    
    /// Current GPU load triggering pre-warm
    pub current_gpu_load: f32,
    
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Heartbeat acknowledgment from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatAck {
    /// Server timestamp
    pub server_time: DateTime<Utc>,
    
    /// Round-trip latency in milliseconds
    pub latency_ms: u32,
    
    /// Any pending messages for client
    pub pending_messages: u32,
    
    /// Server status
    pub server_status: ServerStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerStatus {
    Healthy,
    Degraded { reason: String },
    Maintenance { until: DateTime<Utc> },
}
```

### Escalation

```rust
//! Cloud escalation request/response models

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Cloud model selection
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum CloudModel {
    /// Claude 3.5 Sonnet (default)
    #[default]
    ClaudeSonnet,
    
    /// Claude 3 Opus (highest quality)
    ClaudeOpus,
    
    /// GPT-4 Turbo
    Gpt4Turbo,
    
    /// Let cloud decide based on query
    Auto,
}

/// Request to escalate query to cloud
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationRequest {
    /// Unique request identifier
    pub request_id: String,
    
    /// Session identifier for conversation continuity
    pub session_id: String,
    
    /// The query (already redacted by privacy proxy)
    pub query: String,
    
    /// Context from local processing
    pub context: EscalationContext,
    
    /// Preferred cloud model
    pub model: CloudModel,
    
    /// Maximum tokens to generate
    pub max_tokens: u32,
    
    /// Whether to stream response
    pub stream: bool,
    
    /// LoRA to use (if uploaded)
    pub lora_id: Option<String>,
    
    /// Timeout in seconds
    pub timeout_secs: Option<u32>,
}

/// Context passed with escalation request
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EscalationContext {
    /// Intent framing from Pathos agent
    pub pathos_framing: Option<String>,
    
    /// Relevant chunks from local knowledge vault
    pub local_knowledge: Vec<KnowledgeChunk>,
    
    /// Conversation history
    pub conversation_history: Vec<Message>,
    
    /// Constraints from Ethos agent
    pub constraints: Vec<String>,
    
    /// User preferences
    pub user_preferences: Option<UserPreferences>,
}

/// Knowledge chunk from local RAG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeChunk {
    /// Source document path
    pub source: String,
    
    /// Chunk content
    pub content: String,
    
    /// Relevance score (0.0 - 1.0)
    pub relevance: f32,
}

/// Conversation message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Role: "user" or "assistant"
    pub role: String,
    
    /// Message content
    pub content: String,
    
    /// Timestamp
    pub timestamp: Option<DateTime<Utc>>,
}

/// User preferences for response generation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserPreferences {
    pub preferred_language: Option<String>,
    pub verbosity: Option<Verbosity>,
    pub tone: Option<Tone>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verbosity {
    Concise,
    Normal,
    Detailed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tone {
    Professional,
    Casual,
    Technical,
}

/// Response from cloud escalation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationResponse {
    /// Request identifier (matches request)
    pub request_id: String,
    
    /// Generated content
    pub content: String,
    
    /// Model actually used
    pub model_used: String,
    
    /// Token usage
    pub tokens_used: TokenUsage,
    
    /// Cost in cents (after markup)
    pub cost_cents: u32,
    
    /// Total latency in milliseconds
    pub latency_ms: u64,
    
    /// Sources cited (if any)
    pub sources: Vec<String>,
    
    /// Whether LoRA was used
    pub lora_applied: bool,
}

/// Token usage breakdown
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    /// Input/prompt tokens
    pub prompt: u32,
    
    /// Output/completion tokens
    pub completion: u32,
}

impl TokenUsage {
    pub fn total(&self) -> u32 {
        self.prompt + self.completion
    }
}
```

### Streaming

```rust
//! Streaming response models

use serde::{Deserialize, Serialize};

/// Chunk received during streaming response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamChunk {
    /// Content chunk
    Content {
        text: String,
    },
    
    /// Metadata about current processing phase
    Metadata {
        agent: String,
        phase: String,
    },
    
    /// Stream ended successfully
    End {
        tokens_used: TokenUsage,
        cost_cents: u32,
        total_latency_ms: u64,
    },
    
    /// Error during streaming
    Error {
        code: String,
        message: String,
    },
}

/// Processing phase during streaming
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessingPhase {
    /// Analyzing intent
    Analyzing,
    
    /// Retrieving context
    Retrieving,
    
    /// Generating response
    Generating,
    
    /// Verifying response
    Verifying,
}
```

### Billing

```rust
//! Billing and usage models

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Billing tier
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BillingTier {
    /// Free tier with monthly limit
    Free {
        monthly_limit_cents: u32,
    },
    
    /// Managed tier: 3% markup on costs
    Managed {
        markup_percent: f32,
    },
    
    /// BYOK tier: 30% licensing fee
    Byok {
        licensing_percent: f32,
        anthropic_key: Option<String>,
        openai_key: Option<String>,
    },
}

impl Default for BillingTier {
    fn default() -> Self {
        Self::Managed { markup_percent: 3.0 }
    }
}

/// Usage event for billing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEvent {
    /// Unique event identifier
    pub id: String,
    
    /// Request ID this usage is for
    pub request_id: String,
    
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Input tokens
    pub tokens_in: u32,
    
    /// Output tokens
    pub tokens_out: u32,
    
    /// Model used
    pub model: String,
    
    /// Base cost (before markup)
    pub cost_basis_cents: u32,
    
    /// Final cost (after markup, before credits)
    pub final_cost_cents: u32,
    
    /// Credits applied
    pub credits_applied_cents: u32,
    
    /// Net charge
    pub net_charge_cents: u32,
}

/// Account balance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    /// Unbilled charges
    pub unbilled_cents: u32,
    
    /// Available credits
    pub credits_cents: u32,
    
    /// Credit ceiling (spending limit)
    pub ceiling_cents: u32,
    
    /// Current tier
    pub tier: BillingTier,
    
    /// Next invoice date
    pub next_invoice: Option<DateTime<Utc>>,
}

/// Usage summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSummary {
    /// Period start
    pub period_start: DateTime<Utc>,
    
    /// Period end
    pub period_end: DateTime<Utc>,
    
    /// Total cost in cents
    pub total_cents: u32,
    
    /// Total requests
    pub total_requests: u32,
    
    /// Total input tokens
    pub total_tokens_in: u64,
    
    /// Total output tokens
    pub total_tokens_out: u64,
    
    /// Daily breakdown
    pub daily: Vec<DailyUsage>,
    
    /// By model breakdown
    pub by_model: Vec<ModelUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyUsage {
    pub date: String,  // YYYY-MM-DD
    pub requests: u32,
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub cost_cents: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsage {
    pub model: String,
    pub requests: u32,
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub cost_cents: u32,
}
```

### LoRA

```rust
//! LoRA management models

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::path::PathBuf;

/// Local LoRA information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalLora {
    /// Unique local identifier
    pub id: String,
    
    /// Human-readable name
    pub name: String,
    
    /// Base model this LoRA was trained for
    pub base_model: String,
    
    /// Path to LoRA file
    pub path: PathBuf,
    
    /// File size in bytes
    pub size_bytes: u64,
    
    /// SHA256 checksum
    pub checksum: String,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Whether uploaded to cloud
    pub uploaded: bool,
    
    /// Cloud ID (if uploaded)
    pub cloud_id: Option<String>,
    
    /// Last upload timestamp
    pub uploaded_at: Option<DateTime<Utc>>,
}

/// Cloud LoRA information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudLora {
    /// Cloud identifier
    pub id: String,
    
    /// Human-readable name
    pub name: String,
    
    /// Base model
    pub base_model: String,
    
    /// File size in bytes
    pub size_bytes: u64,
    
    /// Upload timestamp
    pub uploaded_at: DateTime<Utc>,
    
    /// Last used timestamp
    pub last_used: Option<DateTime<Utc>>,
    
    /// Total usage count
    pub usage_count: u64,
    
    /// Regions where available
    pub regions: Vec<String>,
    
    /// Status
    pub status: LoraStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoraStatus {
    Uploading,
    Processing,
    Ready,
    Error { message: String },
}

/// LoRA upload progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadProgress {
    pub upload_id: String,
    pub total_bytes: u64,
    pub uploaded_bytes: u64,
    pub chunks_total: u32,
    pub chunks_uploaded: u32,
    pub status: UploadStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UploadStatus {
    InProgress,
    Completed,
    Failed { error: String },
}
```

### Collaborator

```rust
//! Collaborator and handover models

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Collaborator role
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CollaboratorRole {
    /// Can view agent interactions
    Viewer,
    
    /// Can add comments/feedback
    Commenter,
    
    /// Can modify prompts and settings
    Editor,
}

/// Invite request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteRequest {
    /// Project to invite to
    pub project_id: String,
    
    /// Role for the collaborator
    pub role: CollaboratorRole,
    
    /// Usage quota in cents
    pub quota_cents: u32,
    
    /// Hours until invite expires
    pub expires_hours: u32,
}

/// Created invite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invite {
    /// Invite token
    pub token: String,
    
    /// Full invite URL
    pub url: String,
    
    /// Role
    pub role: CollaboratorRole,
    
    /// Quota
    pub quota_cents: u32,
    
    /// Quota used
    pub quota_used_cents: u32,
    
    /// Creation time
    pub created_at: DateTime<Utc>,
    
    /// Expiration time
    pub expires_at: DateTime<Utc>,
    
    /// Whether accepted
    pub accepted: bool,
    
    /// Guest email (if accepted)
    pub guest_email: Option<String>,
}

/// Active collaborator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collaborator {
    /// User ID
    pub user_id: String,
    
    /// Email
    pub email: String,
    
    /// Role
    pub role: CollaboratorRole,
    
    /// Remaining quota
    pub quota_remaining_cents: u32,
    
    /// Join date
    pub joined_at: DateTime<Utc>,
    
    /// Last active
    pub last_active: Option<DateTime<Utc>>,
}

/// Handover request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoverRequest {
    /// Project to hand over
    pub project_id: String,
    
    /// New owner's email
    pub to_email: String,
    
    /// Include LoRAs in handover
    pub include_loras: bool,
    
    /// Include knowledge vault
    pub include_knowledge: bool,
    
    /// Optional message to new owner
    pub message: Option<String>,
}

/// Handover status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Handover {
    /// Handover token
    pub token: String,
    
    /// Current state
    pub state: HandoverState,
    
    /// Recipient email
    pub to_email: String,
    
    /// Creation time
    pub created_at: DateTime<Utc>,
    
    /// Expiration time
    pub expires_at: DateTime<Utc>,
    
    /// Completion time (if completed)
    pub completed_at: Option<DateTime<Utc>>,
    
    /// Incentive pricing
    pub incentive: HandoverIncentive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandoverState {
    Pending,
    EmailSent,
    Accepted,
    Completed,
    Expired,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoverIncentive {
    /// First year price
    pub first_year_price: f64,
    
    /// Regular price after first year
    pub regular_price: f64,
}
```

---

## TypeScript Data Models (Cloud Workers)

### Common Types

```typescript
// cloud/types/common.ts

export interface User {
  id: string;
  email: string;
  name: string;
  tier: BillingTier;
  createdAt: Date;
  devices: Device[];
}

export interface Device {
  id: string;
  name: string;
  lastSeen: Date;
  certificate: string;
}

export type BillingTier = 
  | { type: 'free'; monthlyLimitCents: number }
  | { type: 'managed'; markupPercent: number }
  | { type: 'byok'; licensingPercent: number; anthropicKey?: string; openaiKey?: string };
```

### Billing Types

```typescript
// cloud/types/billing.ts

export interface LedgerState {
  userId: string;
  tier: BillingTier;
  unbilledCents: number;
  knowledgeCredits: number;
  creditCeiling: number;
  stripeCustomerId: string | null;
  usageHistory: UsageEvent[];
  lastSyncedAt: number;
}

export interface UsageEvent {
  id: string;
  requestId: string;
  timestamp: number;
  tokensIn: number;
  tokensOut: number;
  model: string;
  costBasisCents: number;
  finalChargeCents: number;
  creditsApplied: number;
}

export interface Balance {
  unbilledCents: number;
  creditsCents: number;
  ceilingCents: number;
  tier: BillingTier;
  nextInvoiceDate: string | null;
}
```

### Session Types

```typescript
// cloud/types/session.ts

export interface SessionState {
  id: string;
  userId: string;
  projectId: string;
  conversationHistory: Message[];
  context: SessionContext;
  createdAt: number;
  updatedAt: number;
  expiresAt: number;
}

export interface Message {
  role: 'user' | 'assistant';
  content: string;
  timestamp: number;
  metadata?: MessageMetadata;
}

export interface MessageMetadata {
  model: string;
  tokensUsed: { prompt: number; completion: number };
  latencyMs: number;
  loraApplied?: string;
}

export interface SessionContext {
  pathosFraming?: string;
  constraints: string[];
  userPreferences?: UserPreferences;
}

export interface UserPreferences {
  preferredLanguage?: string;
  verbosity?: 'concise' | 'normal' | 'detailed';
  tone?: 'professional' | 'casual' | 'technical';
}
```

### Escalation Types

```typescript
// cloud/types/escalation.ts

export interface EscalationRequest {
  requestId: string;
  sessionId: string;
  query: string;
  context: EscalationContext;
  model: CloudModel;
  maxTokens: number;
  stream: boolean;
  loraId?: string;
  timeoutSecs?: number;
}

export interface EscalationContext {
  pathosFraming?: string;
  localKnowledge: KnowledgeChunk[];
  conversationHistory: Message[];
  constraints: string[];
  userPreferences?: UserPreferences;
}

export interface KnowledgeChunk {
  source: string;
  content: string;
  relevance: number;
}

export type CloudModel = 'claude_sonnet' | 'claude_opus' | 'gpt4_turbo' | 'auto';

export interface EscalationResponse {
  requestId: string;
  content: string;
  modelUsed: string;
  tokensUsed: { prompt: number; completion: number };
  costCents: number;
  latencyMs: number;
  sources: string[];
  loraApplied: boolean;
}

export type StreamChunk =
  | { type: 'content'; text: string }
  | { type: 'metadata'; agent: string; phase: string }
  | { type: 'end'; tokensUsed: { prompt: number; completion: number }; costCents: number }
  | { type: 'error'; code: string; message: string };
```

### LoRA Types

```typescript
// cloud/types/lora.ts

export interface CloudLora {
  id: string;
  userId: string;
  name: string;
  baseModel: string;
  sizeBytes: number;
  checksum: string;
  r2Key: string;
  uploadedAt: number;
  lastUsed?: number;
  usageCount: number;
  regions: string[];
  status: LoraStatus;
}

export type LoraStatus = 
  | { type: 'uploading'; progress: number }
  | { type: 'processing' }
  | { type: 'ready' }
  | { type: 'error'; message: string };

export interface LoraUpload {
  id: string;
  userId: string;
  name: string;
  baseModel: string;
  sizeBytes: number;
  checksum: string;
  chunksTotal: number;
  chunksReceived: number;
  createdAt: number;
  expiresAt: number;
}
```

### Collaborator Types

```typescript
// cloud/types/collaborator.ts

export type CollaboratorRole = 'viewer' | 'commenter' | 'editor';

export interface InviteRecord {
  token: string;
  projectId: string;
  hostUserId: string;
  role: CollaboratorRole;
  guestQuota: number;
  guestQuotaUsed: number;
  createdAt: number;
  expiresAt: number;
  accepted: boolean;
  guestEmail?: string;
  guestUserId?: string;
}

export interface GuestSession {
  id: string;
  projectId: string;
  guestUserId: string;
  guestEmail: string;
  role: CollaboratorRole;
  remainingQuota: number;
  hostUserId: string;
  createdAt: number;
  lastActiveAt: number;
}

export interface HandoverRecord {
  token: string;
  projectId: string;
  fromUserId: string;
  toEmail: string;
  includeLoras: boolean;
  includeKnowledge: boolean;
  message?: string;
  projectSnapshot: ProjectSnapshot;
  incentive: HandoverIncentive;
  state: HandoverState;
  createdAt: number;
  expiresAt: number;
  completedAt?: number;
}

export type HandoverState = 'pending' | 'email_sent' | 'accepted' | 'completed' | 'expired' | 'cancelled';

export interface HandoverIncentive {
  firstYearPrice: number;
  regularPrice: number;
}

export interface ProjectSnapshot {
  projectId: string;
  name: string;
  loraIds: string[];
  knowledgeDocCount: number;
  sessionCount: number;
  snapshotAt: number;
}
```

---

## Database Schema (D1)

See `/cloud/schema/001_initial.sql` for the complete D1 schema.

Key tables for Phase 2:
- `users` - User accounts
- `api_keys` - API key management
- `user_settings` - User preferences
- `billing_ledger` - Usage and charges
- `usage_records` - Detailed usage logs
- `user_loras` - LoRA metadata
- `chat_sessions` - Conversation history
- `marketplace_listings` - LoRA marketplace (Phase 3)

---

*Document Version: 2.0.0*
*Last Updated: 2026-01-02*
