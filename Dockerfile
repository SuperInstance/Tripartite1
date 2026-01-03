# SuperInstance AI - Docker Build
#
# Multi-stage build for minimal production image
#
# Build: docker build -t superinstance .
# Run:   docker run -v ~/.superinstance:/data superinstance

# ============================================
# Stage 1: Build environment
# ============================================
FROM rust:1.75-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    cmake \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests first for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY crates/synesis-cli/Cargo.toml crates/synesis-cli/
COPY crates/synesis-core/Cargo.toml crates/synesis-core/
COPY crates/synesis-privacy/Cargo.toml crates/synesis-privacy/
COPY crates/synesis-models/Cargo.toml crates/synesis-models/
COPY crates/synesis-knowledge/Cargo.toml crates/synesis-knowledge/

# Create dummy source files for dependency compilation
RUN mkdir -p crates/synesis-cli/src && echo "fn main() {}" > crates/synesis-cli/src/main.rs
RUN mkdir -p crates/synesis-core/src && echo "pub fn dummy() {}" > crates/synesis-core/src/lib.rs
RUN mkdir -p crates/synesis-privacy/src && echo "pub fn dummy() {}" > crates/synesis-privacy/src/lib.rs
RUN mkdir -p crates/synesis-models/src && echo "pub fn dummy() {}" > crates/synesis-models/src/lib.rs
RUN mkdir -p crates/synesis-knowledge/src && echo "pub fn dummy() {}" > crates/synesis-knowledge/src/lib.rs

# Build dependencies only (this layer will be cached)
RUN cargo build --release 2>/dev/null || true
RUN rm -rf crates/*/src

# Copy actual source code
COPY crates crates/
COPY manifests manifests/

# Build the actual application
RUN cargo build --release --bin synesis

# ============================================
# Stage 2: Runtime environment
# ============================================
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libsqlite3-0 \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 synesis

# Create data directories
RUN mkdir -p /data/models /data/knowledge /data/cache /data/logs \
    && chown -R synesis:synesis /data

# Copy binary from builder
COPY --from=builder /app/target/release/synesis /usr/local/bin/

# Copy manifests
COPY --from=builder /app/manifests /usr/share/synesis/manifests

# Set environment
ENV SYNESIS_DATA_DIR=/data
ENV SYNESIS_MANIFESTS_DIR=/usr/share/synesis/manifests
ENV RUST_LOG=info

# Switch to non-root user
USER synesis
WORKDIR /home/synesis

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD synesis status --json || exit 1

# Default command
ENTRYPOINT ["synesis"]
CMD ["--help"]

# ============================================
# Stage 3: Development environment (optional)
# ============================================
FROM rust:1.75-bookworm AS development

# Install development tools
RUN apt-get update && apt-get install -y \
    cmake \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    git \
    curl \
    vim \
    && rm -rf /var/lib/apt/lists/*

# Install Rust tools
RUN rustup component add rustfmt clippy
RUN cargo install cargo-watch cargo-audit

# Create app directory
WORKDIR /app

# Mount points for development
VOLUME ["/app", "/data"]

# Default development command
CMD ["cargo", "watch", "-x", "run"]

# ============================================
# Stage 4: GPU-enabled runtime (CUDA)
# ============================================
FROM nvidia/cuda:12.2.0-runtime-ubuntu22.04 AS gpu-runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libsqlite3-0 \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 synesis

# Create data directories
RUN mkdir -p /data/models /data/knowledge /data/cache /data/logs \
    && chown -R synesis:synesis /data

# Copy binary from builder (would need GPU-enabled build)
# COPY --from=builder-gpu /app/target/release/synesis /usr/local/bin/

# Set environment for CUDA
ENV SYNESIS_DATA_DIR=/data
ENV CUDA_VISIBLE_DEVICES=0
ENV RUST_LOG=info

USER synesis
WORKDIR /home/synesis

ENTRYPOINT ["synesis"]
CMD ["--help"]
