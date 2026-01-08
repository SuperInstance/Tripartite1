//! Escalation context builder
//!
//! Builder pattern for constructing EscalationContext

use crate::escalation::types::{
    EscalationContext, KnowledgeChunk, Message, UserPreferences,
    Verbosity, Tone,
};

/// Builder for EscalationContext
pub struct EscalationContextBuilder {
    context: EscalationContext,
}

impl Default for EscalationContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl EscalationContextBuilder {
    /// Create a new context builder
    pub fn new() -> Self {
        Self {
            context: EscalationContext::default(),
        }
    }

    /// Add Pathos intent framing
    pub fn pathos_framing(mut self, framing: impl Into<String>) -> Self {
        self.context.pathos_framing = Some(framing.into());
        self
    }

    /// Add local knowledge chunk
    pub fn add_knowledge(mut self, chunk: KnowledgeChunk) -> Self {
        self.context.local_knowledge.push(chunk);
        self
    }

    /// Add multiple knowledge chunks
    pub fn add_knowledge_chunks(mut self, chunks: Vec<KnowledgeChunk>) -> Self {
        self.context.local_knowledge.extend(chunks);
        self
    }

    /// Add conversation message
    pub fn add_message(mut self, role: impl Into<String>, content: impl Into<String>) -> Self {
        self.context.conversation_history.push(Message {
            role: role.into(),
            content: content.into(),
            timestamp: Some(chrono::Utc::now()),
        });
        self
    }

    /// Add user message
    pub fn user(self, content: impl Into<String>) -> Self {
        self.add_message("user", content)
    }

    /// Add assistant message
    pub fn assistant(self, content: impl Into<String>) -> Self {
        self.add_message("assistant", content)
    }

    /// Add constraint
    pub fn add_constraint(mut self, constraint: impl Into<String>) -> Self {
        self.context.constraints.push(constraint.into());
        self
    }

    /// Add multiple constraints
    pub fn add_constraints(mut self, constraints: Vec<String>) -> Self {
        self.context.constraints.extend(constraints);
        self
    }

    /// Set user preferences
    pub fn user_preferences(mut self, prefs: UserPreferences) -> Self {
        self.context.user_preferences = Some(prefs);
        self
    }

    /// Set preferred language
    pub fn language(mut self, lang: impl Into<String>) -> Self {
        let mut prefs = self.context.user_preferences.unwrap_or_default();
        prefs.preferred_language = Some(lang.into());
        self.context.user_preferences = Some(prefs);
        self
    }

    /// Set verbosity
    pub fn verbosity(mut self, verbosity: Verbosity) -> Self {
        let mut prefs = self.context.user_preferences.unwrap_or_default();
        prefs.verbosity = Some(verbosity);
        self.context.user_preferences = Some(prefs);
        self
    }

    /// Set tone
    pub fn tone(mut self, tone: Tone) -> Self {
        let mut prefs = self.context.user_preferences.unwrap_or_default();
        prefs.tone = Some(tone);
        self.context.user_preferences = Some(prefs);
        self
    }

    /// Build the context
    pub fn build(self) -> EscalationContext {
        self.context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_basic() {
        let context = EscalationContextBuilder::new()
            .pathos_framing("User wants code help")
            .add_message("user", "Help me write Rust")
            .add_constraint("Use safe Rust")
            .build();

        assert_eq!(context.pathos_framing, Some("User wants code help".to_string()));
        assert_eq!(context.conversation_history.len(), 1);
        assert_eq!(context.constraints.len(), 1);
    }

    #[test]
    fn test_builder_with_knowledge() {
        let chunk = KnowledgeChunk {
            source: "doc.txt".to_string(),
            content: "Rust is safe".to_string(),
            relevance: 0.95,
        };

        let context = EscalationContextBuilder::new()
            .add_knowledge(chunk)
            .build();

        assert_eq!(context.local_knowledge.len(), 1);
        assert_eq!(context.local_knowledge[0].source, "doc.txt");
    }

    #[test]
    fn test_builder_with_preferences() {
        let context = EscalationContextBuilder::new()
            .language("en")
            .verbosity(Verbosity::Detailed)
            .tone(Tone::Technical)
            .build();

        assert!(context.user_preferences.is_some());
        let prefs = context.user_preferences.unwrap();
        assert_eq!(prefs.preferred_language, Some("en".to_string()));
        assert_eq!(prefs.verbosity, Some(Verbosity::Detailed));
        assert_eq!(prefs.tone, Some(Tone::Technical));
    }

    #[test]
    fn test_builder_convenience_methods() {
        let context = EscalationContextBuilder::new()
            .user("What is Rust?")
            .assistant("Rust is a systems language")
            .user("Is it safe?")
            .build();

        assert_eq!(context.conversation_history.len(), 3);
        assert_eq!(context.conversation_history[0].role, "user");
        assert_eq!(context.conversation_history[1].role, "assistant");
        assert_eq!(context.conversation_history[2].role, "user");
    }

    #[test]
    fn test_builder_add_constraints_batch() {
        let constraints = vec![
            "Use async/await".to_string(),
            "Handle errors properly".to_string(),
        ];

        let context = EscalationContextBuilder::new()
            .add_constraints(constraints)
            .build();

        assert_eq!(context.constraints.len(), 2);
    }
}
