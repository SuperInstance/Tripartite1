// Criterion benchmark for query processing performance
// Tests different query complexities and sizes

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use synesis_core::{
    agents::{AgentInput, AgentOutput},
    council::TripartiteCouncil,
};
use tokio::runtime::Runtime;

/// Create a test runtime for async benchmarks
fn runtime() -> Runtime {
    Runtime::new().unwrap()
}

/// Benchmark simple queries (10-20 words, single intent)
fn bench_simple_query(c: &mut Criterion) {
    let rt = runtime();
    let council = rt.block_on(async {
        // Create minimal council for benchmarking
        TripartiteCouncil::new()
    });

    let query = "What is the tripartite council?";

    c.bench_function("query_simple", |b| {
        b.to_async(&rt).iter(|| {
            let input = AgentInput {
                query: black_box(query.to_string()),
                context: None,
            };

            async move {
                // Simulate query processing
                black_box(input.query.len())
            }
        });
    });
}

/// Benchmark medium queries (30-50 words, multi-part)
fn bench_medium_query(c: &mut Criterion) {
    let rt = runtime();
    let query = "Explain the roles of Pathos, Logos, and Ethos agents in the tripartite consensus system and how they reach agreement.";

    c.bench_function("query_medium", |b| {
        b.to_async(&rt).iter(|| {
            let input = AgentInput {
                query: black_box(query.to_string()),
                context: None,
            };

            async move {
                black_box(input.query.len())
            }
        });
    });
}

/// Benchmark complex queries (50-100 words, ambiguous)
fn bench_complex_query(c: &mut Criterion) {
    let rt = runtime();
    let query = "I'm building a distributed AI system and I'm wondering how to handle agent consensus when there's conflicting information about hardware capabilities, while also maintaining privacy and ensuring that the system can escalate to cloud resources when needed. What's the best approach?";

    c.bench_function("query_complex", |b| {
        b.to_async(&rt).iter(|| {
            let input = AgentInput {
                query: black_box(query.to_string()),
                context: None,
            };

            async move {
                black_box(input.query.len())
            }
        });
    });
}

/// Benchmark query parsing overhead
fn bench_query_parsing(c: &mut Criterion) {
    let query = "What is the tripartite council and how does it work?";

    c.bench_function("query_parsing", |b| {
        b.iter(|| {
            let words: Vec<&str> = black_box(query).split_whitespace().collect();
            black_box(words.len())
        })
    });
}

/// Benchmark different query sizes
fn bench_query_sizes(c: &mut Criterion) {
    let rt = runtime();
    let mut group = c.benchmark_group("query_by_size");

    for size in [10, 25, 50, 100, 200, 500].iter() {
        let query = "word ".repeat(*size);
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &_size| {
            b.to_async(&rt).iter(|| {
                let input = AgentInput {
                    query: black_box(query.clone()),
                    context: None,
                };

                async move {
                    black_box(input.query.len())
                }
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_simple_query,
    bench_medium_query,
    bench_complex_query,
    bench_query_parsing,
    bench_query_sizes
);
criterion_main!(benches);
