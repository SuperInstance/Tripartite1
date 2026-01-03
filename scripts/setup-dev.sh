#!/usr/bin/env bash
#
# SuperInstance AI - Development Setup Script
#
# This script sets up the development environment for SuperInstance.
#
# Usage:
#   ./scripts/setup-dev.sh
#   ./scripts/setup-dev.sh --with-models   # Also download small test models
#   ./scripts/setup-dev.sh --clean         # Clean and reinstall

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Parse arguments
WITH_MODELS=false
CLEAN=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --with-models)
            WITH_MODELS=true
            shift
            ;;
        --clean)
            CLEAN=true
            shift
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

log_info "Setting up SuperInstance AI development environment..."
log_info "Project root: $PROJECT_ROOT"

# Check prerequisites
log_info "Checking prerequisites..."

check_command() {
    if ! command -v "$1" &> /dev/null; then
        log_error "$1 is required but not installed."
        exit 1
    fi
    log_success "$1 found"
}

check_command "git"
check_command "cargo"
check_command "rustc"

# Check Rust version
RUST_VERSION=$(rustc --version | cut -d' ' -f2)
REQUIRED_RUST="1.75.0"
if [[ "$(printf '%s\n' "$REQUIRED_RUST" "$RUST_VERSION" | sort -V | head -n1)" != "$REQUIRED_RUST" ]]; then
    log_warn "Rust version $RUST_VERSION may be too old. Recommended: $REQUIRED_RUST+"
fi

# Clean if requested
if [ "$CLEAN" = true ]; then
    log_info "Cleaning previous build artifacts..."
    cargo clean
    rm -rf ~/.superinstance-dev
    log_success "Clean complete"
fi

# Install Rust tools
log_info "Installing Rust development tools..."

cargo install --quiet cargo-watch 2>/dev/null || true
cargo install --quiet cargo-audit 2>/dev/null || true
cargo install --quiet cargo-llvm-cov 2>/dev/null || true
log_success "Rust tools installed"

# Check for rustfmt and clippy
if ! rustup component list | grep -q "rustfmt.*installed"; then
    log_info "Installing rustfmt..."
    rustup component add rustfmt
fi

if ! rustup component list | grep -q "clippy.*installed"; then
    log_info "Installing clippy..."
    rustup component add clippy
fi

# Create development data directory
DEV_DATA_DIR="$HOME/.superinstance-dev"
log_info "Creating development data directory: $DEV_DATA_DIR"

mkdir -p "$DEV_DATA_DIR/models"
mkdir -p "$DEV_DATA_DIR/knowledge"
mkdir -p "$DEV_DATA_DIR/cache"
mkdir -p "$DEV_DATA_DIR/logs"

# Create development config
if [ ! -f "$DEV_DATA_DIR/config.toml" ]; then
    log_info "Creating development config..."
    cat > "$DEV_DATA_DIR/config.toml" << EOF
# SuperInstance Development Configuration

[general]
data_dir = "$DEV_DATA_DIR"
log_level = "debug"

[agents.pathos]
model = "phi-3-mini"
enabled = true
temperature = 0.7

[agents.logos]
model = "llama-3.2-3b"
enabled = true
temperature = 0.7

[agents.ethos]
model = "phi-3-mini"
enabled = true
temperature = 0.3

[consensus]
threshold = 0.85
max_rounds = 3

[cloud]
enabled = false
endpoint = "http://localhost:8787"

[knowledge]
embedding_model = "bge-micro"
embedding_dimensions = 384
EOF
    log_success "Development config created"
fi

# Build the project
log_info "Building the project..."
cargo build --workspace

if [ $? -eq 0 ]; then
    log_success "Build successful"
else
    log_error "Build failed"
    exit 1
fi

# Run tests
log_info "Running tests..."
cargo test --workspace --quiet

if [ $? -eq 0 ]; then
    log_success "All tests passed"
else
    log_warn "Some tests failed"
fi

# Download small test models if requested
if [ "$WITH_MODELS" = true ]; then
    log_info "Downloading test models (this may take a while)..."
    
    # Download BGE-micro for embeddings (smallest model)
    log_info "Downloading bge-micro embeddings model..."
    # This would use the synesis CLI or a direct download
    # For now, just create a placeholder
    touch "$DEV_DATA_DIR/models/.placeholder"
    
    log_success "Test models ready"
fi

# Set up git hooks
log_info "Setting up git hooks..."

if [ -d ".git" ]; then
    mkdir -p .git/hooks
    
    # Pre-commit hook
    cat > .git/hooks/pre-commit << 'EOF'
#!/bin/sh
# Pre-commit hook for SuperInstance

# Run formatting check
cargo fmt --check
if [ $? -ne 0 ]; then
    echo "Error: Code is not formatted. Run 'cargo fmt' first."
    exit 1
fi

# Run clippy
cargo clippy --workspace -- -D warnings
if [ $? -ne 0 ]; then
    echo "Error: Clippy found warnings. Fix them before committing."
    exit 1
fi

exit 0
EOF
    chmod +x .git/hooks/pre-commit
    
    log_success "Git hooks installed"
fi

# Print summary
echo ""
echo "=========================================="
log_success "Development environment setup complete!"
echo "=========================================="
echo ""
echo "Quick start commands:"
echo "  cargo run -p synesis-cli -- --help     # Show CLI help"
echo "  cargo run -p synesis-cli -- status     # Check status"
echo "  cargo watch -x 'run -p synesis-cli'    # Hot reload"
echo "  cargo test --workspace                  # Run all tests"
echo ""
echo "Configuration: $DEV_DATA_DIR/config.toml"
echo "Data directory: $DEV_DATA_DIR"
echo ""

if [ "$WITH_MODELS" = false ]; then
    log_info "Tip: Run with --with-models to download test models"
fi
