# SuperInstance AI - Technical Architecture

> **Audience**: Technical Leads, Senior Engineers, Architects
> **Purpose**: Understand system design, integration points, and technical decisions

---

## System Overview

SuperInstance is a **federated agentic system** with two primary deployment contexts:

1. **Local Hub**: Runs on user hardware (Jetson, AI PC, WSL)
2. **Cloud Mesh**: Runs on Cloudflare's global network

Both contexts implement the same **Tripartite Council** pattern, enabling seamless task migration between local and cloud execution.

## Core Components

### 1. The Tripartite Council

```
                    ┌─────────────────────┐
                    │    USER PROMPT      │
                    └──────────┬──────────┘
                               │
                               ▼
┌──────────────────────────────────────────────────────────────┐
│                        SYNAPSE                                │
│                   (Orchestrator Layer)                        │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │                  PROMPT DISPATCH                         │ │
│  └──────────────────────┬──────────────────────────────────┘ │
│                         │                                     │
│         ┌───────────────┼───────────────┐                    │
│         ▼               ▼               ▼                    │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐            │
│  │   PATHOS    │ │    LOGOS    │ │    ETHOS    │            │
│  │             │ │             │ │             │            │
│  │ • Intent    │ │ • RAG Query │ │ • Fact Check│            │
│  │ • Persona   │ │ • LoRA Load │ │ • Hardware  │            │
│  │ • A2A Xlate │ │ • Synthesis │ │ • Thermal   │            │
│  └──────┬──────┘ └──────┬──────┘ └──────┬──────┘            │
│         │               │               │                    │
│         └───────────────┼───────────────┘                    │
│                         ▼                                    │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              CONSENSUS ENGINE                            │ │
│  │  • Threshold: 0.85 agreement required                   │ │
│  │  • Max Rounds: 3 before Arbiter                         │ │
│  │  • Output: Unified response + confidence score          │ │
│  └─────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────┘
```

### 2. Agent Responsibilities

