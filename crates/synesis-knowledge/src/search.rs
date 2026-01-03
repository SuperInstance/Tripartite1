//! Vector Search
//!
//! Provides similarity search over document embeddings.

use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

use crate::embeddings::{cosine_similarity, EmbeddingProvider};
use crate::vault::KnowledgeVault;
use crate::{KnowledgeError, KnowledgeResult};

/// Search options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    /// Maximum number of results
    pub limit: usize,
    /// Minimum similarity threshold (0.0-1.0)
    pub threshold: f32,
    /// Filter by document types
    pub doc_types: Option<Vec<String>>,
    /// Filter by document IDs
    pub doc_ids: Option<Vec<String>>,
    /// Include chunk content in results
    pub include_content: bool,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            limit: 10,
            threshold: 0.5,
            doc_types: None,
            doc_ids: None,
            include_content: true,
        }
    }
}

/// A search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Chunk ID
    pub chunk_id: String,
    /// Document ID
    pub document_id: String,
    /// Document title
    pub document_title: String,
    /// Similarity score (0.0-1.0)
    pub score: f32,
    /// Chunk content (if requested)
    pub content: Option<String>,
    /// Chunk index within document
    pub chunk_index: u32,
    /// Start offset in document
    pub start_offset: u64,
    /// End offset in document
    pub end_offset: u64,
}

/// Vector search engine
pub struct VectorSearch<'a> {
    vault: &'a KnowledgeVault,
}

impl<'a> VectorSearch<'a> {
    /// Create a new vector search instance
    pub fn new(vault: &'a KnowledgeVault) -> Self {
        Self { vault }
    }

    /// Search for similar chunks
    #[instrument(skip(self, query_embedding))]
    pub async fn search(
        &self,
        query_embedding: &[f32],
        options: &SearchOptions,
    ) -> KnowledgeResult<Vec<SearchResult>> {
        debug!(
            "Searching with {} dimensions, limit={}",
            query_embedding.len(),
            options.limit
        );

        // Get all documents (with optional filtering)
        let documents = self.vault.list_documents(1000)?;

        let mut results = Vec::new();

        for doc in documents {
            // Apply document type filter
            if let Some(ref types) = options.doc_types {
                if !types.contains(&doc.doc_type) {
                    continue;
                }
            }

            // Apply document ID filter
            if let Some(ref ids) = options.doc_ids {
                if !ids.contains(&doc.id) {
                    continue;
                }
            }

            // Get chunks for this document
            let chunks = self.vault.get_chunks(&doc.id)?;

            for chunk in chunks {
                // Get embedding for this chunk
                if let Some(embedding) = self.vault.get_embedding(&chunk.id)? {
                    let score = cosine_similarity(query_embedding, &embedding);

                    if score >= options.threshold {
                        results.push(SearchResult {
                            chunk_id: chunk.id.clone(),
                            document_id: doc.id.clone(),
                            document_title: doc.title.clone(),
                            score,
                            content: if options.include_content {
                                Some(chunk.content.clone())
                            } else {
                                None
                            },
                            chunk_index: chunk.chunk_index,
                            start_offset: chunk.start_offset,
                            end_offset: chunk.end_offset,
                        });
                    }
                }
            }
        }

        // Sort by score descending
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit results
        results.truncate(options.limit);

        debug!("Found {} results above threshold", results.len());
        Ok(results)
    }

    /// Search with text query (generates embedding first)
    pub async fn search_text<E: EmbeddingProvider>(
        &self,
        query: &str,
        embedder: &E,
        options: &SearchOptions,
    ) -> KnowledgeResult<Vec<SearchResult>> {
        let query_embedding = embedder.embed(query).await?;
        self.search(&query_embedding, options).await
    }

    /// Get similar chunks to a specific chunk
    pub async fn find_similar(
        &self,
        chunk_id: &str,
        options: &SearchOptions,
    ) -> KnowledgeResult<Vec<SearchResult>> {
        // Get the embedding for the reference chunk
        let embedding = self
            .vault
            .get_embedding(chunk_id)?
            .ok_or_else(|| KnowledgeError::NotFound(format!("Chunk: {}", chunk_id)))?;

        // Exclude the reference chunk from results
        let modified_options = options.clone();

        let results = self.search(&embedding, &modified_options).await?;

        // Filter out the reference chunk
        Ok(results
            .into_iter()
            .filter(|r| r.chunk_id != chunk_id)
            .collect())
    }
}

/// Hybrid search combining vector and keyword search
pub struct HybridSearch<'a> {
    vector_search: VectorSearch<'a>,
    /// Weight for vector search (0.0-1.0)
    #[allow(dead_code)]
    vector_weight: f32,
    /// Weight for keyword search (0.0-1.0)
    #[allow(dead_code)]
    keyword_weight: f32,
}

impl<'a> HybridSearch<'a> {
    pub fn new(vault: &'a KnowledgeVault, vector_weight: f32, keyword_weight: f32) -> Self {
        Self {
            vector_search: VectorSearch::new(vault),
            vector_weight,
            keyword_weight,
        }
    }

    /// Hybrid search combining vector similarity and keyword matching
    pub async fn search<E: EmbeddingProvider>(
        &self,
        query: &str,
        embedder: &E,
        options: &SearchOptions,
    ) -> KnowledgeResult<Vec<SearchResult>> {
        // Get vector search results
        let vector_results = self
            .vector_search
            .search_text(query, embedder, options)
            .await?;

        // TODO: Add keyword search and combine scores
        // For now, just return vector results

        Ok(vector_results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_options_default() {
        let options = SearchOptions::default();
        assert_eq!(options.limit, 10);
        assert_eq!(options.threshold, 0.5);
        assert!(options.include_content);
    }
}
