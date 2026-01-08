# Contributing to SuperInstance AI

Thank you for your interest in contributing to SuperInstance AI! We're building a privacy-first, tripartite agentic AI system, and we value every contribution.

**Table of Contents**

- [Quick Start](#quick-start)
- [Development Environment Setup](#development-environment-setup)
- [Development Workflow](#development-workflow)
- [Code Standards](#code-standards)
- [Testing Guidelines](#testing-guidelines)
- [Documentation Standards](#documentation-standards)
- [Submitting Changes](#submitting-changes)
- [Community Guidelines](#community-guidelines)
- [Getting Help](#getting-help)

---

## Quick Start

```bash
# Clone the repository
git clone https://github.com/SuperInstance/Tripartite1.git
cd Tripartite1

# Install dependencies and run tests
cargo build --workspace
cargo test --workspace

# Make your changes
# ... edit files ...

# Run tests and linting
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check

# Submit a pull request
```

---

## Development Environment Setup

### Prerequisites

**Required:**
- Rust 1.75+ (stable toolchain)
- Git 2.30+
- SQLite 3.35+

**Recommended:**
- 8GB+ RAM
- 4+ CPU cores
- SSD storage
- GPU with NVIDIA/AMD support (for local inference)

**Optional (for cloud development):**
- Node.js 20+ (for Cloudflare Workers development)
- Wrangler CLI (`npm install -g wrangler`)
- Cloudflare account (for deployment testing)

### Installing Rust

```bash
# Install Rust using rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Configure stable toolchain
rustup default stable
rustup update

# Verify installation
rustc --version
cargo --version
```

### Repository Setup

```bash
# Clone with SSH (recommended if you have SSH keys configured)
git clone git@github.com:SuperInstance/Tripartite1.git

# Or clone with HTTPS
git clone https://github.com/SuperInstance/Tripartite1.git

# Enter the directory
cd Tripartite1

# Install pre-commit hooks (optional)
cargo install pre-commit
pre-commit install

# Build all crates in development mode
cargo build --workspace

# Run all tests to verify setup
cargo test --workspace
```

### IDE Configuration

**VS Code (Recommended):**
1. Install the **rust-analyzer** extension
2. Install the **CodeLLDB** extension for debugging
3. Configure workspace settings:
   ```json
   {
     "rust-analyzer.cargo.loadOutDirsFromCheck": true,
     "rust-analyzer.cargo.features": "all",
     "rust-analyzer.checkOnSave.command": "clippy"
   }
   ```

**IntelliJ IDEA / CLion:**
1. Install the **Rust** plugin
2. Enable external LLM: Settings → Languages & Frameworks → Rust → Rustfmt
3. Use cargo as build system

**Vim/Neovim:**
- Install `rust-analyzer` via your plugin manager
- Configure LSP with `nvim-lspconfig` or `coc-rust-analyzer`

---

## Development Workflow

### The Ralph Wiggum Methodology

SuperInstance follows the **Ralph Wiggum methodology** - persistent iteration on each component until completion. This means:

1. **Work Sequentially** - Complete one task at a time in order
2. **Test Thoroughly** - Don't move on until all tests pass
3. **Fix Bugs Immediately** - When tests fail, debug and fix them
4. **Document Progress** - Keep clear records of what you've done
5. **Verify Acceptance** - Ensure all acceptance criteria are met

> "Ralph is a Bash loop" - Iterate on each component until it's complete, then move forward.

### Workflow Steps

#### 1. Find Something to Work On

- Check [GitHub Issues](https://github.com/SuperInstance/Tripartite1/issues) for open tasks
- Look for labels: `good first issue`, `help wanted`, `documentation`
- Read the [Phase Roadmap](PROJECT_ROADMAP.md) to understand context
- Comment on the issue to claim it (avoid duplicate work)

#### 2. Create a Branch

```bash
# Ensure you're on main and up to date
git checkout main
git pull origin main

# Create a feature branch
git checkout -b feature/your-feature-name

# Or a bug fix branch
git checkout -b fix/issue-number-brief-description

# Or a documentation branch
git checkout -b docs/what-youre-documenting
```

**Branch naming convention:**
- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation changes
- `refactor/` - Code refactoring
- `test/` - Test improvements
- `chore/` - Maintenance tasks

#### 3. Make Your Changes

- Write code following our [Code Standards](#code-standards)
- Add tests for new functionality (see [Testing Guidelines](#testing-guidelines))
- Update documentation as needed
- Keep commits atomic and well-described

#### 4. Commit Your Changes

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
type(scope): description

[optional body]

[optional footer]
```

**Types:**
- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation changes
- `style` - Code style changes (formatting, no logic change)
- `refactor` - Code refactoring
- `test` - Adding or updating tests
- `chore` - Maintenance tasks
- `perf` - Performance improvements

**Examples:**
```bash
git commit -m "feat(core): implement consensus engine voting mechanism"
git commit -m "fix(privacy): correct email regex pattern matching"
git commit -m "docs(readme): update installation instructions for Windows"
git commit -m "test(knowledge): add tests for document chunking edge cases"
```

#### 5. Test Your Changes

```bash
# Run all tests
cargo test --workspace

# Run tests with output
cargo test --workspace -- --nocapture

# Run tests with logging
RUST_LOG=debug cargo test --workspace

# Test specific crate
cargo test --package synesis-core

# Test specific module
cargo test --package synesis-core consensus

# Run with sanitizer (nightly only)
cargo +nightly test -Z sanitizer=address
```

#### 6. Run Linting and Formatting

```bash
# Check formatting
cargo fmt --check

# Auto-format code
cargo fmt

# Run clippy (should produce no warnings)
cargo clippy --workspace --all-targets -- -D warnings

# Fix clippy suggestions
cargo clippy --workspace --all-targets --fix
```

#### 7. Submit a Pull Request

- Push your branch to GitHub:
  ```bash
  git push origin feature/your-feature-name
  ```

- Go to https://github.com/SuperInstance/Tripartite1 and create a PR
- Fill out the PR template (see below)
- Link related issues (e.g., "Fixes #123")
- Request review from maintainers

**Pull Request Template:**

```markdown
## Description
Brief description of changes made.

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update
- [ ] Refactoring
- [ ] Performance improvement

## Testing
- [ ] Tests added/updated
- [ ] All tests passing locally
- [ ] Manual testing performed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review performed
- [ ] Documentation updated
- [ ] No new warnings generated
- [ ] Commits follow conventional commits
```

---

## Code Standards

### Rust Code Style

**General Principles:**
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for all formatting (default settings)
- Use `clippy` for linting (zero warnings)
- Prefer clear, readable code over clever code
- Use descriptive names (no single-letter variables except loop counters)

**Naming Conventions:**
```rust
// Modules: snake_case
mod privacy_proxy;
mod consensus_engine;

// Types: PascalCase
struct PrivacyProxy;
struct ConsensusEngine;

// Functions: snake_case
fn redact_data(input: &str) -> String;

// Constants: SCREAMING_SNAKE_CASE
const MAX_RETRIES: u32 = 3;

// Generic types: PascalCase, single letter (T, E, K, V)
fn process<T: Display>(value: T) -> String {
    value.to_string()
}
```

**Error Handling:**
```rust
// Use Result for recoverable errors
pub fn process_request(input: &str) -> Result<Response, SynesisError> {
    // ...
}

// Use Option for optional values
pub fn find_agent(id: &str) -> Option<&Agent> {
    // ...
}

// Use ? for error propagation
pub fn load_config(path: &Path) -> Result<Config, SynesisError> {
    let content = fs::read_to_string(path)?;
    let config: Config = serde_json::from_str(&content)?;
    Ok(config)
}
```

**Documentation:**
```rust
/// Redacts sensitive information from text using pattern matching.
///
/// # Arguments
///
/// * `input` - The text to redact
/// * `patterns` - List of redaction patterns to apply
///
/// # Returns
///
/// Redacted text with sensitive information replaced by tokens
///
/// # Errors
///
/// Returns `SynesisError::InvalidPattern` if a pattern regex is invalid
///
/// # Examples
///
/// ```
/// use synesis_privacy::Redactor;
///
/// let redactor = Redactor::new();
/// let redacted = redactor.redact("My email is test@example.com")?;
/// assert_eq!(redacted, "My email is [EMAIL_01]");
/// # Ok::<(), SynesisError>(())
/// ```
pub fn redact(&self, input: &str) -> Result<String, SynesisError> {
    // implementation
}
```

**Thread Safety:**
```rust
// Use Arc for shared ownership across threads
use std::sync::Arc;

// Use tokio::sync::Mutex in async code (NOT std::sync::Mutex)
use tokio::sync::Mutex;

// Example: Shared state in async context
#[derive(Clone)]
pub struct SharedState {
    inner: Arc<Mutex<StateData>>,
}

// Use AtomicBool for simple boolean flags
use std::sync::atomic::{AtomicBool, Ordering};

pub struct Agent {
    ready: Arc<AtomicBool>,
}

// Use AtomicU64 for lock-free metrics
use std::sync::atomic::AtomicU64;

pub struct Metrics {
    requests_total: AtomicU64,
}
```

**Async/Await Best Practices:**
```rust
// NEVER hold MutexGuard across await points
// ❌ WRONG
let lock = mutex.lock().await;
async_function().await; // DEADLOCK!
drop(lock);

// ✅ CORRECT
let lock = mutex.lock().await;
let result = sync_operation(&lock);
drop(lock); // Release lock before await
async_function().await; // Safe now
```

### TypeScript Code Style (Cloud Workers)

**General:**
- Use TypeScript strict mode
- Use Prettier for formatting
- Use ESLint for linting
- Prefer functional programming patterns
- Use explicit return types

**Formatting:**
```bash
# Format TypeScript files
npm run format

# Check formatting
npm run format:check

# Run linter
npm run lint
```

### Commit Message Standards

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
type(scope): description

[optional body]

[optional footer]
```

**Scope values:**
- `core` - Core consensus engine
- `cli` - Command-line interface
- `privacy` - Privacy proxy
- `knowledge` - Knowledge vault
- `models` - Model management
- `cloud` - Cloudflare Workers
- `docs` - Documentation
- `tests` - Test infrastructure

**Examples:**
```
feat(core): implement weighted voting in consensus engine

Add weighted voting mechanism where Pathos, Logos, and Ethos
agents have different influence based on confidence scores.

Closes #123
```

```
fix(privacy): prevent token reuse across sessions

The token vault was not properly isolating tokens between sessions,
causing potential privacy leaks. This fix ensures each session
gets unique token identifiers.

Fixes #145
```

---

## Testing Guidelines

### Test Organization

**Unit Tests:**
- Place in the same file as the code
- Use `#[cfg(test)]` module
- Test public APIs and private helpers
- Use descriptive test names

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redaction_replaces_email_with_token() {
        let redactor = Redactor::new();
        let input = "Contact me at test@example.com";
        let result = redactor.redact(input).unwrap();
        assert!(result.contains("[EMAIL_"));
    }

    #[tokio::test]
    async fn test_async_consensus_reaches_threshold() {
        let engine = ConsensusEngine::test_setup().await;
        let result = engine.process(&query).await.unwrap();
        assert!(result.confidence >= 0.85);
    }
}
```

**Integration Tests:**
- Place in `tests/` directory
- Test component interactions
- Use `tokio::test` for async tests

```rust
// tests/integration/full_consensus_flow.rs
#[tokio::test]
async fn test_full_consensus_flow_with_privacy() {
    // Setup
    let council = Council::test_setup().await;

    // Execute
    let query = "What is the privacy policy?";
    let response = council.process(query).await.unwrap();

    // Verify
    assert!(response.confidence >= 0.85);
    assert!(response.content.contains("privacy"));
}
```

### Test Coverage Goals

**Critical Areas (100% coverage required):**
- Consensus engine voting logic
- Privacy redaction patterns
- Token generation and replacement
- Error handling paths
- Security-critical code

**High Coverage (80%+):**
- Agent implementations
- Knowledge vault operations
- Model routing decisions
- CLI command handlers

**Standard Coverage (60%+):**
- Utility functions
- Configuration parsing
- Logging/tracing code

### Test Best Practices

1. **Use Test Fixtures:**
   ```rust
   fn create_test_agent() -> PathosAgent {
       PathosAgent::new(test_config())
   }

   fn test_config() -> AgentConfig {
       AgentConfig::default()
   }
   ```

2. **Test Edge Cases:**
   ```rust
   #[test]
   fn test_empty_input() { /* ... */ }
   #[test]
   fn test_very_long_input() { /* ... */ }
   #[test]
   fn test_special_characters() { /* ... */ }
   #[test]
   fn test_unicode_characters() { /* ... */ }
   ```

3. **Use Property-Based Testing (for algorithms):**
   ```rust
   use proptest::prelude::*;

   proptest! {
       #[test]
       fn test_redaction_roundtrip(s in "\\PC*") {
           let redactor = Redactor::new();
           let redacted = redactor.redact(&s)?;
           let reinflated = redactor.reinflate(&redacted)?;
           assert_eq!(s, reinflated);
       }
   }
   ```

4. **Mock External Dependencies:**
   ```rust
   #[cfg(test)]
   mod mock {
       use super::*;

       pub struct MockEmbedder {
           pub response: Vec<f32>,
       }

       impl EmbeddingProvider for MockEmbedder {
           fn embed(&self, _text: &str) -> Result<Vec<f32>, EmbeddingError> {
               Ok(self.response.clone())
           }
       }
   }
   ```

### Running Tests

```bash
# Run all tests
cargo test --workspace

# Run tests in parallel (faster)
cargo test --workspace -- --test-threads=8

# Run tests with output
cargo test --workspace -- --nocapture

# Run tests with logging
RUST_LOG=debug cargo test --workspace

# Run specific test
cargo test test_redaction_replaces_email

# Run tests for specific crate
cargo test --package synesis-privacy

# Run tests and generate coverage report
cargo install cargo-tarpaulin
cargo tarpaulin --workspace --out Html
```

---

## Documentation Standards

### Code Documentation

**Public APIs:** Every public item must have documentation:
- Functions: What it does, parameters, return value, errors, examples
- Structs: Purpose, field descriptions, usage examples
- Enums: Variants, when to use each
- Traits: Purpose, required methods, implementor notes

**Documentation Template:**
```rust
/// Brief one-line summary.
///
/// More detailed explanation if needed. Can span multiple lines.
/// Explain the "why", not just the "what".
///
/// # Arguments
///
/// * `arg1` - Description of argument
/// * `arg2` - Description of argument
///
/// # Returns
///
/// Description of return value
///
/// # Errors
///
/// * `ErrorType` - When this error occurs
///
/// # Examples
///
/// ```
/// use crate::function_name;
///
/// let result = function_name(arg1, arg2)?;
/// assert_eq!(result, expected);
/// # Ok::<(), ErrorType>(())
/// ```
///
/// # Panics
///
/// Document any panic conditions (should be rare in production code)
///
/// # Safety
///
/// If function is `unsafe`, document invariants and preconditions
pub fn function_name(arg1: Type1, arg2: Type2) -> Result<ReturnType, ErrorType> {
    // implementation
}
```

### Project Documentation

**When to Update Documentation:**
- User-facing features → Update `README.md`
- Architectural changes → Update `ARCHITECTURE.md`
- AI agent instructions → Update `CLAUDE.md`
- API changes → Update relevant crate docs
- New development patterns → Update `CONTRIBUTING.md`

**Documentation Files:**
- `README.md` - User-facing overview and quick start
- `CLAUDE.md` - Development guide and methodology
- `ARCHITECTURE.md` - System design and architecture
- `PROJECT_ROADMAP.md` - Phase timeline and milestones
- `CONTRIBUTING.md` - This file
- `CHANGELOG.md` - Version history and changes

### Inline Comments

**Do Comment:**
- Complex algorithms (explain the "why")
- Non-obvious performance optimizations
- Workarounds for known issues
- TODO/FIXME markers with issues
- Safety invariants in unsafe code

**Don't Comment:**
- Things that are obvious from the code
- Things that would be better as documentation
- Outdated or misleading comments
- Comments that duplicate the code

```rust
// ✅ GOOD - Explains WHY
// We use SHA256 here instead of BGE-Micro because semantic embeddings
// aren't needed for placeholder tokens. This is faster and deterministic.
let hash = sha256(input);

// ❌ BAD - Just repeats the code
// Calculate the SHA256 hash of the input
let hash = sha256(input);

// ✅ GOOD - TODO with issue
// TODO: Replace with BGE-Micro once we have GPU inference
// Tracking: https://github.com/SuperInstance/Tripartite1/issues/42
let embedding = placeholder_embedding(input);

// ✅ GOOD - Explains invariant
// SAFETY: This is safe because we've verified that ptr is non-null
// and points to valid memory for at least `len` bytes.
unsafe { slice::from_raw_parts(ptr, len) }
```

---

## Submitting Changes

### Pull Request Process

1. **Before Submitting:**
   - Ensure all tests pass: `cargo test --workspace`
   - Ensure no warnings: `cargo clippy --workspace -- -D warnings`
   - Ensure formatting is correct: `cargo fmt --check`
   - Update documentation if needed
   - Self-review your changes

2. **Creating the PR:**
   - Use a clear title (follows conventional commits)
   - Fill out the PR template
   - Link related issues
   - Add screenshots for UI changes (if applicable)
   - Request review from maintainers

3. **During Review:**
   - Respond to feedback promptly
   - Make requested changes
   - Push updates to the same branch
   - Ask questions if anything is unclear

4. **After Approval:**
   - Ensure CI is passing
   - Squash commits if requested (maintain conventional commit format)
   - Wait for maintainer to merge

### Review Guidelines

**For Reviewers:**
- Be constructive and respectful
- Explain the reasoning behind suggested changes
- Approve if the changes are good enough (not perfect)
- Test the changes if possible
- Respond within a reasonable time (48 hours)

**For Contributors:**
- Don't take feedback personally
- Ask clarifying questions
- Push back politely if you disagree
- Learn from the review process

### Merge Policy

**Maintainers will merge when:**
- At least one approving review
- All CI checks passing
- No unresolved conversations
- Documentation updated (if needed)

**Merge types:**
- `squash merge` - Default for most PRs (cleaner history)
- `merge commit` - For feature branches with multiple related changes
- `rebase merge` - Rarely used (only for maintaining linear history)

---

## Community Guidelines

### Our Pledge

We aim to foster an inclusive, welcoming community. As contributors and maintainers, we pledge to:

- Use welcoming and inclusive language
- Be respectful of differing viewpoints and experiences
- Gracefully accept constructive criticism
- Focus on what is best for the community
- Show empathy towards other community members

### Expected Behavior

**Please:**
- Be respectful and considerate
- Use inclusive language
- Be collaborative (welcome newcomers, help others learn)
- Focus on constructive criticism (not personal attacks)
- Ask for help when you need it
- Follow the Rust Code of Conduct (see `CODE_OF_CONDUCT.md`)

**Unacceptable Behavior:**
- Harassment, trolling, or derogatory comments
- Personal attacks or insults
- Spam or off-topic discussions
- Discrimination based on race, gender, religion, etc.
- Publishing private information without permission
- Other unethical or unprofessional conduct

### Reporting Issues

If you witness or experience unacceptable behavior:

1. **Email:** conduct@superinstance.ai
2. **DM a maintainer:** On GitHub or Discord
3. **Use the report form:** [GitHub report abuse form](https://github.com/contact/report-abuse)

All reports will be kept confidential. See `CODE_OF_CONDUCT.md` for details.

---

## Getting Help

### Documentation

- [README.md](README.md) - Project overview and quick start
- [ARCHITECTURE.md](ARCHITECTURE.md) - System architecture deep dive
- [PROJECT_ROADMAP.md](PROJECT_ROADMAP.md) - Phase timeline
- [CLAUDE.md](CLAUDE.md) - Development methodology
- [API Documentation](https://docs.rs/synesis) - Rust API docs

### Community Resources

- **GitHub Issues:** Bug reports and feature requests
- **GitHub Discussions:** Questions and ideas
- **Discord Server:** (Coming soon) Real-time chat
- **Email:** dev@superinstance.ai - Direct support

### Common Issues

**Build Errors:**
```bash
# Update Rust toolchain
rustup update stable

# Clean build artifacts
cargo clean

# Rebuild
cargo build --workspace
```

**Test Failures:**
```bash
# Run tests with output to see what's failing
cargo test --workspace -- --nocapture

# Run specific failing test
cargo test test_name

# Run with logging
RUST_LOG=debug cargo test --workspace
```

**IDE Issues:**
- Reload rust-analyzer: Command Palette → "Rust Analyzer: Reload workspace"
- Clear cargo cache: `cargo clean && cargo build`
- Check rust-analyzer health: Command Palette → "Rust Analyzer: Run flycheck"

### Asking Good Questions

Before asking, please:

1. **Search first:** Check existing issues, docs, and discussions
2. **Be specific:** Include error messages, steps to reproduce, your environment
3. **Provide context:** What you're trying to do, what you've already tried
4. **Format code:** Use markdown code blocks
5. **Be patient:** Maintainers are volunteers

**Good question example:**

> Hi! I'm trying to add a new redaction pattern but getting an error.
>
> **What I'm trying to do:** Add a pattern to redact IP addresses
>
> **Error message:**
> ```
> error[E0308]: mismatched types
>   --> src/privacy/patterns.rs:45:20
>    |
> 45 |     Regex::new(r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}")
>    |                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
>    |                    expected `&str`, found `&str`
> ```
>
> **What I've tried:**
> - Checked the docs for Regex::new
> - Looked at other pattern implementations
> - Searched issues for similar errors
>
> **Environment:** Rust 1.75, Ubuntu 22.04
>
> Can someone help me understand what I'm missing?

---

## License

By contributing to SuperInstance AI, you agree that your contributions will be licensed under the **MIT OR Apache-2.0** license, matching the project's dual-license structure.

---

## Recognition

We value all contributions! Contributors will be:

- Listed in `CONTRIBUTORS.md`
- Mentioned in release notes for significant contributions
- Eligible for contributor badges (when we launch our community platform)
- Eligible for Knowledge Credits (future reward system)

Thank you for contributing to SuperInstance AI! 🚀

---

*Last Updated: 2026-01-07*
