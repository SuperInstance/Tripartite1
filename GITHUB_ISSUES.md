# GitHub Issues for SuperInstance AI v0.1.0

**Purpose**: Pre-formatted issues for known limitations and improvements
**Usage**: Copy these templates when creating issues on GitHub
**Repository**: https://github.com/SuperInstance/Tripartite1

---

## Priority 1: Critical (Blocking)

### Issue #1: File Watcher Auto-Indexing Disabled

**Title**: File watcher auto-indexing disabled due to DocumentIndexer lifetime issue

**Priority**: High
**Status**: Known Issue
**Milestone**: Phase 2

**Description**:
The `FileWatcher` component cannot automatically index modified files because `DocumentIndexer` holds a `&'a KnowledgeVault` reference, which is incompatible with async callbacks in Rust.

**Current Behavior**:
- `synesis knowledge watch` command exists but auto-indexing is disabled
- Users must manually run `synesis knowledge index <path>` to update vault

**Expected Behavior**:
- File watcher should automatically index new/modified files
- Debounce file changes by 1 second before indexing
- Support recursive directory watching

**Workaround**:
```bash
# Manual reindexing
synesis knowledge index --watch ~/Projects/

# Or use cron job
*/5 * * * * synesis knowledge index ~/Projects/
```

**Proposed Solutions**:
1. **Option A**: Refactor `DocumentIndexer` to accept `Arc<Mutex<KnowledgeVault>>`
2. **Option B**: Use channel-based message passing (tokio::sync::mpsc)
3. **Option C**: Use tokio::spawn with owned state (not borrowed)

**Code References**:
- `crates/synesis-knowledge/src/indexer.rs:25-35`
- `crates/synesis-knowledge/src/watcher.rs:40-60`
- `ARCHITECTURE.md` - Section: Known Limitations

**Labels**: `bug`, `async`, `lifetime`, `Phase-2`

---

### Issue #2: Placeholder Embeddings Instead of Semantic Search

**Title**: Knowledge vault uses SHA256 hashes instead of semantic embeddings

**Priority**: High
**Status**: Known Limitation
**Milestone**: Phase 2

**Description**:
The RAG system currently uses SHA256 hashes as "embeddings" for vector similarity search. This provides deterministic results but not semantic understanding.

**Current Behavior**:
- Query embeddings are SHA256(query)
- Document embeddings are SHA256(chunk)
- Vector search works but is not semantic (keyword-like)

**Expected Behavior**:
- Use BGE-Micro (1.7MB) or all-MiniLM-L6-v2 for real embeddings
- Semantic similarity search (understands meaning, not just keywords)
- Better relevance scoring for RAG queries

**Impact**:
- RAG retrieval quality is lower than expected
- Synonyms and paraphrases don't match
- Domain-specific terms don't cluster correctly

**Proposed Solution**:
1. Integrate BGE-Micro embedding model via llama.cpp
2. Replace SHA256 placeholder with real embeddings
3. Re-test RAG quality with semantic search
4. Benchmark retrieval accuracy

**Dependencies**:
- llama.cpp backend for embedding model inference
- Phase 2: Cloud integration for fallback to larger models

**Code References**:
- `crates/synesis-knowledge/src/indexer.rs:80-95`
- `crates/synesis-core/src/agents/logos.rs:180-220`

**Labels**: `enhancement`, `RAG`, `embeddings`, `Phase-2`, `machine-learning`

---

## Priority 2: Important (Quality of Life)

### Issue #3: Compiler Warnings (19 warnings)

**Title**: Clean up 19 compiler warnings (unused imports, dead code)

**Priority**: Medium
**Status**: Non-Critical
**Milestone**: v0.2.0

**Description**:
The codebase has 19 compiler warnings, mostly unused imports and dead code for future APIs.

**Warnings Breakdown**:
- Unused imports: ~12 warnings
- Unused variables: ~4 warnings
- Dead code (future APIs): ~3 warnings

**Current Output**:
```
warning: unused import: std::collections::HashMap
  --> crates/synesis-core/src/agents/logos.rs:10:5
   |
10 | use std::collections::HashMap;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^
```

**Expected Behavior**:
- Zero compiler warnings in release builds
- Clean `cargo build --release` output

**Proposed Solution**:
1. Run `cargo fix --allow-dirty` to auto-fix unused imports
2. Add `#[allow(dead_code)]` for intentional future APIs
3. Remove truly dead code or document why it's needed
4. Add CI check to fail on warnings before merge

**Code References**:
- All crates (run `cargo build --release 2>&1 | grep warning`)

**Labels**: `cleanup`, `compiler-warnings`, `good-first-issue`

---

### Issue #4: Sequential Agent Execution (Slow)

**Title**: Agents run sequentially instead of in parallel

**Priority**: Medium
**Status**: Enhancement
**Milestone**: Phase 2

