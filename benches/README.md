# Benchmark Suite - SuperInstance AI

This directory contains comprehensive performance benchmarks for the SuperInstance AI system using [Criterion.rs](https://bgithub.com/bheisler/criterion.rs).

## Benchmarks Overview

### Available Benchmarks

| Benchmark File | Tests | Description |
|----------------|-------|-------------|
| `query_processing.rs` | Query latency | Single/medium/complex queries, parsing, different sizes |
| `agent_execution.rs` | Agent performance | Pathos, Logos, Ethos, parallel vs sequential |
| `consensus_engine.rs` | Consensus speed | Single/multi-round, thresholds, agent counts |
| `knowledge_vault.rs` | Knowledge operations | Chunking, embedding, vector search, indexing |
| `privacy_redaction.rs` | Privacy performance | Pattern matching, redaction, token vault |

### Benchmark Categories

#### 1. Query Processing (`query_processing.rs`)
- **Simple queries**: 10-20 words, single intent
- **Medium queries**: 30-50 words, multi-part questions
- **Complex queries**: 50-100 words, ambiguous, requires revision
- **Query parsing**: Tokenization overhead
- **Query sizes**: Scaling from 10 to 500 words

#### 2. Agent Execution (`agent_execution.rs`)
- **Pathos**: Intent extraction speed
- **Logos**: RAG retrieval and reasoning
- **Ethos**: Verification and fact-checking
- **Parallel execution**: Concurrent agent execution
- **Sequential execution**: Baseline comparison
- **Input sizes**: Scaling from 10 to 500 words

#### 3. Consensus Engine (`consensus_engine.rs`)
- **Single-round**: Fast path (high agreement)
- **Multi-round**: Revision required
- **Thresholds**: 0.75, 0.80, 0.85, 0.90, 0.95
- **Agent counts**: 3, 5, 7, 10 agents
- **Calculation overhead**: Weighted averaging
- **Round tracking**: Round progression logic
- **Timeout handling**: 5-second timeout enforcement

#### 4. Knowledge Vault (`knowledge_vault.rs`)
- **Chunking strategies**: Token-based (128, 256, 512, 1024)
- **Character chunking**: Character-based splitting
- **Embedding generation**: Placeholder embeddings (384 dims)
- **Embedding sizes**: 10 to 1000 words
- **Vector search**: Cosine similarity search
- **Search scaling**: 10 to 1000 documents
- **Document indexing**: Chunk + embed pipeline
- **Top-k retrieval**: 1, 3, 5, 10 results

#### 5. Privacy Redaction (`privacy_redaction.rs`)
- **Email redaction**: Email address patterns
- **Phone redaction**: Phone number formats
- **Credit card**: Luhn algorithm validation
- **IP address**: IPv4 detection
- **API key**: Pattern matching
- **Multiple patterns**: Combined redaction
- **Document sizes**: 1KB to 100KB documents
- **Token vault**: Insert and query operations
- **Reinflation**: Token replacement
- **Pattern matching**: Regex performance

---

## Running Benchmarks

### Quick Start

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench query_processing

# Run specific benchmark function
cargo bench --bench query_processing -- query_simple

# Save baseline for future comparisons
cargo bench -- --save-baseline main

# Compare against baseline
cargo bench -- --baseline main
```

### Using the Scripts

For automated benchmark execution with reporting:

```bash
# Full benchmark suite (5-10 minutes)
./scripts/benchmark.sh

# Quick performance test (1-2 minutes)
./scripts/performance_test.sh
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

**Interpretation**:
- **Mean**: 2.545s (average execution time)
- **Confidence interval**: 2.100s - 2.890s (95% confidence)
- **Change**: +0.5% compared to previous run (within noise, not significant)
- **p-value**: 0.85 (> 0.05 means not statistically significant)

### HTML Reports

After running benchmarks, detailed HTML reports are generated in `target/criterion/`:

```bash
# Open report in browser
open target/criterion/report/index.html  # macOS
xdg-open target/criterion/report/index.html  # Linux
start target/criterion/report/index.html  # Windows
```

**HTML report features**:
- Detailed statistics (mean, median, std dev, min, max)
- Performance over time graphs
- Comparison with previous runs
- Confidence interval visualizations
- Welch t-test for significance testing

---

## Benchmark Writing Guide

### Template

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_my_function(c: &mut Criterion) {
    c.bench_function("my_function", |b| {
        b.iter(|| {
            let input = black_box(my_input);
            let result = my_function(input);
            black_box(result)
        })
    });
}

criterion_group!(benches, bench_my_function);
criterion_main!(benches);
```

### Best Practices

1. **Use `black_box`** to prevent compiler optimizations
   ```rust
   black_box(input)  // Prevents input from being optimized away
   black_box(result) // Prevents result from being discarded
   ```

2. **Parameterized benchmarks** for testing multiple inputs
   ```rust
   let mut group = c.benchmark_group("my_group");
   for size in [10, 50, 100].iter() {
       group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
           b.iter(|| {
               black_box(process(size))
           })
       });
   }
   group.finish();
   ```

3. **Async benchmarks** use `to_async`
   ```rust
   let rt = tokio::runtime::Runtime::new().unwrap();
   c.bench_function("async_function", |b| {
       b.to_async(&rt).iter(|| {
           async {
               black_box(async_function().await)
           }
       })
   });
   ```

4. **Avoid I/O** in benchmark loops (disk, network)
   - Use in-memory data structures
   - Mock external dependencies
   - Isolate the code being tested

5. **Warm-up** happens automatically (Criterion handles this)

---

## Benchmark Baselines

### Creating Baselines

```bash
# Create a baseline for comparison
cargo bench -- --save-baseline before_optimization

