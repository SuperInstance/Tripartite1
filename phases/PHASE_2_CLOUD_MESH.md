# Phase 2: The Cloud Mesh

> **Duration**: Months 5-8
> **Goal**: Activate the "Automated Cash Cow" with seamless local-to-cloud escalation
> **Prerequisite**: Phase 1 complete and stable

---

## Phase 2 Overview

Phase 2 transforms SuperInstance from a local tool into a commercial platform. The local Kernel connects to Cloudflare's global network, enabling cloud escalation, automated billing, and collaborative features.

## Success Criteria

- [ ] QUIC tunnel maintains 24hr uptime with <50ms overhead
- [ ] Billing ledger accurately tracks usage and bills via Stripe
- [ ] LoRA hot-swap works in <200ms
- [ ] Collaborator invite-to-handover flow converts clients
- [ ] First $500 MRR achieved

---

## Milestone 2.1: Cloud Bridge (Weeks 1-4)

### Objective
Establish a persistent, secure connection between local hubs and Cloudflare.

### Deliverables

#### QUIC Tunnel (Rust)
```rust
// cli/src/tunnel/quic.rs

use quinn::{Endpoint, Connection};
use rustls::Certificate;

pub struct CloudBridge {
    endpoint: Endpoint,
    connection: Option<Connection>,
    device_cert: Certificate,
    heartbeat_interval: Duration,
}

impl CloudBridge {
    pub async fn connect(&mut self, cloud_url: &str) -> Result<()> {
        // Configure mTLS with device certificate
        let crypto = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_client_auth_cert(vec![self.device_cert.clone()], self.device_key.clone())?;
        
        self.connection = Some(
            self.endpoint.connect(cloud_url.parse()?, "superinstance.ai")?
                .await?
        );
        
        // Start heartbeat task
        self.spawn_heartbeat();
        
        Ok(())
    }
    
    pub async fn escalate(&self, request: EscalationRequest) -> Result<CloudResponse> {
        let conn = self.connection.as_ref().ok_or(Error::NotConnected)?;
        let (mut send, mut recv) = conn.open_bi().await?;
        
        // Send protobuf-encoded request
        let encoded = request.encode_to_vec();
        send.write_all(&encoded).await?;
        send.finish().await?;
        
        // Receive response
        let response_bytes = recv.read_to_end(MAX_RESPONSE_SIZE).await?;
        let response = CloudResponse::decode(&response_bytes[..])?;
        
        Ok(response)
    }
    
    fn spawn_heartbeat(&self) {
        let conn = self.connection.clone();
        let interval = self.heartbeat_interval;
        
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                
                let vitals = HardwareVitals::collect().await;
                let heartbeat = Heartbeat {
                    device_id: DEVICE_ID.to_string(),
                    timestamp: chrono::Utc::now().timestamp_millis(),
                    vitals: Some(vitals),
                };
                
                if let Some(ref conn) = conn {
                    let mut send = conn.open_uni().await.unwrap();
                    send.write_all(&heartbeat.encode_to_vec()).await.unwrap();
                }
            }
        });
    }
}
```

#### Pre-warm Signaling
When local GPU load exceeds threshold, notify cloud to prepare:

```rust
pub async fn check_prewarm_needed(&self, hardware: &HardwareState) -> bool {
    hardware.gpu_load > 0.8 || hardware.temp_celsius > 80.0
}

pub async fn send_prewarm_signal(&self, project_id: &str, lora_id: Option<&str>) {
    let signal = PrewarmSignal {
        project_id: project_id.to_string(),
        lora_id: lora_id.map(String::from),
        timestamp: chrono::Utc::now().timestamp_millis(),
    };
    
    // Send via heartbeat channel (non-blocking)
    self.heartbeat_tx.send(HeartbeatPayload::Prewarm(signal)).await;
}
```

### Acceptance Criteria
- Tunnel survives network interruptions (auto-reconnect)
- mTLS enforced on all connections
- Heartbeat maintains <50ms latency
- Pre-warm reduces cloud cold-start by 80%

---

## Milestone 2.2: Billing Ledger (Weeks 2-5)

### Objective
Implement real-time billing with 3% managed / 30% BYOK pricing.

### Cloudflare Durable Object

