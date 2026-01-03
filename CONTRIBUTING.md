# Contributing to SuperInstance AI

Thank you for your interest in contributing to SuperInstance! This document provides guidelines and instructions for contributing.

## Code of Conduct

Be respectful, inclusive, and constructive. We're building something meaningful together.

## Getting Started

### Prerequisites

- Rust 1.75+ (stable)
- Node.js 20+ (for cloud workers)
- SQLite 3.35+
- 8GB+ RAM recommended
- GPU optional but helpful for local inference

### Development Setup

```bash
# Clone the repository
git clone https://github.com/superinstance/synesis.git
cd synesis

# Run setup script
./scripts/setup-dev.sh

# Build all crates
cargo build

# Run tests
cargo test

# Run clippy
cargo clippy --all-targets --all-features
```

## Project Structure

```
synesis/
├── crates/
│   ├── synesis-cli        # Command-line interface
│   ├── synesis-core       # Tripartite council orchestration
│   ├── synesis-privacy    # Privacy proxy (redaction/reinflation)
│   ├── synesis-models     # Model management and inference
│   └── synesis-knowledge  # Knowledge vault (RAG)
├── cloud/                 # Cloudflare Workers
├── docs/                  # Documentation
└── tests/                 # Integration tests
```

## Development Workflow

### 1. Pick an Issue

- Check the [issue tracker](https://github.com/superinstance/synesis/issues)
- Look for issues labeled `good first issue` or `help wanted`
- Comment on the issue to claim it

### 2. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/your-bug-fix
```

### 3. Make Changes

- Follow the existing code style
- Add tests for new functionality
- Update documentation as needed
- Keep commits atomic and well-described

### 4. Test Your Changes

```bash
# Run all tests
cargo test

# Run specific crate tests
cargo test -p synesis-core

# Run with logging
RUST_LOG=debug cargo test

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy --all-targets
```

### 5. Submit a Pull Request

- Fill out the PR template
- Reference related issues
- Request review from maintainers

## Code Style

### Rust

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for formatting
- Use `clippy` for linting
- Document public APIs with rustdoc

### TypeScript (Cloud Workers)

- Use Prettier for formatting
- Use ESLint for linting
- Prefer TypeScript strict mode

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
type(scope): description

[optional body]

[optional footer]
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

Examples:
```
feat(core): implement consensus engine
fix(privacy): correct email regex pattern
docs(readme): add installation instructions
```

## Testing Guidelines

### Unit Tests

Place unit tests in the same file as the code:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // ...
    }
}
```

### Integration Tests

Place integration tests in `tests/`:

```rust
// tests/integration/main.rs
#[tokio::test]
async fn test_full_flow() {
    // ...
}
```

### Test Coverage

Aim for high coverage on:
- Core consensus logic
- Privacy redaction patterns
- Model routing decisions

## Documentation

- Update README.md for user-facing changes
- Update CLAUDE.md for AI agent instructions
- Add inline documentation for complex logic
- Update architecture docs for structural changes

## Release Process

1. Update version in `Cargo.toml`
2. Update CHANGELOG.md
3. Create a git tag
4. CI builds and publishes

## Getting Help

- Open a [Discussion](https://github.com/superinstance/synesis/discussions)
- Join our Discord (link TBD)
- Email: dev@superinstance.ai

## License

By contributing, you agree that your contributions will be licensed under the MIT OR Apache-2.0 license.
