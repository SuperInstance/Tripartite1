# SuperInstance AI - Developer Guide

Welcome to the SuperInstance AI development community! This guide will help you become a productive contributor.

---

## Table of Contents

1. [Welcome](#welcome)
2. [Development Setup](#development-setup)
3. [Project Structure](#project-structure)
4. [Development Workflow](#development-workflow)
5. [Code Organization](#code-organization)
6. [Testing Strategy](#testing-strategy)
7. [Documentation Standards](#documentation-standards)
8. [Code Review Process](#code-review-process)
9. [Release Process](#release-process)
10. [Getting Help](#getting-help)

---

## Welcome

### What It's Like to Contribute

SuperInstance is a **privacy-first, local-first AI system** built in Rust. Contributing means:

- **Working with cutting-edge tech**: Rust, async/await, QUIC, vector databases
- **Solving real problems**: Privacy, local AI, knowledge management
- **Building for users**: Developers, researchers, writers, everyone
- **Learning and growing**: Tripartite consensus, agent systems, RAG
- **Great community**: Collaborative, supportive, technical

### Our Values

- **Privacy First**: User data protection is non-negotiable
- **Local First**: Keep processing on-device when possible
- **Quality Over Speed**: Get it right, then make it fast
- **Transparency**: Show users how the system works
- **Accessibility**: Make AI available to everyone

### Expectations

- **Be respectful**: Treat everyone with kindness and professionalism
- **Be thorough**: Test your changes, document your code
- **Be patient**: Reviews may take time, complex problems need discussion
- **Be collaborative**: Ask questions, seek feedback, help others

---

## Development Setup

### Prerequisites

**Required**:
- Rust 1.75+ ([install via rustup](https://rustup.rs/))
- Git
- C compiler (gcc/clang) and OpenSSL headers

**Recommended**:
- VS Code with [rust-analyzer](https://rust-analyzer.github.io/)
- 16GB RAM (8GB minimum)
- Ubuntu 22.04+ / macOS 12+ / Windows 10+

### Step 1: Fork and Clone

```bash
# Fork the repository on GitHub
# Then clone your fork

git clone https://github.com/YOUR_USERNAME/Tripartite1.git
cd Tripartite1

# Add upstream remote
git remote add upstream https://github.com/SuperInstance/Tripartite1.git
```

### Step 2: Install Development Tools

```bash
# Install rustfmt (code formatting)
rustup component add rustfmt

# Install clippy (linter)
rustup component add clippy

# Verify installation
rustfmt --version
clippy --version
```

### Step 3: Build the Project

```bash
# Development build (faster compilation)
cargo build

# Release build (optimized, slower compilation)
cargo build --release

# Run tests to verify setup
cargo test --workspace
```

### Step 4: Set Up Pre-commit Hooks (Optional)

```bash
# Install pre-commit (requires Python)
pip install pre-commit

# Install hooks
pre-commit install
```

### Step 5: Verify Your Setup

```bash
# Run all tests
cargo test --workspace

# Check for warnings
cargo clippy --all -- -D warnings

# Format check
cargo fmt --all -- --check

# Run a basic query
cargo run -- synesis ask "What is 2+2?"
```

**Expected**: All tests pass, no warnings, query succeeds

---

## Project Structure

### Workspace Organization

```
Tripartite1/
├── crates/                    # Rust workspace (6 crates)
│   ├── synesis-cli/           # Command-line interface
│   ├── synesis-core/          # Tripartite council & consensus
│   ├── synesis-knowledge/     # Knowledge vault & RAG
│   ├── synesis-models/        # Hardware detection & models
│   ├── synesis-privacy/       # Privacy proxy & redaction
│   └── synesis-cloud/         # Cloud connectivity (Phase 2)
│
├── docs/                      # User documentation
│   ├── tutorials/             # Step-by-step tutorials
│   ├── guides/                # In-depth guides
│   ├── reference/             # Glossary, FAQ
│   └── contributing/          # Contributor resources
│
├── examples/                  # Runnable code examples
│   ├── basic/                 # Simple usage patterns
│   ├── knowledge/             # RAG examples
│   ├── privacy/               # Privacy examples
│   ├── advanced/              # Custom agents, integration
│   └── cloud/                 # Cloud examples (Phase 2)
│
├── sessions/                  # Development session reports
├── status/                    # Build and completion reports
├── phases/                    # Phase planning documents
│   └── phase2/                # Phase 2 detailed specs
│
├── tests/                     # Integration tests
├── manifests/                 # Hardware profiles
├── architecture/              # Architecture docs
├── agents/                    # Agent documentation
│
├── Cargo.toml                 # Workspace configuration
├── README.md                  # User-facing overview
├── ARCHITECTURE.md            # System architecture
├── DEVELOPER_GUIDE.md         # This file
├── CLAUDE.md                  # Development methodology
└── CONTRIBUTING.md            # Contribution guidelines
```

### Crate Responsibilities

| Crate | Purpose | Lines of Code | Tests |
|-------|---------|---------------|-------|
| **synesis-core** | Agent orchestration, consensus | ~2,500 | 85 |
| **synesis-knowledge** | Vector DB, RAG, embeddings | ~1,800 | 28 |
| **synesis-privacy** | Redaction, token vault | ~1,200 | 37 |
| **synesis-models** | Hardware detection, models | ~800 | 12 |
| **synesis-cli** | Command-line interface | ~1,500 | 7 |
| **synesis-cloud** | QUIC tunnel, escalation | ~3,000 | 68 |

---

## Development Workflow

### Ralph Wiggum Methodology

We use the **Ralph Wiggum** methodology: persistent iteration toward completion.

```
while task not complete:
    implement()
    test()
    fix_bugs()
    verify()
    repeat()
```

### Phase Development

For Phase 2, we work through sessions sequentially:

1. **Read the roadmap**: `phases/PHASE_2_DETAILED_ROADMAP.md`
2. **Study session requirements**: Understand acceptance criteria
3. **Implement**: Write code according to specs
4. **Test frequently**: `cargo test --workspace` after changes
5. **Fix immediately**: Don't accumulate bugs
6. **Document**: Add doc comments as you code
7. **Verify**: Check all acceptance criteria met
8. **Complete**: Mark session done, move to next

### Feature Development

```bash
# 1. Create feature branch
git checkout -b feature/your-feature-name

# 2. Make your changes
# Edit files, add features, fix bugs

# 3. Test frequently
cargo test --workspace
cargo clippy --all -- -D warnings

# 4. Format your code
cargo fmt --all

# 5. Commit your changes
git add .
git commit -m "feat: add your feature description"

# 6. Push to your fork
git push origin feature/your-feature-name

# 7. Create pull request on GitHub
```

### Commit Message Conventions

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

**Examples**:
```
feat(core): add parallel agent execution

Agents now run concurrently using tokio::join!,
reducing query latency by 25-33%.

Closes #123
```

```
fix(privacy): prevent token reuse across sessions

Token vault now generates unique tokens per session
instead of reusing global counters.

Fixes #456
```

---

## Code Organization

### Module Structure

Each crate follows this pattern:

```rust
//! Crate-level documentation
//!
//! Overview of what this crate does and how it fits in the system.

// Re-exports for convenience
pub use important_types::PublicAPI;

// Private modules
mod internal_implementation;

// Public modules
pub mod public_api;
```

### Naming Conventions

**Types**: `PascalCase`
```rust
struct ConsensusEngine { }
enum TunnelState { }
trait Agent { }
type Result<T> = std::result::Result<T, Error>;
```

**Functions**: `snake_case`
```rust
fn process_query() { }
async fn fetch_data() { }
```

**Constants**: `SCREAMING_SNAKE_CASE`
```rust
const MAX_ROUNDS: u32 = 3;
const DEFAULT_THRESHOLD: f32 = 0.85;
```

**Acronyms**: Keep readable
```rust
HttpApi // not HTTPAPI
QuicTunnel // not QUITunnel
SqlDb // not SQLDB
```

### Thread Safety Patterns

**Rule 1**: Use `Arc<tokio::sync::Mutex<T>>` for shared mutable state in async code

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

let shared = Arc::new(Mutex::new(MyData::new()));

// In async task
let lock = shared.lock().await;
// Do work with lock
drop(lock); // Release before .await
async_operation().await;
```

**Rule 2**: Use `Arc<AtomicU64>` for lock-free metrics

```rust
use std::sync::atomic::{AtomicU64, Ordering};

metrics.queries_total.fetch_add(1, Ordering::Relaxed);
```

**Rule 3**: Use `Arc<Vec<T>>` for immutable collections (no lock needed)

```rust
let patterns = Arc::new(vec![pattern1, pattern2]);
// Clone freely, no locking needed
let patterns_clone = Arc::clone(&patterns);
```

### Error Handling

Use our unified error type:

```rust
use synesis_core::SynesisError;
use synesis_core::SynesisResult;

fn do_something() -> SynesisResult<Value> {
    let value = expensive_operation()
        .map_err(|e| SynesisError::processing(format!("Failed: {}", e)))?;

    Ok(value)
}
```

---

## Testing Strategy

### Test Organization

```rust
// Unit test (in same file as code)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consensus_threshold() {
        let engine = ConsensusEngine::default();
        assert!(engine.reached_consensus(0.90));
    }
}

// Integration test (in tests/ directory)
#[tokio::test]
async fn test_full_consensus_round() {
    let mut council = Council::new(CouncilConfig::default());
    council.initialize().await.unwrap();

    let response = council.process(manifest).await.unwrap();
    assert!(response.confidence > 0.85);
}
```

### Test Categories

1. **Unit Tests**: Test individual functions and types
2. **Integration Tests**: Test component interactions
3. **Doc Tests**: Test code examples in documentation

### Writing Good Tests

**DO**:
- ✅ Test one thing per test
- ✅ Use descriptive test names
- ✅ Test both success and failure cases
- ✅ Use `assert!`, `assert_eq!`, `assert_ne!`
- ✅ Add tests for bug fixes

**DON'T**:
- ❌ Test multiple things in one test
- ❌ Use vague names like `test_it_works`
- ❌ Skip error cases
- ❌ Write brittle tests (depends on exact timing, etc.)

### Test-Driven Development

```bash
# 1. Write failing test
# 2. Run test: cargo test
# 3. Write minimal code to pass
# 4. Run test: cargo test
# 5. Refactor
# 6. Run test: cargo test
# 7. Repeat
```

### Running Tests

```bash
# All tests
cargo test --workspace

# Specific crate
cargo test -p synesis-core

# Specific test
cargo test test_consensus_threshold

# With output
cargo test -- --nocapture

# Run doc tests
cargo test --doc
```

---

## Documentation Standards

### Rustdoc Comments

Every public API must have documentation:

```rust
/// Process a query through the tripartite council.
///
/// This function orchestrates the three-agent consensus process:
/// 1. Pathos extracts intent
/// 2. Logos performs reasoning
/// 3. Ethos verifies accuracy
/// 4. Consensus engine evaluates agreement
///
/// # Arguments
///
/// * `manifest` - Query manifest with conversation state
///
/// # Returns
///
/// Returns a `CouncilResponse` containing the consensus response.
///
/// # Errors
///
/// Returns an error if:
/// - Agents are not initialized
/// - Consensus cannot be reached
/// - Model inference fails
///
/// # Examples
///
/// ```
/// use synesis_core::{Council, CouncilConfig, A2AManifest};
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut council = Council::new(CouncilConfig::default());
/// council.initialize().await?;
///
/// let manifest = A2AManifest::new("Explain Rust ownership".to_string());
/// let response = council.process(manifest).await?;
///
/// println!("Response: {}", response.content);
/// # Ok(())
/// # }
/// ```
pub async fn process(&mut self, manifest: A2AManifest) -> SynesisResult<CouncilResponse> {
    // ...
}
```

### Module Documentation

Each module should have module-level docs:

```rust
//! Tripartite consensus engine
//!
//! This module implements the multi-round consensus mechanism where
//! three specialized agents (Pathos, Logos, Ethos) deliberate to reach
//! agreement on query responses.
//!
//! # Consensus Process
//!
//! 1. Each agent processes the query independently
//! 2. Votes are collected and weighted
//! 3. If consensus ≥ threshold, return response
//! 4. Otherwise, enter revision round
//! 5. Repeat until consensus or max rounds
//!
//! # Example
//!
//! ```rust,no_run
//! use synesis_core::ConsensusEngine;
//!
//! let engine = ConsensusEngine::default();
//! let outcome = engine.deliberate(agents, query).await?;
//! ```
```

### Documentation Checklist

Before committing, verify:

- [ ] All new public functions have doc comments
- [ ] Doc comments include: Purpose, Arguments, Returns, Errors, Examples
- [ ] Examples compile and run as tests
- [ ] Complex logic has inline comments
- [ ] Module has module-level documentation

---

## Code Review Process

### Before Submitting

1. **Test**: `cargo test --workspace` passes
2. **Lint**: `cargo clippy --all -- -D warnings` shows no issues
3. **Format**: `cargo fmt --all` applied
4. **Docs**: All new APIs documented
5. **Build**: `cargo build --release` succeeds

### Creating a Pull Request

1. **Title**: Use conventional commit format
   - Good: `feat(core): add parallel agent execution`
   - Bad: `update stuff`

2. **Description**: Include:
   - **What**: Summary of changes
   - **Why**: Motivation for the change
   - **How**: Implementation approach
   - **Testing**: How you tested it
   - **Closes**: Issue number (if applicable)

3. **Checklist**:
   - [ ] Tests pass locally
   - [ ] Added tests for new features
   - [ ] Updated documentation
   - [ ] Added examples (if applicable)
   - [ ] Commit messages follow conventions

### During Review

- **Be responsive**: Address review comments promptly
- **Be patient**: Reviews may take time
- **Be open**: Consider feedback constructively
- **Ask questions**: If something is unclear

### After Approval

1. **Squash commits** (if requested): Combine related commits
2. **Update branch**: `git fetch upstream && git rebase upstream/main`
3. **Merge**: Use "Squash and merge" button
4. **Delete branch**: Clean up after merge

---

## Release Process

### Version Numbers

We use Semantic Versioning: `MAJOR.MINOR.PATCH`

- **MAJOR**: Breaking changes (0.x → 1.0)
- **MINOR**: New features, backward compatible (0.1 → 0.2)
- **PATCH**: Bug fixes (0.2.0 → 0.2.1)

### Pre-Release Checklist

- [ ] All tests passing
- [ ] Zero compiler warnings
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Version bumped in Cargo.toml
- [ ] Tagged in git

### Creating a Release

```bash
# 1. Update version in Cargo.toml
# 2. Update CHANGELOG.md
# 3. Commit changes
git add Cargo.toml CHANGELOG.md
git commit -m "chore: release v0.2.0"

# 4. Create tag
git tag -a v0.2.0 -m "Release v0.2.0: Phase 1 complete, Phase 2 in progress"

# 5. Push to GitHub
git push upstream main
git push upstream v0.2.0

# 6. Create GitHub release with release notes
```

---

## Getting Help

### Resources

- **[ARCHITECTURE.md](ARCHITECTURE.md)** - System design deep dive
- **[CLAUDE.md](CLAUDE.md)** - Development methodology
- **[THREAD_SAFETY_PATTERNS.md](THREAD_SAFETY_PATTERNS.md)** - Concurrency patterns
- **[API Documentation](https://docs.rs/synesis-core/)** - Rust API reference

### Community

- **[GitHub Issues](https://github.com/SuperInstance/Tripartite1/issues)** - Bug reports, feature requests
- **[GitHub Discussions](https://github.com/SuperInstance/Tripartite1/discussions)** - Questions, ideas
- **[Documentation](docs/)** - Tutorials, guides, reference

### Asking Good Questions

When asking for help:

1. **Search first**: Check docs, issues, discussions
2. **Be specific**: Include error messages, stack traces
3. **Show your work**: What have you tried?
4. **Provide context**: OS, Rust version, crate version
5. **Format code**: Use markdown code blocks

**Example**:
```
Hi! I'm working on adding a new agent and getting this error:

```
error[E0277]: trait bound `MyAgent: Agent` is not satisfied
  --> src/agents/my_agent.rs:25:10
   |
25 |     let agents: Vec<Box<dyn Agent>> = vec![my_agent];
   |              ^^^^^^^^^^^^^^^^^^^^^^^^^^^ the trait `Agent` is not implemented for `MyAgent`
```

I've tried:
1. Implementing the `Agent` trait
2. Using `Box<MyAgent>` instead
3. Reading the agent documentation

Rust version: 1.75.0
OS: Ubuntu 22.04

What am I missing?
```

---

## Contributing Quick Reference

| Want to...? | Go to... |
|-------------|----------|
| Report a bug | [GitHub Issues](https://github.com/SuperInstance/Tripartite1/issues) |
| Request feature | [GitHub Issues](https://github.com/SuperInstance/Tripartite1/issues) |
| Ask question | [GitHub Discussions](https://github.com/SuperInstance/Tripartite1/discussions) |
| Submit PR | [Pull Requests](https://github.com/SuperInstance/Tripartite1/pulls) |
| Update docs | See [Documentation Standards](#documentation-standards) |
| Add tests | See [Testing Strategy](#testing-strategy) |

---

## Code of Conduct

### Our Pledge

In the interest of fostering an open and welcoming environment, we pledge to make participation in our project and our community a harassment-free experience for everyone.

### Our Standards

**Positive behavior**:
- Using welcoming and inclusive language
- Being respectful of differing viewpoints and experiences
- Gracefully accepting constructive criticism
- Focusing on what is best for the community
- Showing empathy towards other community members

**Unacceptable behavior**:
- Harassment, trolling, or offensive language
- Personal attacks or derogatory comments
- Public or private harassment
- Publishing others' private information
- Any other unethical or unprofessional conduct

### Reporting Issues

Contact the project maintainers through [GitHub Issues](https://github.com/SuperInstance/Tripartite1/issues) with any concerns.

---

**Welcome to the SuperInstance AI development community!** We're excited to have you.

**Version**: 0.2.0
**Last Updated**: 2026-01-07
**Maintained By**: SuperInstance AI Team
