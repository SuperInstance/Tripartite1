# SuperInstance AI - Claude Code Build Guide

## Quick Start

```bash
# 1. Navigate to project folder
cd ~/superinstance-project

# 2. Start Claude Code
claude

# 3. First command to Claude Code:
"Read CLAUDE.md and PROJECT_ROADMAP.md to understand the project, then confirm you're ready to begin Phase 1.1"
```

---

## Project Overview

**SuperInstance AI** is a tripartite agentic system with:
- **Three specialized agents** (Pathos/Logos/Ethos) that reach consensus before responding
- **Privacy-preserving proxy** that redacts PII before cloud transmission
- **Local-first processing** with intelligent cloud escalation
- **Cost-plus pricing** (3% managed, 30% BYOK)

**Core Innovation**: "An AI that knows when to stay local, when to escalate to cloud, and keeps your secrets safe either way."

---

## Repository Structure to Create

```
superinstance/
├── Cargo.toml                    # Workspace root
├── README.md                     
├── .github/
│   └── workflows/
│       ├── ci.yml                # Rust CI
│       └── manifest-validation.yml
├── crates/
│   ├── synesis-cli/              # CLI binary
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       └── commands/
│   │           ├── mod.rs
│   │           ├── init.rs
│   │           ├── ask.rs
│   │           └── status.rs
│   ├── synesis-core/             # Core orchestration
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── consensus.rs
│   │       ├── agents/
│   │       │   ├── mod.rs
│   │       │   ├── pathos.rs
│   │       │   ├── logos.rs
│   │       │   └── ethos.rs
│   │       └── config.rs
│   ├── synesis-privacy/          # Privacy proxy
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── redactor.rs
│   │       ├── patterns.rs
│   │       └── vault.rs
│   ├── synesis-models/           # Model management
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── loader.rs
│   │       ├── inference.rs
│   │       └── hardware.rs
│   └── synesis-knowledge/        # Vector DB & RAG
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── embeddings.rs
│           ├── sqlite_vss.rs
│           └── retrieval.rs
├── manifests/                    # Hardware manifests
│   ├── schema.json
│   ├── jetson-orin-nano.json
│   ├── rtx-4050-laptop.json
│   └── apple-m2.json
├── cloud/                        # Cloudflare Workers
│   ├── wrangler.toml
│   └── src/
│       ├── index.ts
│       ├── billing-ledger.ts
│       └── cloud-synapse.ts
├── docs/                         # Public documentation
│   ├── quickstart.md
│   ├── architecture.md
│   └── api-reference.md
└── tests/
    ├── integration/
    └── fixtures/
```

---

## Build Phases & Prompts

### PHASE 1.1: CLI Foundation (Weeks 1-4)

#### Session 1: Project Scaffolding
```
Initialize a Rust workspace for SuperInstance AI with the following crates:
- synesis-cli (binary)
- synesis-core (library) 
- synesis-privacy (library)
- synesis-models (library)
- synesis-knowledge (library)

Use Rust 2021 edition. Add these workspace dependencies:
- tokio = { version = "1", features = ["full"] }
- clap = { version = "4", features = ["derive"] }
- serde = { version = "1", features = ["derive"] }
- serde_json = "1"
- anyhow = "1"
- thiserror = "1"
- tracing = "0.1"
- tracing-subscriber = "0.3"

Create the basic folder structure. Don't implement any logic yet - just the scaffolding with placeholder modules.
```

#### Session 2: Hardware Detection
```
Read architecture/LOW_LEVEL.md section on Hardware Detection.

Implement hardware detection in synesis-models/src/hardware.rs:
1. Detect GPU type (NVIDIA CUDA, AMD ROCm, Apple Metal, Intel Arc, None)
2. Detect available VRAM
3. Detect CPU cores and available RAM
4. Detect NPU presence (Qualcomm, Intel, Apple ANE)

Create a HardwareManifest struct that captures:
- device_id: String
- gpu_type: GpuType enum
- vram_mb: u64
- ram_mb: u64
- cpu_cores: u32
- npu_available: bool
- compute_capability: Option<String>

For NVIDIA detection, shell out to nvidia-smi. For others, use appropriate system calls.
Return a comprehensive error if detection fails.
```

