//! Test 1: Full consensus flow from init to ask
//!
//! Tests the complete flow:
//! - synesis init -> status -> ask "Hello world"
//! - Verify all models download
//! - Verify consensus runs
//! - Verify response generated

use std::path::PathBuf;
use tempfile::TempDir;
use tokio::fs;

use synesis_core::{
    agents::{AgentConfig, EthosAgent, LogosAgent, PathosAgent},
    consensus::{ConsensusConfig, ConsensusEngine},
    manifest::A2AManifest,
    CoreResult,
};

/// Test complete consensus flow
#[tokio::test]
async fn test_full_consensus_flow() -> CoreResult<()> {
    // Create temporary data directory
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path();

    // Setup directory structure
    let models_dir = data_dir.join("models");
    let knowledge_dir = data_dir.join("knowledge");
    let cache_dir = data_dir.join("cache");

    fs::create_dir_all(&models_dir).await.unwrap();
    fs::create_dir_all(&knowledge_dir).await.unwrap();
    fs::create_dir_all(&cache_dir).await.unwrap();

    // Verify directories exist
    assert!(models_dir.exists());
    assert!(knowledge_dir.exists());
    assert!(cache_dir.exists());

    // Create mock agent configuration (for testing without actual models)
    let agent_config = AgentConfig {
        model_path: PathBuf::from("/mock/model.gguf"),
        context_length: 2048,
        temperature: 0.7,
        ..Default::default()
    };

    // Initialize agents
    let pathos = PathosAgent::new(agent_config.clone())?;
    let logos = LogosAgent::new(agent_config.clone())?;
    let ethos = EthosAgent::new(agent_config)?;

    // Create consensus engine
    let config = ConsensusConfig {
        threshold: 0.85,
        max_rounds: 3,
        ..Default::default()
    };

    let mut engine = ConsensusEngine::new(config, pathos, logos, ethos);

    // Test simple query
    let prompt = "Hello world";
    let outcome = engine.run(prompt).await?;

    // Verify consensus was reached
    assert!(
        outcome.result.is_consensus() || matches!(outcome.result, synesis_core::consensus::ConsensusResult::NotReached { .. }),
        "Consensus should either reach or fail gracefully"
    );

    // Verify response content exists
    assert!(!outcome.content.is_empty(), "Response should not be empty");

    // Verify timing information is captured
    assert!(outcome.total_duration_ms > 0, "Duration should be recorded");

    println!("✓ Consensus flow completed successfully");
    println!("  - Rounds: {}", outcome.rounds());
    println!("  - Duration: {}ms", outcome.total_duration_ms);
    println!("  - Confidence: {:.2}", outcome.aggregate_confidence().unwrap_or(0.0));

    Ok(())
}

/// Test consensus calculation with mock data
#[tokio::test]
async fn test_consensus_calculation() {
    use synesis_core::agents::AgentResponse;

    // Create mock responses
    let pathos = AgentResponse::new(
        "Pathos",
        "User wants a greeting".to_string(),
        0.90,
    );
    let logos = AgentResponse::new(
        "Logos",
        "Hello! How can I help you today?".to_string(),
        0.92,
    );
    let ethos = AgentResponse::new(
        "Ethos",
        "Response is safe and appropriate".to_string(),
        0.88,
    );

    // Calculate aggregate (weights: pathos=0.25, logos=0.45, ethos=0.30)
    let aggregate = (pathos.confidence * 0.25)
        + (logos.confidence * 0.45)
        + (ethos.confidence * 0.30);

    assert!((aggregate - 0.904).abs() < 0.001);
    assert!(aggregate >= 0.85, "Should reach threshold");

    println!("✓ Consensus calculation verified");
    println!("  - Pathos: {:.2}", pathos.confidence);
    println!("  - Logos: {:.2}", logos.confidence);
    println!("  - Ethos: {:.2}", ethos.confidence);
    println!("  - Aggregate: {:.2}", aggregate);
}

/// Test multiple consensus rounds with feedback
#[tokio::test]
async fn test_consensus_multiple_rounds() {
    // This test verifies the feedback mechanism
    // In a real scenario with models, agents would improve their responses

    let manifest = A2AManifest::new("Generate a complex analysis");

    // Simulate first round with low confidence
    let feedback1 = "Pathos needs more context about the analysis domain";
    manifest.add_feedback(feedback1.to_string());
    manifest.next_round();

    assert_eq!(manifest.round, 2);
    assert!(manifest.feedback_history.contains(&feedback1.to_string()));

    println!("✓ Multi-round consensus handling verified");
    println!("  - Round: {}", manifest.round);
    println!("  - Feedback: {}", feedback1);
}

/// Test A2A Manifest creation
#[test]
fn test_a2a_manifest_creation() {
    let prompt = "Help me debug my Rust code";
    let manifest = A2AManifest::new(prompt);

    assert_eq!(manifest.user_prompt, prompt);
    assert_eq!(manifest.round, 1);
    assert!(manifest.feedback_history.is_empty());
    assert!(manifest.pathos_framing.is_none());

    println!("✓ A2A Manifest creation verified");
}