# Make your code changes...

# Compare against baseline
cargo bench -- --baseline before_optimization
```

### Interpreting Comparisons

```
query_processing/simple
                        time:   [2.100 s 2.545 s 2.890 s]
                        change: [+15.2% +18.3% +21.4%] (p = 0.00 < 0.05)
                        Performance has degraded.
```

**Green text**: Performance improved
**Red text**: Performance degraded
**Gray text**: No significant change

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

---

## Performance Profiling

### Flamegraphs

For deeper performance analysis, generate flamegraphs:

```bash
# Install flamegraph
cargo install flamegraph

# Generate flamegraph for a specific benchmark
cargo flamegraph --bench query_processing
```

Output: `flamegraph.svg` (visualize in browser)

### Perf Statistics

```bash
# Run with perf (Linux)
perf record -g cargo bench --bench query_processing
perf report
```

---

## Troubleshooting

### Issue: "Noise" in Results

**Symptoms**: Large confidence intervals, inconsistent results

**Solutions**:
1. Close other applications (reduce system load)
2. Ensure CPU is not thermal throttling
3. Increase benchmark warm-up time
4. Use `--sample-size` to increase iterations
   ```bash
   cargo bench -- --sample-size 100
   ```

### Issue: Compiler Optimizations

**Symptoms**: Unrealistically fast results

**Solutions**:
1. Always use `black_box()` for inputs and outputs
2. Verify code is actually being executed
3. Use `--profile release` for production-like settings

### Issue: Outliers

**Symptoms**: Extreme max values, long right tail

**Solutions**:
1. Check for background processes
2. Ensure thermal management is active
3. Use `--measurement-time` for longer runs
   ```bash
   cargo bench -- --measurement-time 30
   ```

---

## Contributing

When adding new benchmarks:

1. **Follow naming convention**: `bench_<component>_<operation>`
2. **Add documentation**: Explain what's being tested
3. **Test on multiple machines**: Verify reproducibility
4. **Update BENCHMARKS.md**: Document results
5. **Review baseline comparisons**: Check for regressions

---

## See Also

- **Criterion.rs Book**: https://bheisler.github.io/criterion.rs/book/
- **BENCHMARKS.md**: Detailed performance results
- **scripts/benchmark.sh**: Automated benchmark runner
- **scripts/performance_test.sh**: Quick smoke tests

---

**Last Updated**: 2026-01-07
**Benchmark Suite**: Criterion 0.5
**Maintainer**: SuperInstance AI Team