```typescript
// cloud/src/ledger.ts

export class BillingLedger extends DurableObject {
    private state: LedgerState;
    
    async fetch(request: Request): Promise<Response> {
        const url = new URL(request.url);
        
        switch (url.pathname) {
            case '/log-usage':
                return this.handleLogUsage(request);
            case '/add-credit':
                return this.handleAddCredit(request);
            case '/get-balance':
                return this.handleGetBalance();
            case '/set-tier':
                return this.handleSetTier(request);
            default:
                return new Response('Not Found', { status: 404 });
        }
    }
    
    private async handleLogUsage(request: Request): Promise<Response> {
        const usage: UsageEvent = await request.json();
        
        // Calculate charge based on tier
        const multiplier = this.state.tier === 'managed' ? 1.03 : 1.30;
        let charge = usage.costBasis * multiplier;
        
        // Apply knowledge credits first
        if (this.state.knowledgeCredits > 0) {
            const creditDeduction = Math.min(this.state.knowledgeCredits, charge);
            this.state.knowledgeCredits -= creditDeduction;
            charge -= creditDeduction;
        }
        
        // Update balance
        this.state.unbilledBalance += charge;
        await this.persist();
        
        // Check Stripe threshold
        if (this.state.unbilledBalance >= STRIPE_THRESHOLD) {
            await this.flushToStripe();
        }
        
        // Check credit ceiling (fraud protection)
        if (this.state.unbilledBalance >= this.state.creditCeiling) {
            return new Response(JSON.stringify({
                error: 'credit_ceiling_exceeded',
                action: 'suspend_cloud_access'
            }), { status: 402 });
        }
        
        return new Response(JSON.stringify({
            charged: charge,
            balance: this.state.unbilledBalance,
            credits: this.state.knowledgeCredits,
        }));
    }
    
    private async flushToStripe(): Promise<void> {
        const amountCents = Math.round(this.state.unbilledBalance * 100);
        
        const response = await fetch('https://api.stripe.com/v1/billing/meter_events', {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${this.env.STRIPE_SECRET}`,
                'Idempotency-Key': `${this.state.userId}-${Date.now()}`,
                'Content-Type': 'application/x-www-form-urlencoded',
            },
            body: new URLSearchParams({
                event_name: 'superinstance_usage',
                'payload[stripe_customer_id]': this.state.stripeCustomerId,
                'payload[value]': amountCents.toString(),
            }),
        });
        
        if (response.ok) {
            this.state.unbilledBalance = 0;
            await this.persist();
        }
    }
}
```

### Stripe Integration Setup

1. Create metered product in Stripe Dashboard
2. Configure webhook for payment events
3. Implement customer creation on signup
4. Handle payment failures gracefully

### Acceptance Criteria
- Every inference call is logged
- Markup calculation is accurate (3% vs 30%)
- Knowledge credits offset charges
- Stripe invoices are generated correctly
- Credit ceiling prevents runaway bills

---

## Milestone 2.3: LoRA Hot-Swap (Weeks 4-7)

### Objective
Enable local-trained expertise to be used in cloud inference.

### Upload Flow (`synesis push`)

```bash
$ synesis push --expert

Preparing LoRA for upload...
  ✓ Found: ~/.synesis/loras/project-logos-v1/
  ✓ Size: 127 MB
  ✓ Base model: llama-3.1-8b

Uploading to Cloudflare R2...
  ████████████████████████████████ 100% (127 MB)

Registering with Workers AI...
  ✓ LoRA ID: lora_abc123def456
  ✓ Available in all regions

Your expertise is now available in the cloud!
```

### Cloud-Side Loading

```typescript
// cloud/src/lora-manager.ts

export class LoRAManager {
    constructor(private env: Env) {}
    
