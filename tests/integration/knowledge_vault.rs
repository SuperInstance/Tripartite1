//! Test 3: Knowledge vault workflow
//!
//! Tests the complete RAG pipeline:
//! - Add document
//! - Ask question about document
//! - Verify RAG retrieval in response

use std::path::PathBuf;
use tempfile::TempDir;
use tokio::fs;

use synesis_knowledge::{
    sqlite_vss::{KnowledgeVault, VaultConfig},
    embeddings::{EmbeddingPipeline, EmbeddingRequest},
};

/// Test complete knowledge vault workflow
#[tokio::test]
async fn test_knowledge_vault_workflow() {
    // Create temporary database
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("knowledge.db");

    // Initialize knowledge vault
    let config = VaultConfig {
        db_path: db_path.clone(),
        chunk_size: 500,
        chunk_overlap: 50,
    };

    let vault = KnowledgeVault::new(config).await.unwrap();

    // Test 1: Add a document
    let document_content = r#"
# SuperInstance AI Architecture

SuperInstance is a tripartite agentic AI system with three specialized agents:
- Pathos: Handles user intent and persona extraction
- Logos: Performs logical reasoning and solution synthesis
- Ethos: Verifies safety, accuracy, and feasibility

The consensus engine coordinates these agents to reach agreement before responding.
All three agents must vote with confidence above 0.85 for a response to be emitted.
"#;

    let doc_id = vault
        .add_document(
            "architecture.md",
            document_content,
            "markdown",
        )
        .await
        .unwrap();

    assert!(!doc_id.is_empty());
    println!("✓ Document added with ID: {}", doc_id);

    // Test 2: Query the vault
    let query = "What are the three agents in SuperInstance?";
    let results = vault.search(query, 3).await.unwrap();

    assert!(!results.is_empty(), "Should find relevant chunks");
    println!("✓ Found {} relevant chunks", results.len());

    // Verify results contain relevant information
    let found_pathos = results.iter().any(|r| r.content.contains("Pathos"));
    let found_logos = results.iter().any(|r| r.content.contains("Logos"));
    let found_ethos = results.iter().any(|r| r.content.contains("Ethos"));

    assert!(found_pathos || found_logos || found_ethos,
            "Results should mention at least one agent");

    // Test 3: Get vault statistics
    let stats = vault.get_stats().await.unwrap();
    assert_eq!(stats.document_count, 1);
    assert!(stats.chunk_count > 0);

    println!("✓ Knowledge vault workflow completed");
    println!("  - Documents: {}", stats.document_count);
    println!("  - Chunks: {}", stats.chunk_count);
}

/// Test document retrieval
#[tokio::test]
async fn test_document_retrieval() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("knowledge.db");

    let vault = KnowledgeVault::new(VaultConfig {
        db_path,
        chunk_size: 100,
        chunk_overlap: 20,
    }).await.unwrap();

    // Add multiple documents
    let content1 = "Rust is a systems programming language focused on safety.";
    let content2 = "Python is a high-level language known for simplicity.";

    vault.add_document("rust.txt", content1, "text").await.unwrap();
    vault.add_document("python.txt", content2, "text").await.unwrap();

    // Query for Rust
    let rust_results = vault.search("systems programming language", 5).await.unwrap();
    assert!(!rust_results.is_empty());

    // Should find the Rust document
    let found_rust = rust_results.iter().any(|r| r.content.contains("Rust"));
    assert!(found_rust, "Should retrieve Rust document");

    println!("✓ Document retrieval verified");
}

/// Test chunking strategies
#[tokio::test]
async fn test_chunking_strategies() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("knowledge.db");

    let vault = KnowledgeVault::new(VaultConfig {
        db_path,
        chunk_size: 50,
        chunk_overlap: 10,
    }).await.unwrap();

    // Add long document
    let long_doc = "A".repeat(500);
    vault.add_document("long.txt", &long_doc, "text").await.unwrap();

    // Get stats
    let stats = vault.get_stats().await.unwrap();

    // Should be chunked into multiple pieces
    assert!(stats.chunk_count > 1, "Long document should be chunked");

    println!("✓ Chunking strategy verified");
    println!("  - Chunks created: {}", stats.chunk_count);
}

