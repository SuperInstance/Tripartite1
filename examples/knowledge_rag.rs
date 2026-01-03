//! Knowledge Vault and RAG Example
//!
//! This example demonstrates:
//! 1. Creating a knowledge vault
//! 2. Indexing documents
//! 3. Generating embeddings
//! 4. Performing similarity search (RAG)
//!
//! Run with: cargo run --example knowledge_rag

use std::error::Error;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    println!("SuperInstance AI - Knowledge Vault & RAG Example\n");

    // Step 1: Initialize knowledge vault
    println!("1. Initializing knowledge vault...");
    let vault_path = "/tmp/synesis-example-vault.db";
    let mut vault = KnowledgeVault::new(vault_path)?;
    println!("   ✓ Vault created at: {}", vault_path);

    // Step 2: Initialize embeddings model
    println!("\n2. Loading embedding model...");
    let embedder = Embedder::new("bge-micro", 384);
    println!("   ✓ Model: bge-micro");
    println!("   ✓ Dimensions: 384");

    // Step 3: Create sample documents
    println!("\n3. Creating sample documents...");
    
    let documents = vec![
        Document {
            title: "Rust Programming".to_string(),
            content: "Rust is a systems programming language focused on safety, \
                     concurrency, and performance. It achieves memory safety without \
                     garbage collection through its ownership system.".to_string(),
        },
        Document {
            title: "Python Programming".to_string(),
            content: "Python is a high-level, interpreted programming language known \
                     for its readability and simplicity. It's widely used in web \
                     development, data science, and machine learning.".to_string(),
        },
        Document {
            title: "Machine Learning Basics".to_string(),
            content: "Machine learning is a subset of artificial intelligence that \
                     enables systems to learn from data. Common approaches include \
                     supervised learning, unsupervised learning, and reinforcement learning.".to_string(),
        },
        Document {
            title: "Database Systems".to_string(),
            content: "Databases store and organize data for efficient retrieval. \
                     SQL databases use structured schemas while NoSQL databases \
                     offer flexible document models for scalability.".to_string(),
        },
    ];

    for doc in &documents {
        println!("   - {}", doc.title);
    }

    // Step 4: Chunk and embed documents
    println!("\n4. Chunking and embedding documents...");
    
    let mut all_chunks: Vec<IndexedChunk> = Vec::new();
    
    for (doc_idx, doc) in documents.iter().enumerate() {
        // Chunk the document
        let chunks = chunk_text(&doc.content, 100, 20);
        println!("   {} → {} chunks", doc.title, chunks.len());
        
        for (chunk_idx, chunk) in chunks.iter().enumerate() {
            // Generate embedding
            let embedding = embedder.embed(chunk);
            
            all_chunks.push(IndexedChunk {
                doc_title: doc.title.clone(),
                doc_idx,
                chunk_idx,
                content: chunk.clone(),
                embedding,
            });
        }
    }
    
    println!("   ✓ Total chunks indexed: {}", all_chunks.len());

    // Step 5: Perform similarity search
    println!("\n5. Performing similarity search...");
    
    let query = "How do I write safe concurrent code?";
    println!("   Query: \"{}\"", query);
    
    let query_embedding = embedder.embed(query);
    
    // Calculate similarities
    let mut results: Vec<(f32, &IndexedChunk)> = all_chunks
        .iter()
        .map(|chunk| {
            let sim = cosine_similarity(&query_embedding, &chunk.embedding);
            (sim, chunk)
        })
        .collect();
    
    // Sort by similarity (descending)
    results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    // Show top results
    println!("\n   Top 3 results:");
    for (i, (score, chunk)) in results.iter().take(3).enumerate() {
        println!("\n   {}. Score: {:.3}", i + 1, score);
        println!("      Document: {}", chunk.doc_title);
        println!("      Content: {}...", 
            &chunk.content.chars().take(80).collect::<String>());
    }

    // Step 6: Build RAG context
    println!("\n6. Building RAG context...");
    
    let context: Vec<String> = results
        .iter()
        .take(2)
        .map(|(_, chunk)| chunk.content.clone())
        .collect();
    
    println!("   Retrieved {} chunks for context", context.len());
    println!("\n   === RAG Context ===");
    for (i, ctx) in context.iter().enumerate() {
        println!("   [{}] {}", i + 1, ctx);
    }

    // Step 7: Generate response (mock)
    println!("\n7. Generating response with RAG context...");
    
    let response = format!(
        "Based on the retrieved context about Rust programming, \
         you can write safe concurrent code using Rust's ownership system \
         and borrowing rules. Rust provides memory safety without garbage \
         collection, making it ideal for concurrent programming."
    );
    
    println!("\n   Response:");
    println!("   \"{}\"", response);

    // Cleanup
    std::fs::remove_file(vault_path).ok();
    
    println!("\n✓ Example complete!");

    Ok(())
}

// Mock types for the example

struct KnowledgeVault {
    path: String,
}

impl KnowledgeVault {
    fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        Ok(Self { path: path.to_string() })
    }
}

struct Document {
    title: String,
    content: String,
}

struct IndexedChunk {
    doc_title: String,
    doc_idx: usize,
    chunk_idx: usize,
    content: String,
    embedding: Vec<f32>,
}

struct Embedder {
    model: String,
    dimensions: usize,
}

impl Embedder {
    fn new(model: &str, dimensions: usize) -> Self {
        Self {
            model: model.to_string(),
            dimensions,
        }
    }

    fn embed(&self, text: &str) -> Vec<f32> {
        // Generate deterministic pseudo-embedding based on text hash
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let hash = hasher.finish();
        
        (0..self.dimensions)
            .map(|i| {
                let seed = hash.wrapping_add(i as u64);
                ((seed % 1000) as f32 / 500.0) - 1.0  // Range [-1, 1]
            })
            .collect()
    }
}

fn chunk_text(text: &str, target_size: usize, overlap: usize) -> Vec<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut chunks = Vec::new();
    let mut start = 0;
    
    while start < words.len() {
        let end = (start + target_size).min(words.len());
        let chunk = words[start..end].join(" ");
        chunks.push(chunk);
        
        if end >= words.len() {
            break;
        }
        start = end.saturating_sub(overlap);
    }
    
    if chunks.is_empty() {
        chunks.push(text.to_string());
    }
    
    chunks
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    
    dot / (norm_a * norm_b)
}