    async loadForInference(
        loraId: string,
        prompt: string
    ): Promise<AIResponse> {
        // Check cache first
        const cached = await this.env.LORA_CACHE.get(loraId);
        if (cached) {
            return this.runWithLoRA(cached, prompt);
        }
        
        // Fetch from R2
        const loraObject = await this.env.LORA_BUCKET.get(`adapters/${loraId}.safetensors`);
        if (!loraObject) {
            throw new Error(`LoRA not found: ${loraId}`);
        }
        
        // Run inference with adapter
        const response = await this.env.AI.run(
            '@cf/meta/llama-3.1-8b-instruct',
            {
                prompt,
                lora: loraId,  // Workers AI native LoRA support
            }
        );
        
        return response;
    }
}
```

### Acceptance Criteria
- Upload completes in <60s for 200MB LoRA
- First inference with new LoRA: <500ms
- Subsequent inferences: <200ms (cached)
- LoRA isolation (users can't access others' adapters)

---

## Milestone 2.4: Collaborator System (Weeks 6-9)

### Objective
Enable viral growth through project sharing and handover.

### Invite Flow

```typescript
// cloud/src/collaboration.ts

export async function createInvite(
    projectId: string,
    hostUserId: string,
    options: InviteOptions
): Promise<string> {
    const token = crypto.randomUUID();
    
    const invite: InviteRecord = {
        token,
        projectId,
        hostUserId,
        role: options.role || 'viewer',
        guestQuota: options.guestQuota || 1.00, // $1 default
        expiresAt: Date.now() + (options.expiryHours || 24) * 60 * 60 * 1000,
        createdAt: Date.now(),
    };
    
    await env.INVITE_KV.put(token, JSON.stringify(invite), {
        expirationTtl: options.expiryHours * 60 * 60,
    });
    
    return `https://app.superinstance.ai/join/${token}`;
}

export async function acceptInvite(
    token: string,
    guestEmail: string
): Promise<GuestSession> {
    const inviteData = await env.INVITE_KV.get(token);
    if (!inviteData) {
        throw new Error('Invite expired or invalid');
    }
    
    const invite: InviteRecord = JSON.parse(inviteData);
    
    // Create guest session
    const session: GuestSession = {
        id: crypto.randomUUID(),
        projectId: invite.projectId,
        guestEmail,
        role: invite.role,
        remainingQuota: invite.guestQuota,
        hostUserId: invite.hostUserId,
    };
    
    await env.GUEST_SESSIONS.put(session.id, JSON.stringify(session));
    
    return session;
}
```

### Handover Flow

```typescript
export async function initiateHandover(
    projectId: string,
    fromUserId: string,
    toEmail: string
): Promise<HandoverToken> {
    // Snapshot project state
    const projectState = await getProjectSnapshot(projectId);
    
    const handover: HandoverRecord = {
        token: crypto.randomUUID(),
        projectId,
        fromUserId,
        toEmail,
        projectState,
        incentive: {
            firstYearPrice: 30.00,
            regularPrice: 60.00,
        },
        expiresAt: Date.now() + 7 * 24 * 60 * 60 * 1000, // 7 days
    };
    
    await env.HANDOVER_KV.put(handover.token, JSON.stringify(handover));
    
    // Send email to new user
    await sendHandoverEmail(toEmail, handover);
    
    return handover;
}

export async function completeHandover(
    token: string,
    newUserId: string
): Promise<void> {
    const handoverData = await env.HANDOVER_KV.get(token);
    const handover: HandoverRecord = JSON.parse(handoverData);
    
    // Clone project to new user
    await cloneProject(handover.projectId, handover.fromUserId, newUserId);
    
    // Clone LoRAs
    for (const loraId of handover.projectState.loraIds) {
        await cloneLoRA(loraId, newUserId);
    }
    
    // Set up subscription with incentive pricing
    await createSubscription(newUserId, handover.incentive);
    
    // Cleanup
    await env.HANDOVER_KV.delete(token);
}
```

### Acceptance Criteria
- Invite link works in one click
- Guest sees live agent debate
- Usage bills to host until quota exhausted
- Handover creates independent subscription
- Conversion rate tracking implemented

---

## Milestone 2.5: Cloud Synapse (Weeks 8-12)

### Objective
Replicate the local consensus engine in Cloudflare with full state persistence.

### Durable Object Implementation

```typescript
// cloud/src/orchestrator.ts

export class CloudSynapse extends DurableObject {
    private state: SynapseState;
    private pubsub: PubSubClient;
    
    async fetch(request: Request): Promise<Response> {
        // Handle WebSocket upgrade
        if (request.headers.get('Upgrade') === 'websocket') {
            return this.handleWebSocket(request);
        }
        
        const url = new URL(request.url);
        
        switch (url.pathname) {
            case '/consensus':
                return this.handleConsensusRequest(request);
            case '/state':
                return this.handleStateQuery();
            default:
                return new Response('Not Found', { status: 404 });
        }
    }
    
