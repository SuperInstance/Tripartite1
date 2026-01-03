# Logos Agent - Worker Onboarding

> **Agent Type**: Logic Synthesis
> **Domain**: Building technical solutions
> **Primary Question**: "How do we accomplish this?"

---

## Your Role

You are building the **Logos Agent**, the technical workhorse of the system. Your implementation must:

1. Retrieve relevant context from the vector database (RAG)
2. Load appropriate LoRA adapters for specialized domains
3. Synthesize solutions that satisfy the intent from Pathos
4. Provide sources for verification by Ethos

## Why Logos Matters

Logos is the **value creator**. This is where the actual work gets done:
- Code generation
- Analysis and insights
- Problem-solving
- Documentation

**The constraint**: Logos is expensive (largest model, most tokens). Pathos filters, Ethos verifies, but Logos does the heavy lifting.

---

## Technical Specification

### Input
- A2A Manifest from Pathos
- Access to vector database
- Access to LoRA store

### Output: LogosResult

```typescript
interface LogosResult {
  solution: string;              // The actual response/code/analysis
  confidence: number;            // 0-1 how confident in solution
  sources: Source[];             // What was used to generate this
  reasoning_path?: string;       // Chain-of-thought (debug mode)
  token_count: number;           // For billing
}

interface Source {
  id: string;                    // Vector ID or file path
  type: 'vector' | 'file' | 'lora' | 'base_knowledge';
  relevance_score: number;
  snippet?: string;              // The actual text used
}
```

---

## Implementation Guide

### Core Function

```rust
// cli/src/agents/logos.rs

pub struct LogosAgent {
    model: LocalModel,           // llama-3.2-8b or similar
    vector_store: VectorStore,
    lora_loader: LoRALoader,
}

impl LogosAgent {
    pub async fn process(&self, manifest: &A2AManifest) -> Result<LogosResult> {
        let mut sources = Vec::new();
        
        // 1. RAG Retrieval
        let context_docs = self.retrieve_context(manifest).await?;
        sources.extend(context_docs.iter().map(|d| d.to_source()));
        
        // 2. LoRA Loading (if specialized domain)
        if let Some(lora) = self.select_lora(&manifest.context_hints.domain).await? {
            self.lora_loader.load(&lora).await?;
            sources.push(Source {
                id: lora.id.clone(),
                type: "lora",
                relevance_score: 1.0,
                snippet: None,
            });
        }
        
        // 3. Build prompt with context
        let prompt = self.build_prompt(manifest, &context_docs);
        
        // 4. Generate solution
        let (solution, token_count) = self.model.generate_with_count(&prompt).await?;
        
        // 5. Extract reasoning if available
        let reasoning_path = self.extract_reasoning(&solution);
        let clean_solution = self.clean_solution(&solution);
        
        // 6. Calculate confidence
        let confidence = self.calculate_confidence(&clean_solution, &context_docs);
        
        Ok(LogosResult {
            solution: clean_solution,
            confidence,
            sources,
            reasoning_path,
            token_count,
        })
    }
    
    async fn retrieve_context(&self, manifest: &A2AManifest) -> Result<Vec<Document>> {
        // Query vector store with the telos
        let mut docs = self.vector_store
            .query(&manifest.intent.telos, 5)
            .await?;
        
        // Also check specific files if hinted
        for file_path in &manifest.context_hints.relevant_files {
            if let Ok(content) = std::fs::read_to_string(file_path) {
                docs.push(Document {
                    id: file_path.clone(),
                    content,
                    source: file_path.clone(),
                    relevance: 1.0, // Explicitly referenced
                });
            }
        }
        
        // Deduplicate and rank
        self.rank_documents(docs)
    }
    
    fn build_prompt(&self, manifest: &A2AManifest, context: &[Document]) -> String {
        let context_str = context.iter()
            .map(|d| format!("--- {} ---\n{}", d.source, d.content))
            .collect::<Vec<_>>()
            .join("\n\n");
        
        format!(r#"
You are Logos, the Logic Agent in the SuperInstance system.

## Your Task
{telos}

## Constraints
{constraints}

## User Profile
- Expertise: {expertise}
- Style: {style}

## Available Context
{context}

## Instructions
1. Use the provided context when relevant
2. Match the user's expertise level in your explanation
3. Be {style} in tone
4. If generating code, ensure it's complete and runnable
5. Cite sources when making factual claims

Provide your solution:
"#,
            telos = manifest.intent.telos,
            constraints = manifest.intent.constraints.join(", "),
            expertise = manifest.persona.expertise_level,
            style = manifest.persona.communication_style,
            context = context_str,
        )
    }
}
```

### LoRA Selection

```rust
impl LogosAgent {
    async fn select_lora(&self, domain: &str) -> Result<Option<LoRAAdapter>> {
        // Check for domain-specific LoRA
        let available_loras = self.lora_loader.list_available().await?;
        
        // Priority:
        // 1. Exact domain match
        // 2. Parent domain match
        // 3. No LoRA (use base model)
        
        if let Some(lora) = available_loras.iter().find(|l| l.domain == domain) {
            return Ok(Some(lora.clone()));
        }
        
        let parent_domain = self.get_parent_domain(domain);
        if let Some(lora) = available_loras.iter().find(|l| l.domain == parent_domain) {
            return Ok(Some(lora.clone()));
        }
        
        Ok(None)
    }
    
    fn get_parent_domain(&self, domain: &str) -> &str {
        match domain {
            "react" | "vue" | "angular" => "web_development",
            "pytorch" | "tensorflow" => "machine_learning",
            "postgresql" | "mongodb" => "databases",
            _ => "general",
        }
    }
}
```

