# SuperInstance AI - Troubleshooting Guide

**Version**: 0.1.0
**Last Updated**: 2026-01-02

---

## Table of Contents

1. [Installation Issues](#installation-issues)
2. [Runtime Issues](#runtime-issues)
3. [Performance Issues](#performance-issues)
4. [Knowledge Vault Issues](#knowledge-vault-issues)
5. [Agent/Consensus Issues](#agentconsensus-issues)
6. [Privacy/Redaction Issues](#privacyredaction-issues)
7. [Hardware Detection Issues](#hardware-detection-issues)
8. [Build/Compilation Issues](#buildcompilation-issues)
9. [Getting Help](#getting-help)

---

## Installation Issues

### Issue: "cargo: command not found"

**Symptoms**:
```bash
cargo: command not found
```

**Cause**: Rust toolchain not installed or not in PATH.

**Solution**:

1. Install Rust using rustup:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

2. Verify installation:
```bash
cargo --version
rustc --version
```

3. If still not found, add to PATH manually:
```bash
export PATH="$HOME/.cargo/bin:$PATH"
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
```

---

### Issue: "error: linking with cc failed"

**Symptoms**:
```
error: linking with `cc` failed
error = Non-UTF-8 output: UTF-8 error
```

**Cause**: Missing C compiler or linker.

**Solution**:

**Ubuntu/Debian**:
```bash
sudo apt-get update
sudo apt-get install build-essential
```

**macOS**:
```bash
xcode-select --install
```

**Windows**:
- Install [Build Tools for Visual Studio](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
- Or install [MSYS2](https://www.msys2.org/)

---

### Issue: "failed to run custom build for `openssl-sys`"

**Symptoms**:
```
error: failed to run custom build command for `openssl-sys`
```

**Cause**: Missing OpenSSL headers.

**Solution**:

**Ubuntu/Debian**:
```bash
sudo apt-get install libssl-dev pkg-config
```

**macOS**:
```bash
brew install openssl
export OPENSSL_DIR=/usr/local/opt/openssl
export OPENSSL_LIB_DIR=/usr/local/opt/openssl/lib
```

**Windows**:
- Use vcpkg or static linking feature
- Or set `OPENSSL_DIR` environment variable

---

## Runtime Issues

### Issue: "No such file or directory (os error 2)"

**Symptoms**:
```
Error: Io(Custom { kind: NotFound, error: "No such file or directory" })
```

**Cause**: Missing directory or file (e.g., models directory, config file).

**Solution**:

1. Run `synesis init` to create directories:
```bash
synesis init
```

2. Manually create directory:
```bash
mkdir -p ~/.superinstance/models
mkdir -p ~/.superinstance/vaults
mkdir -p ~/.superinstance/manifests
```

---

### Issue: "Permission denied"

**Symptoms**:
```
Error: PermissionDenied
```

**Cause**: Insufficient permissions on config directory or models.

**Solution**:

1. Check permissions:
```bash
ls -la ~/.superinstance/
```

2. Fix permissions:
```bash
chmod -R 755 ~/.superinstance/
```

3. If models are owned by root:
```bash
sudo chown -R $USER:$USER ~/.superinstance/
```

---

### Issue: "Database is locked"

**Symptoms**:
```
Error: Database(DatabaseIsLocked)
```

**Cause**: Another process is using the SQLite database (token vault or knowledge vault).

**Solution**:

1. Check for other processes:
```bash
ps aux | grep synesis
```

2. Kill other instances:
```bash
pkill synesis
```

3. If stuck, remove lock files:
```bash
rm ~/.superinstance/vaults/*.db-wal
rm ~/.superinstance/vaults/*.db-shm
```

---

## Performance Issues

### Issue: Slow agent response times (> 10 seconds)

**Symptoms**:
- `synesis ask` takes 10+ seconds to respond
- CPU usage spikes during inference

**Cause**: Large model or insufficient hardware.

**Solution**:

1. Check hardware capabilities:
```bash
synesis status
```

2. Use smaller model:
```bash
# Edit ~/.superinstance/config.toml
[agents.logos]
model = "phi-3-mini"  # Instead of llama-3.2-8b
```

3. Enable GPU acceleration (if available):
```bash
# Verify GPU is detected
synesis status | grep GPU

# Install CUDA toolkit (NVIDIA)
# Or ROCm (AMD GPU)
```

---

### Issue: High memory usage (> 4GB)

**Symptoms**:
- System becomes sluggish
- OOM (Out of Memory) errors

**Cause**: Model loaded entirely in memory or memory leak.

**Solution**:

1. Check memory usage:
```bash
ps aux | grep synesis
```

2. Use quantized model (smaller memory footprint):
```bash
[agents.logos]
model = "llama-3.2-8b-q4_0"  # 4-bit quantization
```

3. Restart synesis periodically to free memory:
```bash
pkill synesis
synesis ask "..."
```

---

### Issue: Knowledge vault search is slow

**Symptoms**:
- `synesis knowledge search` takes > 1 second
- RAG queries slow down responses

**Cause**: Large vault (>10K documents) or missing VSS index.

**Solution**:

1. Check vault stats:
```bash
synesis knowledge stats
```

2. Rebuild VSS index:
```bash
synesis knowledge rebuild-index
```

3. Limit search results:
```bash
synesis knowledge search "query" --top-k 3  # Instead of 10
```

---

## Knowledge Vault Issues

### Issue: "No results found" for relevant queries

**Symptoms**:
- `synesis knowledge search` returns no results
- Logos agent doesn't retrieve context

**Cause**: Documents not indexed or embeddings are placeholders.

**Solution**:

1. Check if documents are indexed:
```bash
synesis knowledge stats
# Should show "Documents: N" where N > 0
```

2. Re-index documents:
```bash
synesis knowledge add ~/path/to/docs/
```

3. Check document types (only .rs, .md, .txt supported):
```bash
ls ~/path/to/docs/*.{rs,md,txt}
```

4. Note: Phase 1 uses SHA256 placeholder embeddings (not semantic). Semantic search coming in Phase 2.

---

### Issue: File watcher not auto-indexing

**Symptoms**:
- Modified files not automatically indexed
- Must manually run `synesis knowledge index`

**Cause**: File watcher auto-indexing disabled in Phase 1 (architectural limitation).

**Solution**:

1. Manually index files:
```bash
synesis knowledge index --watch ~/Projects/
```

2. Use cron job or file watcher script:
```bash
# Example: Every 5 minutes
*/5 * * * * synesis knowledge index ~/Projects/
```

3. Note: This will be fixed in Phase 2 with channel-based architecture.

---

## Agent/Consensus Issues

### Issue: "Consensus not reached after 3 rounds"

**Symptoms**:
```
Warning: Consensus not reached after 3 rounds (final confidence: 0.72)
```

**Cause**: Low confidence from agents (query too complex or ambiguous).

**Solution**:

1. Rephrase query more clearly:
```bash
# Instead of: "it thing"
# Try: "How does authentication work in this codebase?"
```

2. Lower consensus threshold:
```bash
# Edit ~/.superinstance/config.toml
[consensus]
threshold = 0.70  # Instead of 0.85
```

3. Increase max rounds:
```bash
[consensus]
max_rounds = 5  # Instead of 3
```

---

### Issue: "Response vetoed by Ethos"

**Symptoms**:
```
Warning: Ethos vetoed the response: Potential security risk detected
```

**Cause**: Ethos agent detected unsafe content (code injection, dangerous command, etc.).

**Solution**:

1. Review the veto reason:
```bash
synesis ask "..." --verbose
```

2. If veto is a false positive, file issue:
```bash
# Report at: https://github.com/SuperInstance/Tripartite1/issues
```

3. Be explicit about safe context:
```bash
# Instead of: "How to delete all files?"
# Try: "How to use rm command safely for cleanup?"
```

---

## Privacy/Redaction Issues

### Issue: Sensitive data not redacted

**Symptoms**:
- Email addresses or API keys appear in logs
- Cloud escalation sends sensitive data

**Cause**: Redaction disabled or pattern not matched.

**Solution**:

1. Verify redaction is enabled:
```bash
# Edit ~/.superinstance/config.toml
[privacy]
redact_emails = true
redact_api_keys = true
redact_phone_numbers = true
```

2. Check supported patterns (see [ARCHITECTURE.md](ARCHITECTURE.md)):
- Email: `user@example.com`
- API Key: `sk-...`, `ghp_...`, `AKIA...`
- Phone: `+1-555-123-4567`
- IPv4/IPv6, URLs, file paths, etc.

3. If pattern is missing, request it:
```bash
# File issue: https://github.com/SuperInstance/Tripartite1/issues
```

---

### Issue: "Token vault error"

**Symptoms**:
```
Error: Vault(InvalidSession)
```

**Cause**: Session expired or token vault cleared.

**Solution**:

1. Check session ID:
```bash
synesis config get session
```

2. Start new session:
```bash
synesis config set session "new-session-id"
```

3. Clear and restart:
```bash
rm ~/.superinstance/vaults/tokens.db
synesis ask "..."
```

---

## Hardware Detection Issues

### Issue: GPU not detected

**Symptoms**:
```
synesis status shows: GPU: None
```

**Cause**: GPU drivers not installed or detection tool missing.

**Solution**:

**NVIDIA GPU**:
1. Install NVIDIA drivers:
```bash
# Ubuntu
sudo apt-get install nvidia-driver-535

# Verify
nvidia-smi
```

2. Install CUDA toolkit:
```bash
sudo apt-get install cuda-toolkit-12
```

**AMD GPU**:
1. Install ROCm:
```bash
# Ubuntu
sudo apt-get install rocm-dev
```

2. Verify:
```bash
rocm-smi
```

**Apple Silicon**:
1. Unified memory detected automatically (system RAM = VRAM)
2. No action needed

---

### Issue: Insufficient disk space

**Symptoms**:
```
Error: Insufficient disk space (need 25GB, have 10GB)
```

**Cause**: Models require significant storage.

**Solution**:

1. Check disk space:
```bash
df -h ~/.superinstance/
```

2. Free up space:
```bash
# Remove unused models
synesis model remove unused

# Or change models directory to external drive
synesis config set models_dir /mnt/external/models
```

3. Use smaller models:
```bash
[agents.logos]
model = "phi-3-mini"  # ~2GB instead of ~8GB
```

---

## Build/Compilation Issues

### Issue: "compiler warnings: 19 warnings"

**Symptoms**:
```
warning: unused import: std::collections::HashMap
warning: variable does not need to be mutable
...
```

**Cause**: Unused code (non-critical).

**Solution**:

1. Auto-fix warnings:
```bash
cargo fix --allow-dirty
```

2. Ignore specific warnings:
```bash
cargo build --release 2>&1 | grep warning
```

3. Note: These will be cleaned up before v0.2.0 release.

---

### Issue: "doctest failed"

**Symptoms**:
```
error: doctest failed, to rerun pass `-p synesis-core --doc`
```

**Cause**: Documentation examples outdated.

**Solution**:

1. Run doctests:
```bash
cargo test --doc
```

2. Fix specific doctest:
```bash
cargo test --doc -p synesis-core
```

3. Note: All doctests should pass in v0.1.0.

---

## Getting Help

### 1. Check Documentation

- [README.md](README.md) - Quick start guide
- [ARCHITECTURE.md](ARCHITECTURE.md) - System architecture
- [ASYNC_PATTERNS_RUST.md](ASYNC_PATTERNS_RUST.md) - Async patterns
- [INTEGRATION_REPORT.md](INTEGRATION_REPORT.md) - Integration status

---

### 2. Search Existing Issues

Check if your issue is already reported:
```bash
# Visit: https://github.com/SuperInstance/Tripartite1/issues
```

Common issues:
- File watcher auto-indexing (#1)
- Placeholder embeddings (#2)
- Compiler warnings (#3)

---

### 3. Collect Debug Information

Before reporting issues, gather diagnostics:

```bash
# System info
synesis status > debug_info.txt

# Verbose output
synesis ask "test" --verbose > verbose_output.txt 2>&1

# Config file
cat ~/.superinstance/config.toml > config.txt

# Logs
ls -la ~/.superinstance/logs/
```

---

### 4. File an Issue

Create a new issue at: https://github.com/SuperInstance/Tripartite1/issues

**Include**:
- OS and version (`uname -a`)
- Rust version (`rustc --version`)
- Synesis version (`synesis --version`)
- Error message (full output)
- Steps to reproduce
- Expected vs actual behavior

---

### 5. Community Support

- GitHub Discussions: https://github.com/SuperInstance/Tripartite1/discussions
- Discord: (coming soon)
- Matrix: (coming soon)

---

## Quick Reference

### Common Commands

```bash
# Diagnostics
synesis status
synesis status --verbose

# Config
synesis config list
synesis config get agents.logos.model
synesis config set agents.logos.model "phi-3-mini"

# Logs
tail -f ~/.superinstance/logs/synesis.log

# Reset
rm -rf ~/.superinstance/
synesis init
```

---

### Useful Paths

| Platform | Config Directory | Models Directory | Logs |
|----------|-----------------|------------------|------|
| Linux | `~/.superinstance/` | `~/.superinstance/models/` | `~/.superinstance/logs/` |
| macOS | `~/.superinstance/` | `~/.superinstance/models/` | `~/.superinstance/logs/` |
| Windows | `%USERPROFILE%\.superinstance\` | `%USERPROFILE%\.superinstance\models\` | `%USERPROFILE%\.superinstance\logs\` |

---

*Last Updated: 2026-01-02*
*Version: 0.1.0*
*Phase: 1 - Local Kernel*