#### Session 3: Model Downloader
```
Read phases/PHASE_1_LOCAL_KERNEL.md section 1.1.

Implement model downloading in synesis-models/src/loader.rs:

1. Create a ModelRegistry that knows about:
   - phi-3-mini-4k-instruct (Q4_K_M) - 2.4GB - for Pathos/Ethos
   - llama-3.2-8b-instruct (Q4_K_M) - 4.7GB - for Logos
   - bge-micro-v1.5 - 50MB - for embeddings

2. For each model, store:
   - name, size_bytes, quantization, huggingface_repo, filename, sha256

3. Implement download_model() that:
   - Checks if model exists at ~/.synesis/models/
   - Downloads from HuggingFace with progress bar
   - Verifies SHA256 checksum
   - Returns path to downloaded file

4. Use reqwest for HTTP, indicatif for progress bars.

Do NOT implement inference yet - just downloading and verification.
```

#### Session 4: CLI Init Command
```
Read phases/PHASE_1_LOCAL_KERNEL.md milestone 1.1.

Implement `synesis init` command in synesis-cli/src/commands/init.rs:

The command should:
1. Print welcome banner with version
2. Run hardware detection and display results
3. Recommend models based on available VRAM:
   - <4GB: phi-3 only, warn about limitations
   - 4-8GB: phi-3 + llama-3.2-8b
   - >8GB: full model suite
4. Ask user to confirm download (y/n)
5. Download selected models with progress
6. Initialize SQLite database at ~/.synesis/knowledge.db
7. Create config file at ~/.synesis/config.toml
8. Print success message with next steps

Use dialoguer for interactive prompts.
Handle Ctrl+C gracefully during downloads.
```

#### Session 5: CLI Status Command
```
Implement `synesis status` command in synesis-cli/src/commands/status.rs:

Display a formatted status report:

┌─────────────────────────────────────────┐
│         SYNESIS STATUS                  │
├─────────────────────────────────────────┤
│ Hardware                                │
│   GPU: NVIDIA RTX 4050 (6GB VRAM)       │
│   RAM: 32GB available                   │
│   NPU: Not detected                     │
├─────────────────────────────────────────┤
│ Models                                  │
│   ✓ phi-3-mini-4k (2.4GB) - loaded      │
│   ✓ llama-3.2-8b (4.7GB) - ready        │
│   ✓ bge-micro-v1.5 (50MB) - loaded      │
├─────────────────────────────────────────┤
│ Knowledge Vault                         │
│   Documents: 47                         │
│   Embeddings: 1,234                     │
│   Last sync: 2 hours ago                │
├─────────────────────────────────────────┤
│ Agents                                  │
│   Pathos: idle                          │
│   Logos: idle                           │
│   Ethos: idle                           │
└─────────────────────────────────────────┘

Use comfy-table or tabled crate for formatting.
Read state from ~/.synesis/ directory.
```

---

### PHASE 1.2: Tripartite Council (Weeks 3-6)

#### Session 6: Agent Trait Definition
```
Read agents/PATHOS_AGENT.md, agents/LOGOS_AGENT.md, agents/ETHOS_AGENT.md.

In synesis-core/src/agents/mod.rs, define:

1. The A2A Manifest structure:
```rust
pub struct A2AManifest {
    pub session_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub telos: String,           // What user wants
    pub persona: Persona,        // User's expertise level
    pub constraints: Vec<Constraint>,
    pub domain: Domain,
    pub urgency: Urgency,
    pub confidence: f32,
}
```

2. The Agent trait:
```rust
#[async_trait]
pub trait Agent {
    fn name(&self) -> &str;
    async fn process(&self, input: AgentInput) -> Result<AgentOutput>;
    fn confidence(&self) -> f32;
}
```

3. AgentInput and AgentOutput enums that handle the different input/output types for each agent.

4. The ConsensusVote structure for voting.

Don't implement the agents yet - just the trait and data structures.
```

#### Session 7: Pathos Agent Implementation
```
Read agents/PATHOS_AGENT.md completely.

Implement Pathos in synesis-core/src/agents/pathos.rs:

1. Load phi-3-mini model via llama-cpp-rs bindings
2. Create the intent extraction prompt from the agent doc
3. Process user input to extract:
   - telos (what they want)
   - persona (beginner/intermediate/expert)
   - constraints (explicit + implicit)
   - domain classification
   - urgency level

4. Calculate confidence using the formula in the agent doc:
   - Penalty for short prompts (<5 words): -0.15
   - Penalty for missing constraints on "generate": -0.10
   - Bonus for clear domain detection: +0.05

5. Return A2AManifest

Use llama-cpp-rs crate for inference. Parse JSON from model output.
Handle malformed model outputs gracefully with defaults.
```

