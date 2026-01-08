//! Hello World - Your First SuperInstance Query
//!
//! This example demonstrates the simplest way to use SuperInstance AI:
//! initialize the council and run a query.
//!
//! # Prerequisites
//!
//! - SuperInstance AI installed and initialized
//! - Run `synesis init` first
//!
//! # Related Documentation
//!
//! - [Getting Started](../../docs/tutorials/getting-started.md)
//! - [Council API](../../docs/api/council-api.md)
//! - [Your First Query](../../docs/tutorials/your-first-query.md)
//!
//! # Expected Output
//!
//! ```text
//! Query: What is Rust?
//!
//! 🤔 Pathos: User wants to know about Rust programming language
//! 🧠 Logos: Retrieving information about Rust...
//! ✅ Ethos: Verifying technical accuracy...
//!
//! ✅ Consensus reached (0.92 confidence)
//!
//! Rust is a systems programming language that runs blazingly fast,
//! prevents segfaults, and guarantees thread safety.
//! ```

use synesis_core::{Council, CouncilConfig};
use synesis_core::manifest::A2AManifest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging (optional, for debugging)
    tracing_subscriber::fmt::init();

    // Create council with default configuration
    println!("🔧 Initializing SuperInstance AI Council...\n");
    let mut council = Council::new(CouncilConfig::default());

    // Initialize all agents (loads models if needed)
    council.initialize().await?;
    println!("✅ Council ready!\n");

    // Create a query
    let query = "What is Rust?".to_string();

    // Create manifest for tracking
    let manifest = A2AManifest::new(query.clone());

    println!("📝 Query: {}\n", query);
    println!("⏳ Processing...\n");

    // Process the query through tripartite council
    let response = council.process(manifest).await?;

    // Display results
    println!("✅ Consensus reached ({:.0}% confidence)\n", response.confidence * 100.0);
    println!("{}\n", response.content);

    // Show metadata
    println!("---");
    println!("Agents: {}/{} agreed", response.agents_agreed, response.total_agents);
    println!("Rounds: {}", response.rounds);
    println!("Time: {:.2}s", response.duration.as_secs_f64());

    Ok(())
}

/// This example shows:
///
/// 1. **Council Creation**: Using `Council::new()` with default config
/// 2. **Initialization**: Loading models and preparing agents
/// 3. **Query Processing**: Using `A2AManifest` to track queries
/// 4. **Response Handling**: Accessing confidence, content, and metadata
///
/// # Key Concepts
///
/// - **Council**: Orchestrates the three-agent system
/// - **Manifest**: Tracks query metadata and conversation state
/// - **Response**: Contains content, confidence, and agent agreement info
///
/// # Next Steps
///
/// - Try [custom_config.rs](custom_config.rs) to customize council behavior
/// - See [batch_queries.rs](batch_queries.rs) for processing multiple queries
/// - Explore [knowledge/](../knowledge/) examples for RAG usage
