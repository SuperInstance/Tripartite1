# SuperInstance AI - Master Orchestrator Guide

> **Current Status**: Phase 1 Complete ✅ (2026-01-02)
> **Tests**: 122/122 passing (100%)
> **Repository**: https://github.com/SuperInstance/Tripartite1
>
> **Your Role**: You are the master orchestrator for building the SuperInstance AI platform. You spawn autoaccept-enabled worker agents, coordinate their work, track progress, maintain documentation, and ensure architectural coherence across all development phases.

## Orchestrator Mode: Auto-Enabled Agents

You operate with **autoaccept enabled** for spawned agents, meaning:
- Workers execute autonomously without waiting for approval
- You monitor their progress and verify completion
- You spawn multiple agents in parallel when tasks are independent
- You review output and fix issues when agents fail
- You continuously drive the build forward through all sessions

### Workflow for Each Session:
1. **Read status** - Check BUILD_STATUS.md and CHANGELOG.md
2. **Identify next task** - From CLAUDE_CODE_BUILD_GUIDE.md sessions
3. **Spawn agents** - Launch autoaccept workers for implementation
4. **Monitor progress** - Use TaskOutput to track agent work
5. **Verify completion** - Run cargo check, tests, and review code
6. **Update documentation** - BUILD_STATUS.md, CHANGELOG.md
7. **Continue** - Move to next session without pausing

## Executive Summary

SuperInstance is a **tripartite agentic AI system** that prioritizes local processing while enabling intelligent cloud escalation. The core innovation is the **Synesis Protocol**: three specialized agent perspectives (Pathos, Logos, Ethos) that reach consensus before acting.

**The Value Proposition**: "An AI that knows when to stay local, when to escalate to cloud, and keeps your secrets safe either way."

## Your Responsibilities

### 1. Build Coordination
- Spawn and direct worker agents for specific implementation tasks
- Ensure each component integrates correctly with the overall architecture
- Maintain consistent coding standards and patterns across the codebase

### 2. Progress Tracking
- Update `status/BUILD_STATUS.md` after each significant milestone
- Track blockers and dependencies between components
- Maintain the `status/CHANGELOG.md` with all changes

### 3. Documentation Maintenance
- Keep this file (`CLAUDE.md`) updated as you learn project nuances
- Update phase documents when scope changes
- Ensure agent onboarding docs reflect current implementation state

### 4. Quality Gates
Before completing any phase:
- [ ] All components pass their defined tests
- [ ] Integration points verified
- [ ] Documentation updated
- [ ] Security considerations addressed
- [ ] Performance benchmarks met (where applicable)

## Project Architecture at a Glance

```
┌──────────────────────────────────────────────────────────────┐
│                    USER INTERFACE LAYER                       │
│  (CLI / Desktop App / Mobile SDK / Web Dashboard)            │
└─────────────────────────┬────────────────────────────────────┘
                          │
┌─────────────────────────▼────────────────────────────────────┐
│                  LOCAL HUB (Jetson/Edge/WSL)                 │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐                       │
│  │ PATHOS  │  │  LOGOS  │  │  ETHOS  │  ← The Tripartite     │
│  │ (Intent)│  │ (Logic) │  │ (Truth) │    Council            │
│  └────┬────┘  └────┬────┘  └────┬────┘                       │
│       └───────────┼───────────┘                              │
│                   ▼                                          │
│         ┌─────────────────┐                                  │
│         │  LOCAL SYNAPSE  │ ← Consensus Engine               │
│         │   (Orchestrator)│                                  │
│         └────────┬────────┘                                  │
│                  │                                           │
│  ┌───────────────┼───────────────┐                           │
│  │               │               │                           │
│  ▼               ▼               ▼                           │
│ SQLite-VSS   LoRA Store    Hardware                          │
│ (Memory)     (Expertise)    Manifest                         │
└──────────────────────────────────┬───────────────────────────┘
                                   │
                    ┌──────────────▼──────────────┐
                    │   PRIVACY PROXY (Redact)    │
                    │   QUIC TUNNEL (Bridge)      │
                    └──────────────┬──────────────┘
                                   │
┌──────────────────────────────────▼───────────────────────────┐
│                    CLOUDFLARE LAYER                          │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              DURABLE OBJECT (Cloud Synapse)              │ │
│  │  - Session State    - Consensus Cache                   │ │
│  │  - Billing Ledger   - Swarm Coordination                │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                                              │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐             │
│  │ Workers AI │  │  Vectorize │  │     R2     │             │
│  │  (Models)  │  │  (Global   │  │   (LoRA    │             │
│  │            │  │   Memory)  │  │   Storage) │             │
│  └────────────┘  └────────────┘  └────────────┘             │
└──────────────────────────────────────────────────────────────┘
```

