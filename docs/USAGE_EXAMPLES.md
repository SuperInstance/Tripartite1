//! # SuperInstance AI - Usage Examples
//!
//! This document provides comprehensive examples for using SuperInstance AI.

## Table of Contents
//!
//! - [Getting Started](#getting-started)
//! - [Local Queries](#local-queries)
//! - [Cloud Escalation](#cloud-escalation)
//! - [Knowledge Management](#knowledge-management)
//! - [Privacy Features](#privacy-features)
//! - [Hardware Manifests](#hardware-manifests)
//! - [Model Management](#model-management)

## Getting Started

### Initialization

```bash
# Initialize SuperInstance (downloads models, creates config)
synesis init

# Check system status
synesis status

# View hardware capabilities
synesis manifest detect
```

### Basic Query

```bash
# Ask a simple question
synesis ask "What is the capital of France?"

# Ask with verbose output (see agent reasoning)
synesis ask --verbose "Explain quantum computing"

# Ask with JSON output
synesis ask --format json "What is 2+2?"
```

## Local Queries

### Consensus Process

```bash
# Run a query through the tripartite council
synesis ask "How do I implement a binary search tree?"

# See what each agent thinks
synesis ask --verbose "Design a REST API"
```

### Knowledge-Aware Queries

```bash
# Query with local knowledge only
synesis ask --local "What did we decide about the database schema?"

# Query specific knowledge sources
synesis ask --knowledge docs/specs --local "Summarize the API spec"
```

## Cloud Escalation

### Force Cloud Processing

```bash
# Escalate to cloud (bypass local processing)
synesis ask --cloud "Explain the theory of relativity"

# Cloud with specific model
synesis ask --cloud --model claude-opus "Compare Python and Rust"

# Streaming response from cloud
synesis ask --cloud --stream "Tell me a story"
```

### Cost-Aware Queries

```bash
# Check your balance before querying
synesis cloud balance

# View recent usage
synesis cloud usage --days 7

# Query with cost estimate shown
synesis ask --cloud "What is machine learning?"
```

## Knowledge Management

### Index Your Project

```bash
# Index current directory
synesis knowledge index .

# Index specific directory
synesis knowledge index ~/projects/myapp

# Index with specific chunk size
synesis knowledge index . --chunk-size 1000

# Auto-index directory (watch for changes)
synesis knowledge watch ~/projects/myapp
```

### Search Knowledge

```bash
# Search indexed knowledge
synesis ask "What does the User model do?"

# Search with RAG
synesis ask --knowledge ~/projects/myapp "How is authentication handled?"
```

### Manage Knowledge Vault

```bash
# List indexed files
synesis knowledge list

# Remove file from vault
synesis knowledge remove ~/projects/myapp/src/deprecated.rs

# Get vault statistics
synesis knowledge stats
```

## Privacy Features

### Redaction Patterns

SuperInstance automatically redacts sensitive information before cloud transmission:

- **Email addresses**: `user@example.com` → `[EMAIL_0001]`
- **Phone numbers**: `(555) 123-4567` → `[PHONE_0001]`
- **API keys**: `sk-abc123...` → `[APIKEY_0001]`
- **SSNs**: `123-45-6789` → `[SSN_0001]`
- **Credit cards**: `4111...` → `[CARD_0001]`
- **IP addresses**: `192.168.1.1` → `[IP_0001]`
- **File paths**: `/etc/passwd` → `[PATH_0001]`
- **URLs with tokens**: `https://api.example.com?token=xxx` → `[URL_0001]`

### See What Was Redacted

```bash
# Show redaction details
synesis ask --show-redactions "My email is test@example.com"

# Output will show:
#   Original: My email is test@example.com
#   Redacted: My email is [EMAIL_0001]
#   Patterns found: 1
```

### Privacy Guarantees

- ✅ Sensitive data never leaves your device unredacted
- ✅ Token vault stored locally (SQLite)
- ✅ Tokens are session-specific (prevent correlation)
- ✅ Constant-time reinflation (prevents timing attacks)
- ✅ Zero knowledge of your data on cloud servers

## Hardware Manifests

### Check Your Hardware

```bash
# Detect hardware capabilities
synesis manifest detect

# Show current manifest
synesis manifest show

# List available manifests
synesis manifest list
```

### Creating Custom Manifests

See `docs/architecture/hardware-manifests.md` for details on creating custom hardware manifests for your platform.

## Model Management

### List Available Models

```bash
# List local models
synesis model list

# List cloud models
synesis model list --cloud

# Get model info
synesis model info claude-sonnet

# Verify model integrity
synesis model verify claude-sonnet
```

### Download Models

```bash
# Download a model
synesis model download claude-sonnet

# Download to specific location
synesis model download claude-sonnet --path ~/models

# Download with progress bar
synesis model download claude-sonnet --progress
```

### Remove Models

```bash
# Remove a model
synesis model remove claude-sonnet

# Remove all models
synesis model remove --all
```

## Cloud Management

### Authentication

