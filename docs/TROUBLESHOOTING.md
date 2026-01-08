# SuperInstance AI - Troubleshooting Guide

This guide helps you diagnose and fix common issues with SuperInstance AI.

## Table of Contents

- [Installation Issues](#installation-issues)
- [Model Management](#model-management)
- [Query Problems](#query-problems)
- [Cloud Connection](#cloud-connection)
- [Knowledge Vault](#knowledge-vault)
- [Performance Issues](#performance-issues)
- [Privacy Concerns](#privacy-concerns)
- [Hardware Issues](#hardware-issues)

## Installation Issues

### Command Not Found

**Problem**: `synesis: command not found`

**Solutions**:
1. Ensure you've installed SuperInstance:
   ```bash
   cargo install --path . --locked
   ```

2. Check your PATH:
   ```bash
   echo $PATH | grep cargo
   ```

3. Add Cargo bin directory to PATH:
   ```bash
   export PATH="$HOME/.cargo/bin:$PATH"
   echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
   ```

### Build Errors

**Problem**: Compilation fails with errors

**Solutions**:
1. Check Rust version:
   ```bash
   rustc --version  # Should be 1.70+
   ```

2. Update dependencies:
   ```bash
   cargo update
   ```

3. Clean build:
   ```bash
   cargo clean
   cargo build --release
   ```

4. Check for missing system dependencies:
   ```bash
   # Linux: Install build essentials
   sudo apt-get install build-essential pkg-config libssl-dev

   # macOS: Install Xcode command line tools
   xcode-select --install
   ```

### Model Download Failures

**Problem**: `synesis model download` fails

**Solutions**:
1. Check internet connection
2. Verify model availability:
   ```bash
   synesis model list --cloud
   ```
3. Try with explicit URL:
   ```bash
   synesis model download claude-sonnet --url https://example.com/model.gguf
   ```
4. Check disk space:
   ```bash
   df -h ~/.superinstance/models
   ```

## Model Management

### Model Not Found

**Problem**: `Model 'claude-sonnet' not found`

**Solutions**:
1. List available models:
   ```bash
   synesis model list
   ```

2. Download the model:
   ```bash
   synesis model download claude-sonnet
   ```

3. Check model integrity:
   ```bash
   synesis model verify claude-sonnet
   ```

### Slow Model Loading

**Problem**: Models take too long to load

**Solutions**:
1. Check hardware acceleration:
   ```bash
   synesis manifest detect
   ```

2. Use smaller models:
   ```bash
   synesis model download tinyllama
   ```

3. Preload models:
   ```bash
   synesis model preload --all
   ```

### Out of Memory

**Problem**: Model loading fails with OOM error

**Solutions**:
1. Check available RAM:
   ```bash
   free -h  # Linux
   vm_stat  # macOS
   ```

2. Use quantized models:
   ```bash
   synesis model download claude-sonnet-q4_0
   ```

3. Adjust model context size:
   ```bash
   export SYNESIS_CONTEXT_SIZE=2048
   ```

## Query Problems

### No Response

**Problem**: Query hangs or returns no response

**Diagnosis**:
1. Check if agents are ready:
   ```bash
   synesis status --verbose
   ```

2. Check model loading:
   ```bash
   synesis model list
   ```

3. Enable debug logging:
   ```bash
   RUST_LOG=debug synesis ask "query"
   ```

**Solutions**:
1. Reinitialize:
   ```bash
   synesis init --force
   ```

2. Reload models:
   ```bash
   synesis model reload --all
   ```

### Poor Quality Responses

**Problem**: Responses are irrelevant or low quality

**Solutions**:
1. Use cloud for complex queries:
   ```bash
   synesis ask --cloud "complex question"
   ```

2. Add knowledge context:
   ```bash
   synesis knowledge index ~/docs
   synesis ask "question based on docs"
   ```

3. Adjust consensus threshold:
   ```bash
   export SYNESIS_CONSENSUS_THRESHOLD=0.95
   ```

4. Increase consensus rounds:
   ```bash
   export SYNESIS_MAX_ROUNDS=5
   ```

### Consensus Timeout

**Problem**: Query times out waiting for consensus

**Solutions**:
1. Lower consensus threshold:
   ```bash
   export SYNESIS_CONSENSUS_THRESHOLD=0.75
   ```

2. Reduce max rounds:
   ```bash
   export SYNESIS_MAX_ROUNDS=2
   ```

3. Use cloud instead:
   ```bash
   synesis ask --cloud "query"
   ```

## Cloud Connection

### Authentication Failures

**Problem**: `synesis cloud login` fails

**Diagnosis**:
```bash
# Check API key validity
synesis cloud ping

# Verify credentials file
cat ~/.superinstance/credentials
```

**Solutions**:
1. Re-enter API key:
   ```bash
   synesis cloud logout
   synesis cloud login
   ```

2. Check credentials format:
   ```bash
   # Should be: sk-ant-api03-...
   synesis cloud login sk-ant-api03-...
   ```

3. Verify account status:
   ```bash
   synesis cloud status
   ```

### Connection Refused

**Problem**: Cannot connect to cloud

**Diagnosis**:
```bash
# Test connectivity
synesis cloud ping

# Check firewall
curl -v https://api.superinstance.ai/health
```

**Solutions**:
1. Check internet connection
2. Verify API endpoint:
   ```bash
   synesis config get cloud.endpoint
   ```
3. Try different endpoint:
   ```bash
   synesis config set cloud.endpoint api.backup.com
   ```
4. Check proxy settings:
   ```bash
   export HTTP_PROXY=http://proxy.example.com:8080
   export HTTPS_PROXY=http://proxy.example.com:8080
   ```

### Slow Cloud Responses

**Problem**: Cloud queries take too long

**Solutions**:
1. Check latency:
   ```bash
   synesis cloud ping
   ```

2. Use streaming for faster first-byte:
   ```bash
   synesis ask --cloud --stream "query"
   ```

3. Try different model:
   ```bash
   synesis ask --cloud --model claude-sonnet "query"
   ```

4. Check network bandwidth:
   ```bash
   speedtest-cli
   ```

## Knowledge Vault

### Indexing Failures

**Problem**: `synesis knowledge index` fails

**Diagnosis**:
```bash
# Check file permissions
ls -la ~/projects/myproject

# Check file types
file ~/projects/myproject/*.pdf
```

**Solutions**:
1. Check supported formats:
   - PDF: `.pdf`
   - Text: `.txt`, `.md`, `.rst`
   - Code: `.rs`, `.py`, `.js`, `.ts`
   - Markdown: `.md`

2. Check file permissions:
   ```bash
   chmod +r ~/projects/myproject/*
   ```

3. Try specific file:
   ```bash
   synesis knowledge index ~/projects/myproject/doc.pdf
   ```

### Poor Search Results

**Problem**: Knowledge search returns irrelevant results

**Solutions**:
1. Re-index with smaller chunks:
   ```bash
   synesis knowledge index . --chunk-size 500
   ```

2. Use better chunking strategy:
   ```bash
   synesis knowledge index . --chunk-strategy semantic
   ```

3. Index only relevant files:
   ```bash
   synesis knowledge index . --include "*.rs" --include "*.md"
   ```

4. Rebuild embeddings:
   ```bash
   synesis knowledge index . --force
   ```

### Vault Corruption

**Problem**: Knowledge vault returns errors

**Diagnosis**:
```bash
# Check vault integrity
synesis knowledge stats

# Check database
sqlite3 ~/.superinstance/knowledge/vault.db "SELECT COUNT(*) FROM chunks;"
```

**Solutions**:
1. Backup current vault:
   ```bash
   cp ~/.superinstance/knowledge/vault.db ~/.superinstance/knowledge/vault.db.backup
   ```

2. Reinitialize vault:
   ```bash
   rm ~/.superinstance/knowledge/vault.db
   synesis knowledge index .
   ```

3. Restore from backup if needed:
   ```bash
   mv ~/.superinstance/knowledge/vault.db.backup ~/.superinstance/knowledge/vault.db
   ```

## Performance Issues

### High CPU Usage

**Problem**: Synesis uses too much CPU

**Diagnosis**:
```bash
# Check process usage
top -p $(pgrep synesis)

# Check agent threads
ps aux | grep synesis
```

**Solutions**:
1. Reduce polling frequency:
   ```bash
   export SYNESIS_POLL_INTERVAL=1000  # milliseconds
   ```

2. Disable file watching:
   ```bash
   # Don't use knowledge watch
   ```

3. Use fewer agents:
   ```bash
   # Use local-only (no consensus overhead)
   synesis ask --local "query"
   ```

### High Memory Usage

**Problem**: Synesis uses too much memory

**Diagnosis**:
```bash
# Check memory usage
ps aux | grep synesis

# Check model memory
synesis model list --verbose
```

**Solutions**:
1. Use smaller models:
   ```bash
   synesis model download tinyllama
   synesis config set model.default tinyllama
   ```

2. Reduce context size:
   ```bash
   export SYNESIS_CONTEXT_SIZE=2048
   ```

3. Clear knowledge vault:
   ```bash
   synesis knowledge clear
   synesis knowledge index ~/critical-docs-only
   ```

### Slow Queries

**Problem**: Queries take too long

**Diagnosis**:
```bash
# Enable timing
time synesis ask "query"

# Check model performance
synesis model benchmark
```

**Solutions**:
1. Use GPU acceleration:
   ```bash
   export CUDA_VISIBLE_DEVICES=0
   ```

2. Reduce document count:
   ```bash
   # Index only essential docs
   synesis knowledge clear
   synesis knowledge index ~/essential-docs
   ```

3. Use cloud for speed:
   ```bash
   synesis ask --cloud "complex query"
   ```

## Privacy Concerns

### Data Leakage Suspected

**Problem**: Sensitive data might be exposed

**Diagnosis**:
```bash
# See what gets redacted
synesis ask --show-redactions "My email is test@example.com"

# Check cloud logs
synesis cloud usage
```

**Solutions**:
1. Verify redaction patterns:
   ```bash
   synesis ask --show-redactions "email: test@example.com, phone: 555-1234"
   ```

2. Check all redaction types are enabled:
   ```bash
   synesis config get privacy.redact_emails
   synesis config get privacy.redact_phones
   ```

3. Use local-only mode:
   ```bash
   synesis ask --local "sensitive query"
   ```

### Token Reuse

**Problem**: Tokens might be reused across sessions

**Diagnosis**:
```bash
# Check token counter
sqlite3 ~/.superinstance/privacy/tokens.db "SELECT category, counter FROM counters;"
```

**Solutions**:
1. Tokens are automatically session-specific
2. Vault is cleared after each query
3. For extra security, restart between sensitive sessions:
   ```bash
   synesis ask "sensitive query"
   # Session ends, tokens cleared
   synesis ask "another sensitive query"  # New tokens generated
   ```

## Hardware Issues

### GPU Not Detected

**Problem**: GPU not being used

**Diagnosis**:
```bash
# Check GPU detection
synesis manifest detect

# Check CUDA
nvidia-smi

# Check Vulkan
vulkaninfo
```

**Solutions**:
1. Install GPU drivers:
   ```bash
   # NVIDIA
   sudo apt install nvidia-driver-535

   # AMD
   sudo apt install mesa-vulkan-drivers
   ```

2. Install CUDA toolkit:
   ```bash
   wget https://developer.download.nvidia.com/compute/cuda/repos/ubuntu2004/x86_64/cuda-keyring_1.0-1_all.deb
   sudo dpkg -i cuda-keyring_1.0-1_all.deb
   sudo apt update
   sudo apt install cuda-toolkit-12-2
   ```

3. Update manifest:
   ```bash
   synesis manifest detect --force
   ```

### Apple Silicon Issues

**Problem**: M1/M2 Mac performance issues

**Solutions**:
1. Use Metal backend:
   ```bash
   export SYNESIS_ACCELERATION=metal
   ```

2. Use ARM-optimized models:
   ```bash
   synesis model download claude-sonnet-arm64
   ```

3. Increase memory limit:
   ```bash
   ulimit -n 65536
   ```

## Getting Help

If none of these solutions work:

1. **Check logs**:
   ```bash
   ~/.superinstance/logs/synesis.log
   ```

2. **Enable debug mode**:
   ```bash
   RUST_LOG=debug synesis ask "query"
   ```

3. **Report bugs**:
   - GitHub Issues: https://github.com/SuperInstance/Tripartite1/issues
   - Include: OS, Rust version, error message, steps to reproduce

4. **Community support**:
   - Discussions: https://github.com/SuperInstance/Tripartite1/discussions
   - Documentation: https://github.com/SuperInstance/Tripartite1/wiki

## Common Error Messages

### `AgentNotReady`

**Meaning**: Model not loaded

**Fix**:
```bash
synesis model download <model-name>
synesis model reload <model-name>
```

### `ConsensusTimeout`

**Meaning**: Agents couldn't agree

**Fix**:
```bash
export SYNESIS_MAX_ROUNDS=5
export SYNESIS_CONSENSUS_THRESHOLD=0.80
```

### `CloudConnectionError`

**Meaning**: Cannot reach cloud API

**Fix**:
```bash
synesis cloud ping
synesis cloud login
```

### `VaultCorruption`

**Meaning**: Knowledge database damaged

**Fix**:
```bash
synesis knowledge clear
synesis knowledge index .
```

### `RedactionError`

**Meaning**: Privacy system failed

**Fix**:
```bash
# Check vault permissions
ls -la ~/.superinstance/privacy/

# Reinitialize if needed
rm ~/.superinstance/privacy/tokens.db
```

## Prevention Tips

### Regular Maintenance

```bash
# Weekly: Check system health
synesis status
synesis model verify --all

# Monthly: Clean up
synesis knowledge clear
synesis knowledge index .

# Quarterly: Update
cargo install --path . --locked
```

### Best Practices

1. **Always verify models** after download
2. **Use local queries** when possible (faster, free)
3. **Index knowledge** before complex projects
4. **Check cloud balance** before heavy usage
5. **Review privacy settings** for sensitive work
6. **Keep backups** of knowledge vault
7. **Monitor metrics** for performance issues
8. **Update regularly** for bug fixes and features
