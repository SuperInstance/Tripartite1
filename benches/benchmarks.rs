//! Benchmarks for SuperInstance AI
//!
//! Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

/// Benchmark consensus calculation
fn bench_consensus_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("consensus");
    
    // Test different weight configurations
    let weights = vec![
        ("default", (0.25, 0.45, 0.30)),
        ("equal", (0.33, 0.34, 0.33)),
        ("logos_heavy", (0.20, 0.60, 0.20)),
    ];
    
    for (name, (pathos, logos, ethos)) in weights {
        group.bench_with_input(
            BenchmarkId::new("aggregate", name),
            &(pathos, logos, ethos),
            |b, (p, l, e)| {
                b.iter(|| {
                    // Simulate consensus calculation
                    let scores = (0.9_f32, 0.85_f32, 0.88_f32);
                    let aggregate = black_box(
                        scores.0 * p + scores.1 * l + scores.2 * e
                    );
                    aggregate
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark privacy redaction patterns
fn bench_redaction(c: &mut Criterion) {
    let mut group = c.benchmark_group("redaction");
    
    // Test inputs of varying sizes
    let inputs = vec![
        ("small", "Contact john@example.com for details."),
        ("medium", "Please reach out to john@example.com or call 555-123-4567. \
                   Our office is at 123 Main St. API key: sk-abc123xyz789"),
        ("large", &"Contact support@company.com for help. ".repeat(100)),
    ];
    
    for (name, input) in inputs {
        group.bench_with_input(
            BenchmarkId::new("email_pattern", name),
            &input,
            |b, input| {
                let pattern = regex::Regex::new(
                    r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}"
                ).unwrap();
                
                b.iter(|| {
                    black_box(pattern.replace_all(input, "[EMAIL_REDACTED]"))
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark token estimation
fn bench_token_estimation(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokens");
    
    let texts = vec![
        ("short", "Hello, world!"),
        ("medium", "The quick brown fox jumps over the lazy dog. ".repeat(10).as_str()),
        ("long", &"Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(100)),
    ];
    
    for (name, text) in texts {
        group.bench_with_input(
            BenchmarkId::new("estimate", name),
            &text,
            |b, text| {
                b.iter(|| {
                    // Simple character-based estimation
                    black_box(text.len() / 4)
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark keyword extraction
fn bench_keyword_extraction(c: &mut Criterion) {
    let stop_words: std::collections::HashSet<&str> = [
        "the", "a", "an", "is", "are", "was", "were", "be", "been", "being",
        "have", "has", "had", "do", "does", "did", "will", "would", "could",
        "should", "may", "might", "must", "can", "to", "of", "in", "for",
        "on", "with", "at", "by", "from", "as", "into", "through", "during",
    ].iter().cloned().collect();
    
    c.bench_function("keyword_extraction", |b| {
        let query = "What is the best way to learn Rust programming language for systems development?";
        
        b.iter(|| {
            let keywords: Vec<&str> = query
                .to_lowercase()
                .split(|c: char| !c.is_alphanumeric())
                .filter(|word| word.len() > 2 && !stop_words.contains(word))
                .take(10)
                .collect();
            black_box(keywords)
        });
    });
}

/// Benchmark manifest serialization
fn bench_manifest_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("manifest");
    
    // Create a sample manifest-like structure
    #[derive(serde::Serialize, serde::Deserialize)]
    struct TestManifest {
        id: String,
        query: String,
        pathos_framing: Option<String>,
        logos_response: Option<String>,
        ethos_verification: Option<String>,
        round: u8,
        metadata: std::collections::HashMap<String, serde_json::Value>,
    }
    
    let manifest = TestManifest {
        id: "test-123".to_string(),
        query: "What is the meaning of life?".to_string(),
        pathos_framing: Some("User is asking a philosophical question".to_string()),
        logos_response: Some("The meaning of life is...".to_string()),
        ethos_verification: Some("Response verified".to_string()),
        round: 1,
        metadata: std::collections::HashMap::new(),
    };
    
    group.bench_function("serialize_json", |b| {
        b.iter(|| {
            black_box(serde_json::to_string(&manifest).unwrap())
        });
    });
    
    let json_str = serde_json::to_string(&manifest).unwrap();
    
    group.bench_function("deserialize_json", |b| {
        b.iter(|| {
            black_box(serde_json::from_str::<TestManifest>(&json_str).unwrap())
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_consensus_calculation,
    bench_redaction,
    bench_token_estimation,
    bench_keyword_extraction,
    bench_manifest_serialization,
);

criterion_main!(benches);
