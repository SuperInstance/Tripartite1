# SuperInstance AI - Project Roadmap

## Vision Statement

SuperInstance is the **"Linux of the Agentic Era"**—a community-driven, privacy-first, hardware-aware AI ecosystem that makes cloud AI accessible while keeping user data sovereign.

## Strategic Positioning

| Era | Focus | Our Advantage |
|-----|-------|---------------|
| 2023-2024 | Scaling (bigger models) | We do **Tailoring** (smarter routing) |
| 2025-2026 | Agent Proliferation | We do **Consensus** (coordinated intelligence) |
| 2026+ | AI Commoditization | We do **Utility** (foundational infrastructure) |

---

## Phase Overview

```
PHASE 1: Local Kernel        PHASE 2: Cloud Mesh         PHASE 3: Marketplace       PHASE 4: Utility
(Months 1-4)                 (Months 5-8)                (Months 9-12)              (Year 2+)
─────────────────────────────────────────────────────────────────────────────────────────────────────
┌─────────────────┐          ┌─────────────────┐         ┌─────────────────┐        ┌─────────────────┐
│ • CLI Tool      │          │ • Cloud Bridge  │         │ • LoRA Market   │        │ • API Standard  │
│ • Local Agents  │────────▶ │ • Billing       │───────▶ │ • Expert Seals  │──────▶ │ • Verification  │
│ • Redact Proxy  │          │ • Hot-Swap LoRA │         │ • Swarm Logic   │        │ • Federation    │
│ • SQLite-VSS    │          │ • Collaborator  │         │ • SDK Release   │        │ • Exit to Util  │
└─────────────────┘          └─────────────────┘         └─────────────────┘        └─────────────────┘
                                                                                              
Revenue: $0                  Revenue: $5/mo subs         Revenue: 3% + Market       Revenue: Utility
(Building trust)             (First cash flow)           (Growing margins)          (Foundational)
```

---

## Phase 1: The Local Kernel (Months 1-4)

**Goal**: Prove that "Tailoring beats Scaling" with a local-first AI that respects privacy.

### Milestones

#### 1.1 CLI Foundation (Week 1-4)
- [ ] `synesis init` - Hardware detection and configuration
- [ ] `synesis ask` - Single-prompt interaction with local council
- [ ] `synesis status` - Display hardware vitals and agent states
- [ ] Config file generation (`synesis.json`)

**Definition of Done**: User can run `synesis ask "Hello"` and receive a consensus response from three local agents.

#### 1.2 Tripartite Council (Week 3-6)
- [ ] Pathos agent: Intent extraction with local LLM
- [ ] Logos agent: RAG-based solution synthesis
- [ ] Ethos agent: Hardware constraint checking
- [ ] Consensus engine: Threshold voting mechanism
- [ ] Inter-agent communication (JSON schema)

**Definition of Done**: All three agents participate in every response, visible in debug mode.

#### 1.3 Privacy Proxy (Week 5-8)
- [ ] PII detection (names, emails, IPs)
- [ ] Code pattern detection (API keys, secrets)
- [ ] UUID tokenization system
- [ ] Token-to-value mapping store
- [ ] Re-inflation on response return

**Definition of Done**: A prompt containing a real name is transmitted to cloud with `[USER_01]` replacing it.

#### 1.4 Knowledge Vault (Week 6-10)
- [ ] SQLite-VSS database setup
- [ ] File watcher for project directories
- [ ] Embedding pipeline (local model)
- [ ] Inventory check before cloud calls
- [ ] Nightly synthesis job (log → wisdom)

**Definition of Done**: After indexing a PDF, queries about its content are answered locally without cloud.

#### 1.5 Hardware Agent Community Seed (Week 8-12)
- [ ] Hardware manifest schema
- [ ] Jetson Orin manifest
- [ ] Generic x86 manifest
- [ ] Apple Silicon manifest (stretch)
- [ ] Manifest contribution guide

**Definition of Done**: GitHub repo with 3+ community-contributed hardware manifests.

### Phase 1 Success Metrics
| Metric | Target |
|--------|--------|
| Local response latency | < 2s for simple queries |
| Privacy leakage | 0 PII to cloud |
| Hardware platforms | 3+ working manifests |
| Community contributors | 10+ |

---

## Phase 2: The Cloud Mesh (Months 5-8)

**Goal**: Activate the "Automated Cash Cow" with seamless local-to-cloud escalation.

### Milestones

#### 2.1 Cloud Bridge (Week 1-4)
- [ ] QUIC tunnel implementation (Rust)
- [ ] mTLS certificate management
- [ ] Heartbeat telemetry stream
- [ ] Pre-warm signaling
- [ ] Federated vector proxy (search-over-tunnel)

**Definition of Done**: Persistent tunnel survives 24hr uptime with <50ms overhead.