#### Session 8: Logos Agent Implementation
```
Read agents/LOGOS_AGENT.md completely.

Implement Logos in synesis-core/src/agents/logos.rs:

1. Load llama-3.2-8b model
2. Accept A2AManifest as input
3. Perform RAG retrieval (stub for now - return empty results)
4. Select appropriate LoRA adapter (stub - use base model)
5. Generate solution using the synthesis prompt from agent doc
6. Track sources used in generation
7. Calculate confidence based on:
   - RAG hit quality
   - Domain match
   - Solution completeness

Return LogosSolution with:
- solution_text: String
- sources: Vec<Source>
- confidence: f32
- tokens_used: u32
```

#### Session 9: Ethos Agent Implementation
```
Read agents/ETHOS_AGENT.md completely.

Implement Ethos in synesis-core/src/agents/ethos.rs:

1. Load phi-3-mini model (same as Pathos)
2. Accept LogosSolution + A2AManifest as input
3. Run verification checks:
   - Hardware constraint validation
   - Safety pattern scanning
   - Fact-checking against local knowledge
   - Code quality checks (if code present)

4. Implement VETO scenarios (always block):
   - Recursive deletion patterns
   - Credential exposure
   - Untrusted URL execution
   - System file modification
   - Thermal limit violations

5. Return EthosVerdict:
   - verdict: Approved | NeedsRevision | Veto
   - feedback: String (for revision rounds)
   - constraints_violated: Vec<String>
   - confidence: f32
```

#### Session 10: Consensus Engine
```
Read architecture/MEDIUM_LEVEL.md section on Consensus Mechanism.

Implement consensus in synesis-core/src/consensus.rs:

```rust
pub struct ConsensusEngine {
    pathos: PathosAgent,
    logos: LogosAgent,
    ethos: EthosAgent,
    threshold: f32,  // 0.85 default
    max_rounds: u32, // 3 default
}

impl ConsensusEngine {
    pub async fn run(&mut self, prompt: &str) -> ConsensusResult {
        // 1. Run Pathos to get A2AManifest
        // 2. Run Logos to get solution
        // 3. Run Ethos to verify
        // 4. Calculate aggregate confidence
        // 5. If below threshold, inject feedback and retry
        // 6. After max_rounds, return ArbiterNeeded
    }
}
```

Aggregate confidence formula:
score = (pathos.confidence * 0.25) + (logos.confidence * 0.45) + (ethos.confidence * 0.30)

Log each round with tracing for debugging.
```

---

### PHASE 1.3: Privacy Proxy (Weeks 5-8)

#### Session 11: Redaction Patterns
```
Read architecture/LOW_LEVEL.md section on Privacy Proxy.

In synesis-privacy/src/patterns.rs, implement regex patterns for:

1. EMAIL: Standard email pattern
2. PHONE: US/international formats
3. SSN: XXX-XX-XXXX pattern
4. API_KEY: Common formats (sk-*, ghp_*, etc.)
5. FILE_PATH: /home/*, C:\Users\*, ~/*, etc.
6. IP_ADDRESS: IPv4 and IPv6
7. CREDIT_CARD: 16-digit patterns with common separators
8. AWS_KEY: AKIA* patterns
9. PRIVATE_KEY: -----BEGIN * PRIVATE KEY-----

Create a RedactionPattern struct with:
- category: RedactionCategory enum
- pattern: Regex
- priority: u8 (higher = process first)

Order matters: API keys before emails (some API keys contain @ symbols).
```

