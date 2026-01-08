//! Protocol message types
//!
//! Defines all messages sent over the QUIC tunnel

use serde::{Deserialize, Serialize};

/// Tunnel message type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum TunnelMessage {
    /// Heartbeat from client to server
    Heartbeat(HeartbeatData),

    /// Heartbeat acknowledgment from server to client
    HeartbeatAck(HeartbeatAckData),

    /// Escalation request from client to server
    EscalationRequest(EscalationRequestData),

    /// Escalation response from server to client
    EscalationResponse(EscalationResponseData),

    /// Streaming chunk from server to client
    StreamChunk(StreamChunkData),

    /// Stream end marker
    StreamEnd(StreamEndData),

    /// Error message
    Error(ErrorData),

    /// Pre-warm signal from client to server
    PrewarmSignal(PrewarmSignalData),
}

/// Heartbeat data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatData {
    /// Unique device identifier
    pub device_id: String,
    /// Unix timestamp (milliseconds)
    pub timestamp: i64,
    /// Monotonically increasing sequence number
    pub sequence: u64,
    /// Device vitals as JSON (CPU, memory, GPU, disk usage)
    pub vitals: serde_json::Value,
}

/// Heartbeat acknowledgment data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatAckData {
    /// Server Unix timestamp (milliseconds)
    pub server_time: i64,
    /// Round-trip latency in milliseconds
    pub latency_ms: u32,
    /// Number of pending messages for this client
    pub pending_messages: u32,
    /// Server status: "healthy", "degraded", or "maintenance"
    pub server_status: String,
}

/// Escalation request data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationRequestData {
    /// Unique request identifier
    pub request_id: String,
    /// Session identifier for conversation continuity
    pub session_id: String,
    /// The query (already redacted by privacy proxy)
    pub query: String,
    /// Context from local processing
    pub context: EscalationContextData,
    /// Model selection: "auto", "claude_sonnet", "claude_opus", or "gpt4_turbo"
    pub model: String,
    /// Maximum tokens to generate
    pub max_tokens: u32,
    /// Whether to stream response
    pub stream: bool,
    /// Optional LoRA identifier to use
    pub lora_id: Option<String>,
}

/// Escalation context data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EscalationContextData {
    /// Intent framing from Pathos agent
    pub pathos_framing: Option<String>,
    /// Relevant chunks from local knowledge vault
    pub local_knowledge: Vec<KnowledgeChunkData>,
    /// Conversation history
    pub conversation_history: Vec<MessageData>,
    /// Constraints from Ethos agent
    pub constraints: Vec<String>,
    /// User preferences for response
    pub user_preferences: Option<UserPreferencesData>,
}

/// Knowledge chunk data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeChunkData {
    /// Source document path
    pub source: String,
    /// Chunk content
    pub content: String,
    /// Relevance score (0.0 - 1.0)
    pub relevance: f32,
}

/// Message data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageData {
    /// Role: "user" or "assistant"
    pub role: String,
    /// Message content
    pub content: String,
    /// Unix timestamp (milliseconds), if available
    pub timestamp: Option<i64>,
}

/// User preferences data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserPreferencesData {
    /// Preferred language code (e.g., "en", "es")
    pub preferred_language: Option<String>,
    /// Verbosity level: "concise", "normal", or "detailed"
    pub verbosity: Option<String>,
    /// Response tone: "professional", "casual", or "technical"
    pub tone: Option<String>,
}

/// Escalation response data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationResponseData {
    /// Request identifier (matches request)
    pub request_id: String,
    /// Generated content
    pub content: String,
    /// Model actually used
    pub model_used: String,
    /// Token usage breakdown
    pub tokens_used: TokenUsageData,
    /// Cost in cents (after markup)
    pub cost_cents: u32,
    /// Total latency in milliseconds
    pub latency_ms: u64,
    /// Sources cited (if any)
    pub sources: Vec<String>,
    /// Whether LoRA was applied
    pub lora_applied: bool,
}

/// Token usage data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsageData {
    /// Input/prompt tokens
    pub prompt: u32,
    /// Output/completion tokens
    pub completion: u32,
}

/// Stream chunk data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunkData {
    /// Request identifier
    pub request_id: String,
    /// Content chunk
    pub content: String,
    /// Sequence number for ordering
    pub sequence: u32,
    /// Whether this is the final chunk
    pub is_final: bool,
}

/// Stream end data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEndData {
    /// Request identifier
    pub request_id: String,
    /// Total tokens used
    pub tokens_used: TokenUsageData,
    /// Total cost in cents
    pub cost_cents: u32,
}

/// Error data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorData {
    /// Error code (e.g., "RATE_LIMITED", "INVALID_REQUEST")
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Additional error details, if any
    pub details: Option<String>,
}

/// Pre-warm signal data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrewarmSignalData {
    /// Device identifier
    pub device_id: String,
    /// Unix timestamp (milliseconds)
    pub timestamp: i64,
    /// Current GPU usage percentage (0.0 - 1.0)
    pub gpu_usage: f32,
    /// Current GPU temperature in Celsius, if available
    pub gpu_temp: Option<f32>,
    /// Reason for pre-warm (e.g., "GPU usage > 80%")
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let msg = TunnelMessage::Heartbeat(HeartbeatData {
            device_id: "test".to_string(),
            timestamp: 123456,
            sequence: 1,
            vitals: serde_json::json!({}),
        });

        let json = serde_json::to_string(&msg).unwrap();
        // Tagged format: {"type":"Heartbeat","data":{...}}
        assert!(json.contains(r#""type":"Heartbeat""#));

        let decoded: TunnelMessage = serde_json::from_str(&json).unwrap();
        assert!(matches!(decoded, TunnelMessage::Heartbeat(_)));
    }

    #[test]
    fn test_escalation_request_data() {
        let data = EscalationRequestData {
            request_id: "req-123".to_string(),
            session_id: "sess-456".to_string(),
            query: "test query".to_string(),
            context: EscalationContextData::default(),
            model: "claude_sonnet".to_string(),
            max_tokens: 1024,
            stream: false,
            lora_id: None,
        };

        assert_eq!(data.request_id, "req-123");
        assert_eq!(data.model, "claude_sonnet");
    }
}
