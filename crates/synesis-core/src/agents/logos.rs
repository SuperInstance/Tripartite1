//! Logos Agent - Logical Reasoning & RAG
//!
//! Logos is the primary reasoning engine. It takes the intent framed by
//! Pathos, retrieves relevant knowledge via RAG, and synthesizes a response.
//!
//! Model: llama-3.2-8b (balanced reasoning capabilities)

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, instrument, warn};

use super::{Agent, AgentConfig, AgentInput, AgentOutput};
use crate::manifest::A2AManifest;
use crate::{SynesisError as CoreError, SynesisResult as CoreResult};

/// Logos agent for logical reasoning and solution synthesis
#[derive(Clone)]
pub struct LogosAgent {
    config: AgentConfig,
    ready: Arc<std::sync::atomic::AtomicBool>,
    // TODO: Add these when integrating with actual modules
    // model: LocalModel,
    // knowledge_vault: Option<KnowledgeVault>,
    // embedder: Option<LocalEmbedder>,
    // lora_loader: LoRALoader,
    // For now, we use placeholder RAG that can be upgraded later
    rag_enabled: bool,
}

impl LogosAgent {
    /// Create a new Logos agent
    pub fn new(config: AgentConfig) -> Self {
        Self {
            config,
            ready: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            rag_enabled: true, // RAG enabled by default
        }
    }

    /// Create a new Logos agent with RAG disabled
    pub fn without_rag(config: AgentConfig) -> Self {
        Self {
            config,
            ready: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            rag_enabled: false,
        }
    }