#### Session 12: Token Vault
```
Implement the token vault in synesis-privacy/src/vault.rs:

1. Create SQLite table for token storage:
```sql
CREATE TABLE tokens (
    id INTEGER PRIMARY KEY,
    token TEXT UNIQUE NOT NULL,     -- [EMAIL_0001]
    category TEXT NOT NULL,          -- EMAIL
    original TEXT NOT NULL,          -- user@example.com
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    session_id TEXT NOT NULL
);
```

2. Implement TokenVault struct:
```rust
impl TokenVault {
    pub fn new(db_path: &Path) -> Result<Self>;
    pub fn store(&mut self, category: &str, original: &str, session_id: &str) -> String;
    pub fn retrieve(&self, token: &str) -> Option<String>;
    pub fn clear_session(&mut self, session_id: &str) -> Result<()>;
}
```

3. Token format: [CATEGORY_NNNN] where NNNN is zero-padded counter per category per session.

4. Add index on session_id for efficient cleanup.
```

#### Session 13: Redactor Implementation
```
Implement the full redactor in synesis-privacy/src/redactor.rs:

```rust
pub struct Redactor {
    vault: TokenVault,
    patterns: Vec<RedactionPattern>,
}

impl Redactor {
    pub fn new(vault: TokenVault) -> Self;
    
    pub fn redact(&mut self, text: &str, session_id: &str) -> RedactedText {
        // Process patterns in priority order
        // For each match: store in vault, replace with token
        // Return RedactedText with token count per category
    }
    
    pub fn reinflate(&self, text: &str) -> String {
        // Find all [CATEGORY_NNNN] tokens
        // Look up each in vault
        // Replace with original value
        // Return restored text
    }
    
    pub fn get_stats(&self, session_id: &str) -> RedactionStats;
}
```

Add comprehensive tests:
- Redact then reinflate returns original
- Multiple same-category items get unique tokens
- Nested patterns handled correctly
- Empty string handling
- Unicode handling
```

#### Session 14: Integration with Consensus
```
Integrate privacy proxy into the consensus engine:

1. Modify ConsensusEngine to accept a Redactor
2. Before Pathos processes: redact the prompt
3. All agents work with redacted text
4. After Ethos approves: reinflate the final response
5. Log redaction stats (count per category, NOT original values)

Update synesis-cli ask command:
- Create session_id for each interaction
- Initialize redactor with session
- Pass through consensus engine
- Reinflate response before display
- Clear session tokens after response

Add --show-redactions flag for debugging that shows token mappings.
```

---

### PHASE 1.4: Knowledge Vault (Weeks 6-10)

#### Session 15: SQLite-VSS Setup
```
Read architecture/LOW_LEVEL.md section on Knowledge Vault.

In synesis-knowledge/src/sqlite_vss.rs:

1. Set up SQLite with sqlite-vss extension:
```sql
CREATE TABLE documents (
    id INTEGER PRIMARY KEY,
    path TEXT UNIQUE,
    content TEXT,
    doc_type TEXT,
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    checksum TEXT
);

CREATE TABLE chunks (
    id INTEGER PRIMARY KEY,
    document_id INTEGER REFERENCES documents(id),
    content TEXT,
    start_offset INTEGER,
    end_offset INTEGER,
    embedding BLOB,
    metadata JSON
);

CREATE VIRTUAL TABLE chunk_embeddings USING vss0(
    embedding(384)  -- BGE-micro dimension
);
```

2. Implement KnowledgeVault struct with:
- add_document(path, content, doc_type)
- search(query_embedding, top_k) -> Vec<ChunkResult>
- delete_document(path)
- get_stats() -> VaultStats
```

#### Session 16: Embedding Pipeline
```
Implement embeddings in synesis-knowledge/src/embeddings.rs:

1. Load BGE-Micro-v1.5 model (50MB)
2. Implement chunking strategies:
   - Code files: chunk by function/class using tree-sitter
   - Markdown: chunk by headings
   - Default: 500-word sliding window, 50-word overlap

3. Create EmbeddingPipeline:
```rust
impl EmbeddingPipeline {
    pub fn new(model_path: &Path) -> Result<Self>;
    pub fn embed(&self, text: &str) -> Vec<f32>;
    pub fn embed_batch(&self, texts: &[&str]) -> Vec<Vec<f32>>;
    pub fn chunk_document(&self, content: &str, doc_type: DocType) -> Vec<Chunk>;
}
```

4. For code chunking, use tree-sitter with rust, python, javascript grammars.
```

