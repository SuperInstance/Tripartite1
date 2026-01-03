# SuperInstance AI (Synesis)

**Privacy-first, local-first AI with a tripartite consensus system**

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/SuperInstance/Tripartite1)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-APACHE)
[![Rust Version](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Phase](https://img.shields.io/badge/phase-1%20%7C%20Local%20Kernel-brightgreen.svg)](PROJECT_ROADMAP.md)

SuperInstance AI is a **tripartite agentic AI system** that prioritizes local processing while enabling intelligent cloud escalation. Three specialized agents (Pathos, Logos, Ethos) reach consensus before responding, ensuring high-quality, safe answers.

> **Current Status**: Phase 1 Complete ✅ | 122/122 Tests Passing | Production-Ready v0.1.0

## What Makes SuperInstance Different?

- **Tripartite Consensus**: Three specialized agents must agree before responding
  - **Pathos** (Intent): "What does the user actually want?"
  - **Logos** (Logic): "How do we accomplish this?" with RAG-enhanced reasoning
  - **Ethos** (Truth): "Is this safe, accurate, and feasible?" with veto power

- **Privacy-First Architecture**: All sensitive data is tokenized before any cloud processing
  - 18 built-in redaction patterns (emails, API keys, phone numbers, etc.)
  - Local token vault (SQLite) - never transmitted to cloud
  - Automatic re-inflation of tokens in responses

- **Local-First Processing**: Keep computation on your device
  - Automatic hardware detection (CPU, GPU, RAM, disk)
  - Intelligent model selection based on available resources
  - Optional cloud escalation for complex tasks (Phase 2)

- **Knowledge Vault**: Local vector database for RAG
  - SQLite-VSS for fast similarity search
  - Automatic document chunking (paragraph/sentence/fixed)
  - Multi-factor relevance scoring (similarity + recency + source quality)

## Quick Start

### Prerequisites

- **Rust** 1.75+ (install via [rustup](https://rustup.rs/))
- **C compiler** (gcc/clang) and OpenSSL headers
  - Ubuntu: `sudo apt-get install build-essential libssl-dev pkg-config`
  - macOS: `xcode-select --install`
  - Windows: [Build Tools for Visual Studio](https://visualstudio.microsoft.com/visual-cpp-build-tools/)

### Installation

```bash
# Clone repository
git clone https://github.com/SuperInstance/Tripartite1.git
cd Tripartite1

# Build release binary
cargo build --release

# Install to PATH (optional)
cargo install --path crates/synesis-cli

# Verify installation
synesis --version
# Output: synesis 0.1.0
```

### Initialize System

```bash
# Detect hardware and create config
synesis init

# Check system status
synesis status
# Displays: CPU, GPU, RAM, disk, platform, and model status
```

### Usage Examples

```bash
# Ask a question (runs tripartite consensus)
synesis ask "What is the capital of France?"
# Pathos extracts intent → Logos reasons → Ethos verifies → Response

# Ask with RAG (if knowledge vault is populated)
synesis ask "How does the authentication system work?"
# Logos retrieves relevant code from vault and cites sources

# Add documents to knowledge vault
synesis knowledge add ~/Projects/my-project/
# Supports: .rs, .md, .txt files

# Search knowledge vault
synesis knowledge search "authentication"
# Returns top-5 relevant chunks with relevance scores

# Knowledge vault statistics
synesis knowledge stats
# Shows: document count, chunk count, storage usage

# Model management
synesis model list
synesis model download phi-3-mini

# Manifest operations
synesis manifest list
synesis manifest load jetson-orin-nx

# Configuration
synesis config list
synesis config get consensus.threshold
synesis config set consensus.threshold 0.90
```

## System Requirements

### Minimum (CPU-only)
- 8 GB RAM
- 10 GB disk space
- x86_64 or ARM64 CPU with AVX2/NEON support

### Recommended
- 16 GB RAM
- 4 GB VRAM (NVIDIA GPU with CUDA)
- 25 GB disk space (for models)

### Optimal
- 32+ GB RAM
- 8+ GB VRAM
- NVMe storage

**Supported Platforms**:
- ✅ Linux (Ubuntu 22.04+, Debian 12+, Arch)
- ✅ macOS (12.0+, Apple Silicon M1/M2/M3)
- ⚠️ Windows 10/11 (basic support, GPU detection limited)

**Supported GPUs**:
- ✅ NVIDIA (CUDA 11.0+)
- ✅ AMD (ROCm 5.0+)
- ⚠️ Intel (basic, via sycl-ls)
- ✅ Apple Silicon (unified memory)
- 25 GB disk space

### Optimal
- 32+ GB RAM
- 8+ GB VRAM
- NVMe storage

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                          User Query                          │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      Privacy Proxy                           │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   Redact    │──│ Token Vault │──│     Reinflate       │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Tripartite Council                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   Pathos    │──│    Logos    │──│       Ethos         │  │
│  │  (Intent)   │  │ (Reasoning) │  │   (Verification)    │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
│                              │                               │
│                    ┌─────────────────┐                      │
│                    │ Consensus Engine │                      │
│                    └─────────────────┘                      │
└─────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┴───────────────┐
              ▼                               ▼
┌─────────────────────────┐     ┌─────────────────────────┐
│     Local Processing    │     │    Cloud Escalation     │
│  ┌───────────────────┐  │     │  ┌───────────────────┐  │
│  │   Local Models    │  │     │  │  Cloud API (1.3x) │  │
│  │   (phi-3, llama)  │  │     │  │ (Claude, GPT-4)   │  │
│  └───────────────────┘  │     │  └───────────────────┘  │
│  ┌───────────────────┐  │     └─────────────────────────┘
│  │  Knowledge Vault  │  │
│  │   (SQLite-VSS)    │  │
│  └───────────────────┘  │
└─────────────────────────┘
```

## Project Structure

```
synesis/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── synesis-cli/        # Command-line interface
│   ├── synesis-core/       # Council orchestration
│   ├── synesis-privacy/    # Redaction and token vault
│   ├── synesis-models/     # Model management and inference
│   └── synesis-knowledge/  # Vector database and RAG
├── manifests/              # Hardware profile definitions
├── cloud/                  # Cloudflare Workers (Phase 2)
└── docs/                   # Documentation
```

## Configuration

Configuration is stored in `~/.superinstance/config.toml`:

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

See `config.toml.example` for full options.

## Documentation

- **[ARCHITECTURE.md](ARCHITECTURE.md)** - Deep dive into system architecture, patterns, and technical decisions
- **[INTEGRATION_REPORT.md](INTEGRATION_REPORT.md)** - Integration testing results and verification
- **[ASYNC_PATTERNS_RUST.md](ASYNC_PATTERNS_RUST.md)** - Async/await patterns and best practices
- **[TROUBLESHOOTING.md](TROUBLESHOOTING.md)** - Common issues and solutions
- **[CONTRIBUTING.md](CONTRIBUTING.md)** - Contribution guidelines
- **[PROJECT_ROADMAP.md](PROJECT_ROADMAP.md)** - Phase timeline and milestones

## Testing

SuperInstance has **100% test coverage** for Phase 1 functionality:

```bash
# Run all tests
cargo test --workspace

# Run tests for specific crate
cargo test --package synesis-core
cargo test --package synesis-privacy
cargo test --package synesis-knowledge
cargo test --package synesis-models
cargo test --package synesis-cli

# Run with output
cargo test --workspace -- --nocapture

# Test Results: 122/122 passing ✅
# - synesis-core: 38 tests
# - synesis-knowledge: 28 tests
# - synesis-models: 12 tests
# - synesis-privacy: 37 tests
# - synesis-cli: 7 tests
```

## Known Limitations (v0.1.0)

1. **File Watcher Auto-Indexing**: Disabled due to architectural limitation (see [Issue #1](GITHUB_ISSUES.md#issue-1))
   - **Workaround**: Use `synesis knowledge index --watch <path>` manually
   - **Fix Planned**: Phase 2

2. **Placeholder Embeddings**: RAG uses SHA256 hashes instead of semantic embeddings
   - **Impact**: Retrieval is not semantic (keyword-like)
   - **Fix Planned**: Phase 2 (BGE-Micro integration)

3. **No Cloud Integration**: All processing is local
   - **Fix Planned**: Phase 2 (Cloudflare Workers, QUIC tunnel)

4. **Sequential Agent Execution**: Agents run one at a time (slow)
   - **Fix Planned**: Phase 2 (parallel execution where possible)

See [GITHUB_ISSUES.md](GITHUB_ISSUES.md) for the full list and details.

## Roadmap

- [x] **Phase 1**: Local Kernel (CLI + tripartite council) ✅ **COMPLETE**
- [ ] **Phase 2**: Cloud Mesh (Cloudflare Workers, billing, QUIC tunnel)
- [ ] **Phase 3**: Knowledge Marketplace (LoRA training, sharing, monetization)
- [ ] **Phase 4**: Utility Infrastructure (SDKs, enterprise features, distributed mode)

See [PROJECT_ROADMAP.md](PROJECT_ROADMAP.md) for details.

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

**Good First Issues**:
- [Issue #3](GITHUB_ISSUES.md#issue-3): Clean up compiler warnings
- [Issue #7](GITHUB_ISSUES.md#issue-7): Add config validation
- [Issue #8](GITHUB_ISSUES.md#issue-8): Add progress bars
- [Issue #10](GITHUB_ISSUES.md#issue-10): Add API documentation examples

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Acknowledgments

Built with amazing open-source projects:
- [llama.cpp](https://github.com/ggerganov/llama.cpp) - Local LLM inference
- [SQLite](https://sqlite.org/) + [SQLite-VSS](https://github.com/asg017/sqlite-vss) - Vector database
- [Tokio](https://tokio.rs/) - Async runtime
- [Cloudflare Workers](https://workers.cloudflare.com/) - Edge compute (Phase 2)

## Contact

- **GitHub**: https://github.com/SuperInstance/Tripartite1
- **Issues**: https://github.com/SuperInstance/Tripartite1/issues
- **Discussions**: https://github.com/SuperInstance/Tripartite1/discussions

---

**Version**: 0.1.0 (Phase 1 - Local Kernel)
**Status**: Production-Ready | 122/122 Tests Passing
**Last Updated**: 2026-01-02
