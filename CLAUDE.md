# SuperInstance AI - Development Guide

> **Current Status**: Phase 2: Cloud Mesh - In Progress (2026-01-07)
> **Tests**: 250+ tests passing (100%)
> **Repository**: https://github.com/SuperInstance/Tripartite1
> **Version**: v0.2.0

---

## Executive Summary

SuperInstance is a **tripartite agentic AI system** that prioritizes local processing while enabling intelligent cloud escalation. The system combines three specialized agents (Pathos, Logos, Ethos) that reach consensus before responding, ensuring high-quality, safe answers with privacy-first architecture.

**The Value Proposition**: "An AI that knows when to stay local, when to escalate to cloud, and keeps your secrets safe either way."

---

## Architecture Overview

```
┌──────────────────────────────────────────────────────────────┐
│                    USER INTERFACE LAYER                       │
│  (CLI / Desktop App / Mobile SDK / Web Dashboard)            │
└─────────────────────────┬────────────────────────────────────┘
                          │
┌─────────────────────────▼────────────────────────────────────┐
│                  LOCAL HUB (Rust)                            │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐                       │
│  │ PATHOS  │  │  LOGOS  │  │  ETHOS  │  ← Tripartite Council │
│  │ (Intent)│  │ (Logic) │  │ (Truth) │                       │
│  └────┬────┘  └────┬────┘  └────┬────┘                       │
│       └───────────┼───────────┘                              │
│                   ▼                                          │
│         ┌─────────────────┐                                  │
│         │ Consensus Engine│ ← Weighted voting (0.85 threshold)│
│         └────────┬────────┘                                  │
│                  │                                           │
│  ┌───────────────┼───────────────┐                           │
│  ▼               ▼               ▼                           │
│ SQLite-VSS   LoRA Store    Hardware                           │
│ (Memory)     (Expertise)    Detection                        │
└──────────────────────────────────┬───────────────────────────┘
                                   │
                    ┌──────────────▼──────────────┐
                    │   PRIVACY PROXY (Redact)    │
                    │   QUIC TUNNEL (Bridge)      │ ← synesis-cloud crate
                    └──────────────┬──────────────┘
                                   │
┌──────────────────────────────────▼───────────────────────────┐
│                    CLOUDFLARE LAYER (Future)                  │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              DURABLE OBJECT (Cloud Synapse)              │ │
│  │  - Session State    - Billing Ledger                    │ │
│  │  - Consensus Cache   - Swarm Coordination               │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                                              │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐             │
│  │ Workers AI │  │  Vectorize │  │     R2     │             │
│  │  (Models)  │  │  (Global   │  │   (LoRA    │             │
│  │            │  │   Memory)  │  │   Storage) │             │
│  └────────────┘  └────────────┘  └────────────┘             │
└──────────────────────────────────────────────────────────────┘
```

### The Tripartite Council

| Agent | Domain | Primary Question | Key Capability |
|-------|--------|------------------|----------------|
| **Pathos** | User Intent | "What does the human actually want?" | Prompt disambiguation, persona learning, A2A translation |
| **Logos** | Project Logic | "How do we accomplish this?" | RAG retrieval, LoRA loading, solution synthesis |
| **Ethos** | Ground Truth | "Is this safe, accurate, and feasible?" | Fact-checking, hardware constraints, thermal limits |

**Consensus Mechanism**: No response is emitted until all three agents agree above a threshold (default 0.85). If consensus cannot be reached after 3 rounds, an Arbiter escalation occurs.

---

## Project Structure

```
/mnt/c/claudesuperinstance/
├── CLAUDE.md                   ← This file (master development guide)
├── README.md                   ← Project overview and quick start
├── Cargo.toml                  ← Workspace configuration
│
├── crates/                     ← Rust workspace (6 crates)
│   ├── synesis-cli/            ← Command-line interface (7 tests)
│   ├── synesis-core/           ← Tripartite council & consensus (85 tests)
│   ├── synesis-knowledge/      ← Knowledge vault & RAG (28 tests)
│   ├── synesis-models/         ← Hardware detection & models (12 tests)
│   ├── synesis-privacy/        ← Privacy proxy & redaction (37 tests)
│   └── synesis-cloud/          ← QUIC tunnel & cloud connectivity (68 tests)
│
├── phases/                     ← Phase documentation
│   ├── PHASE_1_LOCAL_KERNEL.md ← Phase 1: COMPLETE ✅
│   └── PHASE_2_DETAILED_ROADMAP.md ← Phase 2: IN PROGRESS 🔄
│
├── cloud/                      ← Cloudflare Workers (TypeScript, Phase 2)
├── status/                     ← Build tracking & completion reports
├── manifests/                  ← Hardware profile definitions
└── architecture/               ← Detailed architecture docs
```

---

## Development Methodology: Ralph Wiggum

