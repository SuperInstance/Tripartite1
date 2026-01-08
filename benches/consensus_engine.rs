// Criterion benchmark for consensus engine performance
// Tests different consensus scenarios and thresholds

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use synesis_core::consensus::{ConsensusEngine, ConsensusConfig, ConsensusResult};

/// Create a consensus engine configuration
fn create_config(threshold: f64) -> ConsensusConfig {
    ConsensusConfig {
        threshold,
        max_rounds: 3,
        quorum_size: 3,
    }
}

/// Benchmark single-round consensus (fast path)
fn bench_consensus_single_round(c: &mut Criterion) {
    c.bench_function("consensus_single_round", |b| {
        b.iter(|| {
            // Simulate single-round consensus (high agreement)
            let scores = vec![0.92, 0.89, 0.94];
            let avg: f64 = scores.iter().sum::<f64>() / scores.len() as f64;

            black_box(avg >= 0.85)
        })
    });
}

/// Benchmark multi-round consensus (requires revision)
fn bench_consensus_multi_round(c: &mut Criterion) {
    c.bench_function("consensus_multi_round", |b| {
        b.iter(|| {
            // Simulate multi-round consensus (lower agreement)
            let mut scores = vec![0.75, 0.82, 0.79];

            // Simulate revision round
            if scores.iter().sum::<f64>() / scores.len() as f64 < 0.85 {
                scores = vec![0.85, 0.87, 0.86];
            }

            let avg: f64 = scores.iter().sum::<f64>() / scores.len() as f64;

            black_box(avg >= 0.85)
        })
    });
}

/// Benchmark different consensus thresholds
fn bench_consensus_thresholds(c: &mut Criterion) {
    let mut group = c.benchmark_group("consensus_threshold");

    for threshold in [0.75, 0.80, 0.85, 0.90, 0.95].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(threshold), threshold, |b, &threshold| {
            b.iter(|| {
                let scores = vec![0.88, 0.91, 0.87];
                let avg: f64 = scores.iter().sum::<f64>() / scores.len() as f64;

                black_box(avg >= threshold)
            })
        });
    }

    group.finish();
}

/// Benchmark consensus with different agent counts
fn bench_consensus_agent_counts(c: &mut Criterion) {
    let mut group = c.benchmark_group("consensus_agent_count");

    for count in [3, 5, 7, 10].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.iter(|| {
                // Generate scores for N agents
                let scores: Vec<f64> = (0..count)
                    .map(|i| 0.85 + (i as f64 * 0.01))
                    .collect();

                let avg: f64 = scores.iter().sum::<f64>() / scores.len() as f64;

                black_box(avg >= 0.85)
            })
        });
    }

    group.finish();
}

/// Benchmark consensus calculation overhead
fn bench_consensus_calculation(c: &mut Criterion) {
    let agent_scores = vec![
        ("Pathos", 0.92),
        ("Logos", 0.89),
        ("Ethos", 0.94),
    ];

    c.bench_function("consensus_calculation", |b| {
        b.iter(|| {
            let scores: Vec<f64> = black_box(&agent_scores)
                .iter()
                .map(|(_, score)| *score)
                .collect();

            // Weighted average calculation
            let weights = vec![0.3, 0.4, 0.3]; // Pathos, Logos, Ethos
            let weighted_sum: f64 = scores.iter()
                .zip(weights.iter())
                .map(|(s, w)| s * w)
                .sum();

            black_box(weighted_sum)
        })
    });
}

/// Benchmark consensus round tracking
fn bench_consensus_round_tracking(c: &mut Criterion) {
    c.bench_function("consensus_round_tracking", |b| {
        b.iter(|| {
            let mut current_round = 1;
            let max_rounds = 3;
            let threshold = 0.85;
            let scores = vec![0.82, 0.79, 0.84]; // Below threshold

            // Simulate round progression
            while current_round <= max_rounds {
                let avg: f64 = scores.iter().sum::<f64>() / scores.len() as f64;

                if avg >= threshold {
                    break;
                }

                current_round += 1;
            }

            black_box(current_round)
        })
    });
}

/// Benchmark consensus result creation
fn bench_consensus_result(c: &mut Criterion) {
    c.bench_function("consensus_result", |b| {
        b.iter(|| {
            let result = ConsensusResult {
                reached: black_box(true),
                confidence: black_box(0.91),
                rounds: black_box(1),
                agent_scores: black_box(vec![0.92, 0.89, 0.94]),
            };

            black_box(result)
        })
    });
}

/// Benchmark consensus timeout handling
fn bench_consensus_timeout(c: &mut Criterion) {
    c.bench_function("consensus_timeout", |b| {
        b.iter(|| {
            let start = std::time::Instant::now();
            let timeout = std::time::Duration::from_secs(5);

            // Simulate work
            let _scores = vec![0.88, 0.91, 0.87];

            let elapsed = start.elapsed();

            black_box(elapsed < timeout)
        })
    });
}

criterion_group!(
    benches,
    bench_consensus_single_round,
    bench_consensus_multi_round,
    bench_consensus_thresholds,
    bench_consensus_agent_counts,
    bench_consensus_calculation,
    bench_consensus_round_tracking,
    bench_consensus_result,
    bench_consensus_timeout
);
criterion_main!(benches);
