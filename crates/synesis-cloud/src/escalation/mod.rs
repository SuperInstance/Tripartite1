//! Cloud escalation client
//!
//! This module provides the client for escalating queries to cloud LLMs.

pub mod r#types;
pub mod client;
pub mod context;

pub use r#types::{
    CloudModel, EscalationRequest, EscalationResponse, EscalationContext,
    KnowledgeChunk, Message, UserPreferences, TokenUsage,
};
pub use client::{EscalationClient, ClientStats};
pub use context::EscalationContextBuilder;
