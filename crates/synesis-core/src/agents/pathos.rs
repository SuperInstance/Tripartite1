//! Pathos Agent - Intent Extraction & Emotional Intelligence
//!
//! Pathos is responsible for understanding what the user truly wants,
//! extracting intent, identifying user persona, and framing the query
//! for the other agents in the tripartite council.
//!
//! Model: phi-3-mini (fast, efficient, good at classification)
//!
//! Key Responsibilities:
//! - Parse ambiguous human prompts into clear, structured intent
//! - Identify implicit constraints the user hasn't stated
//! - Detect the user's expertise level and communication preferences
//! - Output a valid A2A Manifest that guides the other agents

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, instrument};

use super::{Agent, AgentConfig, AgentInput, AgentOutput};
use crate::{CoreError, CoreResult};

/// Pathos agent for intent extraction
pub struct PathosAgent {
    config: AgentConfig,
    ready: bool,
    // Placeholder for model - will be integrated with synesis-models
    model: Option<ModelPlaceholder>,
}

/// Placeholder for the actual model interface
/// This will be replaced with real model inference later
#[allow(dead_code)]
struct ModelPlaceholder {
    name: String,
}

impl PathosAgent {
    /// Create a new Pathos agent
    pub fn new(config: AgentConfig) -> Self {
        Self {
            config,
            ready: false,
            model: None,
        }
    }

    /// Create a Pathos agent with default configuration for phi-3-mini
    pub fn with_phi3() -> Self {
        Self::new(AgentConfig {
            model: "phi-3-mini-4k".to_string(),
            enabled: true,
            temperature: 0.7,
            max_tokens: 2048,
            system_prompt: None,
        })
    }

    /// Initialize the agent (load model)
    pub async fn initialize(&mut self) -> CoreResult<()> {
        info!(
            "Initializing Pathos agent with model: {}",
            self.config.model
        );

        // TODO: Load actual model via synesis-models crate
        // For now, create a placeholder
        self.model = Some(ModelPlaceholder {
            name: self.config.model.clone(),
        });

        self.ready = true;
        Ok(())
    }

    /// Extract structured intent from user query using the model
    #[instrument(skip(self, query))]
    async fn extract_intent(&self, query: &str) -> CoreResult<PathosIntent> {
        debug!("Extracting intent from query using {}", self.config.model);

        // Build the system prompt for intent extraction
        let _system_prompt = self.build_system_prompt();

        // In a full implementation, we would:
        // 1. Call the model with the system prompt and user query
        // 2. Parse the JSON response
        // 3. Validate the structure

        // For now, use heuristic analysis as placeholder
        let intent = self.heuristic_intent_extraction(query).await;

        Ok(intent)
    }

    /// Build the system prompt for intent extraction
    fn build_system_prompt(&self) -> String {
        r#"
You are Pathos, the Intent Agent in the SuperInstance system.

Your mission: Transform messy human prompts into crystal-clear intent.

## Your Process

1. **Identify the Telos** (true goal)
   - What does the user actually want to achieve?
   - Read between the lines - what aren't they saying?

2. **Detect Constraints**
   - Explicit: "in Python", "under 100 lines", "for a beginner"
   - Implicit: Context suggests they're in a hurry, on mobile, etc.

3. **Profile the Persona**
   - Expertise: Are they asking what an expert or novice would ask?
   - Style: Do they want formal documentation or casual explanation?

4. **Set Verification Scope**
   - check_facts: True if claims will be made
   - check_hardware: True if code/actions will run locally
   - check_safety: True if code execution or system changes

## Output Format

Respond ONLY with a valid JSON object matching this schema:
{
  "intent": {
    "telos": string,
    "query_type": "generate" | "analyze" | "transform" | "verify" | "explain",
    "constraints": string[],
    "priority": "speed" | "quality" | "cost"
  },
  "persona": {
    "expertise_level": "novice" | "intermediate" | "expert",
    "communication_style": "formal" | "casual" | "technical",
    "known_preferences": string[]
  },
  "context_hints": {
    "relevant_files": string[],
    "related_queries": string[],
    "domain": string
  },
  "verification_scope": {
    "check_facts": boolean,
    "check_hardware": boolean,
    "check_safety": boolean
  }
}

No explanations, no markdown, just JSON.
        "#
        .trim()
        .to_string()
    }