    /// Initialize the agent (load model)
    pub async fn initialize(&mut self) -> CoreResult<()> {
        info!("Initializing Logos agent with model: {}", self.config.model);

        // TODO: Load model via synesis_models
        // let model = synesis_models::load(&self.config.model).await?;
        // self.model = model;

        self.ready.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    /// Retrieve relevant context from knowledge vault
    #[instrument(skip(self, manifest))]
    async fn retrieve_context(&self, manifest: &A2AManifest) -> CoreResult<Vec<RetrievedChunk>> {
        if !self.rag_enabled {
            debug!("RAG disabled, skipping retrieval");
            return Ok(vec![]);
        }

        let query = manifest.effective_query();
        debug!("Retrieving context for query: {}", query);

        // 1. Extract key terms from A2AManifest metadata
        let keywords = self.extract_key_terms(manifest);
        debug!("Extracted {} keywords: {:?}", keywords.len(), keywords);

        // 2. Embed the query (placeholder for now)
        let query_embedding = self.embed_query(query).await?;
        debug!(
            "Generated query embedding with {} dimensions",
            query_embedding.len()
        );

        // 3. Search vault for top 5 relevant chunks
        let raw_results = self.search_vault(&query_embedding, &keywords, 5).await?;
        debug!("Retrieved {} raw chunks from vault", raw_results.len());

        // 4. Apply retrieval scoring with recency boost and source quality
        let scored_chunks = self.score_retrieval(raw_results, manifest)?;
        debug!("Scored and sorted {} chunks", scored_chunks.len());

        // 5. Take top results
        let top_chunks = scored_chunks.into_iter().take(5).collect();

        Ok(top_chunks)
    }

    /// Extract key terms from A2AManifest for retrieval
    fn extract_key_terms(&self, manifest: &A2AManifest) -> Vec<String> {
        let mut keywords = Vec::new();

        // Extract from metadata if present
        if let Some(v) = manifest.get_metadata("keywords") {
            if let Ok(kw) = serde_json::from_value::<Vec<String>>(v.clone()) {
                keywords.extend(kw);
            }
        }

        // Extract domain
        if let Some(v) = manifest.get_metadata("domain") {
            if let Some(domain) = v.as_str() {
                keywords.push(domain.to_string());
            }
        }

        // Extract from query (simple word extraction)
        let query_words: Vec<String> = manifest
            .effective_query()
            .split_whitespace()
            .filter(|w| w.len() > 3) // Only meaningful words
            .map(|w| w.to_lowercase())
            .collect();

        keywords.extend(query_words);

        // Deduplicate
        keywords.sort();
        keywords.dedup();

        keywords
    }

    /// Embed the query for vector search
    async fn embed_query(&self, query: &str) -> CoreResult<Vec<f32>> {
        // TODO: Use actual embedder from synesis_knowledge
        // let embedding = self.embedder.as_ref()
        //     .ok_or_else(|| CoreError::AgentError("Embedder not initialized".to_string()))?
        //     .embed(query)
        //     .await
        //     .map_err(|e| CoreError::AgentError(format!("Embedding failed: {}", e)))?;

        // Placeholder: generate deterministic fake embedding based on query
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(query.as_bytes());
        let hash = hasher.finalize();

        let embedding: Vec<f32> = (0..384) // BGE-micro dimension
            .map(|i| {
                let byte_idx = i % hash.len();
                let value = hash[byte_idx] as f32 / 255.0;
                (value * 2.0) - 1.0 // Normalize to [-1, 1]
            })
            .collect();

        Ok(embedding)
    }

    /// Search the knowledge vault with embedding and keywords
    async fn search_vault(
        &self,
        _query_embedding: &[f32],
        _keywords: &[String],
        _limit: usize,
    ) -> CoreResult<Vec<RawChunkResult>> {
        // TODO: Use actual vault search
        // let results = self.knowledge_vault.as_ref()
        //     .ok_or_else(|| CoreError::AgentError("Knowledge vault not initialized".to_string()))?
        //     .search(query_embedding, keywords, limit)
        //     .await
        //     .map_err(|e| CoreError::AgentError(format!("Vault search failed: {}", e)))?;

        // Placeholder: return empty results
        // This will be replaced with actual search when vault is integrated
        warn!("Vault search not yet implemented, returning empty results");
        Ok(vec![])
    }

    /// Score retrieval results with recency boost and source quality
    fn score_retrieval(
        &self,
        raw_results: Vec<RawChunkResult>,
        _manifest: &A2AManifest,
    ) -> CoreResult<Vec<RetrievedChunk>> {
        let mut scored_chunks = Vec::new();

        for raw in raw_results {
            // Calculate relevance score
            let relevance = self.calculate_relevance_score(&raw);

            scored_chunks.push(RetrievedChunk {
                source: raw.source,
                content: raw.content,
                relevance,
                chunk_id: raw.chunk_id,
                doc_type: raw.doc_type,
                days_since_update: raw.days_since_update,
            });
        }

        // Sort by relevance descending
        scored_chunks.sort_by(|a, b| {
            b.relevance
                .partial_cmp(&a.relevance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(scored_chunks)
    }

    /// Calculate relevance score with recency boost and source quality
    fn calculate_relevance_score(&self, raw: &RawChunkResult) -> f32 {
        let cosine_similarity = raw.similarity;

        // Calculate recency boost: More recent = higher boost (capped at 1.5x)
        // 0 days = 1.5x, 5+ days = 1.0x (no boost/penalty)
        let days_penalty = (raw.days_since_update as f32 * 0.1).min(0.5);
        let recency_boost = 1.5 - days_penalty;

        // Source quality multiplier
        let source_quality = match raw.doc_type.as_str() {
            "code" | "rust" | "python" | "javascript" => 1.0, // Code is highest quality
            "markdown" | "docs" => 0.9,                       // Docs are high quality
            "text" | "notes" => 0.8,                          // Notes are medium quality
            _ => 0.7,                                         // Other types
        };

        // Final relevance score
        cosine_similarity * recency_boost * source_quality
    }

    /// Select appropriate LoRA adapter for the domain
    #[instrument(skip(self))]
    async fn select_lora(&self, _domain: &str) -> CoreResult<Option<String>> {
        debug!("Selecting LoRA adapter");

        // TODO: Implement LoRA selection logic
        // Priority:
        // 1. Exact domain match
        // 2. Parent domain match
        // 3. No LoRA (use base model)
        //
        // let available_loras = self.lora_loader.list_available().await?;
        // if let Some(lora) = available_loras.iter().find(|l| l.domain == domain) {
        //     return Ok(Some(lora.clone()));
        // }

        Ok(None) // Use base model for now
    }

    /// Build synthesis prompt from manifest and context
    fn build_synthesis_prompt(&self, manifest: &A2AManifest, context: &[RetrievedChunk]) -> String {
        let mut prompt = String::new();

        // System prompt
        prompt.push_str("You are Logos, the Logic Agent in the SuperInstance system.\n\n");

        // Add Pathos framing if available
        if let Some(framing) = &manifest.pathos_framing {
            prompt.push_str("## Intent Understanding (from Pathos)\n");
            prompt.push_str(framing);
            prompt.push_str("\n\n");
        }

        // Add retrieved context if available with proper formatting
        if !context.is_empty() {
            prompt.push_str("## Relevant Context\n");
            prompt.push_str(&format!(
                "Found {} relevant chunks from the knowledge vault:\n\n",
                context.len()
            ));

            for chunk in context.iter() {
                // Format: [SOURCE: path/to/file.rs:42-58]
                prompt.push_str(&format!(
                    "[SOURCE: {} (relevance: {:.2}, type: {})]\n",
                    chunk.source, chunk.relevance, chunk.doc_type
                ));

                // Format content with language-specific syntax highlighting
                let lang = match chunk.doc_type.as_str() {
                    "rust" => "rust",
                    "python" => "python",
                    "javascript" | "typescript" => "javascript",
                    "markdown" => "markdown",
                    _ => "text",
                };

                prompt.push_str(&format!("```{}\n{}\n```\n\n", lang, chunk.content));
            }
        }

        // Add the actual query
        prompt.push_str("## Task\n");
        prompt.push_str(&manifest.query);
        prompt.push_str("\n\n");

        // Add conversation context if available
        if !manifest.history.is_empty() {
            prompt.push_str("## Conversation Context\n");
            for turn in &manifest.history {
                prompt.push_str(&format!("{}: {}\n", turn.role, turn.content));
            }
            prompt.push('\n');
        }

        // Add instructions
        prompt.push_str("## Instructions\n");
        prompt.push_str("1. Use the provided context when relevant\n");
        prompt.push_str("2. Provide a complete, well-reasoned solution\n");
        prompt.push_str("3. If generating code, ensure it's complete and runnable\n");
        prompt.push_str(
            "4. Cite sources using [SOURCE: path] notation when using retrieved information\n",
        );
        prompt.push_str("5. Show your reasoning process\n");
        if !context.is_empty() {
            prompt.push_str("6. Prioritize information from high-relevance sources (>0.7)\n");
        }
        prompt.push('\n');

        prompt.push_str("## Response\n");

        prompt
    }

    /// Generate solution using model
    #[instrument(skip(self, prompt))]
    async fn generate_solution(&self, prompt: &str) -> CoreResult<GeneratedSolution> {
        debug!("Generating solution");

        // TODO: Actually run through model
        // let (content, tokens) = self.model.generate_with_count(prompt).await?;
        //
        // For now, return a placeholder

        Ok(GeneratedSolution {
            content: format!(
                "[Solution generation not yet implemented]\n\nReceived prompt length: {} chars",
                prompt.len()
            ),
            reasoning: Some(
                "Placeholder reasoning - will be replaced with actual chain-of-thought".to_string(),
            ),
            tokens_used: 500, // Placeholder
        })
    }

    /// Calculate confidence score based on multiple factors including RAG quality
    fn calculate_confidence(
        &self,
        _solution: &str,
        context: &[RetrievedChunk],
        has_good_sources: bool,
    ) -> f32 {
        let mut confidence = 0.5; // Base confidence

        // Factor 1: RAG retrieval quality (0.0 - 0.25 boost)
        if !context.is_empty() {
            // Average relevance of retrieved chunks (safe: context not empty)
            let len = context.len() as f32;
            let avg_relevance: f32 =
                context.iter().map(|c| c.relevance).sum::<f32>() / len;

            // Number of high-relevance chunks (>0.7)
            let high_quality_count = context.iter().filter(|c| c.relevance > 0.7).count();
            let quality_bonus = (high_quality_count as f32 * 0.05).min(0.15);

            confidence += (avg_relevance * 0.1) + quality_bonus;
        }

        // Factor 2: Source relevance and diversity (0.0 - 0.15 boost)
        if has_good_sources {
            confidence += 0.15;
        } else if !context.is_empty() {
            // Partial boost for having any sources
            confidence += 0.05;
        }

        // Factor 3: Source type quality (code > docs > notes)
        if !context.is_empty() {
            let code_ratio = context
                .iter()
                .filter(|c| {
                    matches!(
                        c.doc_type.as_str(),
                        "rust" | "python" | "javascript" | "typescript"
                    )
                })
                .count() as f32
                / context.len() as f32;

            confidence += code_ratio * 0.1;
        }

        // Clamp to [0.0, 1.0]
        confidence.clamp(0.0, 1.0)
    }

    /// Extract clean solution (remove reasoning if present)
    fn clean_solution(&self, solution: &str) -> String {
        // TODO: Implement logic to separate reasoning from solution
        // For now, return as-is
        solution.to_string()
    }

    /// Extract reasoning path from solution (for debug mode)
    fn extract_reasoning(&self, _solution: &str) -> Option<String> {
        // TODO: Implement reasoning extraction
        // For now, return None
        None
    }
}

#[async_trait]
impl Agent for LogosAgent {
    fn name(&self) -> &str {
        "Logos"
    }

    fn role(&self) -> &str {
        "Logical reasoning and knowledge synthesis"
    }

    async fn process(&self, input: AgentInput) -> CoreResult<AgentOutput> {
        if !self.is_ready() {
            return Err(CoreError::AgentError("Logos not initialized".to_string()));
        }

        let start = std::time::Instant::now();
        let manifest = &input.manifest;

        // 1. RAG Retrieval - retrieve relevant context using full manifest
        let context = self.retrieve_context(manifest).await?;

        // 2. LoRA Selection - select specialized domain adapter (placeholder)
        let _domain = manifest
            .metadata
            .get("domain")
            .and_then(|v| v.as_str())
            .unwrap_or("general");
        let _lora_adapter = self.select_lora(_domain).await?;

        // 3. Build synthesis prompt with context
        let prompt = self.build_synthesis_prompt(manifest, &context);

        // 4. Generate solution
        let generated = self.generate_solution(&prompt).await?;

        // 5. Extract reasoning and clean solution
        let clean_solution = self.clean_solution(&generated.content);
        let reasoning_path = self.extract_reasoning(&generated.content);

        // 6. Build source list
        let sources: Vec<Source> = context
            .iter()
            .map(|chunk| Source {
                id: chunk.chunk_id.clone(),
                source_type: SourceType::Vector,
                relevance_score: chunk.relevance,
                snippet: Some(chunk.content.clone()),
            })
            .collect();

        // 7. Calculate confidence
        let has_good_sources = !sources.is_empty() && context.iter().any(|c| c.relevance > 0.7);
        let confidence = self.calculate_confidence(&clean_solution, &context, has_good_sources);

        // 8. Build response metadata
        let mut metadata = std::collections::HashMap::new();
        metadata.insert(
            "sources".to_string(),
            serde_json::to_value(&sources).unwrap_or_default(),
        );
        metadata.insert(
            "context_chunks".to_string(),
            serde_json::Value::Number(context.len().into()),
        );
        metadata.insert(
            "domain".to_string(),
            serde_json::Value::String(_domain.to_string()),
        );
        if let Some(lora) = &_lora_adapter {
            metadata.insert(
                "lora_adapter".to_string(),
                serde_json::Value::String(lora.clone()),
            );
        }

        Ok(AgentOutput {
            agent: self.name().to_string(),
            content: clean_solution,
            confidence,
            reasoning: reasoning_path.or(generated.reasoning),
            tokens_used: generated.tokens_used,
            latency_ms: start.elapsed().as_millis() as u64,
            metadata,
            vote: None, // Logos doesn't vote in consensus
        })
    }

    fn is_ready(&self) -> bool {
        self.ready.load(std::sync::atomic::Ordering::SeqCst)
    }

    fn model(&self) -> &str {
        &self.config.model
    }
}

/// A retrieved chunk from the knowledge vault
#[derive(Debug, Clone)]
pub struct RetrievedChunk {
    /// Source document/file
    pub source: String,
    /// The chunk content
    pub content: String,
    /// Relevance score (0.0-1.0)
    pub relevance: f32,
    /// Chunk ID
    pub chunk_id: String,
    /// Document type (code, markdown, text, etc.)
    pub doc_type: String,
    /// Days since last update (for recency calculation)
    pub days_since_update: u64,
}

/// Raw chunk result from vault search (before scoring)
#[derive(Debug, Clone)]
struct RawChunkResult {
    /// Source document/file
    pub source: String,
    /// The chunk content
    pub content: String,
    /// Chunk ID
    pub chunk_id: String,
    /// Document type
    pub doc_type: String,
    /// Days since last update
    pub days_since_update: u64,
    /// Raw similarity score (before recency/quality boost)
    pub similarity: f32,
}

/// Source reference in Logos response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    /// Source identifier (file path, vector ID, etc.)
    pub id: String,
    /// Type of source
    #[serde(rename = "type")]
    pub source_type: SourceType,
    /// Relevance score (0.0-1.0)
    pub relevance_score: f32,
    /// Content snippet used
    pub snippet: Option<String>,
}

/// Source type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SourceType {
    /// Vector database hit
    #[serde(rename = "vector")]
    Vector,
    /// File reference
    #[serde(rename = "file")]
    File,
    /// LoRA adapter used
    #[serde(rename = "lora")]
    LoRA,
    /// Base knowledge
    #[serde(rename = "base_knowledge")]
    BaseKnowledge,
}

/// Generated solution from model
#[derive(Debug)]
struct GeneratedSolution {
    content: String,
    reasoning: Option<String>,
    tokens_used: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logos_creation() {
        let config = AgentConfig {
            model: "llama-3.2-8b".to_string(),
            ..Default::default()
        };
        let agent = LogosAgent::new(config);
        assert_eq!(agent.name(), "Logos");
        assert!(!agent.is_ready());
    }

    #[test]
    fn test_confidence_calculation() {
        let config = AgentConfig::default();
        let agent = LogosAgent::new(config);

        // Test with no context
        let confidence = agent.calculate_confidence("test solution", &[], false);
        assert_eq!(confidence, 0.5);

        // Test with good context (code chunk)
        let context = vec![RetrievedChunk {
            source: "test.rs".to_string(),
            content: "test content".to_string(),
            relevance: 0.9,
            chunk_id: "1".to_string(),
            doc_type: "rust".to_string(),
            days_since_update: 0,
        }];
        let confidence = agent.calculate_confidence("test solution", &context, true);
        assert!(confidence > 0.5);
        assert!(confidence <= 1.0);

        // Test with high-quality and low-quality chunks
        let mixed_context = vec![
            RetrievedChunk {
                source: "code.py".to_string(),
                content: "def example(): pass".to_string(),
                relevance: 0.85,
                chunk_id: "1".to_string(),
                doc_type: "python".to_string(),
                days_since_update: 1,
            },
            RetrievedChunk {
                source: "notes.md".to_string(),
                content: "some notes".to_string(),
                relevance: 0.6,
                chunk_id: "2".to_string(),
                doc_type: "markdown".to_string(),
                days_since_update: 10,
            },
        ];
        let confidence = agent.calculate_confidence("test solution", &mixed_context, true);
        assert!(confidence > 0.5);
        assert!(confidence <= 1.0);
    }

    #[test]
    fn test_prompt_building() {
        let config = AgentConfig::default();
        let agent = LogosAgent::new(config);

        let manifest = A2AManifest::new("Write a hello world function".to_string());
        let prompt = agent.build_synthesis_prompt(&manifest, &[]);

        assert!(prompt.contains("Logos, the Logic Agent"));
        assert!(prompt.contains("Write a hello world function"));
        assert!(prompt.contains("Instructions"));
    }

    #[test]
    fn test_relevance_scoring() {
        let config = AgentConfig::default();
        let agent = LogosAgent::new(config);

        // Test with recent code chunk (highest score)
        let raw_code = RawChunkResult {
            source: "src/main.rs".to_string(),
            content: "fn main() {}".to_string(),
            chunk_id: "1".to_string(),
            doc_type: "rust".to_string(),
            days_since_update: 0,
            similarity: 0.85,
        };
        let score_code = agent.calculate_relevance_score(&raw_code);
        assert!(score_code > 0.8); // Should be high

        // Test with older markdown chunk (medium score)
        let raw_docs = RawChunkResult {
            source: "README.md".to_string(),
            content: "# Documentation".to_string(),
            chunk_id: "2".to_string(),
            doc_type: "markdown".to_string(),
            days_since_update: 10,
            similarity: 0.85,
        };
        let score_docs = agent.calculate_relevance_score(&raw_docs);
        assert!(score_docs < score_code); // Older docs should score lower

        // Test with old notes (lowest score)
        let raw_notes = RawChunkResult {
            source: "notes.txt".to_string(),
            content: "Some notes".to_string(),
            chunk_id: "3".to_string(),
            doc_type: "text".to_string(),
            days_since_update: 30,
            similarity: 0.85,
        };
        let score_notes = agent.calculate_relevance_score(&raw_notes);
        assert!(score_notes < score_docs); // Old notes should score lowest
    }

    #[test]
    fn test_key_terms_extraction() {
        let config = AgentConfig::default();
        let agent = LogosAgent::new(config);

        let mut manifest =
            A2AManifest::new("How do I implement a binary search tree in Rust?".to_string());

        // Add metadata
        manifest.set_metadata(
            "keywords",
            serde_json::json!(["algorithm", "data structure"]),
        );
        manifest.set_metadata("domain", serde_json::json!("computer science"));

        let keywords = agent.extract_key_terms(&manifest);

        assert!(keywords.contains(&"algorithm".to_string()));
        assert!(keywords.contains(&"data structure".to_string()));
        assert!(keywords.contains(&"computer science".to_string()));
        assert!(keywords.contains(&"binary".to_string()));
        assert!(keywords.contains(&"search".to_string()));
    }
}
