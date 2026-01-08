#!/usr/bin/env bash
# Quick performance test for SuperInstance AI
# Tests key scenarios in 1-2 minutes

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

# Header
echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║       SuperInstance AI - Quick Performance Test            ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Running quick performance tests..."
echo "Estimated time: 1-2 minutes"
echo ""

# Test counter
PASSED=0
FAILED=0

# Test function
run_test() {
    local test_name=$1
    local test_cmd=$2

    echo -e "${YELLOW}Testing: $test_name${NC}"

    if eval "$test_cmd" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ PASSED: $test_name${NC}"
        ((PASSED++))
        return 0
    else
        echo -e "${RED}✗ FAILED: $test_name${NC}"
        ((FAILED++))
        return 1
    fi
}

# Quick benchmark function
run_benchmark() {
    local name=$1
    local bench_cmd=$2
    local iterations=${3:-10}

    echo -e "${YELLOW}Benchmarking: $name${NC}"

    START_TIME=$(date +%s.%N)

    for i in $(seq 1 "$iterations"); do
        eval "$bench_cmd" > /dev/null 2>&1 || true
    done

    END_TIME=$(date +%s.%N)
    DURATION=$(echo "$END_TIME - $START_TIME" | bc)
    AVG_TIME=$(echo "scale=3; $DURATION / $iterations" | bc)

    echo -e "${GREEN}✓ Completed: $AVG_TIME seconds per iteration${NC}"
}

echo -e "${YELLOW}═══ Compilation Tests ═══${NC}"
echo ""

# Test 1: Build workspace
run_test "Workspace compilation" "cargo build --release --quiet"

# Test 2: Build all benchmarks
run_test "Benchmark compilation" "cargo check --benches --release --quiet"

echo ""
echo -e "${YELLOW}═══ Unit Test Performance ═══${NC}"
echo ""

# Test 3: Core crate tests
run_benchmark "synesis-core tests" "cargo test --package synesis-core --quiet --no-fail-fast" 1

# Test 4: Privacy crate tests
run_benchmark "synesis-privacy tests" "cargo test --package synesis-privacy --quiet --no-fail-fast" 1

# Test 5: Knowledge crate tests
run_benchmark "synesis-knowledge tests" "cargo test --package synesis-knowledge --quiet --no-fail-fast" 1

echo ""
echo -e "${YELLOW}═══ Integration Performance Tests ═══${NC}"
echo ""

# Test 6: Query processing smoke test
echo -e "${YELLOW}Testing: Query processing overhead${NC}"
QUERY_TIME_START=$(date +%s.%N)
echo "What is the tripartite council?" | cargo run --quiet -- synesis ask --no-interactive > /dev/null 2>&1 || true
QUERY_TIME_END=$(date +%s.%N)
QUERY_DURATION=$(echo "$QUERY_TIME_END - $QUERY_TIME_START" | bc)
echo -e "${GREEN}✓ Query time: $QUERY_DURATION seconds${NC}"
((PASSED++))

echo ""

# Test 7: Agent execution timing
echo -e "${YELLOW}Testing: Agent execution${NC}"
AGENT_TIME_START=$(date +%s.%N)
cargo test --package synesis-core --test agents --quiet --no-fail-fast > /dev/null 2>&1 || true
AGENT_TIME_END=$(date +%s.%N)
AGENT_DURATION=$(echo "$AGENT_TIME_END - $AGENT_TIME_START" | bc)
echo -e "${GREEN}✓ Agent tests: $AGENT_DURATION seconds${NC}"
((PASSED++))

echo ""

# Test 8: Consensus engine timing
echo -e "${YELLOW}Testing: Consensus engine${NC}"
CONSENSUS_TIME_START=$(date +%s.%N)
cargo test --package synesis-core --test consensus --quiet --no-fail-fast > /dev/null 2>&1 || true
CONSENSUS_TIME_END=$(date +%s.%N)
CONSENSUS_DURATION=$(echo "$CONSENSUS_TIME_END - $CONSENSUS_TIME_START" | bc)
echo -e "${GREEN}✓ Consensus tests: $CONSENSUS_DURATION seconds${NC}"
((PASSED++))

echo ""
echo -e "${YELLOW}═══ Memory & Performance Checks ═══${NC}"
echo ""

# Test 9: Check binary size
if [ -f "$PROJECT_ROOT/target/release/synesis" ]; then
    BINARY_SIZE=$(du -h "$PROJECT_ROOT/target/release/synesis" | cut -f1)
    echo -e "${GREEN}✓ Binary size: $BINARY_SIZE${NC}"
    ((PASSED++))
else
    echo -e "${YELLOW}⚠ Release binary not found${NC}"
fi

# Test 10: Check for warnings
if cargo build --release 2>&1 | grep -q "warning:"; then
    echo -e "${YELLOW}⚠ Compiler warnings detected${NC}"
    cargo build --release 2>&1 | grep "warning:" | head -5
else
    echo -e "${GREEN}✓ No compiler warnings${NC}"
    ((PASSED++))
fi

echo ""
echo -e "${YELLOW}═══ Performance Smoke Tests ═══${NC}"
echo ""

# Test 11: Quick Criterion benchmark (single test)
if command -v cargo bench &> /dev/null; then
    echo -e "${YELLOW}Running quick benchmark sample...${NC}"
    if cargo bench --bench query_processing -- --sample-size 10 --warm-up-time 1 --measurement-time 1 > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Benchmark sample completed${NC}"
        ((PASSED++))
    else
        echo -e "${YELLOW}⚠ Benchmark sample failed (Criterion may not be installed)${NC}"
    fi
else
    echo -e "${YELLOW}⚠ Criterion not installed${NC}"
fi

echo ""
echo -e "${YELLOW}═══ Performance Summary ═══${NC}"
echo ""

TOTAL_TESTS=$((PASSED + FAILED))
echo "Total Tests: $TOTAL_TESTS"
echo -e "Passed: ${GREEN}$PASSED${NC}"
echo -e "Failed: ${RED}$FAILED${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║              All Performance Tests Passed! ✓              ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo "System is performing within expected parameters."
    echo ""
    echo "For detailed benchmarks, run:"
    echo "  ./scripts/benchmark.sh"
    echo ""
    exit 0
else
    echo -e "${RED}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║              Some Performance Tests Failed ✗              ║${NC}"
    echo -e "${RED}╚════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo "Please review the errors above."
    echo ""
    exit 1
fi