/// Test document deletion
#[tokio::test]
async fn test_document_deletion() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("knowledge.db");

    let vault = KnowledgeVault::new(VaultConfig {
        db_path: db_path.clone(),
        chunk_size: 100,
        chunk_overlap: 20,
    }).await.unwrap();

    // Add document
    let doc_id = vault
        .add_document("test.txt", "Test content", "text")
        .await
        .unwrap();

    // Verify it exists
    let stats_before = vault.get_stats().await.unwrap();
    assert_eq!(stats_before.document_count, 1);

    // Delete document
    vault.delete_document("test.txt").await.unwrap();

    // Verify it's gone
    let stats_after = vault.get_stats().await.unwrap();
    assert_eq!(stats_after.document_count, 0);

    println!("✓ Document deletion verified");
}

/// Test embedding similarity
#[test]
fn test_embedding_similarity() {
    // Mock embedding vectors (384 dimensions like BGE-Micro)
    let dim = 384;
    let vec1: Vec<f32> = (0..dim).map(|i| i as f32).collect();
    let vec2: Vec<f32> = (0..dim).map(|i| i as f32).collect(); // Same as vec1
    let vec3: Vec<f32> = (0..dim).map(|i| (i + dim) as f32).collect(); // Different

    // Calculate cosine similarity
    let sim_12 = cosine_similarity(&vec1, &vec2);
    let sim_13 = cosine_similarity(&vec1, &vec3);

    // Same vectors should have similarity 1.0
    assert!((sim_12 - 1.0).abs() < 0.001);

    // Different vectors should have lower similarity
    assert!(sim_13 < sim_12);
    assert!(sim_13 > 0.0); // Should still be positive for these vectors

    println!("✓ Embedding similarity verified");
    println!("  - Same vectors: {:.4}", sim_12);
    println!("  - Different vectors: {:.4}", sim_13);
}

/// Test knowledge vault persistence
#[tokio::test]
async fn test_vault_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("knowledge.db");

    // Add document
    {
        let vault = KnowledgeVault::new(VaultConfig {
            db_path: db_path.clone(),
            chunk_size: 100,
            chunk_overlap: 20,
        }).await.unwrap();

        vault.add_document("test.txt", "Test content", "text").await.unwrap();
    } // Vault goes out of scope and closes

    // Reopen and verify
    let vault = KnowledgeVault::new(VaultConfig {
        db_path: db_path.clone(),
        chunk_size: 100,
        chunk_overlap: 20,
    }).await.unwrap();

    let stats = vault.get_stats().await.unwrap();
    assert_eq!(stats.document_count, 1);

    // Verify we can search
    let results = vault.search("test", 5).await.unwrap();
    assert!(!results.is_empty());

    println!("✓ Vault persistence verified");
}

/// Test code document handling
#[tokio::test]
async fn test_code_document_handling() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("knowledge.db");

    let vault = KnowledgeVault::new(VaultConfig {
        db_path,
        chunk_size: 100,
        chunk_overlap: 20,
    }).await.unwrap();

    // Add code document
    let code = r#"
fn main() {
    println!("Hello, world!");
    let x = 42;
}
"#;

    vault.add_document("main.rs", code, "rust").await.unwrap();

    // Search for function
    let results = vault.search("main function", 5).await.unwrap();
    assert!(!results.is_empty());

    // Verify code is found
    let found_code = results.iter().any(|r| r.content.contains("fn main"));
    assert!(found_code, "Should find the main function");

    println!("✓ Code document handling verified");
}

/// Helper function for cosine similarity
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    dot / (norm_a * norm_b)
}