```bash
# Login with API key
synesis cloud login sk-abc123...

# Interactive login
synesis cloud login

# Device code flow (for headless systems)
synesis cloud login --device

# Logout
synesis cloud logout
```

### Account Management

```bash
# Check account status
synesis cloud status

# View balance
synesis cloud balance

# Add credits
synesis cloud topup 10.00

# View usage
synesis cloud usage

# Usage for specific period
synesis cloud usage --days 30
synesis cloud usage --period week
```

### Connection Testing

```bash
# Test cloud connection
synesis cloud ping

# Sync settings with cloud
synesis cloud sync
```

## Collaboration

### Create Invites

```bash
# Create invite with defaults
synesis invite create --project my-project --email user@example.com

# Create invite with specific role
synesis invite create --project my-project --role editor --email user@example.com

# Create invite with quota
synesis invite create --project my-project --quota 50.00 --email user@example.com

# Create invite with expiration
synesis invite create --project my-project --expires-hours 48 --email user@example.com
```

### Manage Invites

```bash
# List all invites
synesis invite list

# Revoke an invite
synesis invite revoke abc-123-def-456
```

## LoRA Management

### Upload LoRA to Cloud

```bash
# Upload LoRA
synesis push --file my-lora.safetensors --name "My LoRA" --base-model claude-sonnet

# Upload with description
synesis push --file my-lora.safetensors --name "My LoRA" --base-model claude-sonnet \
  --description "Fine-tuned for technical documentation"
```

### Use LoRA in Queries

```bash
# Use LoRA in cloud query
synesis ask --cloud --lora my-lora-id "Explain Rust ownership"

# List available LoRAs
synesis model list --cloud
```

## Metrics and Monitoring

### View Metrics

```bash
# Show all metrics
synesis metrics show

# Export metrics (Prometheus format)
synesis metrics export

# Monitor in real-time
watch -n 5 'synesis metrics show'
```

### Available Metrics

- **Queries**: Total, successful, failed
- **Consensus**: Rounds reached, failures
- **Agents**: Pathos, Logos, Ethos performance
- **Knowledge**: Indexed documents, searches performed
- **Privacy**: Redactions performed, tokens generated
- **Cloud**: Escalations, costs, latency

## Advanced Usage

### Configuration

```bash
# Use custom config file
synesis --config ~/.config/synesis/custom.json ask "query"

# Set config value
synesis config set cloud.endpoint api.example.com

# Get config value
synesis config get cloud.endpoint

# Edit config
synesis config edit
```

### Environment Variables

```bash
# Set log level
export RUST_LOG=debug
synesis ask "query"

# Enable tracing
export RUST_LOG=trace
synesis ask "query"
```

### Performance Tuning

```bash
# Adjust consensus threshold (default: 0.85)
export SYNESIS_CONSENSUS_THRESHOLD=0.90
synesis ask "complex query"

# Adjust max consensus rounds (default: 3)
export SYNESIS_MAX_ROUNDS=5
synesis ask "ambiguous query"

# Set timeout for cloud requests
export SYNESIS_CLOUD_TIMEOUT=30
synesis ask --cloud "slow query"
```

## Troubleshooting

### Debug Mode

```bash
# Enable verbose output
synesis --verbose ask "query"

# Enable debug logging
RUST_LOG=debug synesis ask "query"

# Trace query execution
RUST_LOG=trace synesis ask "query"
```

### Common Issues

**Models not loading**:
```bash
# Re-download models
synesis model download --force claude-sonnet

# Check model integrity
synesis model verify claude-sonnet
```

**Cloud connection issues**:
```bash
# Test connection
synesis cloud ping

# Re-authenticate
synesis cloud logout
synesis cloud login
```

**Knowledge not found**:
```bash
# Re-index directory
synesis knowledge index --force .

# Check vault status
synesis knowledge list
```

## Tips and Best Practices

### Performance

1. **Use local when possible**: Local queries are faster and free
2. **Index your knowledge**: Reduces need for cloud escalation
3. **Chunk large documents**: Improves RAG accuracy
4. **Watch important directories**: Auto-indexing keeps knowledge fresh

### Privacy

1. **Review redactions**: Use `--show-redactions` to verify what's being redacted
2. **Use session tokens**: Tokens expire after each session for privacy
3. **Check before sharing**: Verify no sensitive data in responses

### Cost Management

1. **Check balance regularly**: `synesis cloud balance`
2. **Monitor usage**: `synesis cloud usage --days 7`
3. **Use local first**: Save cloud for complex queries
4. **Knowledge credits**: Contribute to earn credits

### Collaboration

1. **Set appropriate quotas**: Don't give unlimited access
2. **Use time-limited invites`: Auto-expiration for security
3. **Revoke when done**: Clean up old invites
4. **Check invite status**: `synesis invite list`

## Next Steps

- Read [Architecture Guide](../ARCHITECTURE.md)
- Explore [Project Roadmap](../PROJECT_ROADMAP.md)
- Contribute to [Community Hardware Manifests](../docs/contributing/hardware-manifests.md)
- Join the [Discussions](https://github.com/SuperInstance/Tripartite1/discussions)