**Description**:
The three agents (Pathos, Logos, Ethos) run sequentially: Pathos → Logos → Ethos. This increases latency, especially since Pathos and Logos could run in parallel for some queries.

**Current Behavior**:
```rust
let pathos_response = self.pathos.process(input).await?;
let logos_response = self.logos.process(input).await?;
let ethos_response = self.ethos.process(input).await?;
```

**Expected Behavior**:
- Run Pathos and Logos in parallel (where independent)
- Reduce overall latency from ~5s to ~3s for single-round consensus
- Use `tokio::join!` for concurrent execution

**Challenge**:
- Logos depends on Pathos's intent extraction
- Need to determine when agents are independent
- May need two-phase execution (parallel then sequential)

**Proposed Solution**:
1. Analyze dependencies between agents
2. Run Pathos and partial Logos in parallel
3. Merge Pathos intent into Logos input
4. Run Ethos verification
5. Benchmark latency improvement

**Code References**:
- `crates/synesis-core/src/consensus/mod.rs:240-310`

**Labels**: `performance`, `optimization`, `async`, `Phase-2`

---

### Issue #5: No Streaming Responses

**Title**: CLI waits for full response before displaying (no token streaming)

**Priority**: Medium
**Status**: Enhancement
**Milestone**: Phase 2

**Description**:
Currently, the CLI waits for the complete agent response before displaying it to the user. This feels sluggish, especially for long responses.

**Current Behavior**:
```bash
$ synesis ask "Explain quantum computing"
# ... waits 5 seconds ...
# Then displays full response at once
```

**Expected Behavior**:
```bash
$ synesis ask "Explain quantum computing"
# Immediately starts streaming tokens:
Quantum computing is a type of computation that...
# ... continues streaming ...
```

**Challenge**:
- Need streaming interface in model runtime (llama.cpp)
- Consensus engine needs to stream partial results
- Privacy re-inflation must handle streaming tokens
- Terminal display needs to handle partial updates

**Proposed Solution**:
1. Add streaming support to model runtime
2. Modify consensus engine to stream tokens as they arrive
3. Handle token reinflation in streaming mode
4. Update CLI display to show streaming output
5. Add `--no-stream` flag for batch mode

**Dependencies**:
- llama.cpp streaming API
- Phase 2: Better async channel handling

**Code References**:
- `crates/synesis-cli/src/commands/ask.rs`
- `crates/synesis-models/src/runtime.rs` (future)

**Labels**: `enhancement`, `UX`, `streaming`, `Phase-2`

---

## Priority 3: Nice to Have

### Issue #6: Limited GPU Support

**Title**: GPU detection and support primarily for NVIDIA and AMD

**Priority**: Low
**Status**: Enhancement
**Milestone**: Phase 4

**Description**:
Hardware detection works well for NVIDIA and AMD GPUs, but Intel GPU and mobile GPU support is basic or missing.

**Current Support**:
- ✅ NVIDIA: Full (nvidia-smi parsing)
- ✅ AMD: Full (rocm-smi parsing)
- ⚠️ Intel: Basic (sycl-ls parsing)
- ✅ Apple Silicon: Full (unified memory)
- ❌ Mobile (Android/iOS): Not supported
- ❌ Integrated GPUs: Limited detection

**Proposed Enhancements**:
1. Intel OneAPI integration for Arc GPUs
2. Metal (Apple) bindings for better performance
3. Vulkan-based fallback for other GPUs
4. Mobile GPU support (Phase 4)

**Code References**:
- `crates/synesis-models/src/hardware.rs:120-200`

**Labels**: `hardware`, `GPU`, `enhancement`, `Phase-4`

---

### Issue #7: No Configuration Validation

**Title**: Config file errors not caught until runtime

**Priority**: Low
**Status**: Enhancement
**Milestone**: v0.2.0

**Description**:
Invalid configuration in `~/.superinstance/config.toml` is not validated until the specific command is run.

**Current Behavior**:
```bash
# User sets invalid model name
synesis config set agents.logos.model "nonexistent-model"
# Later...
synesis ask "test"
# Error: Model 'nonexistent-model' not found
```

**Expected Behavior**:
```bash
synesis config set agents.logos.model "nonexistent-model"
# Warning: Model 'nonexistent-model' not found in registry
# Did you mean 'phi-3-mini'? [y/N]
```

**Proposed Solution**:
1. Add `synesis config validate` command
2. Validate model names against registry
3. Validate numeric ranges (threshold: 0.0-1.0)
4. Validate file paths exist
5. Auto-suggest corrections for typos

**Code References**:
- `crates/synesis-cli/src/commands/config.rs`

**Labels**: `enhancement`, `config`, `validation`, `good-first-issue`

---

### Issue #8: No Progress Bars for Long Operations

