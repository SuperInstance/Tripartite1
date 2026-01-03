//! Benchmarks for the consensus engine
//!
//! Run with: cargo bench --bench consensus

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::collections::HashMap;

// Mock types for benchmarking (since we can't import from workspace here)

#[derive(Clone)]
struct AgentResponse {
    agent: String,
    content: String,
    confidence: f32,
    reasoning: Option<String>,
    tokens_used: u32,
    latency_ms: u64,
    metadata: HashMap<String, serde_json::Value>,
}

impl AgentResponse {
    fn new(agent: &str, content: &str, confidence: f32) -> Self {
        Self {
            agent: agent.to_string(),
            content: content.to_string(),
            confidence,
            reasoning: None,
            tokens_used: 0,
            latency_ms: 0,
            metadata: HashMap::new(),
        }
    }
}

struct ConsensusConfig {
    threshold: f32,
    max_rounds: u8,
    weights: AgentWeights,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            threshold: 0.85,
            max_rounds: 3,
            weights: AgentWeights::default(),
        }
    }
}

struct AgentWeights {
    pathos: f32,
    logos: f32,
    ethos: f32,
}

impl Default for AgentWeights {
    fn default() -> Self {
        Self {
            pathos: 0.25,
            logos: 0.45,
            ethos: 0.30,
        }
    }
}

struct ConsensusEngine {
    config: ConsensusConfig,
}

impl ConsensusEngine {
    fn new(config: ConsensusConfig) -> Self {
        Self { config }
    }

    fn calculate_aggregate(&self, pathos: f32, logos: f32, ethos: f32) -> f32 {
        (pathos * self.config.weights.pathos)
            + (logos * self.config.weights.logos)
            + (ethos * self.config.weights.ethos)
    }

    fn evaluate(
        &self,
        pathos: &AgentResponse,
        logos: &AgentResponse,
        ethos: &AgentResponse,
    ) -> bool {
        let aggregate = self.calculate_aggregate(
            pathos.confidence,
            logos.confidence,
            ethos.confidence,
        );
        aggregate >= self.config.threshold
    }
}

fn bench_aggregate_calculation(c: &mut Criterion) {
    let engine = ConsensusEngine::new(ConsensusConfig::default());

    c.bench_function("calculate_aggregate", |b| {
        b.iter(|| {
            engine.calculate_aggregate(
                black_box(0.9),
                black_box(0.85),
                black_box(0.88),
            )
        })
    });
}

fn bench_consensus_evaluation(c: &mut Criterion) {
    let engine = ConsensusEngine::new(ConsensusConfig::default());
    
    let pathos = AgentResponse::new("Pathos", "Intent framing", 0.9);
    let logos = AgentResponse::new("Logos", "Response content", 0.85);
    let ethos = AgentResponse::new("Ethos", "Verified", 0.88);

    c.bench_function("evaluate_consensus", |b| {
        b.iter(|| {
            engine.evaluate(
                black_box(&pathos),
                black_box(&logos),
                black_box(&ethos),
            )
        })
    });
}

fn bench_varying_confidence(c: &mut Criterion) {
    let engine = ConsensusEngine::new(ConsensusConfig::default());
    
    let mut group = c.benchmark_group("consensus_with_varying_confidence");
    
    // Test with different confidence levels
    let test_cases = vec![
        ("low", 0.5, 0.5, 0.5),
        ("medium", 0.7, 0.7, 0.7),
        ("high", 0.9, 0.9, 0.9),
        ("mixed", 0.9, 0.6, 0.8),
        ("threshold", 0.85, 0.85, 0.85),
    ];

    for (name, p, l, e) in test_cases {
        let pathos = AgentResponse::new("Pathos", "Content", p);
        let logos = AgentResponse::new("Logos", "Content", l);
        let ethos = AgentResponse::new("Ethos", "Content", e);
        
        group.bench_with_input(
            BenchmarkId::new("evaluate", name),
            &(&pathos, &logos, &ethos),
            |b, (p, l, e)| {
                b.iter(|| engine.evaluate(p, l, e))
            },
        );
    }
    
    group.finish();
}

fn bench_multiple_rounds(c: &mut Criterion) {
    let engine = ConsensusEngine::new(ConsensusConfig::default());
    
    let mut group = c.benchmark_group("consensus_rounds");
    
    for rounds in [1, 2, 3] {
        group.bench_with_input(
            BenchmarkId::new("rounds", rounds),
            &rounds,
            |b, &rounds| {
                b.iter(|| {
                    for _ in 0..rounds {
                        let pathos = AgentResponse::new("Pathos", "Content", 0.85);
                        let logos = AgentResponse::new("Logos", "Content", 0.80);
                        let ethos = AgentResponse::new("Ethos", "Content", 0.82);
                        
                        black_box(engine.evaluate(&pathos, &logos, &ethos));
                    }
                })
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_aggregate_calculation,
    bench_consensus_evaluation,
    bench_varying_confidence,
    bench_multiple_rounds,
);

criterion_main!(benches);
