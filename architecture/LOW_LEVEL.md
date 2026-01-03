# SuperInstance AI - Implementation Details

> **Audience**: Implementing Engineers, Worker Agents
> **Purpose**: Concrete code patterns, API contracts, and implementation specifics

---

## Directory Structure

```
superinstance/
├── cli/                          # Rust CLI application
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands/
│   │   │   ├── init.rs
│   │   │   ├── ask.rs
│   │   │   ├── push.rs
│   │   │   └── status.rs
│   │   ├── agents/
│   │   │   ├── mod.rs
│   │   │   ├── pathos.rs
│   │   │   ├── logos.rs
│   │   │   └── ethos.rs
│   │   ├── synapse/
│   │   │   ├── mod.rs
│   │   │   ├── orchestrator.rs
│   │   │   └── consensus.rs
│   │   ├── privacy/
│   │   │   ├── mod.rs
│   │   │   ├── redactor.rs
│   │   │   └── vault.rs
│   │   ├── storage/
│   │   │   ├── mod.rs
│   │   │   ├── sqlite.rs
│   │   │   └── vectors.rs
│   │   └── tunnel/
│   │       ├── mod.rs
│   │       └── quic.rs
│   └── Cargo.toml
│
├── cloud/                        # Cloudflare Workers
│   ├── src/
│   │   ├── index.ts
│   │   ├── orchestrator.ts       # Durable Object
│   │   ├── ledger.ts             # Billing DO
│   │   ├── gateway.ts            # AI Gateway wrapper
│   │   └── types.ts
│   ├── wrangler.toml
│   └── package.json
│
├── sdk/                          # Client SDKs
│   ├── typescript/
│   │   ├── src/
│   │   │   ├── client.ts
│   │   │   ├── hooks/
│   │   │   │   └── useSynesis.ts
│   │   │   └── types.ts
│   │   └── package.json
│   └── python/
│       ├── synesis/
│       │   ├── __init__.py
│       │   └── client.py
│       └── pyproject.toml
│
├── training/                     # ML training scripts
│   ├── distill.py
│   ├── train_lora.py
│   └── requirements.txt
│
└── manifests/                    # Hardware manifests
    ├── nvidia-jetson-orin-nano.json
    ├── nvidia-jetson-orin-nx.json
    └── generic-x86-nvidia.json
```

## Core Data Structures

### A2A Manifest (Inter-Agent Protocol)

```typescript
// The core communication format between agents
interface A2AManifest {
  // Metadata
  id: string;                     // UUID for tracking
  timestamp: number;              // Unix ms
  round: number;                  // Consensus round number
  
  // From Pathos
  intent: {
    telos: string;                // Final goal in plain language
    constraints: string[];        // User-specified limits
    priority: 'speed' | 'quality' | 'cost';
    persona_hints: {
      expertise_level: 'novice' | 'intermediate' | 'expert';
      communication_style: 'formal' | 'casual' | 'technical';
    };
  };
  
  // For Logos
  logic_request: {
    query_type: 'generate' | 'analyze' | 'transform' | 'verify';
    required_lora?: string;       // Specific expertise needed
    context_ids: string[];        // Vector IDs to include
    max_tokens?: number;
  };
  
  // For Ethos
  verification_scope: {
    check_facts: boolean;
    check_hardware: boolean;
    check_safety: boolean;
    external_sources?: string[];  // URLs to verify against
  };
}
```

### Hardware Manifest Schema