#### Session 17: File Watcher
```
Implement file watching for auto-indexing:

1. Use notify crate for filesystem events
2. Watch ~/.synesis/watched/ directory
3. On file change/create:
   - Compute checksum
   - If changed, re-chunk and re-embed
   - Update database

4. Create CLI command `synesis watch <path>`:
   - Add path to watch list
   - Immediately index existing files
   - Start background watcher

5. Handle common file types:
   - .rs, .py, .js, .ts, .go -> code chunking
   - .md, .txt -> markdown/text chunking  
   - .pdf -> extract text first (use pdf-extract crate)

6. Ignore patterns: .git, node_modules, target, __pycache__
```

#### Session 18: RAG Integration
```
Connect Knowledge Vault to Logos agent:

1. In Logos agent, before synthesis:
   - Extract key terms from A2AManifest
   - Embed the query
   - Search vault for top 5 relevant chunks
   - Include chunks in context

2. Implement retrieval scoring:
   relevance_score = cosine_similarity * recency_boost * source_quality
   
   Where:
   - recency_boost = 1.0 + (0.1 * days_since_update).min(0.5)
   - source_quality = 1.0 for code, 0.9 for docs, 0.8 for notes

3. Format retrieved chunks for LLM context:
```
[SOURCE: path/to/file.rs:42-58]
```rust
fn example() {
    // relevant code
}
```
[/SOURCE]
```

4. Update confidence calculation to factor in RAG quality.
```

---

### PHASE 1.5: Hardware Manifests (Weeks 8-12)

#### Session 19: Manifest Schema
```
Create manifest schema and examples in manifests/:

1. Create schema.json (JSON Schema):
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["device", "compute", "memory", "models"],
  "properties": {
    "device": {
      "type": "object",
      "properties": {
        "name": {"type": "string"},
        "vendor": {"type": "string"},
        "architecture": {"type": "string"}
      }
    },
    "compute": {
      "type": "object", 
      "properties": {
        "gpu_type": {"enum": ["cuda", "rocm", "metal", "vulkan", "none"]},
        "compute_capability": {"type": "string"},
        "tensor_cores": {"type": "boolean"},
        "npu_type": {"type": "string"}
      }
    },
    "memory": {
      "type": "object",
      "properties": {
        "vram_mb": {"type": "integer"},
        "ram_recommended_mb": {"type": "integer"},
        "unified_memory": {"type": "boolean"}
      }
    },
    "models": {
      "type": "object",
      "properties": {
        "max_context": {"type": "integer"},
        "recommended": {"type": "array"},
        "supported_quantizations": {"type": "array"}
      }
    },
    "thermal": {
      "type": "object",
      "properties": {
        "tdp_watts": {"type": "integer"},
        "throttle_temp_c": {"type": "integer"}
      }
    }
  }
}
```

2. Create jetson-orin-nano.json, rtx-4050-laptop.json, apple-m2.json examples.
```

#### Session 20: Manifest Loader
```
Implement manifest loading and validation:

1. In synesis-models/src/hardware.rs, add:
```rust
pub struct HardwareManifest {
    pub device: DeviceInfo,
    pub compute: ComputeInfo,
    pub memory: MemoryInfo,
    pub models: ModelRecommendations,
    pub thermal: ThermalLimits,
}

impl HardwareManifest {
    pub fn load(path: &Path) -> Result<Self>;
    pub fn validate(&self) -> Result<()>;
    pub fn detect_and_load() -> Result<Self>;
}
```

2. detect_and_load() should:
   - Run hardware detection
   - Find matching manifest in ~/.synesis/manifests/
   - Fall back to generic manifest if no match
   - Merge detected values with manifest recommendations

3. Add `synesis manifest` subcommand:
   - `synesis manifest show` - display current
   - `synesis manifest validate <path>` - validate JSON
   - `synesis manifest install <path>` - copy to manifests dir
```

---

### End of Phase 1 - Integration Testing

#### Session 21: Full Integration
```
Create end-to-end integration tests:

1. Test: synesis init -> status -> ask "Hello world"
   - Verify all models download
   - Verify consensus runs
   - Verify response generated

2. Test: Privacy redaction round-trip
   - Input with email, API key, file path
   - Verify redacted before agents
   - Verify reinflated in response

3. Test: Knowledge vault workflow
   - Add document
   - Ask question about document
   - Verify RAG retrieval in response

