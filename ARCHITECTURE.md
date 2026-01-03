# SuperInstance AI - Architecture Documentation

**Version**: 0.1.0 (Phase 1 Complete)
**Last Updated**: 2026-01-02
**Status**: Production-Ready (Local Kernel)

---

## Table of Contents

1. [Overview](#overview)
2. [Design Principles](#design-principles)
3. [System Architecture](#system-architecture)
4. [Component Deep Dive](#component-deep-dive)
5. [Data Flow](#data-flow)
6. [Patterns and Conventions](#patterns-and-conventions)
7. [Technical Decisions](#technical-decisions)
8. [Known Limitations](#known-limitations)
9. [Future Enhancements](#future-enhancements)

---

## Overview

SuperInstance AI is a **privacy-first, local-first AI assistant** that uses a tripartite council of specialized agents to process user queries. The system is designed to keep computation local whenever possible, with intelligent cloud escalation for complex tasks.

### Key Architectural Features

- **Tripartite Consensus**: Three specialized agents (Pathos, Logos, Ethos) must agree before responding
- **Privacy-First**: All sensitive data is tokenized before any cloud transmission
- **Local-First**: Preference for local processing with optional cloud escalation
- **Knowledge Vault**: Local vector database for RAG (Retrieval-Augmented Generation)
- **Hardware Awareness**: Automatic detection and optimization for available hardware

---

## Design Principles

### 1. Privacy by Default

All user data stays on the local device unless explicitly escalated to cloud. Sensitive information (emails, API keys, PII) is automatically redacted before any cloud processing.

**Implementation**: The `Redactor` component uses regex patterns to identify sensitive data and replaces it with tokens stored in a local `TokenVault`.

### 2. Consensus Before Action

No response is emitted until all three agents reach consensus (default threshold: 0.85 confidence). This ensures:
- Intent is correctly understood (Pathos)
- Reasoning is sound (Logos)
- Response is safe and feasible (Ethos)

**Implementation**: The `ConsensusEngine` orchestrates multi-round voting with weighted aggregation.

### 3. Hardware Awareness

The system automatically detects available hardware (CPU, GPU, RAM) and selects appropriate models and configurations.

**Implementation**: `HardwareManifest` system in `synesis-models` crate.

### 4. Modular Crate Design

The codebase is split into focused crates, each with a single responsibility:
- `synesis-core`: Agent orchestration and consensus
- `synesis-privacy`: Redaction and token vault
- `synesis-knowledge`: Vector database and RAG
- `synesis-models`: Model management and hardware detection
- `synesis-cli`: Command-line interface

---

## System Architecture

### High-Level Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                      CLI Interface                          │
│                   (synesis-cli crate)                       │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                  Privacy Proxy Layer                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   Redact    │──│ Token Vault │──│     Reinflate       │  │
│  │  (Patterns) │  │  (SQLite)   │  │    (Reinflation)    │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│               Tripartite Council (Core)                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   Pathos    │  │    Logos    │  │       Ethos         │  │
│  │  (Intent)   │──│  (Reasoning)│──│   (Verification)    │  │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────────────┘  │
│         │                │                 │                 │
│         └────────────────┼─────────────────┘                 │
│                          ▼                                   │
│                 ┌──────────────────┐                         │
│                 │ Consensus Engine │                         │
│                 │  (Orchestrator)  │                         │
│                 └──────────────────┘                         │
└─────────────────────────┬───────────────────────────────────┘
                          │
          ┌───────────────┴───────────────┐
          ▼                               ▼
┌──────────────────────┐     ┌─────────────────────────┐
│   Knowledge Vault    │     │   Model Runtime         │
│  (SQLite-VSS)        │     │  (llama.cpp / TensorRT) │
│  - Vector Search     │     │  - Local Inference      │
│  - Document Index    │     │  - Hardware Offload     │
└──────────────────────┘     └─────────────────────────┘
```

---

## Component Deep Dive

### 1. synesis-core - Agent Orchestration

**Responsibility**: Coordinate the tripartite council and reach consensus.

#### Key Types

- **`Agent` trait**: Interface all agents implement
  ```rust
  pub trait Agent: Send + Sync {
      fn name(&self) -> &str;
      fn role(&self) -> &str;
      async fn process(&self, input: AgentInput) -> CoreResult<AgentOutput>;
      fn is_ready(&self) -> bool;
      fn model(&self) -> &str;
  }
  ```

- **`ConsensusEngine`**: Orchestrates multi-round consensus
  - Runs agents in sequence: Pathos → Logos → Ethos
  - Calculates weighted aggregate confidence
  - Handles veto power from Ethos
  - Supports up to 3 revision rounds with feedback

- **`AgentInput` / `AgentOutput`**: Message passing types
  - `AgentInput`: Contains query, context, and A2A manifest
  - `AgentOutput`: Contains response, confidence, reasoning, and sources

#### Agents

| Agent | Role | Model | Key Capability |
|-------|------|-------|----------------|
| **Pathos** | Intent Extraction | phi-3-mini | Disambiguation, constraint parsing, user profiling |
| **Logos** | Reasoning + RAG | llama-3.2-8b | Knowledge retrieval, solution synthesis, citation |
| **Ethos** | Verification | phi-3-mini-4k | Safety checking, hardware validation, veto power |

#### Integration Points

- **Privacy**: Consensus engine accepts optional `Redactor` for prompt sanitization
- **Knowledge**: Logos agent uses `KnowledgeVault` for RAG queries
- **Models**: All agents load models through `ModelRegistry`

### 2. synesis-privacy - Privacy Proxy

**Responsibility**: Redact sensitive information and manage token vault.

#### Key Components

- **`Pattern`**: Regex-based redaction patterns
  - 18 built-in patterns (email, phone, API keys, credit cards, etc.)
  - Priority ordering (more specific patterns first)
  - Enable/disable per pattern

- **`TokenVault`**: SQLite-backed token storage
  - Global counters per pattern category (not per-document)
  - Session-based isolation
  - Thread-safe access via Arc<Mutex<>> (at application level)

- **`Redactor`**: Orchestrates redaction and reinflation
  - Redacts sensitive text before cloud processing
  - Re-inflates tokens in responses with original values
  - Provides statistics on patterns detected

#### Pattern Priority

1. **SK API Key** (most specific: `sk-` prefix with dash/underscore support)
2. **GitHub Token** (`ghp_`, `gho_`, `ghu_`, etc.)
3. **AWS Keys** (access key ID, secret access key)
4. **Email Addresses**
5. **Phone Numbers**
6. **IPv6 Addresses** (with `::` compression support)
7. **IPv4 Addresses**
8. **URLs**
9. **File Paths**
10. **Credit Cards** (Luhn algorithm validation)
11. **SSN** (US Social Security Numbers)

#### Thread Safety Pattern

**Critical Pattern**: The vault uses `Arc<Mutex<Vault>>` at the **application level**, not inside the library struct. This is because:

1. `rusqlite::Connection` uses `RefCell` internally (not Send/Sync)
2. Cannot hold `MutexGuard` across `await` points
3. Application manages locking scope

**Example**:
```rust
// Correct: Arc<Mutex<TokenVault>> at CLI level
let vault = Arc::new(Mutex::new(TokenVault::in_memory()?));

// Incorrect: Arc<Mutex<TokenVault>> inside Redactor
// This would cause borrowing issues across await points
```

### 3. synesis-knowledge - Knowledge Vault

**Responsibility**: Vector database for document storage and retrieval.

#### Architecture

```
KnowledgeVault (SQLite-VSS)
    ├── Documents (text metadata)
    ├── Chunks (split documents)
    └── Embeddings (vector search via vss0)
         │
         ├── DocumentIndexer (chunking + embeddings)
         ├── FileWatcher (notify-based auto-indexing)
         └── Search (vector + keyword retrieval)
```

#### Key Types

- **`KnowledgeVault`**: Main interface for vault operations
  - Thread-safe: Uses interior mutability at application level
  - Methods: `add_document`, `search`, `get_document_stats`

- **`DocumentIndexer`**: Splits documents into chunks
  - Supports multiple chunking strategies:
    - `Paragraph`: Split by paragraphs with overlap
    - `Sentence`: Split by sentences (NLP-based)
    - `Fixed`: Fixed-size chunks (N tokens)
  - Generates embeddings (currently SHA256 placeholder, will be BGE-Micro)
  - **Known Issue**: Holds `&'a KnowledgeVault` reference incompatible with async callbacks

- **`FileWatcher`**: Monitors directories for changes
  - Uses `notify` crate for cross-platform file watching
  - Debounce delay (default: 1 second)
  - **Known Issue**: Auto-indexing disabled due to DocumentIndexer lifetime issue

- **`Search`**: Vector similarity search
  - SQLite-VSS for fast approximate nearest neighbor (ANN) search
  - Top-K retrieval (default: 5 chunks)
  - Multi-factor relevance scoring:
    - Cosine similarity (base score)
    - Recency boost: `1.0 + (0.1 * days_since_update).min(0.5)`
    - Source quality multiplier: code (1.0) > docs (0.9) > notes (0.8)

#### RAG Integration with Logos

Logos agent retrieves relevant context before answering:

1. Extract key terms from A2A manifest
2. Generate query embedding
3. Search vault for top-K chunks
4. Calculate relevance scores (similarity + recency + source type)
5. Inject formatted context into prompt with citation instructions

**Citation Format**: `[SOURCE: /path/to/file.rs]`

### 4. synesis-models - Model Management

**Responsibility**: Hardware detection, model downloading, and runtime management.

#### Hardware Detection

**`HardwareManifest`**: Comprehensive hardware profile
```rust
pub struct HardwareManifest {
    pub cpu: CpuInfo,
    pub gpus: Vec<GpuInfo>,
    pub ram_bytes: u64,
    pub disk_bytes: u64,
    pub platform: PlatformInfo,
}
```

**Detection Methods**:
- **CPU**: Uses `sysinfo` crate for model, cores, threads, features (AVX, NEON)
- **GPU**:
  - NVIDIA: Parse `nvidia-smi` output for VRAM and CUDA version
  - AMD: Parse `rocm-smi` output
  - Apple Silicon: Unified memory (system RAM = VRAM)
  - Intel: Parse `sycl-ls` output
- **RAM**: `sysinfo` crate for total memory
- **Disk**: `df` command (Unix) or defaults (Windows)
- **Platform**: `/etc/os-release` (Linux), `sw_vers` (macOS), `ver` (Windows)

#### Model Registry

**`ModelRegistry`**: Singleton for model lifecycle
- Download models from URLs with progress tracking
- Verify SHA256 checksums
- Cache in `~/.superinstance/models/`
- Load/unload models based on hardware constraints

#### Model Format Support

- **GGUF**: llama.cpp format (primary)
- **SafeTensors**: Hugging Face format (future)
- **ONNX**: Cross-platform format (future)

### 5. synesis-cli - Command-Line Interface

**Responsibility**: User interaction and command orchestration.

#### Commands

| Command | Purpose | Integration Points |
|---------|---------|-------------------|
| `init` | Initialize system, download models | Hardware detection, ModelRegistry |
| `status` | Display system status | HardwareManifest, ModelRegistry |
| `ask` | Main interaction with agents | ConsensusEngine, Redactor, KnowledgeVault |
| `model` | Manage local models | ModelRegistry |
| `knowledge` | Manage knowledge vault | KnowledgeVault, FileWatcher |
| `manifest` | Manage hardware manifests | HardwareManifest loader |
| `cloud` | Cloud connection (Phase 2) | Durable Objects (future) |
| `config` | Configuration management | TOML config file |

#### Thread Safety Pattern

The CLI manages all shared state using `Arc<Mutex<T>>`:

```rust
struct AppState {
    vault: Arc<Mutex<KnowledgeVault>>,
    vault_ref: Arc<Mutex<KnowledgeVault>>,  // Workaround for lifetime issues
    models: Arc<Mutex<ModelRegistry>>,
    redactor: Arc<Mutex<Redactor>>,
}
```

**Why Two Vaults?** Due to the DocumentIndexer lifetime issue, we maintain two Arc references to the same vault for different use cases.

---

## Data Flow

### Query Processing Flow

```
1. User Input (CLI)
       │
       ▼
2. Privacy Proxy (Redact)
       │
       ├─→ Detect sensitive patterns
       ├─→ Replace with tokens (e.g., [EMAIL_01])
       └─→ Store originals in TokenVault
       │
       ▼
3. Tripartite Council
       │
       ├─→ Round 1:
       │    ├─→ Pathos: Extract intent and constraints
       │    ├─→ Logos: Retrieve context, generate solution
       │    └─→ Ethos: Verify safety and feasibility
       │
       ├─→ Evaluate Consensus
       │    │
       │    ├─→ Consensus Reached? → Return response
       │    ├─→ Vetoed? → Return veto reason
       │    └─→ Below threshold? → Round 2 with feedback
       │
       └─→ Rounds 2-3: Retry with feedback from lowest-confidence agent
       │
       ▼
4. Privacy Proxy (Reinflate)
       │
       ├─→ Replace tokens with original values
       └─→ Return clean response to user
       │
       ▼
5. Output (CLI)
```

### Knowledge Indexing Flow

```
1. File Added to Watch Directory
       │
       ▼
2. Document Indexer
       │
       ├─→ Read file content
       ├─→ Detect document type (code/markdown/text)
       ├─→ Split into chunks (paragraph/sentence/fixed)
       └─→ Generate embeddings (SHA256 placeholder)
       │
       ▼
3. Knowledge Vault
       │
       ├─→ Store document metadata
       ├─→ Store chunks with embeddings
       └─→ Build VSS index
       │
       ▼
4. Search Ready
```

---

## Patterns and Conventions

### Error Handling

All crates use `thiserror` for typed errors:

```rust
#[derive(Debug, thiserror::Error)]
pub enum KnowledgeError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Document not found: {0}")]
    NotFound(String),
}
```

**Result Type**: Each crate defines its own `Result` type alias:
```rust
pub type KnowledgeResult<T> = Result<T, KnowledgeError>;
```

### Async/Await Patterns

**Tokio Runtime**: All async code uses `tokio` runtime:
```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Async code here
}
```

**Critical Pattern**: Cannot hold `MutexGuard` across `await` points:

```rust
// ❌ WRONG: Will cause panic or deadlock
let guard = vault.lock().await;
async_function().await;  // Holding guard across await!
drop(guard);

// ✅ CORRECT: Release lock before await
{
    let guard = vault.lock().await;
    // Do synchronous work
}
async_function().await;  // Lock released
```

### Agent Communication

**A2A Manifest**: Agents communicate via shared manifest:
```rust
pub struct A2AManifest {
    pub original_query: String,
    pub intent: Option<String>,
    pub constraints: Vec<Constraint>,
    pub context: HashMap<String, serde_json::Value>,
    pub sources: Vec<Source>,
    pub round: u8,
    pub feedback: Option<String>,
}
```

**Flow**:
1. Pathos populates `intent` and `constraints`
2. Logos adds `sources` and retrieval context
3. Ethos validates and can veto via constraints
4. Consensus engine aggregates votes

### Configuration

**TOML-based**: All configuration in `~/.superinstance/config.toml`:

```toml
[agents.pathos]
model = "phi-3-mini"
temperature = 0.7

[agents.logos]
model = "llama-3.2-8b"

[consensus]
threshold = 0.85
max_rounds = 3

[privacy]
redact_emails = true
redact_api_keys = true
```

---

## Technical Decisions

### Why SQLite-VSS for Vector Database?

**Decision**: Use SQLite-VSS instead of dedicated vector DB (Milvus, Qdrant, etc.)

**Rationale**:
- Zero-dependency (embedded in SQLite)
- Portable single-file database
- Sufficient performance for local use case
- SQL-based queries (familiar interface)
- ACID guarantees

**Trade-off**: Slower than dedicated vector DBs for large datasets (>1M vectors)

### Why Three Agents Instead of One?

**Decision**: Tripartite council (Pathos, Logos, Ethos) instead of single LLM

**Rationale**:
- Specialization: Each agent optimizes for its domain
- Safety: Ethos has veto power for dangerous responses
- Quality: Consensus reduces hallucinations
- Explainability: Separate reasoning traces

**Trade-off**: Higher latency (3 model invocations) and cost

### Why Rust for Local Kernel?

**Decision**: Rust instead of Python or Go

**Rationale**:
- Zero-cost abstractions (performance critical for local inference)
- Memory safety (no GC pauses)
- Concurrency (async/await for efficient I/O)
- Interop: FFI bindings to llama.cpp (C++)
- Type system: Catches bugs at compile time

**Trade-off**: Steeper learning curve, longer compile times

### Why Placeholder Embeddings?

**Decision**: Use SHA256 hash as placeholder instead of real embeddings

**Rationale**:
- Phase 1 focus: Architecture and integration
- BGE-Micro integration requires model runtime (llama.cpp bindings)
- SHA256 provides deterministic "embeddings" for testing
- Vector search still functional (just not semantic)

**Future**: Replace with BGE-Micro or all-MiniLM-L6-v2 in Phase 2

---

## Known Limitations

### 1. File Watcher Auto-Indexing Disabled

**Issue**: `DocumentIndexer` holds `&'a KnowledgeVault` reference, which is incompatible with async callbacks in `FileWatcher`.

**Impact**: Users must manually trigger reindexing with `synesis knowledge index <path>`

**Mitigation Plan**:
- Refactor `DocumentIndexer` to accept `Arc<Mutex<KnowledgeVault>>`
- Or use channel-based message passing instead of direct callbacks

### 2. Placeholder Embeddings

**Issue**: Current embeddings are SHA256 hashes, not semantic vectors.

**Impact**: RAG retrieval is not semantic (based on hash similarity, not meaning)

**Mitigation Plan**:
- Integrate BGE-Micro (1.7MB embedding model)
- Requires llama.cpp backend for model loading
- Planned for Phase 2

### 3. No Cloud Integration Yet

**Issue**: Cloud escalation (Cloudflare Workers, Durable Objects) not implemented.

**Impact**: All processing is local; cannot leverage cloud models for complex tasks

**Mitigation Plan**: Phase 2 will implement:
- QUIC tunnel to cloud
- Durable Objects for session state
- Workers AI integration (Claude, GPT-4)

### 4. Compiler Warnings

**Issue**: 19 compiler warnings (unused imports, dead code)

**Impact**: Non-critical, but indicates incomplete API surface

**Mitigation Plan**:
- Run `cargo fix` to auto-fix unused imports
- Add `#[allow(dead_code)]` for future APIs
- Clean up before v0.2.0 release

### 5. Limited Hardware Support

**Issue**: GPU detection primarily supports NVIDIA and AMD

**Impact**: Intel GPU and mobile GPU support is basic

**Mitigation Plan**:
- Add Intel OneAPI bindings
- Add Metal (Apple) bindings
- Mobile GPU support (Phase 4)

---

## Future Enhancements

### Short-Term (Phase 2)

1. **Real Embeddings**: Replace SHA256 with BGE-Micro
2. **Cloud Escalation**: Integrate Cloudflare Workers AI
3. **Streaming Responses**: Real-time token streaming from models
4. **File Watcher Fix**: Resolve lifetime issues for auto-indexing

### Medium-Term (Phase 3)

1. **LoRA Support**: Load custom LoRA adapters for domain expertise
2. **Knowledge Marketplace**: Share and sell trained LoRAs
3. **Multi-Modal**: Image and audio understanding
4. **Federated Learning**: Privacy-preserving model updates

### Long-Term (Phase 4)

1. **SDKs**: Python, JavaScript, Go SDKs
2. **Enterprise Features**: SSO, audit logs, RBAC
3. **Distributed Mode**: Swarm coordination across multiple instances
4. **Model Fine-Tuning**: On-device training with user data

---

## Performance Considerations

### Benchmarks (Phase 1)

| Operation | Latency | Throughput |
|-----------|---------|------------|
| Consensus (single round) | ~2-5s | - |
| Redaction | ~10-50ms | ~1000 req/s |
| Vector Search (1K chunks) | ~50-100ms | ~10 queries/s |
| Document Indexing | ~100-500ms/doc | ~2 docs/s |

### Optimization Targets

1. **Parallel Agent Execution**: Run Pathos/Logos in parallel (currently sequential)
2. **Embedding Cache**: Cache query embeddings for repeated queries
3. **Batch Indexing**: Process multiple documents in parallel
4. **Quantization**: Use 4-bit quantization for smaller models

---

## Security Considerations

### Threat Model

1. **Data Exfiltration**: Prevented by local-first architecture
2. **Prompt Injection**: Mitigated by Ethos verification
3. **Model Poisoning**: Mitigated by model verification (SHA256)
4. **Token Vault Leakage**: SQLite encryption at rest (future)

### Best Practices

1. **Never log redacted content**: Always use original values in logs
2. **Validate model checksums**: SHA256 verification before loading
3. **Sandbox model execution**: Run models in separate process (future)
4. **Audit trail**: Log all consensus outcomes and vetoes

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Code Style

- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Document all public APIs with rustdoc
- Write tests for all new features (target: 90% coverage)

---

## References

- [Project Roadmap](PROJECT_ROADMAP.md)
- [Build Guide](CLAUDE_CODE_BUILD_GUIDE.md)
- [Low-Level Architecture](architecture/LOW_LEVEL.md)
- [Medium-Level Architecture](architecture/MEDIUM_LEVEL.md)
- [High-Level Architecture](architecture/HIGH_LEVEL.md)

---

*Last Updated: 2026-01-02*
*Phase 1 Status: Complete (122/122 tests passing)*
*Next Milestone: Phase 2 - Cloud Mesh Integration*