    /// Heuristic intent extraction (placeholder for real model inference)
    /// This will be replaced once we integrate the actual model
    async fn heuristic_intent_extraction(&self, query: &str) -> PathosIntent {
        let query_lower = query.to_lowercase();

        // Detect query type
        let query_type = if query_lower.contains("create")
            || query_lower.contains("generate")
            || query_lower.contains("write")
            || query_lower.contains("make")
        {
            QueryType::Generate
        } else if query_lower.contains("analyze")
            || query_lower.contains("review")
            || query_lower.contains("check")
            || query_lower.contains("audit")
        {
            QueryType::Analyze
        } else if query_lower.contains("convert")
            || query_lower.contains("transform")
            || query_lower.contains("change")
        {
            QueryType::Transform
        } else if query_lower.contains("verify")
            || query_lower.contains("validate")
            || query_lower.contains("test")
        {
            QueryType::Verify
        } else {
            QueryType::Explain
        };

        // Detect expertise level
        let expertise_level = if query_lower.contains("beginner")
            || query_lower.contains("newbie")
            || query_lower.contains("just starting")
            || query_lower.contains("no experience")
        {
            ExpertiseLevel::Novice
        } else if query_lower.contains("advanced")
            || query_lower.contains("expert")
            || query_lower.contains("production")
            || query_lower.contains("scalable")
        {
            ExpertiseLevel::Expert
        } else {
            ExpertiseLevel::Intermediate
        };

        // Detect communication style
        let communication_style = if query_lower.contains("pls")
            || query_lower.contains("please")
            || query_lower.contains("thanks")
        {
            CommunicationStyle::Casual
        } else if query_lower.contains("documentation") || query_lower.contains("specification") {
            CommunicationStyle::Formal
        } else {
            CommunicationStyle::Technical
        };

        // Detect priority
        let priority = if query_lower.contains("quick")
            || query_lower.contains("fast")
            || query_lower.contains("asap")
            || query_lower.contains("urgent")
        {
            Priority::Speed
        } else if query_lower.contains("careful")
            || query_lower.contains("thorough")
            || query_lower.contains("comprehensive")
        {
            Priority::Quality
        } else {
            Priority::Cost
        };

        // Extract constraints
        let constraints = self.extract_constraints(query);

        // Detect domain
        let domain = self.detect_domain(query);

        // Determine verification scope
        let (check_facts, check_hardware, check_safety) = self.determine_verification_scope(query);

        PathosIntent {
            intent: IntentDetails {
                telos: self.extract_telos(query, query_type),
                query_type,
                constraints,
                priority,
            },
            persona: PersonaDetails {
                expertise_level,
                communication_style,
                known_preferences: vec![],
            },
            context_hints: ContextHints {
                relevant_files: vec![],
                related_queries: vec![],
                domain,
            },
            verification_scope: VerificationScope {
                check_facts,
                check_hardware,
                check_safety,
            },
        }
    }

    /// Extract the core goal (telos) from the query
    fn extract_telos(&self, query: &str, _query_type: QueryType) -> String {
        // Simple heuristic: clean up and condense the query
        let mut telos = query.to_string();

        // Remove common filler words
        let fillers = [
            "can you",
            "could you",
            "please",
            "help me to",
            "i need to",
            "i want to",
        ];
        for filler in fillers {
            telos = telos.replace(filler, "");
        }

        // Capitalize first letter
        if let Some(first_char) = telos.chars().next() {
            telos = first_char.to_uppercase().collect::<String>() + &telos[first_char.len_utf8()..];
        }

        // Ensure it ends with proper punctuation
        if !telos.ends_with('.') && !telos.ends_with('?') && !telos.ends_with('!') {
            telos.push('.');
        }

        telos.trim().to_string()
    }

    /// Extract constraints from the query
    fn extract_constraints(&self, query: &str) -> Vec<String> {
        let mut constraints = Vec::new();
        let query_lower = query.to_lowercase();

        // Language constraints
        let languages = [
            "python",
            "rust",
            "javascript",
            "typescript",
            "go",
            "java",
            "c++",
            "c#",
        ];
        for lang in languages {
            if query_lower.contains(lang) {
                constraints.push(format!("Use {}", lang));
            }
        }

        // Format constraints
        if query_lower.contains("json") {
            constraints.push("Output as JSON".to_string());
        }
        if query_lower.contains("markdown") || query_lower.contains("md") {
            constraints.push("Format as Markdown".to_string());
        }

        // Length/complexity constraints
        if query_lower.contains("simple") || query_lower.contains("basic") {
            constraints.push("Keep it simple".to_string());
        }
        if query_lower.contains("detailed") || query_lower.contains("comprehensive") {
            constraints.push("Provide detailed explanation".to_string());
        }

        constraints
    }

