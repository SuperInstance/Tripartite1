# SuperInstance AI - Claude Code Quick Start

## Before You Begin

### Prerequisites Checklist

```bash
# 1. Rust toolchain (latest stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
rustup component add clippy rustfmt

# 2. Claude Code
npm install -g @anthropic-ai/claude-code

# 3. System dependencies (Ubuntu/Debian)
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    cmake \
    clang

# 4. Optional: CUDA for NVIDIA GPUs
# Follow: https://developer.nvidia.com/cuda-downloads

# 5. Verify installations
rustc --version    # Should be 1.75+
claude --version   # Should show version
```

---

## Step 1: Create Project Directory

```bash
# Create the project root
mkdir -p ~/superinstance
cd ~/superinstance

# Copy all documentation files into this directory
# (From the superinstance-project folder you received)
cp -r /path/to/superinstance-project/* .

# Verify structure
ls -la
# Should see: CLAUDE.md, PROJECT_ROADMAP.md, architecture/, phases/, agents/, etc.
```

---

## Step 2: Start Claude Code

```bash
cd ~/superinstance
claude
```

When Claude Code starts, you'll see a prompt. Give it this first command:

---

## Step 3: First Session - Project Initialization

Copy and paste this EXACT prompt:

```
I'm starting development on SuperInstance AI. Please read the following files in order:

1. CLAUDE.md - Your role as orchestrator
2. PROJECT_ROADMAP.md - Timeline and milestones
3. architecture/HIGH_LEVEL.md - System overview
4. phases/PHASE_1_LOCAL_KERNEL.md - Current phase details
5. CLAUDE_CODE_BUILD_GUIDE.md - Session prompts

After reading, confirm:
- You understand the tripartite council (Pathos/Logos/Ethos)
- You understand the privacy-first architecture
- You understand Phase 1.1 goals
- You're ready to begin Session 1: Project Scaffolding

Do NOT start coding yet - just confirm understanding.
```

**Wait for Claude to confirm understanding before proceeding.**

---

## Step 4: Begin Building

After Claude confirms understanding, give it Session 1:

```
Execute Session 1 from CLAUDE_CODE_BUILD_GUIDE.md:

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

Create the basic folder structure from CLAUDE_CODE_BUILD_GUIDE.md.
Don't implement any logic yet - just the scaffolding with placeholder modules.

After creating, run `cargo check` to verify it compiles.
```

---

## Step 5: Session-by-Session Workflow

For each subsequent session:

### Before Starting
```
Read status/BUILD_STATUS.md and status/CHANGELOG.md.
What was completed in the last session? What's next on the roadmap?
```

### Starting a Session
```
Execute Session N from CLAUDE_CODE_BUILD_GUIDE.md.
[Paste the session prompt from the guide]
```

### After Each Session
```
Update status/BUILD_STATUS.md to reflect what was completed.
Add an entry to status/CHANGELOG.md with today's changes.
Run `cargo test` and `cargo clippy` to verify code quality.
```

---

## Key Commands Reference

### Within Claude Code

| Command | Purpose |
|---------|---------|
| `/help` | Show available commands |
| `/clear` | Clear conversation history |
| `/save` | Save current session |
| `/cost` | Show API cost for session |
| `Ctrl+C` | Cancel current operation |
| `Ctrl+D` | Exit Claude Code |

### Useful Prompts

**Check Status:**
```
Show me the current state of BUILD_STATUS.md and summarize what's complete vs remaining.
```

**Resume Work:**
```
I'm resuming work on SuperInstance. Read CHANGELOG.md for recent context, then continue from where we left off.
```

**Debug Issue:**
```
I'm getting this error: [paste error]
Check the relevant source files and fix it.
```

**Run Tests:**
```
Run `cargo test --workspace` and fix any failures.
```

---

## Troubleshooting

### Claude Code Not Starting
```bash
# Reinstall
npm uninstall -g @anthropic-ai/claude-code
npm install -g @anthropic-ai/claude-code

# Check API key
echo $ANTHROPIC_API_KEY
```

### Rust Compilation Errors
```
Show me the error and the relevant source file. Explain what's wrong and fix it.
```

### Session Context Lost
```
I just started a new session. Please read these files to restore context:
1. status/BUILD_STATUS.md
2. status/CHANGELOG.md
3. CLAUDE.md
Then tell me what was last worked on and what should be next.
```

### Out of Memory During Build
```bash
# Limit parallel jobs
export CARGO_BUILD_JOBS=2
cargo build
```

---

## Phase 1 Session Sequence

Complete these in order:

| Session | Focus | Est. Time |
|---------|-------|-----------|
| 1 | Project scaffolding | 30 min |
| 2 | Hardware detection | 1 hr |
| 3 | Model downloader | 1.5 hr |
| 4 | `synesis init` command | 1.5 hr |
| 5 | `synesis status` command | 1 hr |
| 6 | Agent trait definition | 1 hr |
| 7 | Pathos agent | 2 hr |
| 8 | Logos agent | 2 hr |
| 9 | Ethos agent | 2 hr |
| 10 | Consensus engine | 2 hr |
| 11 | Redaction patterns | 1 hr |
| 12 | Token vault | 1 hr |
| 13 | Redactor implementation | 1.5 hr |
| 14 | Privacy pipeline integration | 1 hr |
| 15 | SQLite-VSS setup | 1.5 hr |
| 16 | Embedding pipeline | 2 hr |
| 17 | File watcher | 1 hr |
| 18 | RAG integration | 1.5 hr |
| 19 | Hardware manifest schema | 1 hr |
| 20 | Manifest loader | 1 hr |
| 21 | Integration tests | 2 hr |
| 22 | CLI polish | 1.5 hr |

**Total Phase 1**: ~30 hours of Claude Code sessions

---

## Tips for Effective Sessions

### 1. One Feature Per Session
Don't ask Claude to implement multiple unrelated features. Keep sessions focused.

### 2. Review Before Proceeding
After each session, review the generated code. Ask Claude to explain anything unclear.

### 3. Test Incrementally
Run `cargo check` and `cargo test` after each significant change.

### 4. Use the Agent Docs
When implementing an agent (Pathos/Logos/Ethos), have Claude read the specific agent doc first.

### 5. Keep Status Updated
The status files are your memory between sessions. Keep them current.

---

## Success Criteria for Phase 1

Before moving to Phase 2, verify:

- [ ] `synesis init` completes successfully
- [ ] `synesis status` shows all components
- [ ] `synesis ask "Hello"` returns a response
- [ ] All three agents participate in consensus
- [ ] Privacy redaction works (email/API keys redacted)
- [ ] Knowledge vault indexes documents
- [ ] RAG retrieval returns relevant chunks
- [ ] All tests pass (`cargo test --workspace`)
- [ ] No clippy warnings (`cargo clippy -- -D warnings`)

---

## Next Steps

After completing Phase 1:

1. Read `phases/PHASE_2_CLOUD_MESH.md`
2. Set up Cloudflare account (if not already)
3. Install wrangler CLI: `npm install -g wrangler`
4. Begin Phase 2 sessions

---

## Getting Help

### Within Claude Code
```
I'm stuck on [problem]. Here's what I've tried: [list].
Read the relevant architecture docs and help me solve this.
```

### Check Documentation
- Architecture: `architecture/*.md`
- Current phase: `phases/PHASE_1_LOCAL_KERNEL.md`
- Agent details: `agents/*.md`

### Reset If Needed
```
Something went wrong. Let's start this session over.
Read the current state from status/ files, then show me what needs to be fixed.
```

---

*This guide is part of the SuperInstance AI documentation suite.*
