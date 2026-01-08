//! # SuperInstance Core - Agent Orchestration Engine
//!
//! This crate implements the heart of the SuperInstance AI system: the **tripartite council**
//! that combines three specialized AI agents to process queries through consensus.
//!
//! ## Architecture Overview
//!
//! The SuperInstance system uses three specialized agents, each with a unique perspective:
//!
//! - **Pathos** (Intent Agent): Understands what the user truly wants, extracting intent,
//!   detecting user expertise level, and framing the query for other agents.
//! - **Logos** (Logic Agent): Performs logical reasoning, retrieves relevant knowledge via RAG,
//!   and synthesizes comprehensive solutions.
//! - **Ethos** (Truth Agent): Verifies responses for safety, accuracy, feasibility, and quality.
//!   Has veto power over dangerous outputs.
//!
//! ## Consensus Mechanism
//!
//! The three agents work together through a multi-round consensus process:
//!
//! 1. **Round 1**: Pathos extracts intent → Logos generates solution → Ethos verifies
//! 2. **Evaluation**: Weighted voting determines if consensus is reached (default threshold: 0.85)
//! 3. **Revision**: If consensus isn't reached, feedback is provided and the process repeats
//! 4. **Outcome**: Returns when consensus is reached, vetoed, or max rounds exceeded
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use synesis_core::{Council, CouncilConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create council with default configuration
//!     let mut council = Council::new(CouncilConfig::default());
//!
//!     // Initialize all agents (load models)
//!     council.initialize().await?;
//!
//!     // Process a query
//!     let manifest = synesis_core::A2AManifest::new(
//!         "Explain how vector databases work".to_string()
//!     );
//!     let response = council.process(manifest).await?;
//!
//!     println!("Response: {}", response.content);
//!     println!("Confidence: {:.2}", response.confidence);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Module Structure
//!
//! - [`agents`]: Agent trait and implementations (Pathos, Logos, Ethos)
//! - [`consensus`]: Consensus engine and voting mechanism
//! - [`council`]: High-level council orchestration
//! - [`manifest`]: A2A manifest for query tracking
//! - [`routing`]: Local vs. cloud decision routing
//!
//! ## Privacy Integration
//!
//! The consensus engine integrates with the privacy crate to automatically redact sensitive
//! information before processing and re-inflate it in responses. This ensures that cloud
//! processing never sees raw PII or credentials.

pub mod agents;
pub mod consensus;
pub mod council;
pub mod error;
pub mod manifest;
pub mod metrics;
pub mod routing;

// Re-exports for convenience
pub use agents::{Agent, AgentConfig, AgentResponse};
pub use consensus::{
    AgentWeights, ConsensusConfig, ConsensusEngine, ConsensusOutcome, ConsensusResult, Verdict,
    Votes,
};
pub use council::{Council, CouncilConfig, CouncilResponse};
pub use error::{Result as SynesisResult, SynesisError};
pub use manifest::A2AManifest;
pub use metrics::{Metrics, MetricsSnapshot, QueryTimer};

// Type aliases for backward compatibility during migration
// These allow existing code using CoreError/CoreResult to work with SynesisError
pub type CoreError = SynesisError;
pub type CoreResult<T> = SynesisResult<T>;