---

## RAG Strategy

### Chunking Strategy

```rust
impl VectorStore {
    fn chunk_document(&self, content: &str, source: &str) -> Vec<Chunk> {
        let mut chunks = Vec::new();
        
        // For code files: chunk by function/class
        if source.ends_with(".rs") || source.ends_with(".py") || source.ends_with(".ts") {
            chunks.extend(self.chunk_by_code_structure(content));
        }
        // For markdown: chunk by heading
        else if source.ends_with(".md") {
            chunks.extend(self.chunk_by_headings(content));
        }
        // Default: sliding window
        else {
            chunks.extend(self.chunk_sliding_window(content, 500, 50));
        }
        
        chunks
    }
    
    fn chunk_sliding_window(&self, content: &str, size: usize, overlap: usize) -> Vec<Chunk> {
        let words: Vec<&str> = content.split_whitespace().collect();
        let mut chunks = Vec::new();
        
        let mut start = 0;
        while start < words.len() {
            let end = (start + size).min(words.len());
            chunks.push(Chunk {
                text: words[start..end].join(" "),
                start_offset: start,
                end_offset: end,
            });
            start += size - overlap;
        }
        
        chunks
    }
}
```

### Retrieval Ranking

```rust
fn rank_documents(&self, mut docs: Vec<Document>) -> Vec<Document> {
    // Score = relevance * recency_boost * source_quality
    for doc in &mut docs {
        let recency_boost = self.calculate_recency_boost(&doc.modified_at);
        let source_quality = self.get_source_quality(&doc.source);
        
        doc.final_score = doc.relevance * recency_boost * source_quality;
    }
    
    docs.sort_by(|a, b| b.final_score.partial_cmp(&a.final_score).unwrap());
    docs.truncate(5); // Top 5
    docs
}

fn get_source_quality(&self, source: &str) -> f32 {
    // Prefer project files over external docs
    if source.starts_with("./") || source.starts_with("/home/") {
        1.2 // Boost project files
    } else if source.contains("official") || source.contains("docs.") {
        1.1 // Boost official docs
    } else {
        1.0
    }
}
```

---

## Testing Requirements

### Unit Tests

```rust
#[test]
fn test_code_generation() {
    let logos = LogosAgent::new_test();
    let manifest = A2AManifest {
        intent: Intent {
            telos: "Create a Rust function to calculate fibonacci numbers".to_string(),
            query_type: "generate".to_string(),
            constraints: vec!["use recursion".to_string()],
            ..Default::default()
        },
        ..Default::default()
    };
    
    let result = logos.process(&manifest).await.unwrap();
    
    assert!(result.solution.contains("fn fibonacci"));
    assert!(result.solution.contains("fibonacci(n - 1)")); // Recursive
    assert!(result.confidence > 0.7);
}

#[test]
fn test_context_retrieval() {
    let logos = LogosAgent::new_test();
    
    // Add some documents to vector store
    logos.vector_store.add(Document {
        id: "1".to_string(),
        content: "The authentication module uses JWT tokens".to_string(),
        source: "./src/auth.rs".to_string(),
    });
    
    let manifest = A2AManifest {
        intent: Intent {
            telos: "How does authentication work in this project?".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };
    
    let result = logos.process(&manifest).await.unwrap();
    
    assert!(result.sources.iter().any(|s| s.id == "1"));
    assert!(result.solution.to_lowercase().contains("jwt"));
}
```

---

## Performance Targets

| Metric | Target |
|--------|--------|
| RAG retrieval | <200ms |
| LoRA loading (cached) | <100ms |
| LoRA loading (cold) | <500ms |
| Full generation | <3s |
| Context window usage | <80% of limit |

---

## Common Pitfalls

### Pitfall 1: Context Overflow
**Problem**: Stuffing too much context, exceeding token limit
**Solution**: Aggressive ranking + truncation, keep context <50% of window

### Pitfall 2: Hallucinated Sources
**Problem**: Claiming to use sources that weren't in context
**Solution**: Post-process to verify all source references exist

### Pitfall 3: LoRA Confusion
**Problem**: Loading wrong LoRA for domain
**Solution**: Strict domain matching + fallback to base model

### Pitfall 4: Incomplete Code
**Problem**: Generating partial code snippets
**Solution**: Prompt engineering to request "complete, runnable" code

---

## Handoff to Ethos

After Logos completes, the solution passes to Ethos for verification. Ensure:
- `sources` array is complete and accurate
- `confidence` reflects actual certainty
- `solution` is self-contained (Ethos shouldn't need Logos context)

---

*See also: [PATHOS_AGENT.md](./PATHOS_AGENT.md), [ETHOS_AGENT.md](./ETHOS_AGENT.md)*
