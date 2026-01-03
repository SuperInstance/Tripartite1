# Phase 1: The Local Kernel

> **Duration**: Months 1-4
> **Goal**: Prove that "Tailoring beats Scaling" with a privacy-first local AI

---

## Phase 1 Overview

Phase 1 establishes the foundational "Kernel" of SuperInstance—the local tripartite council that runs on user hardware. Success means a user can run `synesis ask "..."` and receive a consensus response from three coordinated local agents with full privacy.

## Success Criteria

- [ ] CLI tool installable via single command
- [ ] Three agents participate visibly in every response
- [ ] Privacy proxy catches and redacts PII before any cloud call
- [ ] Local vector database answers queries without cloud
- [ ] Hardware manifests for 3+ platforms contributed by community

---

## Milestone 1.1: CLI Foundation (Weeks 1-4)

### Objective
Create the command-line interface that users interact with daily.

### Deliverables

#### 1.1.1 `synesis init`
```bash
$ synesis init

Detecting hardware...
  ✓ GPU: NVIDIA Jetson Orin Nano (8GB)
  ✓ CUDA: 11.4
  ✓ Architecture: arm64

Loading hardware manifest...
  ✓ Manifest: nvidia-jetson-orin-nano.json
  ✓ Max model: 8B parameters
  ✓ Quantization: 4-bit recommended

Downloading models...
  ✓ Pathos: phi-3-mini-4k (3.8B) [2.1 GB]
  ✓ Logos: llama-3.2-8b-q4 (8B) [4.6 GB]
  ✓ Ethos: phi-3-mini-4k (3.8B) [2.1 GB]

Initializing databases...
  ✓ Created: ~/.synesis/synesis.db
  ✓ Created: ~/.synesis/vectors.db

Configuration saved to: ~/.synesis/config.json

SuperInstance is ready. Run 'synesis ask "Hello"' to start.
```

**Acceptance Criteria:**
- Auto-detects NVIDIA GPU (or falls back to CPU)
- Downloads appropriate quantized models
- Creates SQLite databases
- Generates valid config.json

#### 1.1.2 `synesis ask`
```bash
$ synesis ask "What programming language should I use for this project?"

┌─────────────────────────────────────────────────────────────┐
│ SYNESIS CONSENSUS                                            │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│ [PATHOS] Understanding your intent...                       │
│   → Goal: Language recommendation for project               │
│   → Context: Need project details for specificity          │
│   → Confidence: 0.82                                        │
│                                                              │
│ [LOGOS] Analyzing options...                                │
│   → Checking local project files...                         │
│   → Found: package.json, tsconfig.json                      │
│   → Confidence: 0.91                                        │
│                                                              │
│ [ETHOS] Verifying constraints...                            │
│   → Hardware: Jetson supports all common runtimes           │
│   → No conflicts detected                                   │
│   → Confidence: 0.95                                        │
│                                                              │
│ ═══════════════════════════════════════════════════════════ │
│ CONSENSUS REACHED (0.89)                                    │
│                                                              │
│ Based on your existing TypeScript configuration, I          │
│ recommend continuing with TypeScript. Your project          │
│ already has tsconfig.json set up, and TypeScript offers     │
│ the type safety that helps with larger projects.            │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

**Acceptance Criteria:**
- Shows all three agents' contributions
- Displays confidence scores
- Reaches consensus before responding
- Reads local project context

#### 1.1.3 `synesis status`
```bash
$ synesis status

SuperInstance Hub Status
════════════════════════════════════════════════════════════

Hardware:
  GPU Load:     23%
  GPU Temp:     52°C
  VRAM Used:    4.2 GB / 8.0 GB
  CPU Load:     15%

Agents:
  Pathos:       ● Active (phi-3-mini-4k)
  Logos:        ● Active (llama-3.2-8b-q4)
  Ethos:        ● Active (phi-3-mini-4k)

Storage:
  Vectors:      1,247 embeddings
  Interactions: 89 logged
  Wisdom:       12 blocks

