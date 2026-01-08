# Performance Benchmarking Guide - SuperInstance AI

**Version**: v0.2.0
**Last Updated**: 2026-01-07
**Benchmarking Tools**: Criterion 0.5, Custom Scripts

## Quick Start

### Run Quick Performance Test (1-2 minutes)
```bash
./scripts/performance_test.sh
```
This runs smoke tests for key components and verifies system performance.

### Run Full Benchmark Suite (5-10 minutes)
```bash
./scripts/benchmark.sh
```
This runs comprehensive Criterion benchmarks and generates detailed HTML reports.

---

## Benchmark Files Created

### 1. Documentation
- **BENCHMARKS.md** - Comprehensive performance results, methodology, and analysis
- **benches/README.md** - Guide to the benchmark suite

### 2. Benchmark Code (benches/)
- **query_processing.rs** - Query latency, parsing, size scaling
- **agent_execution.rs** - Pathos, Logos, Ethos, parallel vs sequential
- **consensus_engine.rs** - Consensus rounds, thresholds, agent counts
- **knowledge_vault.rs** - Chunking, embeddings, vector search
- **privacy_redaction.rs** - Pattern matching, redaction, token vault

### 3. Scripts (scripts/)
- **benchmark.sh** - Full benchmark suite with reporting
- **performance_test.sh** - Quick performance smoke tests

---

## Running Benchmarks

### Option 1: Quick Performance Test

```bash
./scripts/performance_test.sh
```

**Tests**:
- Workspace compilation
- Benchmark compilation
- Unit test performance (core, privacy, knowledge)
- Query processing overhead
- Agent execution timing
- Consensus engine timing
- Binary size
- Compiler warnings
- Quick Criterion sample

**Time**: 1-2 minutes
**Output**: Pass/fail summary

### Option 2: Full Benchmark Suite

```bash
./scripts/benchmark.sh
```

**Benchmarks**:
- All 5 benchmark files
- 100+ individual benchmark functions
- Statistical analysis with Criterion
- HTML reports with visualizations

**Time**: 5-10 minutes
**Output**:
- Console results
- `benchmark_results/benchmark_TIMESTAMP.txt`
- `target/criterion/report/index.html` (HTML)

### Option 3: Manual Benchmark Execution

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench query_processing

# Run specific test
cargo bench --bench agent_execution -- agent_pathos

# Save baseline for comparison
cargo bench -- --save-baseline main

# Compare against baseline
cargo bench -- --baseline main
```

---

## Understanding Results

### Console Output

```
query_processing/simple
                        time:   [2.100 s 2.545 s 2.890 s]
                        change: [-2.4% +0.5% +3.5%] (p = 0.85 > 0.05)
                        No change in performance detected.
```

**What this means**:
- **Mean**: 2.545s average execution time
- **Range**: 2.100s - 2.890s (95% confidence interval)
- **Change**: +0.5% compared to previous run
- **Significance**: p = 0.85 (> 0.05 means not statistically significant)

### HTML Reports

```bash
# Open HTML report (macOS)
open target/criterion/report/index.html

# Open HTML report (Linux)
xdg-open target/criterion/report/index.html

# Open HTML report (Windows)
start target/criterion/report/index.html
```

**HTML features**:
- Interactive graphs
- Performance over time
- Comparison with previous runs
- Detailed statistics (median, std dev, min, max)
- Welch t-test for significance

---

## Interpreting Changes

### Performance Improved
```
change: [-15.2% -12.3% -9.4%] (p = 0.00 < 0.05)
Performance has improved.
```
- Green text in console
- Negative percentage = faster (good)
- Statistically significant (p < 0.05)

### Performance Degraded
```
change: [+15.2% +18.3% +21.4%] (p = 0.00 < 0.05)
Performance has degraded.
```
- Red text in console
- Positive percentage = slower (bad)
- Statistically significant (p < 0.05)

### No Significant Change
```
change: [-2.4% +0.5% +3.5%] (p = 0.85 > 0.05)
No change in performance detected.
```
- Gray text in console
- Within confidence interval
- Not statistically significant (p > 0.05)

---

## Benchmark Baselines

### Creating a Baseline

```bash
# Before making changes
cargo bench -- --save-baseline before_optimization

# Make code changes...

# Compare against baseline
cargo bench -- --baseline before_optimization
```

### Use Cases

1. **Before optimization**: Establish baseline
2. **After optimization**: Verify improvement
3. **Before refactoring**: Ensure performance isn't lost
4. **CI/CD**: Detect performance regressions

---

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Benchmarks

on: [push, pull_request]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run benchmarks
        run: cargo bench -- --output-format bencher | tee benchmark.txt
      - name: Store benchmark result
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo'
          output-file-path: benchmark.txt
          github-token: ${{ secrets.GITHUB_TOKEN }}
          auto-push: false
```

### GitHub Action with Custom Script

```yaml
- name: Run performance tests
  run: ./scripts/performance_test.sh

- name: Run full benchmarks
  run: ./scripts/benchmark.sh

- name: Upload benchmark results
  uses: actions/upload-artifact@v3
  with:
    name: benchmark-results
    path: benchmark_results/
```

---

## Performance Profiling

### Flamegraphs

For deeper performance analysis:

```bash
# Install flamegraph
cargo install flamegraph

# Generate flamegraph
cargo flamegraph --bench query_processing

# Output: flamegraph.svg
```

### Perf (Linux)

```bash
# Record performance data
perf record -g cargo bench --bench query_processing

# View report
perf report
```

### Valgrind (Linux/macOS)

```bash
# Check for memory leaks
valgrind --leak-check=full cargo bench
```

---

## Writing Custom Benchmarks