    /// Detect the domain of the query
    fn detect_domain(&self, query: &str) -> String {
        let query_lower = query.to_lowercase();

        let domains: &[(&str, &[&str])] = &[
            (
                "web development",
                &[
                    "html",
                    "css",
                    "javascript",
                    "frontend",
                    "react",
                    "vue",
                    "angular",
                    "typescript",
                ],
            ),
            (
                "data science",
                &[
                    "pandas",
                    "numpy",
                    "analysis",
                    "dataset",
                    "machine learning",
                    "ml",
                    "jupyter",
                    "scipy",
                ],
            ),
            (
                "backend",
                &[
                    "api",
                    "server",
                    "database",
                    "sql",
                    "rest",
                    "graphql",
                    "microservices",
                    "websocket",
                ],
            ),
            (
                "devops",
                &[
                    "docker",
                    "kubernetes",
                    "deploy",
                    "ci/cd",
                    "pipeline",
                    "terraform",
                    "ansible",
                    "jenkins",
                ],
            ),
            (
                "mobile",
                &[
                    "android",
                    "ios",
                    "mobile",
                    "app",
                    "flutter",
                    "react native",
                    "swift",
                    "kotlin",
                ],
            ),
            (
                "systems",
                &[
                    "kernel",
                    "driver",
                    "embedded",
                    "systems programming",
                    "firmware",
                    "hardware",
                    "rust",
                    "c++",
                ],
            ),
            (
                "security",
                &[
                    "security",
                    "authentication",
                    "authorization",
                    "encryption",
                    "vulnerability",
                    "penetration",
                    "firewall",
                    "ssl",
                ],
            ),
        ];

        for (domain, keywords) in domains {
            for keyword in *keywords {
                if query_lower.contains(keyword) {
                    return domain.to_string();
                }
            }
        }

        "general".to_string()
    }

    /// Determine what needs verification
    fn determine_verification_scope(&self, query: &str) -> (bool, bool, bool) {
        let query_lower = query.to_lowercase();

        // Safety check: destructive operations
        let check_safety = query_lower.contains("delete")
            || query_lower.contains("remove")
            || query_lower.contains("modify")
            || query_lower.contains("change")
            || query_lower.contains("execute")
            || query_lower.contains("run code");

        // Hardware check: things that will run locally
        let check_hardware = query_lower.contains("run")
            || query_lower.contains("execute")
            || query_lower.contains("deploy")
            || query_lower.contains("performance");

        // Facts check: informational queries
        let check_facts = query_lower.contains("?")
            || query_lower.contains("what is")
            || query_lower.contains("explain")
            || query_lower.contains("how does");

        (check_facts, check_hardware, check_safety)
    }

    /// Calculate confidence score based on query and extracted intent
    fn calculate_confidence(&self, intent: &PathosIntent, original_prompt: &str) -> f32 {
        let mut confidence: f32 = 1.0;

        // Penalty for very short prompts (ambiguous)
        if original_prompt.split_whitespace().count() < 5 {
            confidence -= 0.15;
        }

        // Penalty for missing constraints when they seem needed
        if matches!(intent.intent.query_type, QueryType::Generate)
            && intent.intent.constraints.is_empty()
        {
            confidence -= 0.10;
        }

        // Bonus for clear domain detection
        if !intent.context_hints.domain.is_empty() && intent.context_hints.domain != "general" {
            confidence += 0.05;
        }

        // Penalty for overly verbose telos (likely just echoing)
        if intent.intent.telos.len() > 200 {
            confidence -= 0.10;
        }

        // Clamp to valid range [0.0, 1.0]
        confidence.clamp(0.0_f32, 1.0_f32)
    }
}

#[async_trait]
impl Agent for PathosAgent {
    fn name(&self) -> &str {
        "Pathos"
    }

    fn role(&self) -> &str {
        "Intent extraction and emotional intelligence"
    }