    private async handleConsensusRequest(request: Request): Promise<Response> {
        const { prompt, loraId, localState } = await request.json();
        
        // Load context from previous rounds if continuing
        if (localState) {
            this.state.round = localState.round + 1;
            this.state.previousFeedback = localState.ethosFeedback;
        }
        
        // Run cloud consensus
        const result = await this.runCloudConsensus(prompt, loraId);
        
        // Stream updates via Pub/Sub
        await this.broadcastUpdates(result);
        
        return new Response(JSON.stringify(result));
    }
    
    private async runCloudConsensus(
        prompt: string,
        loraId?: string
    ): Promise<ConsensusResult> {
        // Pathos (intent extraction)
        const pathosPromise = this.runPathos(prompt);
        
        // Pre-load LoRA while Pathos runs
        if (loraId) {
            await this.env.LORA_MANAGER.warmup(loraId);
        }
        
        const pathosResult = await pathosPromise;
        await this.broadcast('pathos', pathosResult);
        
        // Logos (solution synthesis)
        const logosResult = await this.runLogos(pathosResult.manifest, loraId);
        await this.broadcast('logos', logosResult);
        
        // Ethos (verification)
        const ethosResult = await this.runEthos(logosResult.solution, pathosResult.manifest);
        await this.broadcast('ethos', ethosResult);
        
        // Calculate consensus
        const score = this.calculateScore(pathosResult, logosResult, ethosResult);
        
        if (score >= CONSENSUS_THRESHOLD) {
            return {
                status: 'consensus',
                response: logosResult.solution,
                score,
                sources: logosResult.sources,
            };
        }
        
        if (this.state.round >= MAX_ROUNDS) {
            return this.runArbiter(pathosResult, logosResult, ethosResult);
        }
        
        // Continue deliberation
        this.state.round++;
        return this.runCloudConsensus(
            `${prompt}\n\nFeedback: ${ethosResult.feedback}`,
            loraId
        );
    }
    
    private async runArbiter(
        pathos: PathosResult,
        logos: LogosResult,
        ethos: EthosResult
    ): Promise<ConsensusResult> {
        // Use higher-power model for tie-breaking
        const arbiterResponse = await this.env.AI.run(
            '@cf/meta/llama-3.1-70b-instruct',
            {
                prompt: `You are the Arbiter. Three agents could not reach consensus.
                
                Pathos (Intent): ${JSON.stringify(pathos.manifest)}
                Logos (Solution): ${logos.solution}
                Ethos (Concerns): ${ethos.constraints.join(', ')}
                
                Provide a balanced resolution that addresses Ethos's concerns while
                achieving the user's intent. Be decisive.`,
            }
        );
        
        return {
            status: 'arbiter_resolved',
            response: arbiterResponse.response,
            score: 0.75, // Arbiter resolution is lower confidence
            note: 'Resolved by Arbiter due to agent disagreement',
        };
    }
}
```

### Acceptance Criteria
- Full consensus flow works in cloud
- State persists across requests
- Arbiter breaks deadlocks effectively
- Cloud-to-local callback executes commands
- Real-time streaming to dashboard

---

## Phase 2 Definition of Done

### Functional Requirements
- [ ] Tunnel connects and maintains 24hr uptime
- [ ] All usage is billed correctly via Stripe
- [ ] LoRAs upload and hot-swap in cloud
- [ ] Collaborator flow works end-to-end
- [ ] Cloud Synapse matches local quality

### Business Requirements  
- [ ] 100+ paid subscribers
- [ ] $500+ MRR
- [ ] 20%+ handover conversion rate
- [ ] <1% billing errors

### Technical Requirements
- [ ] <50ms tunnel latency
- [ ] <200ms LoRA hot-swap
- [ ] 99.9% uptime
- [ ] Security audit passed

---

*Previous: [PHASE_1_LOCAL_KERNEL.md](./PHASE_1_LOCAL_KERNEL.md)*
*Next: [PHASE_3_MARKETPLACE.md](./PHASE_3_MARKETPLACE.md)*
