<div align="center">

# SuperInstance AI

## Privacy-First, Local-First AI with Tripartite Consensus

[![CI](https://github.com/SuperInstance/Tripartite1/actions/workflows/ci.yml/badge.svg)](https://github.com/SuperInstance/Tripartite1/actions/workflows/ci.yml)
[![Documentation](https://github.com/SuperInstance/Tripartite1/actions/workflows/documentation.yml/badge.svg)](https://github.com/SuperInstance/Tripartite1/actions/workflows/documentation.yml)
[![codecov](https://codecov.io/gh/SuperInstance/Tripartite1/branch/main/graph/badge.svg)](https://codecov.io/gh/SuperInstance/Tripartite1)
[![License](https://img.shields.io/badge/license-MIT%20%7C%20Apache--2.0-blue.svg)](LICENSE-APACHE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Phase](https://img.shields.io/badge/phase-2%20%7C%20Cloud%20Mesh-yellow.svg)](phases/PHASE_2_DETAILED_ROADMAP.md)
[![Tests](https://img.shields.io/badge/tests-234%2F234-brightgreen.svg)](https://github.com/SuperInstance/Tripartite1)

**Your AI, Your Way, Your Privacy.**

[Features](#-key-features) • [Quick Start](#-quick-start) • [Documentation](#-documentation) • [Contributing](#-contributing)

---

</div>

## 🎯 What Makes SuperInstance Different?

### 🧠 The Tripartite Consensus System

**Three specialized AI agents collaborate on every query:**

| Agent | Domain | Primary Question | Color |
|-------|--------|------------------|-------|
| **Pathos** | User Intent | *"What does the user actually want?"* | 🔵 Cyan |
| **Logos** | Project Logic | *"How do we accomplish this?"* | 🟠 Orange |
| **Ethos** | Ground Truth | *"Is this safe, accurate, and feasible?"* | 🟢 Green |

```
     ┌─────────────────────────────────────┐
     │         User Query                  │
     └──────────────┬──────────────────────┘
                    │
     ┌──────────────▼──────────────────────┐
     │     Privacy Proxy (Redact)          │
     └──────────────┬──────────────────────┘
                    │
    ┌───────────────┼───────────────┐
    │               │               │
    ▼               ▼               ▼
┌────────┐    ┌────────┐    ┌────────┐
│Pathos  │    │ Logos  │    │ Ethos  │
│ Intent │    │ Logic  │    │ Truth  │
└───┬────┘    └───┬────┘    └───┬────┘
    │             │             │
    └─────────────┼─────────────┘
                  │
    ┌─────────────▼─────────────┐
    │    Consensus Engine       │
    │  (Weighted Voting, 85%)   │
    └─────────────┬─────────────┘
                  │
            ┌─────▼─────┐
            │  Response │
            └───────────┘
```

**No response is emitted until all three agents agree above a confidence threshold (default 85%).**

---

### 🔒 Privacy-First Architecture

| Feature | Description |
|---------|-------------|
| 🔒 **Tokenization** | All sensitive data replaced with UUIDs before cloud processing |
| 🏠 **Local-First** | Your data stays on your machine by default |
| 🔐 **18 Redaction Patterns** | Emails, API keys, passwords, SSNs, credit cards, and more |
| 🛡️ **Local Token Vault** | Mappings stored locally, never transmitted |
| 🔄 **Re-Inflation** | Responses restored locally with original data |

**Example Privacy Flow:**

```
User Query (Before Redaction):
  "My API key is sk-12345abcde. How do I use it?"

       ▼
Privacy Proxy Redaction
       ▼
Redacted Query (Sent to AI):
  "My API key is [API_KEY_01]. How do I use it?"

       ▼
AI Processing (Never sees actual key)
       ▼
AI Response:
  "To use [API_KEY_01], install it in your .env file..."

       ▼
Privacy Proxy Re-Inflation (Local)
       ▼
Final Response (To User):
  "To use sk-12345abcde, install it in your .env file..."
```

---

### ⚡ Local-First Processing

| Capability | Benefit |
|------------|---------|
| **Automatic Hardware Detection** | CPU, GPU (NVIDIA/AMD/Apple Silicon), RAM, disk |
| **Intelligent Model Selection** | Chooses best model based on available resources |
| **Local Knowledge Vault** | RAG capabilities with SQLite-VSS vector search |
| **Offline-First** | Works completely offline after initial setup |
| **Optional Cloud Escalation** | Use cloud only when local resources insufficient |

<details>
<summary><strong>📊 Hardware Detection Details</strong></summary>

**Supported Hardware:**
- **NVIDIA GPUs**: CUDA 11.0+ with 4GB+ VRAM
- **AMD GPUs**: ROCm support (Linux)
- **Apple Silicon**: Metal Performance Shaders (M1/M2/M3)
- **CPU-Only**: 8GB+ RAM required

**Automatic Detection:**
```bash
$ synesis status

┌──────────────────┬──────────────────┐
│ Component        │ Status           │
├──────────────────┼──────────────────┤
│ CPU              │ 16 cores @ 3.5GHz│
│ GPU              │ NVIDIA RTX 4090  │
│ RAM              │ 32 GB            │
│ Model            │ phi-3-mini       │
│ Knowledge Vault  │ 1,234 documents  │
└──────────────────┴──────────────────┘
```

</details>

---

## 🚀 Quick Start

### Prerequisites

- **Rust** 1.75+ ([install via rustup](https://rustup.rs/))
- **C compiler** and OpenSSL headers
- **8GB RAM** minimum (16GB recommended)

### Installation

```bash
# Clone the repository
git clone https://github.com/SuperInstance/Tripartite1.git
cd Tripartite1

# Build release binary
cargo build --release

# Initialize the system
./target/release/synesis init

# Run your first query
./target/release/synesis ask "What is the capital of France?"
```

**Output:**

```
🤔 Pathos (Intent): User wants factual information about French geography
🧠 Logos (Logic): Retrieving knowledge about capital cities...
✅ Ethos (Truth): Verifying factual accuracy...

✅ Consensus reached (0.95 confidence)

The capital of France is Paris.

---
Agents: 3/3 agreed | Confidence: 95% | Time: 2.3s
```

---

## 📚 Usage Examples

### Basic Query

```bash
synesis ask "Explain how vector databases work"
```

### Knowledge Vault (RAG)

```bash
# Add your documents
synesis knowledge add ~/Documents/my-project/

# Query your codebase
synesis ask "How does the authentication system work?"
```

### Custom Configuration

```bash
# Adjust consensus threshold
synesis config set consensus.threshold 0.90

# Change model
synesis config set agents.pathos.model phi-3-mini
```

<details>
<summary><strong>🔧 Advanced Configuration</strong></summary>

**Available Settings:**

```bash
# Consensus Configuration
synesis config set consensus.threshold 0.90      # Agreement threshold (0.0-1.0)
synesis config set consensus.max_rounds 3        # Maximum negotiation rounds

# Agent Configuration
synesis config set agents.pathos.model phi-3-mini
synesis config set agents.logos.model phi-3-mini
synesis config set agents.ethos.model phi-3-mini

# Privacy Configuration
synesis config set privacy.auto_redact true      # Auto-redact sensitive data
synesis config set privacy.patterns email,api_key,ssn

# Knowledge Vault Configuration
synesis config set knowledge.chunk_size 1000     # Document chunk size
synesis config set knowledge.chunk_strategy semantic
```

</details>

---

## 🏗️ Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                    USER INTERFACE LAYER                       │
│              (CLI / Desktop / Mobile / Web)                   │
└─────────────────────────┬────────────────────────────────────┘
                          │
┌─────────────────────────▼────────────────────────────────────┐
│                  LOCAL HUB (Your Device)                      │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐                       │
│  │ PATHOS  │  │  LOGOS  │  │  ETHOS  │  ← Tripartite Council │
│  │ (Intent)│  │ (Logic) │  │ (Truth) │                       │
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

<details>
<summary><strong>🔍 Deep Dive: Consensus Flow</strong></summary>

**Step-by-Step Consensus Process:**

1. **User submits query** → Privacy Proxy checks for sensitive data
2. **Privacy Proxy** → Redacts emails, API keys, credentials
3. **Redacted query** → Sent to all three agents simultaneously
4. **Each agent** → Processes query through their specialized lens
5. **Agent outputs** → Sent to Consensus Engine with confidence scores
6. **Consensus Engine** → Calculates weighted agreement score
7. **If score < 85%** → Revision round (agents negotiate)
8. **If score ≥ 85%** → Response emitted
9. **Response** → Re-inflated locally and presented to user

**Weighted Voting:**
- Pathos (Intent): Weight 1.2x (understanding user needs)
- Logos (Logic): Weight 1.0x (technical correctness)
- Ethos (Truth): Weight 1.5x (safety and accuracy veto power)

</details>

**Learn More**: [ARCHITECTURE.md](ARCHITECTURE.md) | [Developer Guide](CONTRIBUTING.md)

---

## 🎓 Key Features

### Tripartite Consensus

| Feature | Description |
|---------|-------------|
| **Multi-Agent Deliberation** | Each agent brings unique perspective to every query |
| **Weighted Voting** | Ethos has veto power (1.5x weight) for safety |
| **Revision Rounds** | Agents negotiate if initial consensus is low |
| **Transparent** | See how each agent contributed to the final answer |

### Privacy & Security

| Feature | Description |
|---------|-------------|
| **18 Redaction Patterns** | Emails, API keys, phone numbers, SSNs, credit cards, passwords |
| **Local Token Vault** | SQLite database, mappings never transmitted |
| **Re-Inflation** | Only happens locally on your device |
| **mTLS** | All cloud communication uses mutual TLS (Phase 2) |

### Knowledge Vault (RAG)

| Feature | Description |
|---------|-------------|
| **SQLite-VSS** | Fast vector search on local documents |
| **Automatic Chunking** | Paragraph, sentence, or fixed-size strategies |
| **Semantic Search** | Find relevant information in your codebase |
| **Source Citation** | Responses include where information came from |

### Performance

| Metric | Local (CPU) | Local (GPU) | Cloud |
|--------|------------|-------------|-------|
| First query | 5-8s | 3-5s | 2-3s |
| Subsequent | 2-3s | 1-2s | 1-2s |
| Memory usage | 4-8 GB | 6-12 GB | N/A |
| Privacy | 100% | 100% | Tokenized |

*Benchmarks on: Intel i7-12700K, 32GB RAM, NVIDIA RTX 4090*

---

## 📖 Documentation

### For Users

- **[Getting Started Tutorial](docs/tutorials/getting-started.md)** - Installation and first query
- **[Your First Query](docs/tutorials/your-first-query.md)** - Understanding the tripartite system
- **[Knowledge Vault Guide](docs/tutorials/knowledge-vault.md)** - Using RAG with your documents
- **[Privacy Basics](docs/tutorials/privacy-basics.md)** - How privacy features work
- **[FAQ](docs/reference/faq.md)** - Frequently asked questions
- **[Glossary](docs/reference/glossary.md)** - Terminology and concepts

### For Developers

- **[Developer Guide](CONTRIBUTING.md)** - Contribution and development workflow
- **[Architecture Deep Dive](ARCHITECTURE.md)** - System design and internals
- **[API Documentation](https://docs.rs/synesis-core/)** - Rust API reference
- **[Examples](examples/)** - Runnable code examples
- **[Testing Guide](docs/contributing/testing-guide.md)** - How to write tests

### Phase Documentation

- **[Phase 1: Local Kernel](phases/PHASE_1_LOCAL_KERNEL.md)** ✅ Complete
- **[Phase 2: Cloud Mesh](phases/PHASE_2_DETAILED_ROADMAP.md)** 🔄 In Progress (33%)
- **[Phase 3: Marketplace](phases/PHASE_3_MARKETPLACE.md)** ⏳ Planned
- **[Phase 4: Utility](phases/PHASE_4_UTILITY.md)** ⏳ Planned

---

## 🛠️ CLI Commands

<details>
<summary><strong>📝 Query Commands</strong></summary>

```bash
# Ask a question
synesis ask "Your question here"

# Stream response (real-time)
synesis ask --stream "Explain async Rust"

# Use specific model
synesis ask --model phi-3-mini "What is RAG?"
```

</details>

<details>
<summary><strong>📚 Knowledge Commands</strong></summary>

```bash
# Add documents to vault
synesis knowledge add <path>

# Search knowledge vault
synesis knowledge search "query"

# View vault statistics
synesis knowledge stats

# Reindex documents
synesis knowledge reindex

# Remove documents
synesis knowledge remove <pattern>
```

</details>

<details>
<summary><strong>⚙️ Configuration Commands</strong></summary>

```bash
# List all settings
synesis config list

# Get specific setting
synesis config get <key>

# Change setting
synesis config set <key> <value>

# Reset to default
synesis config reset <key>
```

</details>

<details>
<summary><strong>📊 System Commands</strong></summary>

```bash
# View system status
synesis status

# View performance metrics
synesis metrics show

# Export metrics (Prometheus format)
synesis metrics export metrics.txt
```

</details>

<details>
<summary><strong>🤖 Model Management</strong></summary>

```bash
# List available models
synesis model list

# Download a model
synesis model download <model>

# View model details
synesis model info <model>

# Remove a model
synesis model remove <model>
```

</details>

---

## 💡 Use Cases

### For Developers

- **Code Understanding**: "How does the authentication flow work?"
- **Bug Investigation**: "Why is this function returning an error?"
- **Code Review**: "What are the potential issues with this code?"
- **Documentation**: "Generate docs for this API endpoint"

### For Researchers

- **Literature Review**: "Summarize recent papers on vector databases"
- **Concept Explanation**: "Explain Rust ownership with examples"
- **Technical Writing**: "Write a technical description of this system"

### For Writers

- **Content Generation**: "Write blog post about async Rust"
- **Editing**: "Improve clarity and flow of this paragraph"
- **Ideation**: "Brainstorm features for a mobile app"

### For Everyone

- **Learning**: "Teach me about machine learning"
- **Analysis**: "Compare and contrast these two approaches"
- **Decision Making**: "What are the trade-offs between SQL and NoSQL?"

---

## 🔧 System Requirements

### Minimum (CPU-only)

| Component | Requirement |
|-----------|-------------|
| RAM | 8 GB |
| Disk Space | 10 GB |
| CPU | x86_64 or ARM64 |
| GPU | Not required |

### Recommended

| Component | Requirement |
|-----------|-------------|
| RAM | 16 GB |
| Disk Space | 25 GB |
| GPU | 4 GB VRAM (NVIDIA) |
| OS | Ubuntu 22.04+ / macOS 12+ / Windows 10+ |

### Optimal

| Component | Requirement |
|-----------|-------------|
| RAM | 32 GB |
| Disk Space | NVMe SSD |
| GPU | 8 GB VRAM (NVIDIA RTX 3060+) |
| GPU Support | NVIDIA, AMD, or Apple Silicon |

---

## 📦 Project Status

<div align="center">

### **v0.2.0** - Phase 1 Complete | Phase 2 In Progress (33%)

**Tests**: 234/234 Passing (100%) | **Code Quality**: Zero Warnings | **Documentation**: Comprehensive

</div>

### Completed Features (Phase 1)

- ✅ Tripartite council with three agents
- ✅ Consensus engine with multi-round negotiation
- ✅ Privacy proxy with 18 redaction patterns
- ✅ Knowledge vault with RAG and semantic search
- ✅ Hardware detection and model management
- ✅ CLI with all commands
- ✅ Comprehensive testing (234 tests)
- ✅ Zero compiler warnings

### In Progress (Phase 2)

- 🔄 QUIC tunnel with mTLS (Sessions 2.1-2.2 complete)
- 🔄 Device telemetry and heartbeat (Session 2.3 complete)
- 🔄 Cloud escalation client (Session 2.4 next)
- ⏳ Billing integration (Session 2.6)
- ⏳ Cloudflare Workers deployment (Session 2.7)

<details>
<summary><strong>🗺️ Phase Roadmap</strong></summary>

**Phase 1: Local Kernel ✅ COMPLETE**
- Tripartite consensus system
- Privacy proxy with redaction
- Knowledge vault with RAG
- Hardware detection
- CLI interface

**Phase 2: Cloud Mesh 🔄 IN PROGRESS (33%)**
- QUIC tunnel with mTLS
- Cloud escalation (Claude, GPT-4)
- Billing and metering
- LoRA hot-swap
- Collaborator system

**Phase 3: Marketplace ⏳ PLANNED**
- LoRA training
- Knowledge marketplace
- Model sharing
- Monetization

**Phase 4: Utility ⏳ PLANNED**
- SDKs (Python, JavaScript)
- Desktop application
- Mobile SDK
- Distributed mode

**See**: [PROJECT_ROADMAP.md](PROJECT_ROADMAP.md) for details

</details>

---

## 🤝 Contributing

We welcome contributions! SuperInstance is a community-driven project.

### Good First Issues

- 📚 Improve documentation
- 🧪 Add tests
- 🐛 Fix bugs
- ✨ Add features

**See**: [CONTRIBUTING.md](CONTRIBUTING.md) | [Developer Guide](CONTRIBUTING.md)

### Development Workflow

1. Read [Developer Guide](CONTRIBUTING.md)
2. Set up development environment
3. Pick an issue or create one
4. Fork and create a branch
5. Make your changes
6. Add tests and documentation
7. Submit a pull request

---

## 🧪 Testing

```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p synesis-core
cargo test -p synesis-knowledge
cargo test -p synesis-privacy

# Run with output
cargo test --workspace -- --nocapture

# Test coverage
cargo test --workspace --all-features
```

**Test Results**: 234/234 passing (100%)

---

## 🔐 Privacy & Security

SuperInstance is designed with privacy as a core principle:

### Data Protection

| Feature | Status |
|---------|--------|
| Local processing by default | ✅ Your data never leaves your device |
| Tokenization before cloud | ✅ Sensitive info replaced with UUIDs |
| Local token vault | ✅ Mappings stored locally (SQLite) |
| mTLS encryption | ✅ All cloud communication encrypted (Phase 2) |
| Open source | ✅ Fully auditable codebase |

### Redaction Patterns

Built-in patterns for:
- Email addresses
- API keys (GitHub, AWS, OpenAI, etc.)
- Phone numbers
- Social Security Numbers
- Credit card numbers
- Passwords
- IP addresses
- And 10 more...

**See**: [Privacy Basics Tutorial](docs/tutorials/privacy-basics.md)

---

## 📝 License

Licensed under either of:

- **MIT License** ([LICENSE-MIT](LICENSE-MIT))
- **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE))

at your option.

---

## 🙏 Acknowledgments

Built with amazing open-source projects:

- **[llama.cpp](https://github.com/ggerganov/llama.cpp)** - Local LLM inference
- **[SQLite](https://sqlite.org/)** + **[SQLite-VSS](https://github.com/asg017/sqlite-vss)** - Vector database
- **[Tokio](https://tokio.rs/)** - Async runtime
- **[Quinn](https://github.com/quinn-rs/quinn)** - QUIC implementation
- **[Cloudflare Workers](https://workers.cloudflare.com/)** - Edge compute (Phase 2)

---

## 📞 Contact & Support

### Getting Help

- **[Documentation](docs/)** - Start here
- **[FAQ](docs/reference/faq.md)** - Common questions
- **[Troubleshooting](TROUBLESHOOTING.md)** - Solve problems
- **[GitHub Issues](https://github.com/SuperInstance/Tripartite1/issues)** - Report bugs
- **[GitHub Discussions](https://github.com/SuperInstance/Tripartite1/discussions)** - Ask questions

### Community

- **GitHub**: [SuperInstance/Tripartite1](https://github.com/SuperInstance/Tripartite1)
- **Star** ⭐ us if you find SuperInstance useful!
- **Watch** 👀 to track progress
- **Fork** 🍴 to contribute

---

<div align="center">

**SuperInstance AI** - *Your AI, your way, your privacy.*

**Version**: 0.2.0 | **Status**: Production-Ready (Phase 1) | **Tests**: 234 Passing ✅

*Last Updated: 2026-01-07*

</div>