#### 2.2 Billing Ledger (Week 2-5)
- [ ] Durable Object class deployment
- [ ] Cost-Plus calculation (3%/30%)
- [ ] Knowledge credit system
- [ ] Stripe metered billing integration
- [ ] Credit ceiling circuit breaker

**Definition of Done**: A user exceeding $5 in usage triggers automatic Stripe invoice.

#### 2.3 LoRA Hot-Swap (Week 4-7)
- [ ] R2 storage for adapter files
- [ ] Expert-injection middleware
- [ ] `synesis push` command for LoRA upload
- [ ] Cloud-side adapter loading
- [ ] Latency optimization (caching)

**Definition of Done**: Local-trained LoRA is usable in cloud inference within 100ms of request.

#### 2.4 Collaborator System (Week 6-9)
- [ ] Project-bound JWT generation
- [ ] Guest view dashboard
- [ ] Cost attribution to host
- [ ] Guest quota enforcement
- [ ] Handover workflow

**Definition of Done**: Freelancer shares link, client watches agents, clicks approve to execute.

#### 2.5 Cloud Synapse (Week 8-12)
- [ ] Durable Object orchestrator
- [ ] Internal Pub/Sub gossip
- [ ] Context-pinning (state persistence)
- [ ] Arbiter tie-breaker logic
- [ ] Cloud-to-local callback

**Definition of Done**: Complex prompt triggers multi-round cloud consensus with visible debate.

### Phase 2 Success Metrics
| Metric | Target |
|--------|--------|
| Paid subscribers | 100+ |
| Monthly recurring revenue | $500+ |
| Cloud escalation latency | <1s perceived |
| Handover conversion rate | 20%+ |

---

## Phase 3: The Marketplace (Months 9-12)

**Goal**: Enable developers to productize expertise and create network effects.

### Milestones

#### 3.1 Expert Marketplace
- [ ] LoRA listing and discovery
- [ ] Rating and review system
- [ ] Revenue sharing (70/30 split)
- [ ] Expert verification badges

#### 3.2 Swarm Logic
- [ ] Cross-project service bindings
- [ ] ZKP-based consultation (privacy)
- [ ] Swarm leader election
- [ ] Multi-agent consensus streaming

#### 3.3 SDK Release
- [ ] TypeScript SDK (React Native, Electron, Web)
- [ ] Python SDK
- [ ] Partner ID attribution headers
- [ ] Hardware detection bridge

#### 3.4 Knowledge Marketplace
- [ ] Premium fact fragments
- [ ] Contributor payout system
- [ ] Verification workflow

### Phase 3 Success Metrics
| Metric | Target |
|--------|--------|
| Listed experts (LoRAs) | 100+ |
| SDK integrations | 50+ apps |
| Knowledge contributors | 500+ |
| Monthly transaction volume | $10k+ |

---

## Phase 4: Exit to Utility (Year 2+)

**Goal**: Become foundational infrastructure—the "SSL of AI Verification."

### Strategic Milestones

#### 4.1 Synesis-Verified API
- Public verification-as-a-service
- Trust badge integration
- Enterprise SLAs

#### 4.2 A2A Protocol Standardization
- Open standard publication
- Industry adoption campaign
- Reference implementations

#### 4.3 Federation
- Multi-provider backend support
- Decentralized identity
- Cross-ecosystem interoperability

#### 4.4 Utility Status
- Direct Cloudflare/infrastructure partnerships
- Government/regulatory adoption
- Self-sustaining economic model

---

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Cloudflare pricing changes | Medium | High | Abstract provider layer, maintain BYOK option |
| LoRA hot-swap limitations | High | Medium | Graceful fallback to standard models |
| Low Phase 1 adoption | Medium | High | Focus on specific vertical (legal, medical) |
| Competition from majors | High | Medium | Speed + privacy moat, community lock-in |
| Complexity overwhelm | High | High | Strict phase gates, defer features |

---

## Decision Log

| Date | Decision | Rationale | Reversible? |
|------|----------|-----------|-------------|
| [Initial] | Rust for CLI | Speed + safety for persistent networking | No (major) |
| [Initial] | SQLite-VSS for local | Portable, no server dependency | Yes |
| [Initial] | 3% margin | Volume > margin, utility positioning | Yes |
| [Initial] | Tripartite model | Separation of concerns, consensus quality | No (architectural) |

---

## Checkpoint Reviews

### Phase 1 Exit Review
- [ ] All 1.x milestones complete
- [ ] 10+ users running locally
- [ ] No critical bugs in privacy proxy
- [ ] Architecture document validated

### Phase 2 Exit Review
- [ ] Positive cash flow achieved
- [ ] Cloud reliability > 99%
- [ ] Handover workflow proven
- [ ] Scaling plan validated

### Phase 3 Exit Review
- [ ] Network effects visible
- [ ] SDK adoption growing
- [ ] Community self-sustaining
- [ ] Series A metrics achievable

---

*Document Version: 1.0*
*Last Updated: [Orchestrator updates this]*
*Next Review: End of Phase 1.1*
