// Criterion benchmark for knowledge vault performance
// Tests document chunking, embedding, and vector search

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use synesis_knowledge::{
    chunker::{DocumentChunker, ChunkStrategy},
    embeddings::PlaceholderEmbedder,
};

/// Benchmark document chunking with different strategies
fn bench_chunking_strategies(c: &mut Criterion) {
    let document = "This is a sample document. ".repeat(100);

    let mut group = c.benchmark_group("chunking_strategy");

    // Test different chunk sizes
    for size in [128, 256, 512, 1024].iter() {
        group.bench_with_input(BenchmarkId::new("tokens", size), size, |b, &size| {
            b.iter(|| {
                let chunker = DocumentChunker::new(size);
                let chunks = chunker.chunk(black_box(&document), ChunkStrategy::Token);

                black_box(chunks.len())
            })
        });
    }

    group.finish();
}

/// Benchmark character-based chunking
fn bench_character_chunking(c: &mut Criterion) {
    let document = "This is a sample document for testing character-based chunking. ".repeat(50);

    c.bench_function("chunking_character", |b| {
        b.iter(|| {
            let chunker = DocumentChunker::new(512);
            let chunks = chunker.chunk(black_box(&document), ChunkStrategy::Character);

            black_box(chunks.len())
        })
    });
}

/// Benchmark embedding generation (placeholder)
fn bench_embedding_generation(c: &mut Criterion) {
    let embedder = PlaceholderEmbedder::new();
    let text = "This is a sample text for embedding generation.";

    c.bench_function("embedding_generation", |b| {
        b.iter(|| {
            let embedding = embedder.embed(black_box(text));

            black_box(embedding.len())
        })
    });
}

/// Benchmark embedding generation for different text sizes
fn bench_embedding_sizes(c: &mut Criterion) {
    let embedder = PlaceholderEmbedder::new();
    let mut group = c.benchmark_group("embedding_text_size");

    for size in [10, 50, 100, 500, 1000].iter() {
        let text = "word ".repeat(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &_size| {
            b.iter(|| {
                let embedding = embedder.embed(black_box(&text));

                black_box(embedding.len())
            })
        });
    }

    group.finish();
}

/// Benchmark vector similarity search
fn bench_vector_search(c: &mut Criterion) {
    // Create mock embeddings (384 dimensions)
    let query_embedding: Vec<f32> = (0..384).map(|i| i as f32 / 384.0).collect();
    let doc_embeddings: Vec<Vec<f32>> = (0..100)
        .map(|_| (0..384).map(|i| (i as f32 + 10.0) / 384.0).collect())
        .collect();

    c.bench_function("vector_search", |b| {
        b.iter(|| {
            let mut best_score = 0.0;
            let mut best_idx = 0;

            for (idx, doc_emb) in black_box(&doc_embeddings).iter().enumerate() {
                // Calculate cosine similarity
                let dot_product: f32 = query_embedding.iter()
                    .zip(doc_emb.iter())
                    .map(|(a, b)| a * b)
                    .sum();

                let norm_a: f32 = query_embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
                let norm_b: f32 = doc_emb.iter().map(|x| x * x).sum::<f32>().sqrt();

                let similarity = dot_product / (norm_a * norm_b);

                if similarity > best_score {
                    best_score = similarity;
                    best_idx = idx;
                }
            }

            black_box((best_idx, best_score))
        })
    });
}

/// Benchmark vector search with different database sizes
fn bench_vector_search_scales(c: &mut Criterion) {
    let query_embedding: Vec<f32> = (0..384).map(|i| i as f32 / 384.0).collect();
    let mut group = c.benchmark_group("vector_search_scale");

    for size in [10, 50, 100, 500, 1000].iter() {
        let doc_embeddings: Vec<Vec<f32>> = (0..*size)
            .map(|_| (0..384).map(|i| (i as f32 + 10.0) / 384.0).collect())
            .collect();

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &_size| {
            b.iter(|| {
                let mut best_score = 0.0;

                for doc_emb in black_box(&doc_embeddings).iter() {
                    // Simplified similarity calculation
                    let similarity: f32 = query_embedding.iter()
                        .zip(doc_emb.iter())
                        .map(|(a, b)| a * b)
                        .sum();

                    if similarity > best_score {
                        best_score = similarity;
                    }
                }

                black_box(best_score)
            })
        });
    }

    group.finish();
}

/// Benchmark document indexing (chunking + embedding)
fn bench_document_indexing(c: &mut Criterion) {
    let document = "This is a sample document that needs to be chunked and embedded for vector search. ".repeat(20);
    let chunker = DocumentChunker::new(512);
    let embedder = PlaceholderEmbedder::new();

    c.bench_function("document_indexing", |b| {
        b.iter(|| {
            // Chunk the document
            let chunks = chunker.chunk(black_box(&document), ChunkStrategy::Token);

            // Embed each chunk
            let embeddings: Vec<_> = chunks.iter()
                .map(|chunk| embedder.embed(chunk))
                .collect();

            black_box(embeddings.len())
        })
    });
}

/// Benchmark top-k retrieval
fn bench_top_k_retrieval(c: &mut Criterion) {
    let query_embedding: Vec<f32> = (0..384).map(|i| i as f32 / 384.0).collect();
    let doc_embeddings: Vec<Vec<f32>> = (0..100)
        .map(|_| (0..384).map(|i| (i as f32 + 10.0) / 384.0).collect())
        .collect();

    let mut group = c.benchmark_group("top_k_retrieval");

    for k in [1, 3, 5, 10].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(k), k, |b, &k| {
            b.iter(|| {
                // Calculate all similarities
                let mut similarities: Vec<(usize, f32)> = doc_embeddings.iter()
                    .enumerate()
                    .map(|(idx, doc_emb)| {
                        let sim: f32 = query_embedding.iter()
                            .zip(doc_emb.iter())
                            .map(|(a, b)| a * b)
                            .sum();
                        (idx, sim)
                    })
                    .collect();

                // Sort by similarity (descending)
                similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

                // Get top-k
                let top_k: Vec<_> = similarities.iter().take(k).collect();

                black_box(top_k)
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_chunking_strategies,
    bench_character_chunking,
    bench_embedding_generation,
    bench_embedding_sizes,
    bench_vector_search,
    bench_vector_search_scales,
    bench_document_indexing,
    bench_top_k_retrieval
);
criterion_main!(benches);