```json
{
  "$schema": "https://superinstance.ai/schemas/hardware-manifest-v1.json",
  "device": {
    "name": "NVIDIA Jetson Orin Nano",
    "vendor": "NVIDIA",
    "architecture": "arm64",
    "category": "edge_ai"
  },
  "compute": {
    "gpu": {
      "name": "Ampere",
      "cuda_cores": 1024,
      "tensor_cores": 32,
      "vram_mb": 8192,
      "vram_type": "unified"
    },
    "cpu": {
      "cores": 6,
      "threads": 6,
      "max_freq_mhz": 1500
    }
  },
  "limits": {
    "max_model_params_billions": 8,
    "recommended_quantization": "4bit",
    "max_context_tokens": 4096,
    "concurrent_agents": 1,
    "thermal_throttle_celsius": 80,
    "max_power_watts": 15
  },
  "capabilities": {
    "local_training": true,
    "training_max_params_billions": 3,
    "video_input": true,
    "audio_input": true
  },
  "optimizations": {
    "use_flash_attention": true,
    "use_kv_cache": true,
    "prefetch_model_on_boot": true,
    "cuda_graph_capture": true
  }
}
```

### Consensus State Machine

```rust
// cli/src/synapse/consensus.rs

#[derive(Debug, Clone)]
pub enum ConsensusStatus {
    Initializing,
    AwaitingPathos,
    AwaitingLogos,
    AwaitingEthos,
    Deliberating { round: u8 },
    ConsensusReached { score: f32 },
    ArbiterNeeded,
    Failed { reason: String },
}

pub struct ConsensusEngine {
    threshold: f32,           // Default 0.85
    max_rounds: u8,           // Default 3
    current_round: u8,
    state: ConsensusState,
}

impl ConsensusEngine {
    pub fn new(threshold: f32, max_rounds: u8) -> Self {
        Self {
            threshold,
            max_rounds,
            current_round: 0,
            state: ConsensusState::default(),
        }
    }
    
    pub async fn run_cycle(&mut self, prompt: &str) -> ConsensusResult {
        // Phase 1: Intent Extraction
        let pathos_result = self.agents.pathos.process(prompt).await?;
        self.state.pathos_confidence = pathos_result.confidence;
        
        // Phase 2: Logic Synthesis  
        let logos_result = self.agents.logos.process(&pathos_result.manifest).await?;
        self.state.logos_confidence = logos_result.confidence;
        
        // Phase 3: Truth Verification
        let ethos_result = self.agents.ethos.verify(&logos_result.solution).await?;
        self.state.ethos_confidence = ethos_result.confidence;
        
        // Calculate aggregate score
        let aggregate = self.calculate_aggregate();
        
        if aggregate >= self.threshold {
            ConsensusResult::Reached {
                response: logos_result.solution,
                confidence: aggregate,
                sources: logos_result.sources,
            }
        } else if self.current_round >= self.max_rounds {
            ConsensusResult::ArbiterNeeded {
                pathos_view: pathos_result,
                logos_view: logos_result,
                ethos_view: ethos_result,
            }
        } else {
            self.current_round += 1;
            // Inject feedback and retry
            self.run_cycle_with_feedback(ethos_result.feedback).await
        }
    }
    
    fn calculate_aggregate(&self) -> f32 {
        // Weighted average - Ethos has veto power
        let weights = (0.2, 0.4, 0.4); // Pathos, Logos, Ethos
        
        if self.state.ethos_verdict == Verdict::Veto {
            return 0.0; // Automatic fail
        }
        
        self.state.pathos_confidence * weights.0 +
        self.state.logos_confidence * weights.1 +
        self.state.ethos_confidence * weights.2
    }
}
```

## API Contracts

### CLI → Cloud Tunnel Protocol

```protobuf
// proto/tunnel.proto
syntax = "proto3";
package synesis.tunnel;

message Heartbeat {
  string device_id = 1;
  int64 timestamp = 2;
  HardwareVitals vitals = 3;
}

message HardwareVitals {
  float gpu_load_percent = 1;
  float gpu_temp_celsius = 2;
  int32 vram_used_mb = 3;
  int32 vram_total_mb = 4;
  float cpu_load_percent = 5;
}

message EscalationRequest {
  string request_id = 1;
  string redacted_prompt = 2;
  string lora_id = 3;
  string project_id = 4;
  ConsensusState local_state = 5;
}

message ConsensusState {
  int32 round = 1;
  float pathos_confidence = 2;
  float logos_confidence = 3;
  float ethos_confidence = 4;
  string ethos_feedback = 5;
}

message CloudResponse {
  string request_id = 1;
  string response_text = 2;
  float confidence = 3;
  repeated string sources = 4;
  UsageMetrics usage = 5;
}

message UsageMetrics {
  int32 input_tokens = 1;
  int32 output_tokens = 2;
  float cost_usd = 3;
}
```

