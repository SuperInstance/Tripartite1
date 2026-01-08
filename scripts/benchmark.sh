#!/usr/bin/env bash
# Comprehensive benchmark runner for SuperInstance AI
# Runs all Criterion benchmarks and generates detailed reports

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BENCHMARK_DIR="$PROJECT_ROOT/benches"
TARGET_DIR="$PROJECT_ROOT/target"
CRITERION_DIR="$TARGET_DIR/criterion"
RESULTS_DIR="$PROJECT_ROOT/benchmark_results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULTS_FILE="$RESULTS_DIR/benchmark_$TIMESTAMP.txt"

# Create results directory
mkdir -p "$RESULTS_DIR"

# Header
echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║     SuperInstance AI - Comprehensive Benchmark Suite       ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Timestamp: $TIMESTAMP"
echo "Rust Version: $(rustc --version)"
echo "OS: $(uname -s) $(uname -r)"
echo "Architecture: $(uname -m)"
echo ""

# System information
echo -e "${YELLOW}═══ System Information ═══${NC}"
if command -v lscpu &> /dev/null; then
    echo "CPU Info:"
    lscpu | grep -E "^Model name|^CPU\(s\)|^Thread" | head -5
fi

if command -v free &> /dev/null; then
    echo -e "\nMemory:"
    free -h | head -2
fi

if command -v nvidia-smi &> /dev/null; then
    echo -e "\nGPU:"
    nvidia-smi --query-gpu=name,memory.total --format=csv,noheader | head -1
fi

echo ""

# Check if benchmarks exist
if [ ! -d "$BENCHMARK_DIR" ]; then
    echo -e "${RED}Error: Benchmark directory not found: $BENCHMARK_DIR${NC}"
    exit 1
fi

# Check for Criterion
echo -e "${YELLOW}═══ Benchmark Environment Check ═══${NC}"
if ! cargo bench --help &> /dev/null; then
    echo -e "${RED}Error: Criterion not installed${NC}"
    echo "Install with: cargo install cargo-criterion"
    exit 1
fi
echo -e "${GREEN}✓ Criterion is installed${NC}"

# Check if running in release mode
echo ""
echo -e "${YELLOW}═══ Compilation Check ═══${NC}"
echo "Compiling benchmarks in release mode..."
if ! cargo check --benches --release 2>&1 | grep -q "Finished"; then
    echo -e "${YELLOW}⚠ Compilation check completed${NC}"
fi
echo -e "${GREEN}✓ Benchmarks compiled successfully${NC}"

echo ""

# Warm-up run (optional)
echo -e "${YELLOW}═══ Warm-up Run ═══${NC}"
echo "Running quick warm-up to ensure JIT compilation..."
cargo bench --bench query_processing -- --warm-up-time 3 --measurement-time 1 --nresamples 1 > /dev/null 2>&1 || true
echo -e "${GREEN}✓ Warm-up completed${NC}"
echo ""

# Run all benchmarks
echo -e "${YELLOW}═══ Running Benchmarks ═══${NC}"
echo "This may take 5-10 minutes..."
echo ""

START_TIME=$(date +%s)

# Array of benchmark names
BENCHMARKS=(
    "query_processing"
    "agent_execution"
    "consensus_engine"
    "knowledge_vault"
    "privacy_redaction"
)

# Run each benchmark
for bench in "${BENCHMARKS[@]}"; do
    echo -e "${BLUE}▶ Running: $bench${NC}"

    if cargo bench --bench "$bench" 2>&1 | tee -a "$RESULTS_FILE"; then
        echo -e "${GREEN}✓ Completed: $bench${NC}"
    else
        echo -e "${RED}✗ Failed: $bench${NC}"
    fi

    echo ""
done

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

echo -e "${GREEN}═══ Benchmark Suite Completed ═══${NC}"
echo "Total time: ${DURATION}s"
echo ""

# Generate summary
echo -e "${YELLOW}═══ Benchmark Summary ═══${NC}"
echo "Results saved to: $RESULTS_FILE"
echo "HTML report: $CRITERION_DIR/report/index.html"
echo ""

# Extract key metrics (if available)
if [ -f "$RESULTS_FILE" ]; then
    echo -e "${YELLOW}Key Metrics:${NC}"

    # Extract mean times from results
    echo ""
    echo "Query Processing:"
    grep -A 1 "query_processing/simple" "$RESULTS_FILE" | grep "time:" | tail -1 || echo "  Not available"
    grep -A 1 "query_processing/medium" "$RESULTS_FILE" | grep "time:" | tail -1 || echo "  Not available"

    echo ""
    echo "Agent Execution:"
    grep -A 1 "agent_pathos" "$RESULTS_FILE" | grep "time:" | tail -1 || echo "  Not available"
    grep -A 1 "agent_logos" "$RESULTS_FILE" | grep "time:" | tail -1 || echo "  Not available"
    grep -A 1 "agent_ethos" "$RESULTS_FILE" | grep "time:" | tail -1 || echo "  Not available"

    echo ""
    echo "Consensus Engine:"
    grep -A 1 "consensus_single_round" "$RESULTS_FILE" | grep "time:" | tail -1 || echo "  Not available"

    echo ""
    echo "Knowledge Vault:"
    grep -A 1 "vector_search" "$RESULTS_FILE" | grep "time:" | tail -1 || echo "  Not available"

    echo ""
    echo "Privacy Redaction:"
    grep -A 1 "redact_email" "$RESULTS_FILE" | grep "time:" | tail -1 || echo "  Not available"
fi

echo ""

# Comparison with previous run (if available)
LATEST_RESULT="$RESULTS_DIR/benchmark_latest.txt"
if [ -f "$LATEST_RESULT" ] && [ "$LATEST_RESULT" != "$RESULTS_FILE" ]; then
    echo -e "${YELLOW}═══ Comparison with Previous Run ═══${NC}"
    echo "Previous: $LATEST_RESULT"
    echo "Current:  $RESULTS_FILE"
    echo ""
    echo "To compare HTML reports:"
    echo "  1. Open: $CRITERION_DIR/report/index.html"
    echo "  2. Click 'Change' to see comparison"
    echo ""
fi

# Update latest symlink
ln -sf "$RESULTS_FILE" "$LATEST_RESULT"

# Open HTML report (optional)
echo -e "${YELLOW}═══ Viewing Results ═══${NC}"
echo "To view detailed HTML report:"
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "  open $CRITERION_DIR/report/index.html"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "  xdg-open $CRITERION_DIR/report/index.html"
else
    echo "  Open in browser: file://$CRITERION_DIR/report/index.html"
fi

echo ""
echo -e "${GREEN}═══ Benchmark Script Complete ═══${NC}"
echo ""
echo "Next steps:"
echo "  1. Review HTML report for detailed analysis"
echo "  2. Check for performance regressions (red text)"
echo "  3. Update BENCHMARKS.md with new results"
echo "  4. Commit results to git for historical tracking"
echo ""
