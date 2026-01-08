//! Cloud escalation types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Cloud model selection
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum CloudModel {
    /// Let cloud decide based on query
    #[default]
    Auto,

    /// Claude 3.5 Sonnet (default)
    ClaudeSonnet,

    /// Claude 3 Opus (highest quality)
    ClaudeOpus,

    /// GPT-4 Turbo
    Gpt4Turbo,
}

/// Request to escalate query to cloud
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationRequest {
    /// Unique request identifier
    pub request_id: String,

    /// Session identifier for conversation continuity
    pub session_id: String,

    /// The query (already redacted by privacy proxy)
    pub query: String,

    /// Context from local processing
    pub context: EscalationContext,

    /// Preferred cloud model
    pub model: CloudModel,

    /// Maximum tokens to generate
    pub max_tokens: u32,

    /// Whether to stream response
    pub stream: bool,

    /// LoRA to use (if uploaded)
    pub lora_id: Option<String>,

    /// Timeout in seconds
    pub timeout_secs: Option<u32>,
}

impl Default for EscalationRequest {
    fn default() -> Self {
        Self {
            request_id: String::new(),
            session_id: String::new(),
            query: String::new(),
            context: EscalationContext::default(),
            model: CloudModel::Auto,
            max_tokens: 1024,
            stream: false,
            lora_id: None,
            timeout_secs: Some(30),
        }
    }
}

/// Context passed with escalation request
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EscalationContext {
    /// Intent framing from Pathos agent
    pub pathos_framing: Option<String>,

    /// Relevant chunks from local knowledge vault
    pub local_knowledge: Vec<KnowledgeChunk>,

    /// Conversation history
    pub conversation_history: Vec<Message>,

    /// Constraints from Ethos agent
    pub constraints: Vec<String>,

    /// User preferences
    pub user_preferences: Option<UserPreferences>,
}

/// Knowledge chunk from local RAG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeChunk {
    /// Source document path
    pub source: String,

    /// Chunk content
    pub content: String,

    /// Relevance score (0.0 - 1.0)
    pub relevance: f32,
}

/// Conversation message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Role: "user" or "assistant"
    pub role: String,

    /// Message content
    pub content: String,

    /// Timestamp
    pub timestamp: Option<DateTime<Utc>>,
}

/// User preferences for response generation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserPreferences {
    pub preferred_language: Option<String>,
    pub verbosity: Option<Verbosity>,
    pub tone: Option<Tone>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Verbosity {
    Concise,
    Normal,
    Detailed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Tone {
    Professional,
    Casual,
    Technical,
}

/// Response from cloud escalation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationResponse {
    /// Request identifier (matches request)
    pub request_id: String,

    /// Generated content
    pub content: String,

    /// Model actually used
    pub model_used: String,

    /// Token usage
    pub tokens_used: TokenUsage,

    /// Cost in cents (after markup)
    pub cost_cents: u32,

    /// Total latency in milliseconds
    pub latency_ms: u64,

    /// Sources cited (if any)
    pub sources: Vec<String>,

    /// Whether LoRA was used
    pub lora_applied: bool,
}

/// Token usage breakdown
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    /// Input/prompt tokens
    pub prompt: u32,

    /// Output/completion tokens
    pub completion: u32,
}

impl TokenUsage {
    pub fn total(&self) -> u32 {
        self.prompt + self.completion
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escalation_request_defaults() {
        let request = EscalationRequest::default();
        assert_eq!(request.model, CloudModel::Auto);
        assert_eq!(request.max_tokens, 1024);
        assert!(!request.stream);
    }

    #[test]
    fn test_token_usage() {
        let usage = TokenUsage {
            prompt: 100,
            completion: 200,
        };
        assert_eq!(usage.total(), 300);
    }

    #[test]
    fn test_cloud_model_serde() {
        let model = CloudModel::ClaudeSonnet;
        let json = serde_json::to_string(&model).unwrap();
        assert_eq!(json, "\"claude_sonnet\"");

        let decoded: CloudModel = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, CloudModel::ClaudeSonnet);
    }
}