### Cloudflare Worker Endpoints

```typescript
// cloud/src/index.ts

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);
    
    // Route table
    switch (url.pathname) {
      // WebSocket upgrade for tunnel
      case '/v1/tunnel':
        return handleTunnelUpgrade(request, env);
      
      // REST API endpoints
      case '/v1/consensus':
        return handleConsensusRequest(request, env);
      
      case '/v1/verify':
        return handleVerificationRequest(request, env);
      
      // Billing endpoints
      case '/v1/usage':
        return handleUsageQuery(request, env);
      
      case '/v1/credits':
        return handleCreditsQuery(request, env);
      
      // Collaboration endpoints
      case '/v1/invite':
        return handleInviteCreate(request, env);
      
      case '/v1/handover':
        return handleProjectHandover(request, env);
      
      default:
        return new Response('Not Found', { status: 404 });
    }
  }
};

// Durable Object bindings in wrangler.toml
// [durable_objects]
// bindings = [
//   { name = "SYNAPSE", class_name = "CloudSynapse" },
//   { name = "LEDGER", class_name = "BillingLedger" }
// ]
```

### SDK Client Interface

```typescript
// sdk/typescript/src/client.ts

export interface SynesisConfig {
  apiKey: string;
  projectId: string;
  tier: 'managed' | 'byok';
  localBridge?: {
    host: string;
    port: number;
  };
}

export class SynesisClient {
  private config: SynesisConfig;
  private socket: WebSocket | null = null;
  
  constructor(config: SynesisConfig) {
    this.config = config;
  }
  
  /**
   * Stream a consensus request with real-time updates
   */
  async stream(
    prompt: string,
    onUpdate: (update: ConsensusUpdate) => void,
    options?: StreamOptions
  ): Promise<ConsensusResult> {
    const url = `wss://api.superinstance.ai/v1/stream/${this.config.projectId}`;
    
    return new Promise((resolve, reject) => {
      this.socket = new WebSocket(url);
      
      this.socket.onopen = () => {
        this.socket?.send(JSON.stringify({
          type: 'consensus_request',
          prompt,
          tier: this.config.tier,
          options,
        }));
      };
      
      this.socket.onmessage = (event) => {
        const data = JSON.parse(event.data);
        
        if (data.type === 'update') {
          onUpdate(data as ConsensusUpdate);
        } else if (data.type === 'complete') {
          resolve(data as ConsensusResult);
          this.socket?.close();
        } else if (data.type === 'error') {
          reject(new Error(data.message));
          this.socket?.close();
        }
      };
      
      this.socket.onerror = (error) => {
        reject(error);
      };
    });
  }
  
  /**
   * Simple request/response (no streaming)
   */
  async ask(prompt: string, options?: AskOptions): Promise<ConsensusResult> {
    const response = await fetch(
      `https://api.superinstance.ai/v1/consensus`,
      {
        method: 'POST',
        headers: this.getHeaders(),
        body: JSON.stringify({ prompt, options }),
      }
    );
    
    if (!response.ok) {
      throw new Error(`API error: ${response.status}`);
    }
    
    return response.json();
  }
  
  private getHeaders(): Record<string, string> {
    return {
      'Authorization': `Bearer ${this.config.apiKey}`,
      'X-Synesis-Project': this.config.projectId,
      'X-Synesis-Tier': this.config.tier,
      'Content-Type': 'application/json',
    };
  }
}
```

## Key Implementation Patterns

### Privacy Proxy (Redaction)

```rust
// cli/src/privacy/redactor.rs