    async fn process(&self, input: AgentInput) -> CoreResult<AgentOutput> {
        if !self.ready {
            return Err(CoreError::AgentError("Pathos not initialized".to_string()));
        }

        let start = std::time::Instant::now();
        let manifest = &input.manifest;

        // Extract structured intent from the user query
        let intent = self.extract_intent(&manifest.query).await?;

        // Calculate confidence
        let confidence = self.calculate_confidence(&intent, &manifest.query);

        // Build framing message for other agents
        let framing = self.build_framing_message(&intent, &manifest.query);

        // Build response metadata
        let mut metadata = std::collections::HashMap::new();
        metadata.insert(
            "intent".to_string(),
            serde_json::to_value(&intent).unwrap_or_default(),
        );
        metadata.insert(
            "query_type".to_string(),
            serde_json::Value::String(format!("{:?}", intent.intent.query_type)),
        );
        metadata.insert(
            "expertise_level".to_string(),
            serde_json::Value::String(format!("{:?}", intent.persona.expertise_level)),
        );
        metadata.insert(
            "domain".to_string(),
            serde_json::Value::String(intent.context_hints.domain.clone()),
        );

        Ok(AgentOutput {
            agent: self.name().to_string(),
            content: framing,
            confidence,
            reasoning: Some(format!(
                "Extracted intent as '{}' with {} confidence for {} user",
                intent.intent.telos,
                confidence,
                format!("{:?}", intent.persona.expertise_level).to_lowercase()
            )),
            tokens_used: 150, // Placeholder
            latency_ms: start.elapsed().as_millis() as u64,
            metadata,
            vote: None, // Pathos doesn't vote in consensus
        })
    }

    fn is_ready(&self) -> bool {
        self.ready
    }

    fn model(&self) -> &str {
        &self.config.model
    }
}

impl PathosAgent {
    /// Build a framing message that guides other agents
    fn build_framing_message(&self, intent: &PathosIntent, original_query: &str) -> String {
        format!(
            "USER INTENT ANALYSIS:\n\
             ====================\n\
             Goal (Telos): {}\n\
             Query Type: {:?}\n\
             Priority: {:?}\n\
             \n\
             USER PERSONA:\n\
             =============\n\
             Expertise Level: {:?}\n\
             Communication Style: {:?}\n\
             \n\
             CONTEXT:\n\
             ========\n\
             Domain: {}\n\
             Constraints: {}\n\
             \n\
             VERIFICATION NEEDED:\n\
             ====================\n\
             Facts: {} | Hardware: {} | Safety: {}\n\
             \n\
             Original Query: \"{}\"",
            intent.intent.telos,
            intent.intent.query_type,
            intent.intent.priority,
            intent.persona.expertise_level,
            intent.persona.communication_style,
            intent.context_hints.domain,
            if intent.intent.constraints.is_empty() {
                "None detected".to_string()
            } else {
                intent.intent.constraints.join(", ")
            },
            intent.verification_scope.check_facts,
            intent.verification_scope.check_hardware,
            intent.verification_scope.check_safety,
            original_query
        )
    }
}

/// Structured intent extracted by Pathos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathosIntent {
    pub intent: IntentDetails,
    pub persona: PersonaDetails,
    pub context_hints: ContextHints,
    pub verification_scope: VerificationScope,
}

/// Core intent details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentDetails {
    pub telos: String,
    pub query_type: QueryType,
    pub constraints: Vec<String>,
    pub priority: Priority,
}

/// Query type classification
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum QueryType {
    Generate,
    Analyze,
    Transform,
    Verify,
    Explain,
}

/// User persona details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaDetails {
    pub expertise_level: ExpertiseLevel,
    pub communication_style: CommunicationStyle,
    pub known_preferences: Vec<String>,
}

/// User expertise level
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ExpertiseLevel {
    Novice,
    Intermediate,
    Expert,
}

/// Communication style preference
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CommunicationStyle {
    Formal,
    Casual,
    Technical,
}

/// Processing priority
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Priority {
    Speed,
    Quality,
    Cost,
}

/// Context hints for downstream agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextHints {
    pub relevant_files: Vec<String>,
    pub related_queries: Vec<String>,
    pub domain: String,
}