**Title**: Model download and document indexing show no progress

**Priority**: Low
**Status**: Enhancement
**Milestone**: v0.2.0

**Description**:
Long-running operations like model downloads and document indexing don't show progress bars, making it unclear if the system is working.

**Current Behavior**:
```bash
$ synesis model download llama-3.2-8b
# ... no output for 5 minutes ...
# Done!
```

**Expected Behavior**:
```bash
$ synesis model download llama-3.2-8b
Downloading llama-3.2-8b (4.2 GB)...
[████████████████████████████░░] 85% (3.6 GB / 4.2 GB)
ETA: 45s
```

**Proposed Solution**:
1. Use `indicatif` crate for progress bars
2. Add progress to:
   - Model downloads
   - Document indexing
   - Vault rebuilding
   - Consensus rounds (show current round/agent)

**Dependencies**:
- `indicatif` crate (already in dependencies)
- Model download progress tracking

**Code References**:
- `crates/synesis-cli/src/display.rs` (has `StreamingDisplay` but unused)

**Labels**: `enhancement`, `UX`, `progress-bar`, `good-first-issue`

---

## Priority 4: Documentation

### Issue #9: User Guides Missing

**Title**: Need comprehensive user guides for common workflows

**Priority**: Low
**Status**: Documentation
**Milestone**: v0.2.0

**Description**:
Current documentation is developer-focused. Need user-friendly guides for non-technical users.

**Needed Guides**:
1. "Getting Started" tutorial (5-minute walkthrough)
2. "Setting Up Your First Knowledge Vault" guide
3. "Optimizing Performance for Your Hardware" guide
4. "Privacy and Security Best Practices" guide
5. "Troubleshooting Common Issues" guide (partially done)

**Proposed Solution**:
1. Create `docs/user-guide/` directory
2. Write Markdown guides with screenshots
3. Add video tutorials (YouTube)
4. Create interactive examples

**Code References**:
- `README.md` (high-level overview)
- `TROUBLESHOOTING.md` (technical issues)

**Labels**: `documentation`, `good-first-issue`, `help-wanted`

---

### Issue #10: API Documentation Missing Examples

**Title**: Rustdoc API docs lack usage examples

**Priority**: Low
**Status**: Documentation
**Milestone**: v0.2.0

**Description**:
Public API functions lack rustdoc examples, making it hard for developers to use the library.

**Current State**:
```rust
/// Adds a document to the vault
pub fn add_document(&mut self, doc: Document) -> Result<()> {
    // ...
}
```

**Expected State**:
```rust
/// Adds a document to the vault
///
/// # Examples
///
/// ```
/// use synesis_knowledge::{KnowledgeVault, Document};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut vault = KnowledgeVault::in_memory()?;
/// let doc = Document::new("test.md", "# Hello\nWorld");
/// vault.add_document(doc)?;
/// # Ok(())
/// # }
/// ```
pub fn add_document(&mut self, doc: Document) -> Result<()> {
    // ...
}
```

**Proposed Solution**:
1. Add examples to all public APIs
2. Run `cargo test --doc` to verify examples compile
3. Include `trybuild` tests for error cases

**Code References**:
- All public APIs in `synesis-core`, `synesis-knowledge`, `synesis-privacy`

**Labels**: `documentation`, `good-first-issue`, `rustdoc`

---

## How to Use These Templates

### Creating a New Issue

1. Go to: https://github.com/SuperInstance/Tripartite1/issues/new
2. Copy the template from above
3. Fill in additional details:
   - **Steps to reproduce**: If bug
   - **Expected behavior**: What should happen
   - **Actual behavior**: What actually happens
   - **Environment**: OS, Rust version, etc.
4. Add relevant labels (see templates)
5. Submit issue

---

### Label Reference

| Label | Usage |
|-------|-------|
| `bug` | Software bug |
| `enhancement` | Feature request |
| `good-first-issue` | Suitable for new contributors |
| `help-wanted` | Community help needed |
| `documentation` | Docs improvement |
| `performance` | Performance issue |
| `Phase-2` | Planned for Phase 2 |
| `Phase-4` | Planned for Phase 4 |
| `async` | Async/await related |
| `RAG` | Knowledge vault / retrieval |
| `UX` | User experience |
| `cleanup` | Code cleanup |

---

### Issue Lifecycle

1. **Open**: Issue created
2. **Triaged**: Maintainer reviews and assigns priority
3. **Assigned**: Contributor assigned to work on it
4. **In Progress**: Work being done
5. **PR Created**: Pull request submitted
6. **Review**: Code review in progress
7. **Merged**: PR merged, issue closed
8. **Verified**: Tested in release, issue verified as fixed

---

*Last Updated: 2026-01-02*
*Repository: https://github.com/SuperInstance/Tripartite1*
*Version: 0.1.0*