Cloud Status:   ○ Not connected (local-only mode)
```

**Acceptance Criteria:**
- Real-time hardware metrics
- Agent health status
- Storage statistics
- Cloud connection state

### Technical Implementation

**File Structure:**
```
cli/src/commands/
├── init.rs       # Hardware detection, model download
├── ask.rs        # Prompt handling, consensus orchestration
├── status.rs     # System health reporting
└── mod.rs        # Command routing
```

**Key Dependencies:**
```toml
# Cargo.toml
[dependencies]
clap = { version = "4", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
rusqlite = { version = "0.29", features = ["bundled"] }
nvml-wrapper = "0.9"  # NVIDIA monitoring
indicatif = "0.17"    # Progress bars
colored = "2"         # Terminal colors
```

---

## Milestone 1.2: Tripartite Council (Weeks 3-6)

### Objective
Implement the three specialized agents and their consensus mechanism.

### Agent Specifications

#### Pathos Agent (Intent)
- **Model**: phi-3-mini-4k (small, fast)
- **Purpose**: Understand what the user actually wants
- **Output**: A2A Manifest with structured intent

```rust
// cli/src/agents/pathos.rs

pub struct PathosAgent {
    model: LocalModel,
    persona_cache: PersonaCache,
}

impl PathosAgent {
    pub async fn process(&self, prompt: &str, context: &ProjectContext) -> PathosResult {
        let system_prompt = r#"
            You are Pathos, the Intent Agent. Your job is to:
            1. Understand what the user actually wants (their telos)
            2. Identify any implicit constraints
            3. Determine expertise level from communication style
            4. Output a structured A2A Manifest
            
            Respond ONLY with valid JSON matching the A2AManifest schema.
        "#;
        
        let response = self.model.generate(system_prompt, prompt, context).await?;
        let manifest: A2AManifest = serde_json::from_str(&response)?;
        
        PathosResult {
            manifest,
            confidence: self.calculate_confidence(&response),
        }
    }
}
```

#### Logos Agent (Logic)
- **Model**: llama-3.2-8b-q4 (powerful, larger)
- **Purpose**: Build the technical solution
- **Output**: Solution text with sources

```rust
// cli/src/agents/logos.rs

pub struct LogosAgent {
    model: LocalModel,
    rag: VectorStore,
    lora_loader: LoRALoader,
}

impl LogosAgent {
    pub async fn process(&self, manifest: &A2AManifest) -> LogosResult {
        // 1. Retrieve relevant context from vector store
        let context_docs = self.rag.query(&manifest.intent.telos, 5).await?;
        
        // 2. Load project-specific LoRA if available
        if let Some(lora_id) = &manifest.logic_request.required_lora {
            self.lora_loader.load(lora_id).await?;
        }
        
        // 3. Generate solution
        let system_prompt = format!(r#"
            You are Logos, the Logic Agent. Your job is to:
            1. Build a technical solution for the intent
            2. Use the provided context documents
            3. Be specific and actionable
            
            Intent: {}
            Constraints: {:?}
            Context: {}
        "#, manifest.intent.telos, manifest.intent.constraints, 
            context_docs.join("\n---\n"));
        
        let solution = self.model.generate(&system_prompt, "", &[]).await?;
        
        LogosResult {
            solution,
            confidence: self.calculate_confidence(&solution),
            sources: context_docs.iter().map(|d| d.source.clone()).collect(),
        }
    }
}
```

#### Ethos Agent (Truth)
- **Model**: phi-3-mini-4k (fast verification)
- **Purpose**: Verify accuracy and feasibility
- **Output**: Verdict with constraints

```rust
// cli/src/agents/ethos.rs

pub struct EthosAgent {
    model: LocalModel,
    hardware: HardwareManifest,
    fact_checker: FactChecker,
}

impl EthosAgent {
    pub async fn verify(&self, solution: &str, manifest: &A2AManifest) -> EthosResult {
        let mut constraints = Vec::new();
        let mut verdict = Verdict::Approved;
        
        // 1. Hardware constraint check
        if manifest.verification_scope.check_hardware {
            if let Some(hw_issue) = self.check_hardware_constraints(solution).await? {
                constraints.push(hw_issue);
                verdict = Verdict::NeedsRevision;
            }
        }
        
        // 2. Fact check against known sources
        if manifest.verification_scope.check_facts {
            let facts = self.fact_checker.verify(solution).await?;
            for fact in facts.iter().filter(|f| !f.verified) {
                constraints.push(format!("Unverified claim: {}", fact.claim));
                verdict = Verdict::NeedsRevision;
            }
        }
        
        // 3. Safety check
        if manifest.verification_scope.check_safety {
            if let Some(safety_issue) = self.check_safety(solution).await? {
                constraints.push(safety_issue);
                verdict = Verdict::Veto; // Safety issues are vetoes
            }
        }
        
        EthosResult {
            verdict,
            constraints,
            confidence: self.calculate_confidence(&constraints),
            feedback: self.generate_feedback(&constraints),
        }
    }
    
    async fn check_hardware_constraints(&self, solution: &str) -> Option<String> {
        // Check if solution would exceed hardware limits
        if solution.contains("max_tokens=16000") && self.hardware.limits.max_context_tokens < 16000 {
            return Some(format!(
                "Solution requires 16K context but hardware limit is {}",
                self.hardware.limits.max_context_tokens
            ));
        }
        None
    }
}
```

### Consensus Engine

```rust
// cli/src/synapse/consensus.rs

pub struct ConsensusEngine {
    pathos: PathosAgent,
    logos: LogosAgent,
    ethos: EthosAgent,
    config: ConsensusConfig,
}

impl ConsensusEngine {
    pub async fn run(&mut self, prompt: &str) -> Result<ConsensusResult> {
        let context = self.load_project_context().await?;
        
        for round in 0..self.config.max_rounds {
            // Phase 1: Intent
            let pathos_result = self.pathos.process(prompt, &context).await?;
            self.emit_update(AgentUpdate::Pathos(pathos_result.clone())).await;
            
            // Phase 2: Logic
            let logos_result = self.logos.process(&pathos_result.manifest).await?;
            self.emit_update(AgentUpdate::Logos(logos_result.clone())).await;
            
            // Phase 3: Truth
            let ethos_result = self.ethos.verify(&logos_result.solution, &pathos_result.manifest).await?;
            self.emit_update(AgentUpdate::Ethos(ethos_result.clone())).await;
            
            // Calculate consensus
            let score = self.calculate_score(&pathos_result, &logos_result, &ethos_result);
            
            if score >= self.config.threshold {
                return Ok(ConsensusResult::Reached {
                    response: logos_result.solution,
                    score,
                    sources: logos_result.sources,
                });
            }
            
            if ethos_result.verdict == Verdict::Veto {
                return Ok(ConsensusResult::Vetoed {
                    reason: ethos_result.constraints.join(", "),
                });
            }
            
            // Inject feedback for next round
            prompt = format!("{}\n\nPrevious attempt feedback: {}", 
                prompt, ethos_result.feedback);
        }
        
        Ok(ConsensusResult::ArbiterNeeded)
    }
}
```

**Acceptance Criteria:**
- All three agents execute for every prompt
- Consensus score accurately reflects agreement
- Veto power works (Ethos can block unsafe outputs)
- Maximum 3 rounds before escalation

---

## Milestone 1.3: Privacy Proxy (Weeks 5-8)

### Objective
Ensure no sensitive data leaves the local device unredacted.

### Implementation Details

See `architecture/LOW_LEVEL.md` for the full `Redactor` implementation.

**Key Test Cases:**

```rust
#[test]
fn test_email_redaction() {
    let mut redactor = Redactor::new(TokenVault::new_in_memory());
    let input = "Contact john.doe@example.com for details";
    let output = redactor.redact(input);
    
    assert!(!output.contains("john.doe@example.com"));
    assert!(output.contains("[EMAIL_"));
}

#[test]
fn test_api_key_redaction() {
    let mut redactor = Redactor::new(TokenVault::new_in_memory());
    let input = "Use API key sk_live_abc123def456ghi789jkl012";
    let output = redactor.redact(input);
    
    assert!(!output.contains("sk_live_"));
    assert!(output.contains("[SECRET_"));
}

#[test]
fn test_roundtrip() {
    let mut redactor = Redactor::new(TokenVault::new_in_memory());
    let original = "John's email is john@test.com and his key is sk_test_123";
    
    let redacted = redactor.redact(original);
    let restored = redactor.reinflate(&redacted);
    
    assert_eq!(original, restored);
}
```

**Acceptance Criteria:**
- Catches: emails, phone numbers, SSNs, API keys, file paths, IP addresses
- Token vault persists across sessions
- Round-trip (redact → reinflate) is lossless
- No PII appears in cloud-bound traffic

---

## Milestone 1.4: Knowledge Vault (Weeks 6-10)

### Objective
Build local memory that reduces cloud dependency over time.

### Implementation

```rust
// cli/src/storage/vectors.rs

pub struct VectorStore {
    db: rusqlite::Connection,
    embedder: LocalEmbedder,  // Using BGE-Micro-v1.5
}

impl VectorStore {
    pub async fn index_file(&mut self, path: &Path) -> Result<usize> {
        let content = std::fs::read_to_string(path)?;
        let chunks = self.chunk_document(&content);
        
        let mut count = 0;
        for chunk in chunks {
            let embedding = self.embedder.embed(&chunk.text).await?;
            
            self.db.execute(
                "INSERT INTO vectors (id, embedding, text, source, created_at) 
                 VALUES (?, ?, ?, ?, ?)",
                params![
                    Uuid::new_v4().to_string(),
                    embedding.as_bytes(),
                    chunk.text,
                    path.to_str().unwrap(),
                    chrono::Utc::now().timestamp()
                ],
            )?;
            count += 1;
        }
        
        Ok(count)
    }
    
    pub async fn query(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let query_embedding = self.embedder.embed(query).await?;
        
        // SQLite-VSS similarity search
        let results = self.db.prepare(
            "SELECT text, source, distance 
             FROM vectors 
             WHERE embedding MATCH ? 
             ORDER BY distance 
             LIMIT ?"
        )?.query_map(
            params![query_embedding.as_bytes(), limit],
            |row| Ok(SearchResult {
                text: row.get(0)?,
                source: row.get(1)?,
                distance: row.get(2)?,
            })
        )?.collect::<Result<Vec<_>, _>>()?;
        
        Ok(results)
    }
    
    fn chunk_document(&self, content: &str) -> Vec<Chunk> {
        // Split into ~500 token chunks with 50 token overlap
        // ... implementation
    }
}
```

### File Watcher

```rust
// cli/src/storage/watcher.rs

pub struct ProjectWatcher {
    watcher: notify::RecommendedWatcher,
    vector_store: Arc<Mutex<VectorStore>>,
}

impl ProjectWatcher {
    pub fn watch(&mut self, project_dir: &Path) -> Result<()> {
        self.watcher.watch(project_dir, RecursiveMode::Recursive)?;
        
        // Handle events
        loop {
            match self.rx.recv()? {
                Event { kind: EventKind::Create(_) | EventKind::Modify(_), paths, .. } => {
                    for path in paths {
                        if self.should_index(&path) {
                            let mut store = self.vector_store.lock().unwrap();
                            store.index_file(&path).await?;
                        }
                    }
                }
                _ => {}
            }
        }
    }
    
    fn should_index(&self, path: &Path) -> bool {
        let ext = path.extension().and_then(|e| e.to_str());
        matches!(ext, Some("rs" | "py" | "ts" | "js" | "md" | "txt" | "json"))
    }
}
```

**Acceptance Criteria:**
- Auto-indexes project files on change
- Queries return relevant context in <100ms
- Vector DB size stays manageable (<1GB for typical project)
- Nightly synthesis job consolidates learnings

---

## Milestone 1.5: Hardware Manifests (Weeks 8-12)

### Objective
Enable community-driven hardware optimization.

### Manifest Repository Structure

```
manifests/
├── CONTRIBUTING.md          # How to add a manifest
├── schema.json              # JSON Schema for validation
├── nvidia/
│   ├── jetson-orin-nano.json
│   ├── jetson-orin-nx.json
│   └── rtx-4090.json
├── amd/
│   └── radeon-rx-7900.json
├── intel/
│   └── lunar-lake.json
└── apple/
    └── m3-pro.json
```

### Validation Script

```python
# scripts/validate_manifest.py

import json
import jsonschema
import sys

def validate(manifest_path):
    with open('schema.json') as f:
        schema = json.load(f)
    
    with open(manifest_path) as f:
        manifest = json.load(f)
    
    try:
        jsonschema.validate(manifest, schema)
        print(f"✓ {manifest_path} is valid")
        return True
    except jsonschema.ValidationError as e:
        print(f"✗ {manifest_path} failed validation: {e.message}")
        return False

if __name__ == '__main__':
    sys.exit(0 if validate(sys.argv[1]) else 1)
```

**Acceptance Criteria:**
- Schema enforces all required fields
- CI validates manifests on PR
- At least 3 manifests from different vendors
- Community contribution guide is clear

---

## Phase 1 Definition of Done

### Functional Requirements
- [ ] `synesis init` works on Jetson Orin and x86 Linux
- [ ] `synesis ask` returns consensus responses
- [ ] Privacy proxy catches all PII patterns
- [ ] Vector store indexes and retrieves accurately
- [ ] Hardware manifests load and constrain correctly

### Non-Functional Requirements
- [ ] Response latency < 3s for simple queries
- [ ] Memory usage < 12GB total
- [ ] Test coverage > 80%
- [ ] Documentation complete

### Community Requirements
- [ ] Open source repo with MIT license
- [ ] README with quick start guide
- [ ] Contributing guide for manifests
- [ ] Discord or discussion forum active

---

## Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| llama.cpp compatibility issues | Medium | High | Maintain fallback to pure Python inference |
| SQLite-VSS performance | Low | Medium | Benchmark early, optimize or switch to FAISS |
| Model download failures | Medium | Low | Multiple mirrors, resume support |
| CUDA version conflicts | High | Medium | Document supported versions, provide containers |

---

*Next Phase: [PHASE_2_CLOUD_MESH.md](./PHASE_2_CLOUD_MESH.md)*
