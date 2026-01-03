//! # Tripartite Agent System
//!
//! This module defines the agent trait and implementations for the three specialized agents
//! that form the SuperInstance council.
//!
//! ## The Three Agents
//!
//! ### Pathos - Intent Agent
//!
//! **Role**: Understand what the user truly wants
//!
//! Pathos extracts clear intent from ambiguous human prompts by:
//! - Identifying the user's true goal (telos)
//! - Detecting implicit constraints not explicitly stated
//! - Profiling user expertise level (Novice/Intermediate/Expert)
//! - Determining communication style preference (Formal/Casual/Technical)
//! - Setting verification scope for Ethos (facts, hardware, safety)
//!
//! **Model**: phi-3-mini (fast, efficient, good at classification)
//!
//! ### Logos - Logic Agent
//!
//! **Role**: Synthesize solutions using reasoning and knowledge
//!
//! Logos generates comprehensive solutions by:
//! - Retrieving relevant context from the knowledge vault via RAG
//! - Selecting appropriate LoRA adapters for domain expertise
//! - Synthesizing information from retrieved sources
//! - Generating well-reasoned, complete solutions
//! - Citing sources used in the response
//!
//! **Model**: llama-3.2-8b (balanced reasoning capabilities)
//!
//! ### Ethos - Truth Agent
//!
//! **Role**: Verify responses are safe, accurate, and feasible
//!
//! Ethos ensures quality and safety by:
//! - Checking for dangerous patterns (veto scenarios)
//! - Validating hardware constraints
//! - Fact-checking claims when possible
//! - Verifying code quality (error handling, best practices)
//! - Checking thermal limits for intensive operations
//! - Providing constructive feedback for revisions
//!
//! **Model**: phi-3-mini-4k (fast verification, good at following instructions)
//!
//! ## Agent Trait
//!
//! All agents implement the [`Agent`] trait:
//!
//! ```rust,ignore
//! use async_trait::async_trait;
//! use synesis_core::agents::{Agent, AgentInput, AgentOutput};
//! use synesis_core::CoreResult;
//!
//! #[async_trait]
//! pub trait Agent: Send + Sync {
//!     /// Agent's name for logging/display
//!     fn name(&self) -> &str;
//!
//!     /// Agent's role description
//!     fn role(&self) -> &str;
//!
//!     /// Process a query and return a response
//!     async fn process(&self, input: AgentInput) -> CoreResult<AgentOutput>;
//!
//!     /// Check if the agent is ready (model loaded, etc.)
//!     fn is_ready(&self) -> bool;
//!
//!     /// Get the model this agent uses
//!     fn model(&self) -> &str;
//! }
//! ```
//!
//! ## Communication Flow
//!
//! ```text
//! User Query
//!     │
//!     ▼
//! ┌─────────┐
//! │ Pathos  │ ← Extracts intent, profiles user, sets verification scope
//! └────┬────┘
//!      │
//!      ▼ (framing + metadata)
//! ┌─────────┐
//! │ Logos   │ ← Retrieves context, synthesizes solution
//! └────┬────┘
//!      │
//!      ▼ (solution + reasoning)
//! ┌─────────┐
//! │ Ethos   │ ← Verifies safety, accuracy, feasibility
//! └────┬────┘
//!      │
//!      ▼ (verdict + vote)
//!   Consensus
//! ```

pub mod ethos;
pub mod logos;
pub mod pathos;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::manifest::A2AManifest;
use crate::CoreResult;

// ============================================================================
// Core Agent Trait
// ============================================================================

/// Core agent trait that all council members must implement.
///
/// The agent trait defines the interface that Pathos, Logos, and Ethos
/// all implement to participate in the consensus process.
///
/// # Lifecycle
///
/// 1. Agent is created with [`AgentConfig`]
/// 2. [`Agent::initialize()`] is called to load models
/// 3. [`Agent::process()`] is called for each query
/// 4. [`Agent::is_ready()`] checks if the agent is available
///
/// # Example
///
/// ```rust,ignore
/// use synesis_core::agents::{Agent, AgentInput, AgentOutput};
/// use synesis_core::CoreResult;
///
/// struct MyAgent;
///
/// #[async_trait::async_trait]
/// impl Agent for MyAgent {
///     fn name(&self) -> &str {
///         "MyAgent"
///     }
///
///     fn role(&self) -> &str {
///         "Custom agent implementation"
///     }
///
///     async fn process(&self, input: AgentInput) -> CoreResult<AgentOutput> {
///         // Process the input...
///         Ok(AgentOutput::new("MyAgent", "Response".to_string(), 0.9))
///     }
///
///     fn is_ready(&self) -> bool {
///         true
///     }
///
///     fn model(&self) -> &str {
///         "my-model"
///     }
/// }
/// ```
#[async_trait]
pub trait Agent: Send + Sync {
    /// Returns the agent's name for logging and display purposes.
    ///
    /// This should be a short, memorable identifier like "Pathos", "Logos", or "Ethos".
    fn name(&self) -> &str;

