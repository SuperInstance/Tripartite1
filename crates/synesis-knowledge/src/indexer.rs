//! Document Indexer
//!
//! Handles ingestion of documents into the knowledge vault.

use sha2::{Digest, Sha256};
use std::path::Path;
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

use crate::chunker::{detect_document_type, ChunkOptions, Chunker};
use crate::embeddings::EmbeddingProvider;
use crate::vault::{Document, KnowledgeVault};
use crate::{KnowledgeError, KnowledgeResult};

/// Result of indexing a document
#[derive(Debug)]
pub struct IndexResult {
    /// Document ID
    pub document_id: String,
    /// Number of chunks created
    pub chunk_count: u32,
    /// Whether the document was updated (vs new)
    pub updated: bool,
    /// Indexing time in milliseconds
    pub indexing_time_ms: u64,
}

/// Document indexer
pub struct DocumentIndexer<'a, E: EmbeddingProvider> {
    vault: &'a KnowledgeVault,
    embedder: &'a E,
    chunker: Chunker,
    /// Skip if content hash matches existing document
    skip_duplicates: bool,
}

impl<'a, E: EmbeddingProvider> DocumentIndexer<'a, E> {
    /// Create a new indexer
    pub fn new(vault: &'a KnowledgeVault, embedder: &'a E) -> Self {
        Self {
            vault,
            embedder,
            chunker: Chunker::new(),
            skip_duplicates: true,
        }
    }

    /// Configure chunker options
    pub fn with_chunk_options(mut self, options: ChunkOptions) -> Self {
        self.chunker = Chunker::with_options(options);
        self
    }

    /// Configure duplicate handling
    pub fn skip_duplicates(mut self, skip: bool) -> Self {
        self.skip_duplicates = skip;
        self
    }

    /// Index a file from disk
    #[instrument(skip(self))]
    pub async fn index_file(&self, path: &Path) -> KnowledgeResult<IndexResult> {
        info!("Indexing file: {:?}", path);

        // Read file content
        let content = tokio::fs::read_to_string(path).await?;

        // Get filename for title
        let filename = path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        // Detect document type
        let doc_type = detect_document_type(&filename);

        self.index_content(&content, &filename, doc_type, Some(path))
            .await
    }

    /// Index content directly
    #[instrument(skip(self, content))]
    pub async fn index_content(
        &self,
        content: &str,
        title: &str,
        doc_type: &str,
        path: Option<&Path>,
    ) -> KnowledgeResult<IndexResult> {
        let start = std::time::Instant::now();

        // Calculate content hash
        let content_hash = calculate_hash(content);

        // Check for duplicates
        if self.skip_duplicates && self.vault.has_document_hash(&content_hash)? {
            info!("Document already indexed with same content hash");
            return Ok(IndexResult {
                document_id: String::new(),
                chunk_count: 0,
                updated: false,
                indexing_time_ms: start.elapsed().as_millis() as u64,
            });
        }

        // Generate document ID
        let doc_id = format!("doc_{}", Uuid::new_v4().simple());

        // Chunk the content
        let chunks = self.chunker.chunk(content)?;
        let chunk_count = chunks.len() as u32;

        debug!("Created {} chunks", chunk_count);

        // Create document record
        let now = chrono::Utc::now();
        let document = Document {
            id: doc_id.clone(),
            path: path.map(|p| p.to_string_lossy().to_string()),
            title: title.to_string(),
            doc_type: doc_type.to_string(),
            content_hash,
            chunk_count,
            size_bytes: content.len() as u64,
            indexed_at: now,
            updated_at: now,
            metadata: std::collections::HashMap::new(),
        };

        // Save document
        self.vault.insert_document(&document)?;

        // Process chunks
        for (i, chunk) in chunks.iter().enumerate() {
            let chunk_id = format!("chunk_{}_{}", doc_id, i);

            // Save chunk
            self.vault.insert_chunk(
                &chunk_id,
                &doc_id,
                i as u32,
                &chunk.content,
                chunk.start_offset,
                chunk.end_offset,
                chunk.token_count,
            )?;

            // Generate and save embedding
            let embedding = self.embedder.embed(&chunk.content).await?;
            self.vault.insert_embedding(&chunk_id, &embedding)?;
        }

        info!(
            "Indexed document {} with {} chunks in {}ms",
            doc_id,
            chunk_count,
            start.elapsed().as_millis()
        );

        Ok(IndexResult {
            document_id: doc_id,
            chunk_count,
            updated: false,
            indexing_time_ms: start.elapsed().as_millis() as u64,
        })
    }

    /// Index multiple files
    pub async fn index_files(&self, paths: &[&Path]) -> KnowledgeResult<Vec<IndexResult>> {
        let mut results = Vec::with_capacity(paths.len());

        for path in paths {
            match self.index_file(path).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    warn!("Failed to index {:?}: {}", path, e);
                    // Continue with other files
                },
            }
        }

        Ok(results)
    }

    /// Index a directory recursively
    #[instrument(skip(self))]
    pub async fn index_directory(
        &self,
        dir: &Path,
        extensions: Option<&[&str]>,
    ) -> KnowledgeResult<Vec<IndexResult>> {
        info!("Indexing directory: {:?}", dir);

        let mut results = Vec::new();
        let mut stack = vec![dir.to_path_buf()];

        while let Some(current) = stack.pop() {
            let mut entries = tokio::fs::read_dir(&current).await?;

            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();

                if path.is_dir() {
                    // Skip hidden directories
                    if !path
                        .file_name()
                        .map(|n| n.to_string_lossy().starts_with('.'))
                        .unwrap_or(false)
                    {
                        stack.push(path);
                    }
                } else if path.is_file() {
                    // Check extension filter
                    let should_index = if let Some(exts) = extensions {
                        path.extension()
                            .and_then(|e| e.to_str())
                            .map(|e| exts.contains(&e))
                            .unwrap_or(false)
                    } else {
                        true
                    };

                    if should_index {
                        match self.index_file(&path).await {
                            Ok(result) => results.push(result),
                            Err(e) => {
                                warn!("Failed to index {:?}: {}", path, e);
                            },
                        }
                    }
                }
            }
        }

        info!("Indexed {} files from directory", results.len());
        Ok(results)
    }

    /// Reindex a specific document
    pub async fn reindex(&self, document_id: &str) -> KnowledgeResult<IndexResult> {
        // Get existing document
        let doc = self
            .vault
            .get_document(document_id)?
            .ok_or_else(|| KnowledgeError::NotFound(document_id.to_string()))?;

        // If we have the original path, read and reindex
        if let Some(path) = &doc.path {
            let path = Path::new(path);
            if path.exists() {
                // Delete existing document
                self.vault.delete_document(document_id)?;
                // Reindex
                return self.index_file(path).await;
            }
        }

        Err(KnowledgeError::Internal(
            "Cannot reindex: original file not found".to_string(),
        ))
    }
}

/// Calculate SHA256 hash of content
fn calculate_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_hash() {
        let hash1 = calculate_hash("hello world");
        let hash2 = calculate_hash("hello world");
        let hash3 = calculate_hash("different content");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_eq!(hash1.len(), 64); // SHA256 = 64 hex chars
    }
}
