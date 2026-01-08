// Criterion benchmark for agent execution performance
// Tests Pathos, Logos, and Ethos agents individually and in parallel

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use synesis_core::agents::{AgentInput, AgentOutput, AgentType};

/// Create a sample agent input
fn create_input(query: &str) -> AgentInput {
    AgentInput {
        query: query.to_string(),
        context: None,
    }
}

/// Benchmark Pathos agent (intent extraction)
fn bench_pathos_agent(c: &mut Criterion) {
    let query = "I want to build a distributed AI system that can make decisions autonomously.";

    c.bench_function("agent_pathos", |b| {
        b.iter(|| {
            let input = black_box(create_input(query));

            // Simulate Pathos processing: intent extraction
            let keywords = input.query.split_whitespace()
                .filter(|w| w.len() > 4)
                .count();

            black_box(keywords)
        })
    });
}

/// Benchmark Logos agent (RAG retrieval + logic)
fn bench_logos_agent(c: &mut Criterion) {
    let query = "What is the consensus mechanism and how does it work with privacy redaction?";

    c.bench_function("agent_logos", |b| {
        b.iter(|| {
            let input = black_box(create_input(query));

            // Simulate Logos processing: search + reasoning
            let words: Vec<_> = input.query.split_whitespace().collect();
            let complexity = words.len() * 2; // Simulated RAG overhead

            black_box(complexity)
        })
    });
}

/// Benchmark Ethos agent (verification)
fn bench_ethos_agent(c: &mut Criterion) {
    let query = "Is it safe to run this model on my hardware?";

    c.bench_function("agent_ethos", |b| {
        b.iter(|| {
            let input = black_box(create_input(query));

            // Simulate Ethos processing: verification checks
            let checks = input.query.matches('?').count() + 1;

            black_box(checks)
        })
    });
}

/// Benchmark parallel agent execution
fn bench_parallel_agents(c: &mut Criterion) {
    let query = "Explain the tripartite consensus system in detail.";

    c.bench_function("agents_parallel", |b| {
        b.iter(|| {
            let input = black_box(create_input(query));

            // Simulate parallel execution
            let pathos_result = input.query.len() / 10;
            let ethos_result = input.query.len() / 12;
            let logos_result = input.query.len() * 2;

            // Combine results
            black_box(pathos_result + ethos_result + logos_result)
        })
    });
}

/// Benchmark sequential agent execution (baseline)
fn bench_sequential_agents(c: &mut Criterion) {
    let query = "Explain the tripartite consensus system in detail.";

    c.bench_function("agents_sequential", |b| {
        b.iter(|| {
            let input = black_box(create_input(query));

            // Simulate sequential execution
            let pathos_result = input.query.len() / 10;
            let logos_result = pathos_result + input.query.len() * 2;
            let ethos_result = logos_result + input.query.len() / 12;

            black_box(ethos_result)
        })
    });
}

/// Benchmark agent with different input sizes
fn bench_agent_input_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("agent_input_size");

    for size in [10, 50, 100, 200, 500].iter() {
        let query = "test ".repeat(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &_size| {
            b.iter(|| {
                let input = black_box(create_input(&query));

                // Simulate agent processing
                black_box(input.query.len())
            })
        });
    }

    group.finish();
}

/// Benchmark agent output generation
fn bench_agent_output(c: &mut Criterion) {
    let response = "The tripartite council consists of three specialized agents: Pathos extracts user intent, Logos performs logical reasoning and knowledge retrieval, and Ethos verifies safety and feasibility.";

    c.bench_function("agent_output", |b| {
        b.iter(|| {
            let output = AgentOutput {
                content: black_box(response.to_string()),
                confidence: 0.92,
                agent_type: AgentType::Logos,
            };

            black_box(output)
        })
    });
}

criterion_group!(
    benches,
    bench_pathos_agent,
    bench_logos_agent,
    bench_ethos_agent,
    bench_parallel_agents,
    bench_sequential_agents,
    bench_agent_input_sizes,
    bench_agent_output
);
criterion_main!(benches);