### Template

```rust
// benches/my_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_my_function(c: &mut Criterion) {
    c.bench_function("my_function", |b| {
        b.iter(|| {
            let input = black_box(42);
            let result = my_function(input);
            black_box(result)
        })
    });
}

criterion_group!(benches, bench_my_function);
criterion_main!(benches);
```

### Key Rules

1. **Always use `black_box()`**
   - Prevents compiler from optimizing away code
   - Use for inputs and outputs

2. **Test the right thing**
   - Isolate the code you want to measure
   - Don't include I/O (disk, network)
   - Use in-memory data

3. **Parameterized benchmarks**
   ```rust
   let mut group = c.benchmark_group("my_group");
   for size in [10, 50, 100].iter() {
       group.bench_with_input(
           BenchmarkId::from_parameter(size),
           size,
           |b, &size| {
               b.iter(|| black_box(process(size)))
           }
       );
   }
   group.finish();
   ```

4. **Async benchmarks**
   ```rust
   let rt = tokio::runtime::Runtime::new().unwrap();
   c.bench_function("async_function", |b| {
       b.to_async(&rt).iter(|| {
           async { black_box(async_fn().await) }
       })
   });
   ```

---

## Troubleshooting

### Issue: High Variance / Wide Confidence Intervals

**Symptoms**: Large range in results, inconsistent runs

**Solutions**:
1. Close other applications
2. Check CPU thermal throttling
3. Increase sample size:
   ```bash
   cargo bench -- --sample-size 100
   ```
4. Increase measurement time:
   ```bash
   cargo bench -- --measurement-time 30
   ```

### Issue: Unrealistic Results

**Symptoms**: Results are too fast, don't match reality

**Solutions**:
1. Ensure `black_box()` is used correctly
2. Verify code is actually executing
3. Check for compiler optimizations:
   ```bash
   cargo bench -- --verbose
   ```

### Issue: Criterion Not Installed

**Symptoms**: `cargo bench` command not found

**Solution**:
```bash
# Install cargo-criterion
cargo install cargo-criterion

# Or add to Cargo.toml
[dev-dependencies]
criterion = "0.5"
```

### Issue: Outliers

**Symptoms**: Extreme max values, long right tail

**Solutions**:
1. Ensure system is idle
2. Disable power saving
3. Increase warm-up time:
   ```bash
   cargo bench -- --warm-up-time 10
   ```

---

## Performance Targets

### Query Latency
- **Target**: < 3.5s for medium queries
- **Current**: ~2.8s average
- **Status**: ✅ On target

### Consensus Engine
- **Target**: < 1.5s for single-round consensus
- **Current**: ~1.12s average
- **Status**: ✅ On target

### Knowledge Search
- **Target**: < 20ms for vector search
- **Current**: ~11.5ms (1k documents)
- **Status**: ✅ On target

### Privacy Redaction
- **Target**: < 2ms per document
- **Current**: ~0.85ms average
- **Status**: ✅ Excellent

### Memory Usage
- **Target**: < 1GB with 3B model
- **Current**: ~280MB
- **Status**: ✅ Excellent

---

## Contributing Benchmarks

When adding new benchmarks:

1. **Choose appropriate location**
   - Query operations → `query_processing.rs`
   - Agent logic → `agent_execution.rs`
   - Consensus → `consensus_engine.rs`
   - Knowledge → `knowledge_vault.rs`
   - Privacy → `privacy_redaction.rs`

2. **Follow naming convention**
   - `bench_<component>_<operation>`
   - Example: `bench_consensus_thresholds`

3. **Add documentation**
   - What's being tested?
   - What are the expectations?
   - Any special considerations?

4. **Test on multiple machines**
   - Verify reproducibility
   - Document environment differences

5. **Update documentation**
   - Add results to `BENCHMARKS.md`
   - Update this guide

---

## Best Practices

### Before Benchmarking
1. **Close unnecessary applications**
2. **Ensure CPU is cool** (not throttling)
3. **Use release mode**: `cargo build --release`
4. **Stable power source** (not battery)
5. **Consistent environment** (same hardware, OS, Rust version)

### During Benchmarking
1. **Let warm-up complete** (Criterion handles this)
2. **Don't disturb the system**
3. **Run multiple times** ( Criterion handles this)
4. **Check for outliers** in results

### After Benchmarking
1. **Review HTML reports**
2. **Check for regressions** (red text)
3. **Save baselines** for future comparison
4. **Document findings**
5. **Commit results** to git

---

## Reference Documentation

### Internal Documentation
- **BENCHMARKS.md** - Detailed performance results
- **benches/README.md** - Benchmark suite guide
- **CLAUDE.md** - Project overview

### External Resources
- [Criterion.rs User Guide](https://bheisler.github.io/criterion.rs/book/index.html)
- [Rust Benchmarking Guide](https://doc.rust-lang.org/1.70.0/book/ch15-01-box.html)
- [Flamegraph Guide](https://github.com/flamegraph-rs/flamegraph)

---

## Quick Reference

```bash
# Quick test
./scripts/performance_test.sh

# Full benchmark
./scripts/benchmark.sh

# Manual benchmark
cargo bench --bench query_processing

# Save baseline
cargo bench -- --save-baseline main

# Compare baseline
cargo bench -- --baseline main

# View HTML report
open target/criterion/report/index.html

# Generate flamegraph
cargo flamegraph --bench query_processing

# Run specific test
cargo bench --bench agent_execution -- agent_pathos
```

---

**Last Updated**: 2026-01-07
**Version**: v0.2.0
**Maintainer**: SuperInstance AI Team
**Questions**: https://github.com/SuperInstance/Tripartite1/issues