use regex::Regex;
use uuid::Uuid;

pub struct Redactor {
    vault: TokenVault,
    patterns: RedactionPatterns,
}

struct RedactionPatterns {
    email: Regex,
    phone: Regex,
    ssn: Regex,
    api_key: Regex,
    file_path: Regex,
    ip_address: Regex,
}

impl Redactor {
    pub fn new(vault: TokenVault) -> Self {
        Self {
            vault,
            patterns: RedactionPatterns {
                email: Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap(),
                phone: Regex::new(r"\b\d{3}[-.]?\d{3}[-.]?\d{4}\b").unwrap(),
                ssn: Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap(),
                api_key: Regex::new(r"(sk|pk|api)[_-]?[a-zA-Z0-9]{20,}").unwrap(),
                file_path: Regex::new(r"(/[\w.-]+)+|([A-Z]:\\[\w.-\\]+)").unwrap(),
                ip_address: Regex::new(r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b").unwrap(),
            },
        }
    }
    
    pub fn redact(&mut self, text: &str) -> String {
        let mut result = text.to_string();
        
        // Order matters - more specific patterns first
        result = self.redact_pattern(&result, &self.patterns.api_key, "SECRET");
        result = self.redact_pattern(&result, &self.patterns.ssn, "SSN");
        result = self.redact_pattern(&result, &self.patterns.email, "EMAIL");
        result = self.redact_pattern(&result, &self.patterns.phone, "PHONE");
        result = self.redact_pattern(&result, &self.patterns.file_path, "PATH");
        result = self.redact_pattern(&result, &self.patterns.ip_address, "IP");
        
        result
    }
    
    fn redact_pattern(&mut self, text: &str, pattern: &Regex, category: &str) -> String {
        let mut result = text.to_string();
        let mut offset: i64 = 0;
        
        for capture in pattern.find_iter(text) {
            let original = capture.as_str();
            let token = self.vault.store(original, category);
            
            let start = (capture.start() as i64 + offset) as usize;
            let end = (capture.end() as i64 + offset) as usize;
            
            result.replace_range(start..end, &token);
            offset += token.len() as i64 - original.len() as i64;
        }
        
        result
    }
    
    pub fn reinflate(&self, text: &str) -> String {
        self.vault.reinflate(text)
    }
}

pub struct TokenVault {
    db: rusqlite::Connection,
}

impl TokenVault {
    pub fn store(&mut self, value: &str, category: &str) -> String {
        let token = format!("[{}_{:04}]", category, self.next_id(category));
        
        self.db.execute(
            "INSERT INTO tokens (token, value, category, created_at) VALUES (?, ?, ?, ?)",
            params![token, value, category, chrono::Utc::now().timestamp()],
        ).unwrap();
        
        token
    }
    
    pub fn reinflate(&self, text: &str) -> String {
        let re = Regex::new(r"\[([A-Z]+)_(\d+)\]").unwrap();
        let mut result = text.to_string();
        
        for capture in re.captures_iter(text) {
            let token = capture.get(0).unwrap().as_str();
            if let Some(value) = self.lookup(token) {
                result = result.replace(token, &value);
            }
        }
        
        result
    }
    
    fn lookup(&self, token: &str) -> Option<String> {
        self.db.query_row(
            "SELECT value FROM tokens WHERE token = ?",
            params![token],
            |row| row.get(0),
        ).ok()
    }
}
```

### Billing Ledger (Durable Object)

```typescript
// cloud/src/ledger.ts

import { DurableObject } from "cloudflare:workers";

export class BillingLedger extends DurableObject {
  private balance: number = 0;
  private knowledgeCredits: number = 0;
  private tier: 'managed' | 'byok' = 'managed';
  
