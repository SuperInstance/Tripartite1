# Frequently Asked Questions (FAQ)

Common questions about SuperInstance AI.

---

## General

### What is SuperInstance AI?

SuperInstance is a **privacy-first, local-first AI system** that uses three specialized agents (Pathos, Logos, Ethos) to reach consensus before responding. It keeps your data on your device and only escalates to the cloud when necessary.

**Learn More**: [Getting Started](../tutorials/getting-started.md)

### How is SuperInstance different from ChatGPT/Claude?

| Feature | SuperInstance | ChatGPT/Claude |
|---------|---------------|----------------|
| **Privacy** | All data stays local (by default) | Data sent to cloud |
| **Transparency** | See all three agents' reasoning | Black-box response |
| **Consensus** | Three agents must agree | Single model response |
| **Customization** | Custom agents, knowledge vault | Limited customization |
| **Cost** | Free (local), cheap (cloud) | Subscription required |
| **Offline** | Works without internet | Requires internet |

### Is SuperInstance free?

**Yes and no**:

- **Local Processing**: 100% free (runs on your hardware)
- **Cloud Escalation**: Pay-per-use (3% markup on Cloudflare costs)
- **BYOK Tier**: 30% licensing fee for using your own API keys

**Learn More**: [Cost Economics](../../CLAUDE.md#cost-plus-economics)

### What are the system requirements?

**Minimum**:
- 8 GB RAM
- 10 GB disk space
- x86_64 or ARM64 CPU

**Recommended**:
- 16 GB RAM
- 4 GB VRAM (NVIDIA GPU)
- 25 GB disk space

**See Also**: [Getting Started - Prerequisites](../tutorials/getting-started.md#step-1-install-prerequisites)

---

## Privacy and Security

### Does SuperInstance send my data to the cloud?

**No, not by default**:

- ✅ Local processing: Everything stays on your device
- ✅ Knowledge vault: Stored locally
- ✅ Models: Run locally (optional)

**Cloud escalation** (your choice):
- Redacts sensitive info first
- Only sends necessary data
- Never sees raw credentials/PII

**Learn More**: [Privacy Basics](../tutorials/privacy-basics.md)

### What data does SuperInstance collect?

**Local Only**:
- Configuration (in `~/.superinstance/`)
- Knowledge vault (SQLite database)
- Token vault (redaction mappings)
- Query history (if enabled)

**Never Collected**:
- We don't track you
- No telemetry sent home
- No account required (for local use)

### Is my data encrypted?

**Yes**:
- Knowledge vault: SQLite with file system encryption (OS-dependent)
- Token vault: UUID substitutions (reversible only locally)
- Cloud communication: TLS 1.3 with mTLS (Phase 2)

### Can SuperInstance work offline?

**Yes!** SuperInstance works completely offline for:
- ✅ All local queries
- ✅ Knowledge vault searches
- ✅ Model inference

**Only requires internet for**:
- Downloading models (one-time)
- Cloud escalation (optional)
- Updates

---

## Usage

### How do I ask a question?

```bash
synesis ask "Your question here"
```

**Example**:
```bash
synesis ask "What is the capital of France?"
synesis ask "How does the authentication system work?"
synesis ask "Explain Rust ownership with examples"
```

**Learn More**: [Your First Query](../tutorials/your-first-query.md)

### How do I add documents to the knowledge vault?

```bash
# Add a directory
synesis knowledge add ~/Documents/my-project/

# Add specific file types
synesis knowledge add ~/Documents/*.pdf

# Search your knowledge
synesis knowledge search "authentication"
```

**Learn More**: [Knowledge Vault Tutorial](../tutorials/knowledge-vault.md)

### Can I use SuperInstance as a library?

**Yes!** Use it programmatically:

```rust
use synesis_core::{Council, CouncilConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut council = Council::new(CouncilConfig::default());
    council.initialize().await?;

    let response = council.process(your_query).await?;
    println!("{}", response.content);

    Ok(())
}
```

**See Also**: [API Reference](../api/), [Examples](../../examples/)

### How accurate are the responses?

**Depends on**:
- Model quality (local vs cloud)
- Query clarity
- Knowledge in vault (for RAG)
- Consensus threshold (default: 85%)

**Typical accuracy**:
- Factual queries: 90-95%
- Technical explanations: 85-90%
- Creative tasks: 80-90%
- Code generation: 85-95% (with context)

**Tips for better accuracy**:
1. Be specific in your queries
2. Add relevant code/docs to knowledge vault
3. Use cloud models for complex tasks
4. Adjust consensus threshold if needed

---

## Performance

### Why is my first query slow?

**Reason**: Models need to be loaded into memory (~5-10 seconds once)

**Solution**: Subsequent queries are fast (~2-3 seconds)

### How can I speed up queries?

**Quick wins**:
1. **Use smaller models**: `phi-3-mini` instead of `llama-3.2-8b`
2. **Disable RAG**: `synesis ask "query" --no-rag` (if not needed)
3. **Reduce rounds**: `synesis config set consensus.max_rounds 1`

**Hardware upgrades**:
1. **More RAM**: 16 GB → 32 GB
2. **GPU**: NVIDIA RTX 3060 or better
3. **SSD**: Faster disk I/O

**See Also**: [Performance Tuning Guide](../guides/performance-tuning.md)

### Does SuperInstance use my GPU?

**Yes, if available**:
- ✅ NVIDIA GPUs (CUDA 11.0+)
- ✅ AMD GPUs (ROCm 5.0+)
- ✅ Apple Silicon (unified memory)
- ⚠️ Intel GPUs (basic support)

**Check your GPU**:
```bash
synesis status
```

---

## Troubleshooting

### "command not found: synesis"

**Solution**: Install or add to PATH:

```bash
# Install
cargo install --path crates/synesis-cli

# Or add to PATH
export PATH="$PATH:$(pwd)/target/release"
```

### "GPU not detected"

**Solutions**:

**NVIDIA**:
```bash
# Install drivers
sudo apt-get install nvidia-driver-535

# Verify
nvidia-smi
```

**AMD**:
```bash
# Install ROCm
# See: https://rocm.docs.amd.com/
```

**Apple Silicon**:
- Should work automatically (M1/M2/M3)

### "Out of memory"

**Solutions**:
1. Use smaller model: `synesis config set agents.pathos.model phi-3-mini`
2. Close other applications
3. Add swap space
4. Upgrade RAM

### Slow performance

**See**: [Performance Tuning Guide](../guides/performance-tuning.md)

**Quick checks**:
```bash
# Check what's using resources
synesis status

# Disable verbose output
synesis ask "query" --quiet

# Use CPU only (for testing)
synesis ask "query" --cpu
```

---

## Development

### How do I contribute?

**Start here**: [Contributing Guide](../contributing/)

**Quick start**:
1. Read [Onboarding Guide](../contributing/onboarding.md)
2. Set up development environment
3. Pick a good first issue
4. Make your first PR!

### How do I build from source?

```bash
# Clone repository
git clone https://github.com/SuperInstance/Tripartite1.git
cd Tripartite1

# Build
cargo build --release

# Run tests
cargo test --workspace

# Run
./target/release/synesis --version
```

### Can I create custom agents?

**Yes!** Implement the `Agent` trait:

```rust
use synesis_core::Agent;

pub struct MyCustomAgent {
    // Your fields
}

impl Agent for MyCustomAgent {
    fn process(&self, input: AgentInput) -> AgentOutput {
        // Your implementation
    }
}
```

**Learn More**: [Agent API](../api/agent-api.md), [Examples](../../examples/advanced/custom_agent.rs)

---

## Cloud and Billing (Phase 2)

### When will cloud features be available?

**Status**: Phase 2 in progress (33% complete)

**Completed**:
- ✅ QUIC tunnel implementation
- ✅ Telemetry and heartbeat
- 🔄 Escalation client (in progress)

**Planned**:
- Billing integration
- Cloudflare Workers deployment
- LoRA sharing

**Track progress**: [Phase 2 Roadmap](../../phases/PHASE_2_DETAILED_ROADMAP.md)

### How much will cloud escalation cost?

**Estimated costs** (managed tier, 3% markup):

| Model | Input (per 1M tokens) | Output (per 1M tokens) |
|-------|----------------------|-----------------------|
| Claude Sonnet | $3.00 × 1.03 = $3.09 | $15.00 × 1.03 = $15.45 |
| Claude Opus | $15.00 × 1.03 = $15.45 | $75.00 × 1.03 = $77.25 |
| GPT-4 Turbo | $10.00 × 1.03 = $10.30 | $30.00 × 1.03 = $30.90 |

**Example query**:
- Input: 500 tokens
- Output: 1,000 tokens
- Claude Sonnet: ~$0.02

**See Also**: [Billing Guide](../guides/billing.md) (when available)

---

## Advanced

### What is the tripartite consensus system?

**Three specialized agents**:

1. **Pathos** (Intent): What do you want?
2. **Logos** (Logic): How do we do it?
3. **Ethos** (Truth): Is it safe/accurate?

All three must agree before responding.

**Learn More**: [Tripartite Consensus Tutorial](../tutorials/tripartite-consensus.md)

### Can I use SuperInstance for commercial purposes?

**Yes!** Licensed under:
- MIT License ✅
- Apache-2.0 License ✅

Choose either. No restrictions on commercial use.

**See**: [LICENSE-MIT](../../LICENSE-MIT), [LICENSE-APACHE](../../LICENSE-APACHE)

### Can I run SuperInstance on a server?

**Yes!** Options:

1. **Headless mode**: No GUI needed
2. **Docker**: Containerize it
3. **Cloud**: Deploy to GPU instances (AWS, GCP, Azure)
4. **Edge**: Run on Jetson/edge devices

**See**: [Deployment Guide](../guides/deployment.md) (when available)

---

## Still Have Questions?

### Get Help

- **Documentation**: [Docs Index](../README.md)
- **Troubleshooting**: [Troubleshooting Guide](troubleshooting.md)
- **Glossary**: [Define terms](glossary.md)
- **GitHub Issues**: [Ask a question](https://github.com/SuperInstance/Tripartite1/issues)
- **GitHub Discussions**: [Start a discussion](https://github.com/SuperInstance/Tripartite1/discussions)

### Community

- **Star us on GitHub**: [SuperInstance/Tripartite1](https://github.com/SuperInstance/Tripartite1)
- **Watch for updates**: Click "Watch" on GitHub
- **Share feedback**: Open an issue or PR

---

## Can't Find Your Answer?

1. **Search the docs**: Use the search bar
2. **Check the glossary**: [Define unfamiliar terms](glossary.md)
3. **Ask the community**: [GitHub Discussions](https://github.com/SuperInstance/Tripartite1/discussions)
4. **Report an issue**: [GitHub Issues](https://github.com/SuperInstance/Tripartite1/issues/new)

---

**FAQ Version**: v0.2.0
**Last Updated**: 2026-01-07
**Total Questions**: 40+