    /// Returns a brief description of the agent's role.
    ///
    /// Example: "Intent extraction and emotional intelligence" for Pathos.
    fn role(&self) -> &str;

    /// Process an input manifest and generate a response.
    ///
    /// This is the core method where agents perform their specialized processing.
    /// Each agent receives the same input but produces different outputs based on
    /// their role:
    ///
    /// - **Pathos**: Extracts intent, frames the query
    /// - **Logos**: Generates solutions, retrieves knowledge
    /// - **Ethos**: Verifies safety, accuracy, feasibility
    ///
    /// # Arguments
    ///
    /// * `input` - The query manifest with context from previous agents
    ///
    /// # Returns
    ///
    /// An [`AgentOutput`] containing the response, confidence, and optional vote.
    async fn process(&self, input: AgentInput) -> CoreResult<AgentOutput>;

    /// Check if the agent is ready to process queries.
    ///
    /// Returns `true` if the agent has been initialized and has loaded its model.
    /// Returns `false` if the agent still needs initialization.
    fn is_ready(&self) -> bool;

    /// Returns the identifier of the model this agent uses.
    ///
    /// Examples: "phi-3-mini", "llama-3.2-8b", "mistral-7b-instruct"
    fn model(&self) -> &str;
}

// ============================================================================
// Agent Input/Output Types
// ============================================================================

/// Input passed to agents for processing.
///
/// Each agent receives the same [`A2AManifest`] but can include additional
/// context specific to their role.
///
/// # Fields
///
/// - `manifest` - The A2A manifest containing the query and all previous agent outputs
/// - `context` - Additional metadata specific to this agent invocation
///
/// # Example
///
/// ```rust,ignore
/// use synesis_core::agents::AgentInput;
/// use synesis_core::A2AManifest;
/// use serde_json::json;
///
/// let manifest = A2AManifest::new("Explain Rust ownership".to_string());
///
/// // Add context for Logos to prioritize certain files
/// let input = AgentInput::new(manifest)
///     .with_context("priority_files", json!(["src/memory.rs", "src/ownership.rs"]));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInput {
    /// The manifest being processed
    pub manifest: A2AManifest,

    /// Additional context specific to this agent
    #[serde(default)]
    pub context: HashMap<String, serde_json::Value>,
}

impl AgentInput {
    /// Creates a new agent input from a manifest.
    ///
    /// # Arguments
    ///
    /// * `manifest` - The query manifest to process
    pub fn new(manifest: A2AManifest) -> Self {
        Self {
            manifest,
            context: HashMap::new(),
        }
    }

    /// Adds a key-value pair to the context.
    ///
    /// # Arguments
    ///
    /// * `key` - The context key
    /// * `value` - The context value (must be JSON-serializable)
    pub fn with_context(mut self, key: &str, value: serde_json::Value) -> Self {
        self.context.insert(key.to_string(), value);
        self
    }
}

/// Output produced by an agent during processing.
///
/// Contains the agent's response along with metadata for consensus evaluation.
///
/// # Fields
///
/// - `agent` - Name of the agent that produced this output
/// - `content` - The primary response content
/// - `confidence` - Agent's confidence in this response (0.0-1.0)
/// - `reasoning` - Optional explanation of the reasoning process
/// - `tokens_used` - Number of tokens consumed (for billing/cost tracking)
/// - `latency_ms` - Processing time in milliseconds
/// - `metadata` - Additional agent-specific metadata
/// - `vote` - Optional consensus vote (for Ethos)
///
/// # Example
///
/// ```rust,ignore
/// use synesis_core::agents::AgentOutput;
///
/// let output = AgentOutput::new("Logos", "Here's how ownership works...".to_string(), 0.92)
///     .with_reasoning("Retrieved from memory module docs".to_string())
///     .with_tokens(450)
///     .with_latency(150);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutput {
    /// The agent's name
    pub agent: String,

    /// The generated content
    pub content: String,

    /// Confidence score (0.0-1.0)
    pub confidence: f32,

    /// Reasoning/explanation (optional)
    pub reasoning: Option<String>,

    /// Tokens used
    pub tokens_used: u32,

    /// Processing time in milliseconds
    pub latency_ms: u64,

    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,

    /// Consensus vote (for council voting)
    pub vote: Option<ConsensusVote>,
}

impl AgentOutput {
    /// Creates a new agent output with minimal fields.
    ///
    /// # Arguments
    ///
    /// * `agent` - The agent's name
    /// * `content` - The response content
    /// * `confidence` - Confidence score (0.0-1.0)
    pub fn new(agent: &str, content: String, confidence: f32) -> Self {
        Self {
            agent: agent.to_string(),
            content,
            confidence,
            reasoning: None,
            tokens_used: 0,
            latency_ms: 0,
            metadata: HashMap::new(),
            vote: None,
        }
    }