4. Test: Hardware constraint enforcement
   - Mock low-VRAM scenario
   - Verify Ethos blocks oversized model request

5. Performance benchmarks:
   - Time to first token < 500ms
   - Full consensus round < 2s
   - Privacy proxy overhead < 10ms

Create tests/integration/ with these scenarios.
Use cargo-nextest for parallel test execution.
```

#### Session 22: CLI Polish
```
Final CLI polish before Phase 2:

1. Add `synesis ask` interactive mode:
   - REPL loop with history
   - Show agent activity in real-time
   - Ctrl+C graceful exit

2. Add global flags:
   - --verbose / -v: Show debug output
   - --config <path>: Custom config file
   - --offline: Never attempt cloud
   - --json: Output as JSON

3. Add `synesis config` subcommand:
   - Show current config
   - Set individual values
   - Reset to defaults

4. Improve error messages:
   - Colored output (use owo-colors)
   - Suggestions for common errors
   - Link to docs for complex issues

5. Add shell completions:
   - Generate for bash, zsh, fish
   - Include in `synesis completions <shell>`
```

---

## Additional Prompts for Common Scenarios

### Debugging Consensus Failures
```
The consensus engine is failing to reach agreement after 3 rounds.

Debug by:
1. Adding detailed tracing to each agent's process() method
2. Logging the confidence scores at each round
3. Logging Ethos feedback that's being injected
4. Check if agents are seeing the feedback correctly

Show me the trace output format and how to interpret stuck consensus.
```

### Optimizing Model Loading
```
Model loading is taking too long on startup.

Implement lazy loading:
1. Don't load models until first use
2. Keep models in memory after first load
3. Add --preload flag to force eager loading
4. Add model unloading for memory pressure

Use tokio::sync::OnceCell for lazy initialization.
```

### Adding New PII Pattern
```
I need to add redaction for custom pattern: employee IDs in format EMP-XXXXX.

1. Add new RedactionCategory::EmployeeId
2. Add pattern to patterns.rs with appropriate priority
3. Add tests for the new pattern
4. Update RedactionStats to track this category

Show the complete diff for this change.
```

### Debugging RAG Retrieval
```
RAG is returning irrelevant chunks for queries.

Debug by:
1. Logging the query embedding
2. Logging top 10 results with scores
3. Showing the relevance calculation breakdown
4. Adding a --debug-rag flag to ask command

Also consider:
- Is chunking too aggressive?
- Is the embedding model appropriate?
- Are we using the right similarity metric?
```

---

## Development Tips

### Running Tests
```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p synesis-privacy

# Run integration tests
cargo test --test '*'

# Run with output
cargo test -- --nocapture
```

### Debugging
```bash
# Enable debug logging
RUST_LOG=debug synesis ask "test"

# Enable specific module logging
RUST_LOG=synesis_core::consensus=trace synesis ask "test"

# Profile with flamegraph
cargo flamegraph --bin synesis -- ask "test"
```

### Code Quality
```bash
# Format
cargo fmt --all

# Lint
cargo clippy --workspace -- -D warnings

# Check for unused deps
cargo +nightly udeps

# Security audit
cargo audit
```

---

## Common Issues & Solutions

### Issue: Model download fails
```
Solution: Check network, try --mirror flag for alternate source.
If behind proxy, set HTTPS_PROXY environment variable.
```

### Issue: GPU not detected
```
Solution: 
1. Verify CUDA/ROCm drivers installed
2. Check nvidia-smi / rocm-smi works
3. Ensure user in video group
4. Try --force-cpu flag to continue without GPU
```

### Issue: SQLite-VSS extension not found
```
Solution:
1. Install sqlite-vss: cargo install sqlite-vss-cli
2. Or build from source with --features bundled
3. Check LD_LIBRARY_PATH includes extension directory
```

### Issue: Out of memory during inference
```
Solution:
1. Use smaller quantization (Q4_K_M -> Q4_0)
2. Reduce context length in config
3. Enable memory-mapped loading
4. Close other applications
```

---

## Next Phase Preview

After Phase 1 is complete, Phase 2 introduces:
- Cloud Bridge (QUIC tunnel to Cloudflare)
- Billing Ledger (Durable Objects)
- LoRA Hot-Swap
- Collaborator System

Read phases/PHASE_2_CLOUD_MESH.md when ready to proceed.