**Ralph Wiggum** is an autonomous AI development technique based on **persistent iteration**:

### Core Principles

1. **Methodical Sequential Work** - Complete each session in order before moving to the next
2. **Self-Referential Improvement** - Your previous work exists in files and git history; learn from it
3. **Test-Driven Iteration** - Write code, test it, fix bugs, repeat until all tests pass
4. **Clear Completion Criteria** - Each session has defined acceptance criteria
5. **Persistent Progress** - When you encounter bugs, fix them. When tests fail, debug them. Keep going.

### The Ralph Loop

```bash
while not complete:
    1. Study the session requirements (from roadmap)
    2. Read companion documents (phases/phase2/*.md)
    3. Implement the feature
    4. Run tests
    5. Fix failures
    6. Verify acceptance criteria
    7. Document completion
    8. Move to next session
```

### Starting Each Session

```bash
# 1. Read the roadmap
cat phases/PHASE_2_DETAILED_ROADMAP.md | grep -A 50 "Session 2.X"

# 2. Study companion docs
ls phases/phase2/
# - CLOUD_API_SPECIFICATION.md
# - DATA_MODELS.md
# - QUIC_TUNNEL_GUIDE.md
# - SECURITY_CONSIDERATIONS.md

# 3. Check current state
git log --oneline -10
cargo test --workspace

# 4. Set acceptance criteria from roadmap
# (Keep visible while working)
```

### During Implementation

```bash
# Write code according to specs
# Test frequently
cargo test --package synesis-cloud
cargo check --package synesis-cloud

# Fix issues immediately
# Re-test to verify

# Add doc comments as you go
```

### Completing a Session

```bash
# 1. Verify all acceptance criteria
cargo test --workspace

# 2. Create completion report
# Create status/PHASE_2_SESSION_2.X_COMPLETE.md

# 3. Update progress
# Mark session complete in roadmap

# 4. Move to next session
# Only when 100% complete
```

---

## Current Status

### Phase 1: Local Kernel ✅ COMPLETE

**Status**: Production-ready, all systems operational

**Completed Milestones**:
- ✅ CLI Foundation (hardware detection, model management)
- ✅ Tripartite Council (Pathos, Logos, Ethos agents)
- ✅ Consensus Engine (multi-round coordination)
- ✅ Privacy Proxy (18 redaction patterns, token vault)
- ✅ Knowledge Vault (SQLite-VSS, RAG integration)
- ✅ Integration & CLI (all commands working)

**Key Improvements Delivered**:
- File watcher auto-indexing (channel-based refactor)
- BGE-Micro embedding infrastructure (trait-based, 384 dimensions)
- Parallel agent execution (25-33% latency reduction)
- Thread safety patterns documented (100% compliance)
- Unified error handling (SynesisError across all crates)
- Metrics infrastructure (Prometheus-compatible)

**Test Results**:
```
synesis-core:    85/85 passing
synesis-knowledge: 28/28 passing
synesis-models:  12/12 passing
synesis-privacy:  37/37 passing
synesis-cli:      7/7 passing
```

### Phase 2: Cloud Mesh 🔄 IN PROGRESS (33% Complete)

**Current Focus**: Session 2.4 - Cloud Escalation Client

**Completed Sessions**:
- ✅ Session 2.0: Planning & Setup
- ✅ Session 2.1: synesis-cloud Crate Setup (11/11 tests)
- ✅ Session 2.2: QUIC Tunnel Core Implementation (27/27 tests)
- ✅ Session 2.3: Heartbeat & Telemetry Enhancement (34/34 tests)

**Remaining Sessions**:
- 🔄 Session 2.4: Cloud Escalation Client (NEXT)
- ⏳ Session 2.5: Message Protocol Definition
- ⏳ Session 2.6: Billing Client Implementation
- ⏳ Session 2.7: Cloudflare Workers Durable Objects
- ⏳ Sessions 2.8-2.12: Advanced features

**Test Results**:
```
synesis-cloud: 68/68 passing (100%)
```

---

## Critical Implementation Patterns

### Thread Safety

**Comprehensive Documentation**: See `THREAD_SAFETY_PATTERNS.md` for complete patterns.

**Core Rules**:
1. **NEVER hold `MutexGuard` across `.await` points** - Causes deadlocks
2. **Use `tokio::sync::Mutex` in async code, NOT `std::sync::Mutex`**
3. **Use `Arc<T>` for shared state, NOT `Rc<T>`** - Rc is not thread-safe
4. **Prefer atomic operations for simple counters** - Lock-free is faster

**Pattern: Arc<tokio::sync::Mutex<T>> for Async Code**
```rust
let vault = Arc::new(tokio::sync::Mutex::new(KnowledgeVault::open(...)?));

// In async task
let vault_lock = vault.lock().await;
let result = sync_operation(&vault_lock);
drop(vault_lock); // CRITICAL: Drop before await
async_operation().await; // Safe now
```

