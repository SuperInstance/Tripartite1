# Getting Started with SuperInstance AI

**Time**: 10 minutes
**Difficulty**: Beginner
**Prerequisites**:
- Rust 1.75+ installed ([install rustup](https://rustup.rs/))
- C compiler and OpenSSL headers
- 8GB RAM minimum (16GB recommended)

---

## What You'll Learn

By the end of this tutorial, you will:
- ✅ Install SuperInstance AI
- ✅ Initialize the system
- ✅ Run your first AI query
- ✅ Understand the basic components

---

## Step 1: Install Prerequisites

### Option A: Ubuntu/Debian

```bash
# Install Rust (via rustup)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install C compiler and OpenSSL
sudo apt-get update
sudo apt-get install build-essential libssl-dev pkg-config
```

### Option B: macOS

```bash
# Install Rust (via rustup)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Xcode command line tools
xcode-select --install
```

### Option C: Windows

1. Install [Rustup](https://rustup.rs/)
2. Install [Build Tools for Visual Studio](https://visualstudio.microsoft.com/visual-cpp-build-tools/)

### Verify Installation

```bash
rustc --version  # Should show rustc 1.75.0 or later
cargo --version  # Should show cargo 1.75.0 or later
```

---

## Step 2: Clone and Build

```bash
# Clone repository
git clone https://github.com/SuperInstance/Tripartite1.git
cd Tripartite1

# Build release binary (takes ~2-5 minutes)
cargo build --release

# Verify build
./target/release/synesis --version
# Expected output: synesis 0.2.0
```

**Tip**: Add `./target/release` to your PATH for easier access:

```bash
# Temporary (current session only)
export PATH="$PATH:$(pwd)/target/release"

# Permanent (add to ~/.bashrc or ~/.zshrc)
echo 'export PATH="$PATH:/path/to/Tripartite1/target/release"' >> ~/.bashrc
source ~/.bashrc
```

---

## Step 3: Initialize SuperInstance

```bash
# Detect hardware and create config
synesis init

# Expected output:
# ✅ Detected hardware:
#    CPU: 16 cores (x86_64)
#    RAM: 32 GB
#    GPU: NVIDIA RTX 4090 (24 GB VRAM)
#    Disk: 500 GB available
#
# ✅ Created configuration: ~/.superinstance/config.toml
# ✅ Initialized database: ~/.superinstance/vault.db
```

This command:
1. Detects your hardware capabilities
2. Creates configuration file
3. Initializes local databases
4. Sets up the knowledge vault

### Check Your Status

```bash
synesis status

# Expected output:
# ┌─────────────┬──────────────────┐
# │ Component   │ Status           │
# ├─────────────┼──────────────────┤
# │ CPU         │ 16 cores @ 3.5GHz│
# │ GPU         │ NVIDIA RTX 4090  │
# │ RAM         │ 32 GB            │
# │ Disk        │ 500 GB available │
# │ Platform    │ Linux x86_64     │
# │ Model       │ Not loaded       │
# └─────────────┴──────────────────┘
```

---

## Step 4: Run Your First Query

```bash
synesis ask "What is the capital of France?"

# Expected output:
# 🤔 Pathos (Intent): User wants factual information about French geography
# 🧠 Logos (Logic): Retrieving knowledge about capital cities...
# ✅ Ethos (Truth): Verifying factual accuracy...
#
# ✅ Consensus reached (0.95 confidence)
#
# The capital of France is Paris.
#
# ---
# Tokens: 156 | Time: 2.3s | Agents: 3/3 agreed
```

**What just happened?**

1. **Pathos** analyzed your intent: "factual query about geography"
2. **Logos** formulated a response using its knowledge
3. **Ethos** verified the information was accurate
4. **Consensus Engine** confirmed all agents agreed (95% confidence)
5. **Response** delivered to you

---

## Step 5: Explore the Knowledge Vault

The knowledge vault lets you add your own documents for RAG (Retrieval-Augmented Generation).

```bash
# Add documents to the knowledge vault
synesis knowledge add ~/Documents/my-project/

# Expected output:
# 📁 Scanning: ~/Documents/my-project/
# ✅ Found 45 files
# 📊 Processing:
#    - .rs files: 20
#    - .md files: 15
#    - .txt files: 10
# ✅ Indexed 237 chunks from 45 documents
# 💾 Storage: 2.3 MB
```

Now query your documents:

```bash
synesis ask "How does the authentication system work?"

# Expected output:
# 🤔 Pathos: User wants technical explanation of auth system
# 🧠 Logos: Found 3 relevant documents:
#    - src/auth/mod.rs (relevance: 0.92)
#    - docs/auth.md (relevance: 0.87)
#    - tests/auth_test.rs (relevance: 0.71)
# ✅ Ethos: Technical accuracy verified
#
# Based on the codebase documentation:
#
# The authentication system uses JWT tokens with the following flow:
# 1. Client sends credentials to /auth/login
# 2. Server validates against database
# 3. Server issues JWT with 24-hour expiry
# 4. Client includes JWT in subsequent requests
# ...
#
# 📚 Sources: src/auth/mod.rs:45-67, docs/auth.md:12-34
```

---

## Step 6: Configure Your System

View and edit your configuration:

```bash
# View current config
synesis config list

# Get specific setting
synesis config get consensus.threshold

# Change setting
synesis config set consensus.threshold 0.90

# Available settings:
# - consensus.threshold: Agreement level (0.0-1.0, default: 0.85)
# - consensus.max_rounds: Max consensus rounds (1-5, default: 3)
# - agents.pathos.model: Model for Pathos agent
# - agents.logos.model: Model for Logos agent
# - agents.ethos.model: Model for Ethos agent
```

---

## Common First-Time Issues

### Issue: "command not found: synesis"

**Solution**: Add to PATH or use full path:
```bash
export PATH="$PATH:$(pwd)/target/release"
# Or use: ./target/release/synesis
```

### Issue: "GPU not detected"

**Solution**: Install NVIDIA drivers:
```bash
# Ubuntu
sudo apt-get install nvidia-driver-535

# Verify
nvidia-smi
```

### Issue: "Out of memory"

**Solution**: Reduce model size or use CPU-only:
```bash
synesis config set agents.pathos.model phi-3-mini
```

---

## What's Next?

Congratulations! You've successfully:
- ✅ Installed SuperInstance AI
- ✅ Initialized the system
- ✅ Run your first query
- ✅ Added documents to knowledge vault
- ✅ Configured the system

### Continue Learning

1. **[Your First Query](your-first-query.md)** - Deep dive into query processing
2. **[Knowledge Vault](knowledge-vault.md)** - Master RAG and semantic search
3. **[Privacy Basics](privacy-basics.md)** - Understand privacy features
4. **[Advanced Usage](advanced-usage.md)** - Power user features

### Next Tutorial

**[Your First Query](your-first-query.md)** - Learn what happens during a query, how the three agents work together, and how to interpret results.

---

## Need Help?

- **Troubleshooting**: [Troubleshooting Guide](../reference/troubleshooting.md)
- **FAQ**: [Frequently Asked Questions](../reference/faq.md)
- **GitHub Issues**: [Post a question](https://github.com/SuperInstance/Tripartite1/issues)

---

**Tutorial Version**: v0.2.0
**Last Updated**: 2026-01-07
**Feedback**: Open an issue or PR to suggest improvements