    /// Adds reasoning information to the output.
    ///
    /// # Arguments
    ///
    /// * `reasoning` - Explanation of the agent's reasoning process
    pub fn with_reasoning(mut self, reasoning: String) -> Self {
        self.reasoning = Some(reasoning);
        self
    }

    /// Sets the number of tokens consumed.
    ///
    /// # Arguments
    ///
    /// * `tokens` - Token count
    pub fn with_tokens(mut self, tokens: u32) -> Self {
        self.tokens_used = tokens;
        self
    }

    /// Sets the processing latency.
    ///
    /// # Arguments
    ///
    /// * `ms` - Latency in milliseconds
    pub fn with_latency(mut self, ms: u64) -> Self {
        self.latency_ms = ms;
        self
    }

    /// Adds a consensus vote to the output.
    ///
    /// # Arguments
    ///
    /// * `vote` - The vote to include
    pub fn with_vote(mut self, vote: ConsensusVote) -> Self {
        self.vote = Some(vote);
        self
    }
}

/// Legacy alias for backward compatibility
pub type AgentResponse = AgentOutput;

// ============================================================================
// Consensus Types
// ============================================================================

/// Vote from an agent in the consensus process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusVote {
    /// Agent name
    pub agent: String,

    /// Approval decision
    pub approve: bool,

    /// Confidence in this vote (0.0-1.0)
    pub confidence: f32,

    /// Reasoning for the vote
    pub reasoning: Option<String>,

    /// Any concerns or constraints identified
    #[serde(default)]
    pub concerns: Vec<Constraint>,
}

impl ConsensusVote {
    /// Create a new vote
    pub fn new(agent: &str, approve: bool, confidence: f32) -> Self {
        Self {
            agent: agent.to_string(),
            approve,
            confidence,
            reasoning: None,
            concerns: vec![],
        }
    }

    /// Add reasoning
    pub fn with_reasoning(mut self, reasoning: String) -> Self {
        self.reasoning = Some(reasoning);
        self
    }

    /// Add concerns
    pub fn with_concerns(mut self, concerns: Vec<Constraint>) -> Self {
        self.concerns = concerns;
        self
    }
}

// ============================================================================
// Pathos-Specific Types
// ============================================================================

/// User persona profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persona {
    /// Expertise level
    pub expertise_level: ExpertiseLevel,

    /// Communication style
    pub communication_style: CommunicationStyle,

    /// Known preferences from interaction history
    #[serde(default)]
    pub known_preferences: Vec<String>,
}

impl Default for Persona {
    fn default() -> Self {
        Self {
            expertise_level: ExpertiseLevel::Intermediate,
            communication_style: CommunicationStyle::Casual,
            known_preferences: vec![],
        }
    }
}

/// User expertise level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpertiseLevel {
    /// Beginner - needs simple explanations
    Novice,
    /// Some experience - balanced explanations
    Intermediate,
    /// Expert - wants technical depth
    Expert,
}

/// Communication style preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommunicationStyle {
    /// Formal, detailed explanations
    Formal,
    /// Conversational, friendly tone
    Casual,
    /// Technical jargon acceptable
    Technical,
}

/// Intent structure from Pathos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    /// The actual goal in clear language
    pub telos: String,

    /// Type of query
    pub query_type: QueryType,

    /// Explicit + inferred limits
    #[serde(default)]
    pub constraints: Vec<String>,

    /// Priority preference
    pub priority: Priority,
}

impl Default for Intent {
    fn default() -> Self {
        Self {
            telos: String::new(),
            query_type: QueryType::Explain,
            constraints: vec![],
            priority: Priority::Quality,
        }
    }
}

/// Type of query
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryType {
    /// Generate new content/code
    Generate,
    /// Analyze existing content
    Analyze,
    /// Transform content
    Transform,
    /// Verify claims
    Verify,
    /// Explain concepts
    Explain,
}

/// Priority preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    /// Speed over quality
    Speed,
    /// Quality over speed
    Quality,
    /// Minimize cost
    Cost,
}

/// Context hints from Pathos
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContextHints {
    /// Files Logos should check
    #[serde(default)]
    pub relevant_files: Vec<String>,

    /// Similar past queries
    #[serde(default)]
    pub related_queries: Vec<String>,

    /// Domain (e.g., "web development", "data science")
    #[serde(default)]
    pub domain: String,
}

/// Verification scope from Pathos
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VerificationScope {
    /// Check facts
    #[serde(default)]
    pub check_facts: bool,

    /// Check hardware constraints
    #[serde(default)]
    pub check_hardware: bool,

    /// Check safety concerns
    #[serde(default)]
    pub check_safety: bool,
}

