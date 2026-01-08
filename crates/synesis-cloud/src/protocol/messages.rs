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
    pub device_id: String,
    pub timestamp: i64,
    pub sequence: u64,
    pub vitals: serde_json::Value, // Device vitals as JSON
}

/// Heartbeat acknowledgment data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatAckData {
    pub server_time: i64,
    pub latency_ms: u32,
    pub pending_messages: u32,
    pub server_status: String, // "healthy" | "degraded" | "maintenance"
}

/// Escalation request data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationRequestData {
    pub request_id: String,
    pub session_id: String,
    pub query: String,
    pub context: EscalationContextData,
    pub model: String, // "auto" | "claude_sonnet" | "claude_opus" | "gpt4_turbo"
    pub max_tokens: u32,
    pub stream: bool,
    pub lora_id: Option<String>,
}

/// Escalation context data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EscalationContextData {
    pub pathos_framing: Option<String>,
    pub local_knowledge: Vec<KnowledgeChunkData>,
    pub conversation_history: Vec<MessageData>,
    pub constraints: Vec<String>,
    pub user_preferences: Option<UserPreferencesData>,
}

/// Knowledge chunk data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeChunkData {
    pub source: String,
    pub content: String,
    pub relevance: f32,
}

/// Message data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageData {
    pub role: String,
    pub content: String,
    pub timestamp: Option<i64>,
}

/// User preferences data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserPreferencesData {
    pub preferred_language: Option<String>,
    pub verbosity: Option<String>,
    pub tone: Option<String>,
}

/// Escalation response data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationResponseData {
    pub request_id: String,
    pub content: String,
    pub model_used: String,
    pub tokens_used: TokenUsageData,
    pub cost_cents: u32,
    pub latency_ms: u64,
    pub sources: Vec<String>,
    pub lora_applied: bool,
}

/// Token usage data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsageData {
    pub prompt: u32,
    pub completion: u32,
}

/// Stream chunk data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunkData {
    pub request_id: String,
    pub content: String,
    pub sequence: u32,
    pub is_final: bool,
}

/// Stream end data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEndData {
    pub request_id: String,
    pub tokens_used: TokenUsageData,
    pub cost_cents: u32,
}

/// Error data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorData {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
}

/// Pre-warm signal data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrewarmSignalData {
    pub device_id: String,
    pub timestamp: i64,
    pub gpu_usage: f32,
    pub gpu_temp: Option<f32>,
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