## The Tripartite Council

| Agent | Domain | Primary Question | Key Capability |
|-------|--------|------------------|----------------|
| **Pathos** | User Intent | "What does the human actually want?" | Prompt disambiguation, persona learning, A2A translation |
| **Logos** | Project Logic | "How do we accomplish this?" | RAG retrieval, LoRA loading, solution synthesis |
| **Ethos** | Ground Truth | "Is this safe, accurate, and feasible?" | Fact-checking, hardware constraints, thermal limits |

### Consensus Mechanism
No response is emitted until all three agents agree above a threshold (default 0.85). If consensus cannot be reached after 3 rounds, an Arbiter escalation occurs.

## Directory Structure

```
superinstance-project/
├── CLAUDE.md                 ← You are here (Orchestrator Guide)
├── PROJECT_ROADMAP.md        ← Phase timeline and milestones
├── architecture/
│   ├── HIGH_LEVEL.md         ← C-suite/investor overview
│   ├── MEDIUM_LEVEL.md       ← Technical lead overview  
│   └── LOW_LEVEL.md          ← Implementation details
├── phases/
│   ├── PHASE_1_LOCAL_KERNEL.md
│   ├── PHASE_2_CLOUD_MESH.md
│   ├── PHASE_3_MARKETPLACE.md
│   └── PHASE_4_UTILITY.md
├── agents/
│   ├── PATHOS_AGENT.md       ← Onboarding for intent workers
│   ├── LOGOS_AGENT.md        ← Onboarding for logic workers
│   ├── ETHOS_AGENT.md        ← Onboarding for truth workers
│   ├── INFRASTRUCTURE_AGENT.md ← Cloudflare/DevOps workers
│   └── FRONTEND_AGENT.md     ← UI/UX workers
├── prompts/
│   ├── WORKER_PROMPTS.md     ← Effective prompts for spawning workers
│   └── REVIEW_PROMPTS.md     ← Prompts for code review/QA
└── status/
    ├── BUILD_STATUS.md       ← Current state of the build
    └── CHANGELOG.md          ← History of changes
```

## Orchestration Workflow

### When Starting a New Session
1. Read `status/BUILD_STATUS.md` to understand current state
2. Check `status/CHANGELOG.md` for recent context
3. Identify the next priority from `PROJECT_ROADMAP.md`
4. Load relevant phase document from `phases/`

### When Spawning a Worker Agent
1. Identify the appropriate agent type from `agents/`
2. Provide the worker with:
   - Their specific onboarding document
   - The relevant section of the architecture doc
   - Clear acceptance criteria
   - Current context from status files
3. Review their output against the architecture
4. Update status files after completion

### When Completing a Milestone
1. Verify all acceptance criteria are met
2. Update `status/BUILD_STATUS.md` with completion
3. Add entry to `status/CHANGELOG.md`
4. Update `PROJECT_ROADMAP.md` if timeline affected
5. Update this file if you learned important patterns

## Critical Implementation Notes

### Privacy-First Architecture
The **Redact & Re-inflate** proxy is the cornerstone feature:
- All sensitive data is tokenized before cloud transmission
- Tokens are replaced with UUIDs: `[USER_01]`, `[SECRET_CODE_A]`
- Cloud never sees raw PII or proprietary code
- Local hub re-inflates tokens in cloud responses

### Cost-Plus Economics
- **Managed Tier**: 3% markup on Cloudflare wholesale costs
- **BYOK Tier**: 30% licensing fee for using Synesis Protocol
- **Knowledge Credits**: Contributors earn credits that offset costs

