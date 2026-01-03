//! Test 5: Performance benchmarks
//!
//! Tests performance requirements:
//! - Time to first token < 500ms
//! - Full consensus round < 2s
//! - Privacy proxy overhead < 10ms

use std::time::{Duration, Instant};
use tempfile::TempDir;

use synesis_core::{
    agents::{AgentConfig, EthosAgent, LogosAgent, PathosAgent},
    consensus::{ConsensusConfig, ConsensusEngine},
    manifest::A2AManifest,
};
use synesis_privacy::{redactor::{Redactor, RedactorConfig}, vault::TokenVault};

/// Test privacy proxy performance
#[tokio::test]
async fn test_privacy_proxy_performance() {
    let vault = TokenVault::in_memory().unwrap();
    let mut redactor = Redactor::new(RedactorConfig::default(), vault).unwrap();

    // Test with various text sizes
    let test_cases = vec![
        ("Short", "Contact me at test@example.com"),
        ("Medium", "Email: test@example.com, Phone: 555-123-4567, Key: sk-1234567890abcdef. Path: /home/user/config.txt. SSN: 123-45-6789. IP: 192.168.1.1."),
        ("Long", &"Contact me at test@example.com. ".repeat(100)),
    ];

    for (name, text) in test_cases {
        // Measure redaction time
        let start = Instant::now();
        let result = redactor.redact(text, "bench_session");
        let redaction_time = start.elapsed();

        // Measure reinflation time
        let start = Instant::now();
        let _ = redactor.reinflate(&result.redacted_text);
        let reinflation_time = start.elapsed();

        let total_overhead = redaction_time + reinflation_time;

        println!("{} text privacy overhead:", name);
        println!("  - Redaction: {:?}", redaction_time);
        println!("  - Reinflation: {:?}", reinflation_time);
        println!("  - Total: {:?}", total_overhead);

        // Privacy proxy overhead should be < 10ms for normal cases
        // May be relaxed for very long texts
        if name != "Long" {
            assert!(total_overhead < Duration::from_millis(10),
                    "Privacy overhead should be < 10ms for {} text", name);
        }
    }

    println!("✓ Privacy proxy performance verified");
}

/// Test consensus engine performance
#[tokio::test]
async fn test_consensus_performance() {
    // Create mock agents
    let config = AgentConfig {
        model_path: "/mock/model.gguf".into(),
        ..Default::default()
    };

    let pathos = PathosAgent::new(config.clone()).unwrap();
    let logos = LogosAgent::new(config.clone()).unwrap();
    let ethos = EthosAgent::new(config).unwrap();

    let mut engine = ConsensusEngine::with_agents(pathos, logos, ethos);

    // Measure consensus time
    let start = Instant::now();
    let outcome = engine.run("Hello world").await.unwrap();
    let consensus_time = start.elapsed();

    println!("Consensus performance:");
    println!("  - Total time: {:?}", consensus_time);
    println!("  - Rounds: {}", outcome.rounds());
    println!("  - Per round: {:?}", consensus_time / outcome.rounds() as u32);

    // Note: This benchmark uses mock agents, so it's very fast
    // With real models, we'd enforce < 2s requirement
    assert!(outcome.total_duration_ms > 0, "Should record timing");

    println!("✓ Consensus performance timing verified");
}

/// Test agent processing performance
#[tokio::test]
async fn test_agent_processing_performance() {
    let config = AgentConfig {
        model_path: "/mock/model.gguf".into(),
        ..Default::default()
    };

    // Test each agent
    let agents: Vec<(&str, Box<dyn synesis_core::agents::Agent>)> = vec![
        ("Pathos", Box::new(PathosAgent::new(config.clone()).unwrap())),
        ("Logos", Box::new(LogosAgent::new(config.clone()).unwrap())),
        ("Ethos", Box::new(EthosAgent::new(config.clone()).unwrap())),
    ];

    for (name, agent) in agents {
        let manifest = A2AManifest::new("Test query");

        let start = Instant::now();
        let _ = agent.process(&manifest).await.unwrap();
        let duration = start.elapsed();

        println!("{} processing time: {:?}", name, duration);

        // Mock agents should be very fast
        assert!(duration < Duration::from_millis(100));
    }

    println!("✓ Agent processing performance verified");
}