  constructor(state: DurableObjectState, env: Env) {
    super(state, env);
    
    // Restore state from storage
    this.ctx.blockConcurrencyWhile(async () => {
      this.balance = await this.ctx.storage.get('balance') ?? 0;
      this.knowledgeCredits = await this.ctx.storage.get('credits') ?? 0;
      this.tier = await this.ctx.storage.get('tier') ?? 'managed';
    });
  }
  
  async logUsage(usage: UsageEvent): Promise<LedgerResponse> {
    const multiplier = this.tier === 'managed' ? 1.03 : 1.30;
    let charge = usage.costBasis * multiplier;
    
    // Deduct from knowledge credits first
    if (this.knowledgeCredits > 0) {
      const creditDeduction = Math.min(this.knowledgeCredits, charge);
      this.knowledgeCredits -= creditDeduction;
      charge -= creditDeduction;
      await this.ctx.storage.put('credits', this.knowledgeCredits);
    }
    
    this.balance += charge;
    await this.ctx.storage.put('balance', this.balance);
    
    // Log to SQL for history
    await this.ctx.storage.sql.exec(`
      INSERT INTO transactions (type, amount, timestamp, request_id)
      VALUES ('usage', ?, ?, ?)
    `, charge, Date.now(), usage.requestId);
    
    // Check for Stripe flush threshold
    if (this.balance >= STRIPE_THRESHOLD) {
      await this.flushToStripe();
    }
    
    return {
      charged: charge,
      newBalance: this.balance,
      creditsRemaining: this.knowledgeCredits,
    };
  }
  
  async addKnowledgeCredit(amount: number, reason: string): Promise<void> {
    this.knowledgeCredits += amount;
    await this.ctx.storage.put('credits', this.knowledgeCredits);
    
    await this.ctx.storage.sql.exec(`
      INSERT INTO transactions (type, amount, timestamp, reason)
      VALUES ('credit', ?, ?, ?)
    `, amount, Date.now(), reason);
  }
  
  private async flushToStripe(): Promise<void> {
    const userId = await this.ctx.storage.get('userId');
    
    const response = await fetch('https://api.stripe.com/v1/billing/meter_events', {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${this.env.STRIPE_SECRET}`,
        'Idempotency-Key': `${userId}-${Date.now()}`,
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: new URLSearchParams({
        event_name: 'synesis_usage',
        payload: JSON.stringify({
          stripe_customer_id: userId,
          value: Math.round(this.balance * 100).toString(), // cents
        }),
      }),
    });
    
    if (response.ok) {
      this.balance = 0;
      await this.ctx.storage.put('balance', 0);
    }
  }
}

interface UsageEvent {
  requestId: string;
  costBasis: number;  // Wholesale cost from AI Gateway
  inputTokens: number;
  outputTokens: number;
  model: string;
}

interface LedgerResponse {
  charged: number;
  newBalance: number;
  creditsRemaining: number;
}

const STRIPE_THRESHOLD = 5.00; // Flush at $5
```

## Testing Requirements

### Unit Test Coverage Targets

| Component | Minimum Coverage | Critical Paths |
|-----------|-----------------|----------------|
| Privacy Redactor | 95% | All PII patterns |
| Consensus Engine | 90% | State transitions |
| Token Vault | 95% | Store/retrieve/reinflate |
| Billing Ledger | 95% | All calculation paths |
| Hardware Detection | 80% | Platform-specific |

### Integration Test Scenarios

1. **Full Consensus Flow**: Prompt → Pathos → Logos → Ethos → Response
2. **Privacy Round-Trip**: Original → Redact → Cloud → Reinflate → Original
3. **Escalation Path**: Local fail → Redact → Tunnel → Cloud → Return
4. **Billing Accuracy**: Usage → Ledger → Stripe verification
5. **LoRA Hot-Swap**: Upload → R2 → Load → Inference → Verify

---

*This document is the source of truth for implementation details. Update when patterns change.*