// ============================================================================
// Logos-Specific Types
// ============================================================================

/// Source reference from Logos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    /// Vector ID or file path
    pub id: String,

    /// Type of source
    pub source_type: SourceType,

    /// Relevance score (0.0-1.0)
    pub relevance_score: f32,

    /// The actual text used (optional)
    pub snippet: Option<String>,
}

/// Type of source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceType {
    /// From vector database
    Vector,
    /// From file system
    File,
    /// From LoRA adapter
    Lora,
    /// From base knowledge
    BaseKnowledge,
}

// ============================================================================
// Ethos-Specific Types
// ============================================================================

/// Verification constraint from Ethos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    /// Type of constraint
    pub constraint_type: ConstraintType,

    /// Severity level
    pub severity: Severity,

    /// Description of the issue
    pub description: String,

    /// Evidence for the constraint
    pub source: Option<String>,

    /// How to fix it
    pub suggestion: Option<String>,
}

/// Type of constraint
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintType {
    /// Factual accuracy
    Fact,
    /// Hardware limitations
    Hardware,
    /// Safety concern
    Safety,
    /// Code quality
    Quality,
}

/// Severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    /// Just a warning
    Warning,
    /// Needs fixing
    Error,
    /// Critical - blocks execution
    Critical,
}

/// Ethos verdict
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verdict {
    /// Approved to ship
    Approved,
    /// Needs revisions
    NeedsRevision,
    /// Veto - blocked
    Veto,
}

// ============================================================================
// Domain Types
// ============================================================================

/// Domain classification
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Domain {
    /// Primary domain
    #[serde(default)]
    pub primary: String,

    /// Sub-domains
    #[serde(default)]
    pub subdomains: Vec<String>,
}

// ============================================================================
// Urgency Types
// ============================================================================

/// Urgency level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Urgency {
    /// Not urgent
    Low,
    /// Normal urgency
    Medium,
    /// High urgency
    High,
    /// Critical urgency
    Critical,
}

/// Configuration for an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Model identifier
    pub model: String,

    /// Whether this agent is enabled
    pub enabled: bool,

    /// Temperature for generation (0.0-1.0)
    pub temperature: f32,

    /// Maximum tokens to generate
    pub max_tokens: u32,

    /// System prompt for this agent
    pub system_prompt: Option<String>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            model: String::new(),
            enabled: true,
            temperature: 0.7,
            max_tokens: 2048,
            system_prompt: None,
        }
    }
}

// ============================================================================
// Re-exports
// ============================================================================

// Re-export agent implementations
pub use ethos::EthosAgent;
pub use logos::LogosAgent;
pub use pathos::PathosAgent;

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_output_builder() {
        let response = AgentOutput::new("test", "Hello".to_string(), 0.9)
            .with_reasoning("Because...".to_string())
            .with_tokens(100)
            .with_latency(50);

        assert_eq!(response.agent, "test");
        assert_eq!(response.confidence, 0.9);
        assert_eq!(response.tokens_used, 100);
        assert_eq!(response.latency_ms, 50);
    }

    #[test]
    fn test_consensus_vote() {
        let vote = ConsensusVote::new("pathos", true, 0.85)
            .with_reasoning("Clear intent".to_string())
            .with_concerns(vec![]);

        assert_eq!(vote.agent, "pathos");
        assert!(vote.approve);
        assert_eq!(vote.confidence, 0.85);
        assert!(vote.reasoning.is_some());
    }

    #[test]
    fn test_persona_default() {
        let persona = Persona::default();
        assert_eq!(persona.expertise_level, ExpertiseLevel::Intermediate);
        assert_eq!(persona.communication_style, CommunicationStyle::Casual);
        assert!(persona.known_preferences.is_empty());
    }

    #[test]
    fn test_constraint_types() {
        let constraint = Constraint {
            constraint_type: ConstraintType::Safety,
            severity: Severity::Critical,
            description: "Dangerous operation".to_string(),
            source: None,
            suggestion: Some("Use alternative approach".to_string()),
        };

        assert_eq!(constraint.severity, Severity::Critical);
        assert!(constraint.suggestion.is_some());
    }

    #[test]
    fn test_agent_input_builder() {
        let manifest = A2AManifest::new("Test query".to_string());
        let input = AgentInput::new(manifest).with_context("key", serde_json::json!("value"));

        assert!(input.context.contains_key("key"));
    }

    #[test]
    fn test_source_type_enum() {
        let source = Source {
            id: "test_id".to_string(),
            source_type: SourceType::Vector,
            relevance_score: 0.9,
            snippet: Some("test snippet".to_string()),
        };

        assert_eq!(source.source_type, SourceType::Vector);
        assert_eq!(source.relevance_score, 0.9);
        assert!(source.snippet.is_some());
    }
}