### Local-First Decision Tree
```
User submits prompt
        │
        ▼
┌───────────────────┐
│ Check Local Cache │
│ (SQLite-VSS)      │
└─────────┬─────────┘
          │
    Found locally?
     │         │
    YES        NO
     │         │
     ▼         ▼
  Return    ┌─────────────────┐
  cached    │ Check Hardware  │
            │ Constraints     │
            └────────┬────────┘
                     │
              Can run locally?
               │         │
              YES        NO
               │         │
               ▼         ▼
         Run locally   Redact → Cloud
         (save cost)   (full power)
```

## Technology Stack Reference

| Layer | Technology | Purpose |
|-------|------------|---------|
| Local Orchestrator | Rust | Zero-latency thread management |
| Model Runtime | llama.cpp / TensorRT | Native inference |
| Privacy Proxy | Rust/Go | High-speed token replacement |
| Local Memory | SQLite-VSS | Portable vector database |
| Cloud Tunnel | QUIC (quinn crate) | Resilient bi-directional stream |
| Cloud State | Durable Objects | Stateful session management |
| Cloud Inference | Workers AI | Serverless GPU access |
| Cloud Memory | Vectorize | Global vector index |
| Storage | R2 | LoRA and asset storage |
| Payments | Stripe Metered Billing | Automated invoicing |

## Periodic Review Checklist

Run this checklist weekly or after major milestones:

### Architecture Coherence
- [ ] Are all new components following the tripartite pattern?
- [ ] Is privacy maintained at every transmission point?
- [ ] Do local and cloud implementations mirror each other?

### Documentation Currency
- [ ] Does BUILD_STATUS.md reflect actual state?
- [ ] Are agent onboarding docs still accurate?
- [ ] Have any new patterns emerged that should be documented?

### Technical Debt
- [ ] Are there any shortcuts that need revisiting?
- [ ] Have any security concerns been deferred?
- [ ] Are test coverages adequate?

## Notes Section (Update as you learn)

<!-- Add your observations, patterns, and learnings here -->

### Current Phase: Phase 1 Complete ✅ (2026-01-02)

**Completed Components**:
- ✅ CLI Foundation (hardware detection, model downloader, init/status commands)
- ✅ Tripartite Council (Pathos, Logos, Ethos agents + Consensus Engine)
- ✅ Privacy Proxy (redaction patterns, token vault, redactor)
- ✅ Knowledge Vault (SQLite-VSS, embeddings, chunking, file watcher)
- ✅ Integration & CLI (thread safety fixed, all commands working)

**Test Results**: 122/122 passing (100%)
- synesis-core: 38 tests
- synesis-knowledge: 28 tests
- synesis-models: 12 tests
- synesis-privacy: 37 tests
- synesis-cli: 7 tests

**Known Issues**:
- ⚠️ File watcher auto-indexing disabled (DocumentIndexer holds &KnowledgeVault across await)
- ℹ️ 19 compiler warnings (unused imports/variables, non-critical)

### Patterns Discovered
1. **Thread Safety Pattern**: Use `Arc<Mutex<T>>` at application level, not inside library structs
2. **Async/Await with SQLite**: Cannot hold `MutexGuard` across await points with rusqlite
3. **Agent System**: All agents implement `Agent` trait with `process(&self, input: AgentInput) -> AgentOutput`
4. **Privacy-First**: Token vault with global counters per category (not per-document)

### Gotchas Encountered
1. **rusqlite Connection**: Not Send/Sync - uses RefCell internally
2. **DocumentIndexer Lifetime**: Holds `&'a KnowledgeVault` incompatible with async callbacks
3. **Privacy Patterns**: Need priority ordering, IPv6 :: compression, SK API key underscore/dash support
4. **Chunker Empty Chunks**: Small documents (< threshold) return empty chunks (fixed)
5. **Relevance Scoring**: Was inverted (higher distance = better, should be lower = better)

### Questions for Human Review
- None currently - project ready for integration testing

---

*Last Updated: 2026-01-02*
*Current Phase: Phase 1 Complete - Comprehensive Audit In Progress*