**Pattern: Arc<AtomicU64> for Lock-Free Metrics**
```rust
metrics.queries_total.fetch_add(1, Ordering::Relaxed);
```

### Privacy-First Architecture

The **Redact & Re-inflate** proxy is the cornerstone:
- All sensitive data is tokenized before cloud transmission
- Tokens replaced with UUIDs: `[USER_01]`, `[SECRET_CODE_A]`
- Cloud never sees raw PII or proprietary code
- Local hub re-inflates tokens in cloud responses

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

### Agent System Pattern

All agents implement the `Agent` trait:

```rust
pub trait Agent {
    fn process(&self, input: AgentInput) -> AgentOutput;
}
```

**Communication Flow**:
1. User query → AgentInput
2. AgentInput → Pathos (intent extraction)
3. Pathos output → Logos (RAG retrieval)
4. Logos output → Ethos (verification)
5. All outputs → Consensus Engine (weighted voting)
6. If consensus < threshold → Revision round
7. If consensus ≥ threshold → Return to user

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

---

## Technology Stack

| Layer | Technology | Purpose |
|-------|------------|---------|
| Local Orchestrator | Rust | Zero-latency thread management |
| Model Runtime | llama.cpp / TensorRT | Native inference |
| Privacy Proxy | Rust | High-speed token replacement |
| Local Memory | SQLite-VSS | Portable vector database |
| Cloud Tunnel | QUIC (quinn crate) | Resilient bi-directional stream |
| Cloud State | Durable Objects | Stateful session management |
| Cloud Inference | Workers AI | Serverless GPU access |
| Cloud Memory | Vectorize | Global vector index |
| Storage | R2 | LoRA and asset storage |
| Payments | Stripe Metered | Automated invoicing |

---

## Quick Reference Commands

```bash
# Build project
cargo build --release

# Run all tests
cargo test --workspace

# Check for warnings
cargo clippy --lib --all -- -D warnings

# Format code
cargo fmt --all

# Generate documentation
cargo doc --no-deps --open

# Run specific package tests
cargo test --package synesis-core
cargo test --package synesis-cloud

# Run CLI
cargo run -- synesis status
cargo run -- synesis init
cargo run -- synesis ask "What is the tripartite council?"
```

---

## Documentation

**Primary Documentation**:
- `README.md` - Project overview and quick start for users
- `ARCHITECTURE.md` - System architecture deep dive
- `phases/PHASE_2_DETAILED_ROADMAP.md` - Master Phase 2 implementation plan
- `CONTRIBUTING.md` - Contribution guidelines

**Phase 2 Companion Documents** (`phases/phase2/`):
- `CLOUD_API_SPECIFICATION.md` - API endpoints and contracts
- `DATA_MODELS.md` - All data structures and types
- `QUIC_TUNNEL_GUIDE.md` - QUIC implementation patterns
- `SECURITY_CONSIDERATIONS.md` - Security requirements
- `TESTING_GUIDE.md` - Testing strategies

**Reference Guides**:
- `THREAD_SAFETY_PATTERNS.md` - Complete thread safety patterns
- `ASYNC_PATTERNS_RUST.md` - Async/await best practices
- `TROUBLESHOOTING.md` - Common issues and solutions

**Status Reports** (`status/`):
- `PHASE_2_SESSION_2.3_COMPLETE.md` - Latest session completion report
- `PHASE_1_REFINEMENTS_ALL_COMPLETE.md` - Phase 1 final report

---

## Quality Standards

**Code Quality Gates**:
- ✅ All tests must pass (100% pass rate required)
- ✅ Zero compiler warnings (all library crates)
- ✅ Zero clippy warnings (all library crates)
- ✅ All public APIs documented
- ✅ Thread safety verified (see `THREAD_SAFETY_PATTERNS.md`)

**Before Committing**:
```bash
# Run full test suite
cargo test --workspace

# Check for warnings
cargo clippy --lib --all -- -D warnings

# Format code
cargo fmt --all

# Check documentation
cargo doc --no-deps
```

---

## Cost-Plus Economics

- **Managed Tier**: 3% markup on Cloudflare wholesale costs
- **BYOK Tier**: 30% licensing fee for using Synesis Protocol
- **Knowledge Credits**: Contributors earn credits that offset costs

---

## Contact & Repository

- **GitHub**: https://github.com/SuperInstance/Tripartite1
- **Issues**: https://github.com/SuperInstance/Tripartite1/issues
- **Discussions**: https://github.com/SuperInstance/Tripartite1/discussions

---

*Last Updated: 2026-01-07*
*Current Phase: Phase 2: Cloud Mesh (33% complete)*
*Tests: 250+ passing (100%)*
*Version: v0.2.0*
*Methodology: Ralph Wiggum - Persistent Iterative Development*