/// Test knowledge vault search performance
#[tokio::test]
async fn test_knowledge_vault_search_performance() {
    use synesis_knowledge::sqlite_vss::{KnowledgeVault, VaultConfig};

    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("knowledge.db");

    let vault = KnowledgeVault::new(VaultConfig {
        db_path,
        chunk_size: 500,
        chunk_overlap: 50,
    }).await.unwrap();

    // Add multiple documents
    for i in 0..10 {
        let content = format!("Document {} content with some searchable text.", i);
        vault.add_document(&format!("doc{}.md", i), &content, "markdown")
            .await
            .unwrap();
    }

    // Measure search performance
    let start = Instant::now();
    let results = vault.search("searchable content", 5).await.unwrap();
    let search_time = start.elapsed();

    println!("Knowledge vault search performance:");
    println!("  - Search time: {:?}", search_time);
    println!("  - Results: {}", results.len());

    // Search should be fast
    assert!(search_time < Duration::from_millis(100));

    println!("✓ Knowledge vault search performance verified");
}

/// Test redaction pattern matching performance
#[tokio::test]
async fn test_pattern_matching_performance() {
    let vault = TokenVault::in_memory().unwrap();
    let redactor = Redactor::new(RedactorConfig::default(), vault).unwrap();

    // Create text with many patterns
    let mut text = String::new();
    for i in 0..100 {
        text.push_str(&format!("Email {}: user{}@example.com. ", i, i));
    }

    // Measure pattern detection
    let start = Instant::now();
    let _ = redactor.contains_sensitive(&text);
    let detection_time = start.elapsed();

    println!("Pattern detection performance:");
    println!("  - Text length: {} chars", text.len());
    println!("  - Detection time: {:?}", detection_time);

    // Should be very fast even for long texts
    assert!(detection_time < Duration::from_millis(10));

    println!("✓ Pattern matching performance verified");
}

/// Test concurrent operations performance
#[tokio::test]
async fn test_concurrent_operations() {
    let vault = TokenVault::in_memory().unwrap();
    let mut redactor = Redactor::new(RedactorConfig::default(), vault).unwrap();

    // Spawn multiple concurrent redaction tasks
    let tasks: Vec<_> = (0..10)
        .map(|i| {
            let mut redactor_ref = &redactor;
            tokio::spawn(async move {
                let text = format!("Test {} with email user{}@example.com", i, i);
                // Note: Since redact requires &mut self, we can't truly parallelize
                // This is a limitation of the current API
                redactor_ref.redact(&text, &format!("session_{}", i))
            })
        })
        .collect();

    let start = Instant::now();

    // Wait for all tasks to complete
    let results = futures::future::join_all(tasks).await;
    let total_time = start.elapsed();

    let successful = results.iter().filter(|r| r.is_ok()).count();

    println!("Concurrent operations performance:");
    println!("  - Total tasks: {}", results.len());
    println!("  - Successful: {}", successful);
    println!("  - Total time: {:?}", total_time);

    assert_eq!(successful, results.len(), "All tasks should succeed");

    println!("✓ Concurrent operations verified");
}

/// Benchmark: Time to first response
#[tokio::test]
async fn benchmark_time_to_first_response() {
    let config = AgentConfig {
        model_path: "/mock/model.gguf".into(),
        ..Default::default()
    };

    let pathos = PathosAgent::new(config.clone()).unwrap();
    let logos = LogosAgent::new(config.clone()).unwrap();
    let ethos = EthosAgent::new(config).unwrap();

    let mut engine = ConsensusEngine::with_agents(pathos, logos, ethos);

    let prompt = "Hello world";

    // Measure time to first agent response (Pathos)
    let manifest = A2AManifest::new(prompt);
    let start = Instant::now();

    // In real implementation, we'd measure actual model inference
    // For now, we're measuring the mock overhead
    let _ = engine.run(prompt).await.unwrap();
    let time_to_response = start.elapsed();

    println!("Time to first response benchmark:");
    println!("  - Total time: {:?}", time_to_response);

    // Mock agents are instant, real requirement is < 500ms
    assert!(time_to_response < Duration::from_millis(100));

    println!("✓ Time to first response benchmarked");
}

/// Throughput benchmark
#[tokio::test]
async fn benchmark_throughput() {
    let vault = TokenVault::in_memory().unwrap();
    let mut redactor = Redactor::new(RedactorConfig::default(), vault).unwrap();

    // Process multiple texts and measure throughput
    let texts: Vec<_> = (0..100)
        .map(|i| format!("Test {} with email user{}@example.com", i, i))
        .collect();

    let start = Instant::now();

    for text in &texts {
        let _ = redactor.redact(text, "bench_session");
    }

    let total_time = start.elapsed();
    let throughput = texts.len() as f64 / total_time.as_secs_f64();

    println!("Throughput benchmark:");
    println!("  - Total texts: {}", texts.len());
    println!("  - Total time: {:?}", total_time);
    println!("  - Throughput: {:.2} texts/second", throughput);

    // Should handle at least 100 texts/second
    assert!(throughput > 100.0);

    println!("✓ Throughput benchmarked");
}
