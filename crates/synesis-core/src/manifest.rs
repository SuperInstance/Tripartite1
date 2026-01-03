//! A2A Manifest - Agent-to-Agent Communication Protocol
//!
//! The manifest is the standard message format passed between agents
//! in the tripartite council. It accumulates context as it moves
//! from Pathos → Logos → Ethos.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Agent-to-Agent Manifest
///
/// This is the core data structure that flows through the tripartite council.
/// Each agent reads from and writes to the manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AManifest {
    /// Unique identifier for this conversation turn
    pub id: String,

    /// Session ID for multi-turn conversations
    pub session_id: String,

    /// The original user query
    pub query: String,

    /// Query after privacy redaction
    pub redacted_query: Option<String>,

    /// Conversation history (previous turns)
    #[serde(default)]
    pub history: Vec<ConversationTurn>,

    /// Intent framing from Pathos
    pub pathos_framing: Option<String>,

    /// Pathos confidence score
    pub pathos_confidence: Option<f32>,

    /// Response from Logos
    pub logos_response: Option<String>,

    /// Logos confidence score
    pub logos_confidence: Option<f32>,

    /// Verification notes from Ethos
    pub ethos_verification: Option<String>,

    /// Ethos confidence score
    pub ethos_confidence: Option<f32>,

    /// Current consensus round
    pub round: u8,

    /// Feedback from previous rounds
    #[serde(default)]
    pub feedback: Vec<String>,

    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,

    /// Timestamps
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,

    /// Processing flags
    #[serde(default)]
    pub flags: ManifestFlags,
}

impl A2AManifest {
    /// Create a new manifest for a query
    pub fn new(query: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            session_id: Uuid::new_v4().to_string(),
            query,
            redacted_query: None,
            history: vec![],
            pathos_framing: None,
            pathos_confidence: None,
            logos_response: None,
            logos_confidence: None,
            ethos_verification: None,
            ethos_confidence: None,
            round: 0,
            feedback: vec![],
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
            flags: ManifestFlags::default(),
        }
    }

    /// Create a manifest with session context
    pub fn with_session(query: String, session_id: String, history: Vec<ConversationTurn>) -> Self {
        let mut manifest = Self::new(query);
        manifest.session_id = session_id;
        manifest.history = history;
        manifest
    }

    /// Set the redacted query
    pub fn set_redacted_query(&mut self, redacted: String) {
        self.redacted_query = Some(redacted);
        self.updated_at = chrono::Utc::now();
    }

    /// Set Pathos results
    pub fn set_pathos_result(&mut self, framing: String, confidence: f32) {
        self.pathos_framing = Some(framing);
        self.pathos_confidence = Some(confidence);
        self.updated_at = chrono::Utc::now();
    }

    /// Set Logos results
    pub fn set_logos_result(&mut self, response: String, confidence: f32) {
        self.logos_response = Some(response);
        self.logos_confidence = Some(confidence);
        self.updated_at = chrono::Utc::now();
    }

    /// Set Ethos results
    pub fn set_ethos_result(&mut self, verification: String, confidence: f32) {
        self.ethos_verification = Some(verification);
        self.ethos_confidence = Some(confidence);
        self.updated_at = chrono::Utc::now();
    }

    /// Add feedback for next round
    pub fn add_feedback(&mut self, feedback: String) {
        self.feedback.push(feedback);
        self.updated_at = chrono::Utc::now();
    }

    /// Increment round counter
    pub fn next_round(&mut self) {
        self.round += 1;
        // Clear previous results for fresh evaluation
        self.logos_response = None;
        self.logos_confidence = None;
        self.ethos_verification = None;
        self.ethos_confidence = None;
        self.updated_at = chrono::Utc::now();
    }

    /// Get the query to process (redacted if available)
    pub fn effective_query(&self) -> &str {
        self.redacted_query.as_deref().unwrap_or(&self.query)
    }

    /// Check if this is a continuation of a conversation
    pub fn is_continuation(&self) -> bool {
        !self.history.is_empty()
    }

    /// Add metadata
    pub fn set_metadata(&mut self, key: &str, value: serde_json::Value) {
        self.metadata.insert(key.to_string(), value);
        self.updated_at = chrono::Utc::now();
    }

    /// Get metadata
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }
}

/// A turn in the conversation history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTurn {
    /// Role: "user" or "assistant"
    pub role: String,
    /// Content of the message
    pub content: String,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ConversationTurn {
    pub fn user(content: String) -> Self {
        Self {
            role: "user".to_string(),
            content,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn assistant(content: String) -> Self {
        Self {
            role: "assistant".to_string(),
            content,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Processing flags for the manifest
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ManifestFlags {
    /// Query requires cloud processing
    pub requires_cloud: bool,
    /// Query contains sensitive data (redacted)
    pub has_sensitive_data: bool,
    /// Query needs knowledge retrieval
    pub needs_knowledge: bool,
    /// Query is urgent
    pub urgent: bool,
    /// Query is a simple/fast query
    pub simple_query: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_creation() {
        let manifest = A2AManifest::new("Hello, world!".to_string());
        assert!(!manifest.id.is_empty());
        assert_eq!(manifest.query, "Hello, world!");
        assert_eq!(manifest.round, 0);
    }

    #[test]
    fn test_manifest_round_progression() {
        let mut manifest = A2AManifest::new("Test".to_string());

        manifest.set_pathos_result("Intent framing".to_string(), 0.9);
        manifest.set_logos_result("Response".to_string(), 0.85);
        manifest.set_ethos_result("Verified".to_string(), 0.88);

        assert!(manifest.pathos_framing.is_some());
        assert!(manifest.logos_response.is_some());

        manifest.next_round();

        assert_eq!(manifest.round, 1);
        assert!(manifest.pathos_framing.is_some()); // Pathos preserved
        assert!(manifest.logos_response.is_none()); // Logos cleared
        assert!(manifest.ethos_verification.is_none()); // Ethos cleared
    }

    #[test]
    fn test_effective_query() {
        let mut manifest = A2AManifest::new("Original query".to_string());
        assert_eq!(manifest.effective_query(), "Original query");

        manifest.set_redacted_query("Redacted query".to_string());
        assert_eq!(manifest.effective_query(), "Redacted query");
    }
}