#### Pathos (Intent Layer)
- **Input**: Raw user prompt (text, images, context)
- **Output**: Structured A2A Manifest
- **Key Functions**:
  - Contextual decoding (what does the user actually want?)
  - Persona alignment (match user's communication style)
  - Ambiguity resolution (clarify before proceeding)
  - Constraint identification for Logos

#### Logos (Logic Layer)
- **Input**: A2A Manifest from Pathos
- **Output**: Technical solution/response
- **Key Functions**:
  - RAG retrieval from SQLite-VSS/Vectorize
  - LoRA adapter selection and loading
  - Solution synthesis and code generation
  - Chain-of-thought reasoning

#### Ethos (Truth Layer)
- **Input**: Proposed solution from Logos
- **Output**: Verification result + constraints
- **Key Functions**:
  - Fact-checking against Ground Truth index
  - Hardware constraint validation
  - Thermal/VRAM limit checking
  - External source verification (web, PDFs)

### 3. Data Flow Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      LOCAL HUB                                   │
│                                                                  │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐        │
│  │  SQLite-VSS │     │  LoRA Store │     │  Hardware   │        │
│  │  (Vectors)  │     │ (Adapters)  │     │  Manifest   │        │
│  └──────┬──────┘     └──────┬──────┘     └──────┬──────┘        │
│         │                   │                   │                │
│         └───────────────────┼───────────────────┘                │
│                             │                                    │
│                             ▼                                    │
│                    ┌─────────────────┐                          │
│                    │ LOCAL SYNAPSE   │                          │
│                    │ (Rust Binary)   │                          │
│                    └────────┬────────┘                          │
│                             │                                    │
│              ┌──────────────┼──────────────┐                    │
│              ▼              │              ▼                    │
│       ┌───────────┐        │       ┌───────────────┐           │
│       │ RESPOND   │        │       │ ESCALATE      │           │
│       │ LOCALLY   │        │       │ TO CLOUD      │           │
│       └───────────┘        │       └───────┬───────┘           │
│                            │               │                    │
└────────────────────────────┼───────────────┼────────────────────┘
                             │               │
                             │               ▼
                             │    ┌─────────────────────┐
                             │    │   PRIVACY PROXY     │
                             │    │   (Redact & UUID)   │
                             │    └──────────┬──────────┘
                             │               │
                             │               ▼
                             │    ┌─────────────────────┐
                             │    │   QUIC TUNNEL       │
                             │    │   (mTLS Secured)    │
                             │    └──────────┬──────────┘
                             │               │
─────────────────────────────┼───────────────┼─────────────────────
                             │               │
┌────────────────────────────┼───────────────┼────────────────────┐
│                    CLOUDFLARE LAYER        │                    │
│                             │               │                    │
│                             │               ▼                    │
│                             │    ┌─────────────────────┐        │
│                             │    │   AI GATEWAY        │        │
│                             │    │   (Metering)        │        │
│                             │    └──────────┬──────────┘        │
│                             │               │                    │
│                             │               ▼                    │
│                             │    ┌─────────────────────┐        │
│                             │    │   DURABLE OBJECT    │        │
│                             │    │   (Cloud Synapse)   │        │
│                             │    └──────────┬──────────┘        │
│                             │               │                    │
│              ┌──────────────┼───────────────┼──────────────┐    │
│              ▼              ▼               ▼              ▼    │
│       ┌───────────┐  ┌───────────┐  ┌───────────┐  ┌─────────┐ │
│       │ Workers   │  │ Vectorize │  │    R2     │  │ Billing │ │
│       │ AI        │  │ (Global)  │  │ (LoRAs)   │  │ Ledger  │ │
│       └───────────┘  └───────────┘  └───────────┘  └─────────┘ │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

## Key Technical Decisions

### Language Selection

| Component | Language | Rationale |
|-----------|----------|-----------|
| CLI Orchestrator | Rust | Zero-cost abstractions, memory safety, async |
| Privacy Proxy | Rust | Performance-critical string manipulation |
| Model Runtime | C++ (llama.cpp) | Direct hardware access, CUDA integration |
| Training Scripts | Python | ML ecosystem, Unsloth compatibility |
| Cloud Workers | TypeScript | Cloudflare native, rapid iteration |
| Dashboard | React/TS | Standard frontend, SSR support |

### Database Architecture

```
LOCAL STORAGE
├── synesis.db (SQLite)
│   ├── interaction_logs      # All prompt/response pairs
│   ├── consensus_scores      # Agreement metrics
│   ├── user_feedback         # Thumbs up/down
│   └── hardware_telemetry    # Thermal, VRAM snapshots
│
├── vectors.db (SQLite-VSS)
│   ├── project_embeddings    # Indexed project files
│   ├── conversation_memory   # Recent context vectors
│   └── wisdom_blocks         # Distilled learnings
│
└── lora_store/
    ├── active/               # Currently loaded adapters
    └── archive/              # Previous versions

CLOUD STORAGE
├── Vectorize Index
│   ├── global_knowledge      # Shared Ground Truth
│   └── project:{id}          # Per-project namespaces
│
├── R2 Buckets
│   ├── lora-adapters/        # .safetensors files
│   └── project-assets/       # PDFs, docs, media
│
└── D1 Database
    ├── users                 # Account data
    ├── projects              # Metadata
    ├── marketplace_listings  # LoRA marketplace
    └── transactions          # Billing history
```

### Consensus Protocol

```typescript
interface ConsensusState {
  round: number;                    // Current deliberation round
  threshold: number;                // Agreement threshold (default: 0.85)
  maxRounds: number;                // Before Arbiter (default: 3)
  
  pathos: {
    intent: A2AManifest;
    confidence: number;
  };
  
  logos: {
    solution: string;
    confidence: number;
    sources: string[];
  };
  
  ethos: {
    verdict: 'APPROVED' | 'VETO' | 'NEEDS_REVISION';
    constraints: string[];
    confidence: number;
  };
  
  aggregateScore: number;           // Weighted average
  status: 'DELIBERATING' | 'CONSENSUS' | 'ARBITER_NEEDED';
}
```

### Privacy Proxy Protocol

```
REDACTION FLOW
═══════════════════════════════════════════════════════════

Original:  "Fix the auth bug in John's code at /home/john/secret.py"
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│  PATHOS SCAN                                             │
│  • PII: "John" → [USER_01]                              │
│  • PATH: "/home/john/secret.py" → [PATH_01]             │
└─────────────────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│  LOGOS SCAN                                              │
│  • CODE: (attached code block) → [CODE_BLOCK_A]         │
│  • API_KEY: "sk-abc123..." → [SECRET_01]                │
└─────────────────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│  TOKEN VAULT (Local SQLite)                              │
│  ┌─────────────┬────────────────────────┐               │
│  │ Token       │ Original Value          │               │
│  ├─────────────┼────────────────────────┤               │
│  │ [USER_01]   │ John                    │               │
│  │ [PATH_01]   │ /home/john/secret.py    │               │
│  │ [CODE_A]    │ (full code block)       │               │
│  │ [SECRET_01] │ sk-abc123...            │               │
│  └─────────────┴────────────────────────┘               │
└─────────────────────────────────────────────────────────┘
                     │
                     ▼
Transmitted: "Fix the auth bug in [USER_01]'s code at [PATH_01]"

RE-INFLATION FLOW (on response return)
═══════════════════════════════════════════════════════════

Cloud Response: "The issue in [PATH_01] is in line 42..."
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│  TOKEN REPLACEMENT                                       │
│  [PATH_01] → /home/john/secret.py                       │
└─────────────────────────────────────────────────────────┘
                     │
                     ▼
User Sees: "The issue in /home/john/secret.py is in line 42..."
```

## Integration Points

### Local → Cloud Escalation

Escalation triggers when:
1. **Complexity**: Token count exceeds local model's effective window
2. **Hardware**: GPU load > 80% or temperature > 80°C
3. **Capability**: Requested model/LoRA not available locally
4. **Explicit**: User requests cloud power

```rust
fn should_escalate(request: &Request, hardware: &HardwareState) -> bool {
    let complexity_trigger = request.estimated_tokens > LOCAL_TOKEN_LIMIT;
    let hardware_trigger = hardware.gpu_load > 0.8 || hardware.temp_celsius > 80;
    let capability_trigger = !local_lora_available(&request.required_lora);
    let explicit_trigger = request.force_cloud;
    
    complexity_trigger || hardware_trigger || capability_trigger || explicit_trigger
}
```

### Billing Integration

```
┌─────────────────────────────────────────────────────────┐
│                     REQUEST FLOW                         │
└─────────────────────────────────────────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────────────────┐
│  AI GATEWAY                                              │
│  • Captures: model, tokens_in, tokens_out               │
│  • Calculates: wholesale_cost                           │
│  • Header: cf-ai-billable-cost                          │
└─────────────────────────────────────────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────────────────┐
│  BILLING LEDGER (Durable Object)                         │
│                                                          │
│  tier = request.headers['X-Synesis-Tier']               │
│  multiplier = tier == 'managed' ? 1.03 : 1.30           │
│  charge = wholesale_cost * multiplier                   │
│                                                          │
│  if (knowledge_credits > 0) {                           │
│    deduction = min(knowledge_credits, charge)           │
│    knowledge_credits -= deduction                       │
│    charge -= deduction                                  │
│  }                                                       │
│                                                          │
│  unbilled_balance += charge                             │
│                                                          │
│  if (unbilled_balance > STRIPE_THRESHOLD) {             │
│    stripe.reportUsage(user_id, unbilled_balance)        │
│    unbilled_balance = 0                                 │
│  }                                                       │
└─────────────────────────────────────────────────────────┘
```

## Performance Targets

| Metric | Local | Cloud | Combined |
|--------|-------|-------|----------|
| Time to First Token | <500ms | <1s | <1s |
| Consensus Round | <2s | <3s | <3s |
| Privacy Proxy Overhead | <10ms | N/A | <10ms |
| Tunnel Latency | N/A | <50ms | <50ms |
| LoRA Hot-Swap | <100ms | <200ms | <200ms |

## Security Model

### Trust Boundaries

```
┌─────────────────────────────────────────────────────────────┐
│  TRUSTED ZONE (User's Hardware)                              │
│  • Raw data, PII, secrets                                    │
│  • Full prompt history                                       │
│  • Token vault mappings                                      │
│  • Hardware telemetry                                        │
└─────────────────────────────────────────────────────────────┘
                          │
                    mTLS + Redaction
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│  SEMI-TRUSTED ZONE (Cloudflare)                              │
│  • Redacted prompts only                                     │
│  • Anonymous usage metrics                                   │
│  • Encrypted LoRA files                                      │
│  • Session state (no raw data)                               │
└─────────────────────────────────────────────────────────────┘
```

### Authentication Flow

1. **Initial Setup**: `synesis login` → OAuth2 with Cloudflare Access
2. **Device Cert**: Cloudflare Origin CA issues device-specific certificate
3. **Request Auth**: Every request includes:
   - mTLS client certificate (device identity)
   - Bearer token (user identity)
   - Project ID (scope restriction)

---

*For implementation details, see LOW_LEVEL.md*
