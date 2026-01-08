# SuperInstance AI - Code Examples

This directory contains runnable code examples demonstrating various aspects of SuperInstance AI.

---

## Directory Structure

```
examples/
├── basic/           # Simple usage patterns
├── knowledge/       # Knowledge vault and RAG
├── privacy/         # Privacy proxy and redaction
├── advanced/        # Custom agents and integration
└── cloud/           # Cloud integration (Phase 2)
```

---

## Running Examples

### Run from Source

```bash
# Navigate to example directory
cd examples/basic

# Compile and run
cargo run --example hello_world

# Or compile only
cargo build --example hello_world
./target/debug/examples/hello_world
```

### Run with Cargo Directly

```bash
# From repository root
cargo run --example hello_world
cargo run --example semantic_search
```

---

## Examples by Category

### 📦 Basic ([`basic/`](basic/))

Simple examples to get you started:

| Example | Description |
|---------|-------------|
| [`hello_world.rs`](basic/hello_world.rs) | Run your first AI query |
| [`custom_config.rs`](basic/custom_config.rs) | Use custom configuration |
| [`batch_queries.rs`](basic/batch_queries.rs) | Process multiple queries |

**Prerequisites**: Complete [Getting Started](../docs/tutorials/getting-started.md) tutorial

### 🧠 Knowledge ([`knowledge/`](knowledge/))

Working with the knowledge vault and RAG:

| Example | Description |
|---------|-------------|
| [`add_documents.rs`](knowledge/add_documents.rs) | Add documents to knowledge base |
| [`semantic_search.rs`](knowledge/semantic_search.rs) | Perform semantic searches |
| [`custom_chunker.rs`](knowledge/custom_chunker.rs) | Custom chunking strategies |

**Prerequisites**: Understand [Knowledge Vault](../docs/tutorials/knowledge-vault.md)

### 🔒 Privacy ([`privacy/`](privacy/))

Privacy and redaction features:

| Example | Description |
|---------|-------------|
| [`custom_patterns.rs`](privacy/custom_patterns.rs) | Create custom redaction patterns |
| [`token_vault.rs`](privacy/token_vault.rs) | Manage token vault |
| [`reinflate.rs`](privacy/reinflate.rs) | Re-inflate redacted responses |

**Prerequisites**: Read [Privacy Basics](../docs/tutorials/privacy-basics.md)

### 🚀 Advanced ([`advanced/`](advanced/))

Advanced usage and customization:

| Example | Description |
|---------|-------------|
| [`custom_agent.rs`](advanced/custom_agent.rs) | Build a custom agent |
| [`consensus_config.rs`](advanced/consensus_config.rs) | Configure consensus engine |
| [`integration.rs`](advanced/integration.rs) | Use SuperInstance as a library |

**Prerequisites**: Study [API Reference](../docs/api/)

### ☁️ Cloud ([`cloud/`](cloud/))

Cloud integration features (Phase 2):

| Example | Description |
|---------|-------------|
| [`cloud_escalation.rs`](cloud/cloud_escalation.rs) | Escalate queries to cloud |
| [`streaming.rs`](cloud/streaming.rs) | Stream cloud responses |
| [`lora_upload.rs`](cloud/lora_upload.rs) | Upload and manage LoRAs |

**Prerequisites**: Phase 2 complete

---

## Example Standards

All examples follow these standards:

### ✅ Characteristics

- **Runnable**: Every example compiles and runs
- **Self-contained**: Each example is complete and independent
- **Well-commented**: Code explains what's happening
- **Best practices**: Demonstrates recommended patterns
- **Error handling**: Shows proper error handling

### 📝 Structure

Each example includes:

1. **File header**: Description and prerequisites
2. **Imports**: Clear and organized
3. **Main logic**: Well-commented code
4. **Output**: Shows expected results
5. **Related docs**: Links to documentation

### Example Template

```rust
//! Example Name - Brief description
//!
//! This example demonstrates [what it does].
//!
//! # Prerequisites
//!
//! - [Requirement 1]
//! - [Requirement 2]
//!
//! # Related Documentation
//!
//! - [Link to relevant docs]
//!
//! # Expected Output
//!
//! ```text
//! [Show what the output looks like]
//! ```

use synesis_core::{Council, CouncilConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize
    let mut council = Council::new(CouncilConfig::default());
    council.initialize().await?;

    // Do something
    let response = council.process(/* ... */).await?;
    println!("{}", response.content);

    Ok(())
}
```

---

## Contributing Examples

Want to add an example? Great!

### Guidelines

1. **Keep it simple**: Focus on one concept per example
2. **Make it runnable**: Test before submitting
3. **Add comments**: Explain the "why", not just the "what"
4. **Update docs**: Link from relevant documentation
5. **Follow template**: Use the example template above

### Adding a New Example

1. Create the example file in appropriate directory
2. Add it to this README's table of contents
3. Test it works: `cargo run --example your_example`
4. Add documentation links
5. Submit PR

### Example PR Checklist

- [ ] Example compiles without warnings
- [ ] Example runs successfully
- [ ] Code is well-commented
- [ ] Expected output is documented
- [ ] Linked from relevant docs
- [ ] Added to README table

---

## Testing Examples

All examples are tested as part of the test suite:

```bash
# Test all examples
cargo test --examples

# Test specific example
cargo test --example hello_world

# Run example with clippy
cargo clippy --example hello_world -- -D warnings
```

---

## Need Help?

- Example not working? Check [Troubleshooting](../docs/reference/troubleshooting.md)
- Want to understand more? Read [Tutorials](../docs/tutorials/)
- Found a bug? [Open an issue](https://github.com/SuperInstance/Tripartite1/issues)

---

**Examples Version**: v0.2.0
**Last Updated**: 2026-01-07