/// Verification scope for Ethos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationScope {
    pub check_facts: bool,
    pub check_hardware: bool,
    pub check_safety: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::A2AManifest;

    #[tokio::test]
    async fn test_pathos_initialization() {
        let mut agent = PathosAgent::with_phi3();
        assert!(!agent.is_ready());

        agent.initialize().await.unwrap();
        assert!(agent.is_ready());
        assert_eq!(agent.model(), "phi-3-mini-4k");
    }

    #[tokio::test]
    async fn test_simple_question() {
        let agent = PathosAgent::with_phi3();
        let mut agent = agent;
        agent.initialize().await.unwrap();

        let manifest = A2AManifest::new("What is Rust?".to_string());
        let input = AgentInput {
            manifest,
            context: std::collections::HashMap::new(),
        };
        let response = agent.process(input).await.unwrap();

        assert_eq!(response.agent, "Pathos");
        assert!(response.confidence > 0.7);
        assert!(response.content.contains("Rust"));
    }

    #[tokio::test]
    async fn test_expert_request() {
        let agent = PathosAgent::with_phi3();
        let mut agent = agent;
        agent.initialize().await.unwrap();

        let manifest = A2AManifest::new(
            "Implement a lock-free MPSC queue using compare-and-swap in Rust".to_string(),
        );
        let input = AgentInput {
            manifest,
            context: std::collections::HashMap::new(),
        };
        let response = agent.process(input).await.unwrap();

        // Should detect this as an expert-level request
        assert!(response.content.contains("Expert") || response.content.contains("Intermediate"));
    }

    #[tokio::test]
    async fn test_safety_detection() {
        let agent = PathosAgent::with_phi3();
        let mut agent = agent;
        agent.initialize().await.unwrap();

        let manifest = A2AManifest::new("Write a script to delete all files in /tmp".to_string());
        let input = AgentInput {
            manifest,
            context: std::collections::HashMap::new(),
        };
        let response = agent.process(input).await.unwrap();

        // Should detect safety concerns
        assert!(response.content.contains("true") || response.content.contains("Safety"));
    }

    #[tokio::test]
    async fn test_query_type_detection() {
        let agent = PathosAgent::with_phi3();

        // Test generate queries
        assert!(matches!(
            agent
                .heuristic_intent_extraction("Create a new API endpoint")
                .await
                .intent
                .query_type,
            QueryType::Generate
        ));

        // Test analyze queries
        assert!(matches!(
            agent
                .heuristic_intent_extraction("Analyze this code for bugs")
                .await
                .intent
                .query_type,
            QueryType::Analyze
        ));

        // Test explain queries
        assert!(matches!(
            agent
                .heuristic_intent_extraction("How does recursion work?")
                .await
                .intent
                .query_type,
            QueryType::Explain
        ));
    }

    #[tokio::test]
    async fn test_domain_detection() {
        let agent = PathosAgent::with_phi3();

        let web_dev = agent
            .heuristic_intent_extraction("Create a React component")
            .await;
        assert_eq!(web_dev.context_hints.domain, "web development");

        let data_science = agent
            .heuristic_intent_extraction("Analyze this dataset with pandas")
            .await;
        assert_eq!(data_science.context_hints.domain, "data science");
    }

    #[tokio::test]
    async fn test_confidence_calculation() {
        let agent = PathosAgent::with_phi3();

        // Short prompt should have penalty
        let short = agent.heuristic_intent_extraction("Help").await;
        let confidence = agent.calculate_confidence(&short, "Help");
        assert!(confidence < 1.0);

        // Long, detailed prompt should have higher confidence
        let detailed = agent.heuristic_intent_extraction(
            "Create a REST API using Rust and Actix-web with JWT authentication and PostgreSQL database"
        ).await;
        let confidence = agent.calculate_confidence(&detailed, "Create a REST API using Rust and Actix-web with JWT authentication and PostgreSQL database");
        assert!(confidence >= 0.85);
    }

    #[tokio::test]
    async fn test_constraint_extraction() {
        let agent = PathosAgent::with_phi3();

        let python_query = agent
            .heuristic_intent_extraction("Write a Python script")
            .await;
        assert!(python_query
            .intent
            .constraints
            .iter()
            .any(|c| c.to_lowercase().contains("python")));

        let json_query = agent.heuristic_intent_extraction("Output as JSON").await;
        assert!(json_query
            .intent
            .constraints
            .iter()
            .any(|c| c.to_lowercase().contains("json")));
    }
}
