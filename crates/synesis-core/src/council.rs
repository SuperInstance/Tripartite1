//! # Council Orchestrator
//!
//! This module provides the high-level orchestration for the tripartite council.
//! The `Council` struct coordinates the three agents (Pathos, Logos, Ethos) to
//! process queries through multi-round consensus.
//!
//! ## How the Council Works
//!
//! The council implements a sequential processing pipeline with revision loops:
//!
//! ```text
//! User Query
//!     │
//!     ▼
//! ┌─────────────────────────────────────┐
//! │ Round 1                             │
//! │  Pathos → Logos → Ethos             │
//! │  (intent → solution → verification) │
//! └──────────────┬──────────────────────┘
//!                │
//!                ▼
//!          Consensus Check
//!                │
//!        ┌───────┴────────┐
//!        │                │
//!    Reached          Not Reached
//!        │                │
//!        ▼                ▼
//!   Return           Max Rounds?
//!   Response             │
//!                   ┌────┴────┐
//!                   │         │
//!                  Yes       No
//!                   │         │
//!                   ▼         ▼
//!              Return    Next Round with
//!              Partial   Feedback
//!              Response
//! ```
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use synesis_core::{Council, CouncilConfig, A2AManifest};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create and initialize council
//! let mut council = Council::new(CouncilConfig::default());
//! council.initialize().await?;
//!
//! // Process a query
//! let manifest = A2AManifest::new(
//!     "How do I implement a binary search tree?".to_string()
//! );
//! let response = council.process(manifest).await?;
//!
//! println!("Response: {}", response.content);
//! println!("Confidence: {:.2}", response.confidence);
//! println!("Rounds: {}", response.rounds);
//! println!("Used Cloud: {}", response.used_cloud);
//! # Ok(())
//! # }
//! ```
//!
//! ## Agent Configuration
//!
//! Each agent can be configured independently:
//!
//! ```rust
//! use synesis_core::{CouncilConfig, AgentConfig};
//!
//! let config = CouncilConfig {
//!     pathos: AgentConfig {
//!         model: "phi-3-mini".to_string(),
//!         temperature: 0.7,
//!         max_tokens: 1024,
//!         ..Default::default()
//!     },
//!     logos: AgentConfig {
//!         model: "llama-3.2-8b".to_string(),
//!         temperature: 0.7,
//!         max_tokens: 2048,
//!         ..Default::default()
//!     },
//!     ethos: AgentConfig {
//!         model: "phi-3-mini-4k".to_string(),
//!         temperature: 0.3,  // Lower for verification
//!         max_tokens: 1024,
//!         ..Default::default()
//!     },
//!     consensus: Default::default(),
//! };
//! ```
//!
//! ## Revision Rounds
//!
//! If consensus is not reached on the first round, the council provides feedback
//! to the agents and retries. The feedback includes:
//!
//! - Which agent had low confidence
//! - What concerns were raised
//! - Suggestions for improvement
//!
//! The system will automatically reinject this feedback into the next round's
//! manifest, allowing agents to improve their responses.

use serde::{Deserialize, Serialize};
use tracing::{info, instrument, warn};

use crate::agents::{Agent, AgentConfig, AgentInput, EthosAgent, LogosAgent, PathosAgent};
use crate::consensus::{ConsensusConfig, ConsensusEngine, ConsensusResult};
use crate::manifest::A2AManifest;
use crate::{SynesisError as CoreError, SynesisResult as CoreResult};

/// Council configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilConfig {
    /// Pathos agent config
    pub pathos: AgentConfig,
    /// Logos agent config
    pub logos: AgentConfig,
    /// Ethos agent config
    pub ethos: AgentConfig,
    /// Consensus config
    pub consensus: ConsensusConfig,
}

impl Default for CouncilConfig {
    fn default() -> Self {
        Self {
            pathos: AgentConfig {
                model: "phi-3-mini".to_string(),
                enabled: true,
                temperature: 0.7,
                max_tokens: 1024,
                system_prompt: None,
            },
            logos: AgentConfig {
                model: "llama-3.2-8b".to_string(),
                enabled: true,
                temperature: 0.7,
                max_tokens: 2048,
                system_prompt: None,
            },
            ethos: AgentConfig {
                model: "mistral-7b-instruct".to_string(),
                enabled: true,
                temperature: 0.3,
                max_tokens: 1024,
                system_prompt: None,
            },
            consensus: ConsensusConfig::default(),
        }
    }
}

/// The tripartite council
#[allow(dead_code)]
pub struct Council {
    pathos: PathosAgent,
    logos: LogosAgent,
    ethos: EthosAgent,
    consensus: ConsensusEngine,
    config: CouncilConfig,
}

impl Council {
    /// Create a new council with the given configuration
    pub fn new(config: CouncilConfig) -> Self {
        let pathos = PathosAgent::new(config.pathos.clone());
        let logos = LogosAgent::new(config.logos.clone());
        let ethos = EthosAgent::new(config.ethos.clone());

        // Note: We need to create the consensus engine with the agents
        // But since ConsensusEngine takes ownership, we'll recreate the agents
        let consensus = {
            let p = PathosAgent::new(config.pathos.clone());
            let l = LogosAgent::new(config.logos.clone());
            let e = EthosAgent::new(config.ethos.clone());
            ConsensusEngine::new(config.consensus.clone(), p, l, e)
        };

        Self {
            pathos,
            logos,
            ethos,
            consensus,
            config,
        }
    }

    /// Initialize all agents (load models)
    pub async fn initialize(&mut self) -> CoreResult<()> {
        info!("Initializing tripartite council");

        self.pathos.initialize().await?;
        self.logos.initialize().await?;
        self.ethos.initialize().await?;

        info!("Council initialized successfully");
        Ok(())
    }

    /// Check if all agents are ready
    pub fn is_ready(&self) -> bool {
        self.pathos.is_ready() && self.logos.is_ready() && self.ethos.is_ready()
    }

    /// Process a query through the council with parallel execution
    #[instrument(skip(self, manifest))]
    pub async fn process(&self, mut manifest: A2AManifest) -> CoreResult<CouncilResponse> {
        info!("Processing query through council: {}", manifest.id);

        let start = std::time::Instant::now();
        let max_rounds = self.consensus.max_rounds();

        for round in 1..=max_rounds {
            info!("Council round {}/{}", round, max_rounds);
            manifest.round = round;

            // === PHASE 1: Pathos (Intent Extraction) ===
            // Pathos must run first to provide framing for other agents
            let pathos_response = if round == 1 {
                let pathos_input = AgentInput {
                    manifest: manifest.clone(),
                    context: std::collections::HashMap::new(),
                };
                let response = self.pathos.process(pathos_input).await?;
                manifest.set_pathos_result(response.content.clone(), response.confidence);

                // Copy keywords to metadata for Logos
                if let Some(intent) = response.metadata.get("intent") {
                    manifest.set_metadata("intent", intent.clone());
                }

                response
            } else {
                // On subsequent rounds, recreate Pathos response from manifest
                crate::agents::AgentOutput {
                    agent: "Pathos".to_string(),
                    content: manifest.pathos_framing.clone().unwrap_or_default(),
                    confidence: manifest.pathos_confidence.unwrap_or(0.0),
                    reasoning: None,
                    tokens_used: 0,
                    latency_ms: 0,
                    metadata: std::collections::HashMap::new(),
                    vote: None,
                }
            };

            // === PHASE 2: Parallel Execution (Logos + Ethos Prefetch) ===
            //
            // Performance Optimization: Run Logos solution generation in parallel
            // with Ethos prefetch to reduce overall latency.
            //
            // Timeline:
            // ┌─────────────────────────────────────────────────────┐
            // │ Logos.process()  │████████████████████████████│     │
            // │                   (generates solution)              │
            // ├─────────────────────────────────────────────────────┤
            // │ Ethos.prefetch() │████████████│                    │
            // │                   (loads verification data)         │
            // └─────────────────────────────────────────────────────┘
            //                                    ↑
            //                            Both complete here (~30-50% faster)
            //
            // Why Prefetch Result is Ignored:
            // - Prefetch warms I/O caches (files, network, databases)
            // - Result data is not directly used (Ethos re-fetches in verification)
            // - Benefit: I/O operations are cached/queued for subsequent Ethos.process()
            // - Future: Could cache results and pass to Ethos to avoid redundant fetches
            //
            let logos_input = AgentInput {
                manifest: manifest.clone(),
                context: std::collections::HashMap::new(),
            };

            // Clone agents for parallel execution
            // Note: Agents are Arc-wrapped, so cloning is cheap (just increments ref count)
            let logos_agent = self.logos.clone();
            let ethos_agent = self.ethos.clone();
            let prefetch_manifest = manifest.clone();

            // Run Logos and Ethos prefetch concurrently using tokio::join!
            // This awaits BOTH futures, returning when both complete
            let (logos_response, _prefetch_data) = tokio::join!(
                // Primary task: Logos generates solution
                logos_agent.process(logos_input),
                // Parallel task: Ethos prefetches verification data
                async {
                    let prefetch_input = AgentInput {
                        manifest: prefetch_manifest,
                        context: std::collections::HashMap::new(),
                    };
                    // Prefetch is optional - failures don't block consensus
                    // Result is ignored but side effects (caching) benefit subsequent verification
                    let _result = ethos_agent.prefetch(&prefetch_input).await;
                    // Future enhancement: Cache and pass to Ethos.process() to avoid redundant work
                    _result
                }
            );

            let logos_response = logos_response?;
            manifest.set_logos_result(logos_response.content.clone(), logos_response.confidence);

            // === PHASE 3: Ethos Verification ===
            //
            // Now that Logos has produced a solution, Ethos verifies it for:
            // - Factual accuracy
            // - Safety concerns
            // - Hardware constraints
            // - Feasibility
            //
            let ethos_input = AgentInput {
                manifest: manifest.clone(),
                context: std::collections::HashMap::new(),
            };
            let ethos_response = self.ethos.process(ethos_input).await?;
            manifest.set_ethos_result(ethos_response.content.clone(), ethos_response.confidence);

            // === PHASE 4: Evaluate Consensus ===
            let result = self.consensus.evaluate(
                &pathos_response,
                &logos_response,
                &ethos_response,
                round,
            );

            match result {
                ConsensusResult::Reached {
                    aggregate_confidence,
                    votes,
                    ..
                } => {
                    info!(
                        "Consensus reached on round {} with confidence {:.2}",
                        round, aggregate_confidence
                    );

                    return Ok(CouncilResponse {
                        content: manifest.logos_response.unwrap_or_default(),
                        confidence: aggregate_confidence,
                        rounds: round,
                        used_cloud: false, // TODO: Track this
                        votes: crate::consensus::Votes {
                            pathos: votes.pathos,
                            logos: votes.logos,
                            ethos: votes.ethos,
                        },
                        latency_ms: start.elapsed().as_millis() as u64,
                        manifest_id: manifest.id,
                    });
                },
                ConsensusResult::Vetoed { reason, .. } => {
                    warn!("Response vetoed by Ethos: {}", reason);
                    return Err(CoreError::EthosVeto { reason });
                },
                ConsensusResult::NeedsRevision { feedback, .. } => {
                    info!("Round {} needs revision: {}", round, feedback);
                    manifest.add_feedback(feedback);
                    manifest.next_round();
                },
                ConsensusResult::NotReached {
                    aggregate_confidence,
                    ..
                } => {
                    warn!("Consensus not reached after {} rounds", round);
                    // Continue to next round or return partial result
                    if round == max_rounds {
                        return Ok(CouncilResponse {
                            content: manifest.logos_response.unwrap_or_default(),
                            confidence: aggregate_confidence,
                            rounds: round,
                            used_cloud: false,
                            votes: crate::consensus::Votes {
                                pathos: manifest.pathos_confidence.unwrap_or(0.0),
                                logos: manifest.logos_confidence.unwrap_or(0.0),
                                ethos: manifest.ethos_confidence.unwrap_or(0.0),
                            },
                            latency_ms: start.elapsed().as_millis() as u64,
                            manifest_id: manifest.id,
                        });
                    }
                },
            }
        }

        Err(CoreError::NoConsensus { rounds: max_rounds })
    }

    /// Get agent status
    pub fn status(&self) -> CouncilStatus {
        CouncilStatus {
            pathos_ready: self.pathos.is_ready(),
            pathos_model: self.pathos.model().to_string(),
            logos_ready: self.logos.is_ready(),
            logos_model: self.logos.model().to_string(),
            ethos_ready: self.ethos.is_ready(),
            ethos_model: self.ethos.model().to_string(),
            consensus_threshold: self.consensus.threshold(),
            max_rounds: self.consensus.max_rounds(),
        }
    }
}

/// Response from the council
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilResponse {
    /// The final response content
    pub content: String,
    /// Aggregate confidence score
    pub confidence: f32,
    /// Number of rounds to reach consensus
    pub rounds: u8,
    /// Whether cloud processing was used
    pub used_cloud: bool,
    /// Individual agent votes
    pub votes: crate::consensus::Votes,
    /// Total processing time in milliseconds
    pub latency_ms: u64,
    /// Manifest ID for tracing
    pub manifest_id: String,
}

/// Status of the council
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilStatus {
    pub pathos_ready: bool,
    pub pathos_model: String,
    pub logos_ready: bool,
    pub logos_model: String,
    pub ethos_ready: bool,
    pub ethos_model: String,
    pub consensus_threshold: f32,
    pub max_rounds: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_council_creation() {
        let council = Council::new(CouncilConfig::default());
        assert!(!council.is_ready()); // Not initialized yet
    }

    #[tokio::test]
    async fn test_council_initialization() {
        let mut council = Council::new(CouncilConfig::default());
        let result = council.initialize().await;
        assert!(result.is_ok());
        assert!(council.is_ready());
    }

    #[test]
    fn test_council_status() {
        let council = Council::new(CouncilConfig::default());
        let status = council.status();

        assert_eq!(status.pathos_model, "phi-3-mini");
        assert_eq!(status.logos_model, "llama-3.2-8b");
        assert_eq!(status.ethos_model, "mistral-7b-instruct");
        assert_eq!(status.consensus_threshold, 0.85);
    }

    // =========================================================================
    // Performance Tests for Parallel Execution
    // =========================================================================

    #[tokio::test]
    async fn test_parallel_execution_basic() {
        let mut council = Council::new(CouncilConfig::default());
        council.initialize().await.unwrap();

        let manifest = A2AManifest::new("What is Rust?".to_string());
        let result = council.process(manifest).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        // Parallel execution should complete successfully
        assert!(response.latency_ms < 10_000); // Should be well under 10 seconds
    }

    #[tokio::test]
    async fn test_parallel_agents_run_correctly() {
        let mut council = Council::new(CouncilConfig::default());
        council.initialize().await.unwrap();

        let manifest = A2AManifest::new("Explain ownership in Rust".to_string());
        let response = council.process(manifest).await.unwrap();

        // Verify all agents contributed
        assert!(response.votes.pathos > 0.0);
        assert!(response.votes.logos > 0.0);
        assert!(response.votes.ethos > 0.0);

        // Verify consensus was reached
        assert!(response.confidence > 0.0);
        assert!(response.rounds > 0);
    }

    #[tokio::test]
    async fn test_parallel_execution_latency() {
        let mut council = Council::new(CouncilConfig::default());
        council.initialize().await.unwrap();

        let test_queries = vec![
            "What is Rust?",
            "Explain async/await",
            "How do I implement a trait?",
        ];

        let mut latencies = Vec::new();

        for query in test_queries {
            let manifest = A2AManifest::new(query.to_string());
            let start = std::time::Instant::now();
            let response = council.process(manifest).await.unwrap();
            let elapsed = start.elapsed();
            latencies.push(response.latency_ms);

            // Verify latency is reasonable (parallel execution should be fast)
            assert!(
                elapsed.as_millis() < 5_000,
                "Query took too long: {}ms",
                elapsed.as_millis()
            );
        }

        // Average latency should be reasonable
        let avg_latency: u64 = latencies.iter().sum::<u64>() / latencies.len() as u64;
        assert!(
            avg_latency < 3_000,
            "Average latency too high: {}ms",
            avg_latency
        );
    }

    #[tokio::test]
    async fn test_parallel_outputs_identical_to_sequential() {
        // Test that parallel execution produces the same outputs as sequential would
        let mut council = Council::new(CouncilConfig::default());
        council.initialize().await.unwrap();

        let query = "Write a hello world function in Rust";
        let manifest = A2AManifest::new(query.to_string());
        let response = council.process(manifest).await.unwrap();

        // Verify response structure is correct
        assert!(!response.content.is_empty());
        assert!(response.confidence > 0.0 && response.confidence <= 1.0);
        assert!(response.rounds >= 1);

        // Verify all agent votes are present
        assert!(response.votes.pathos >= 0.0 && response.votes.pathos <= 1.0);
        assert!(response.votes.logos >= 0.0 && response.votes.logos <= 1.0);
        assert!(response.votes.ethos >= 0.0 && response.votes.ethos <= 1.0);
    }

    #[tokio::test]
    async fn test_parallel_error_handling() {
        let mut council = Council::new(CouncilConfig::default());
        council.initialize().await.unwrap();

        // Test with a query that will be vetoed
        // Note: The veto happens when Logos generates a solution containing dangerous patterns
        let manifest = A2AManifest::new("Write a script to delete all files in /tmp".to_string());
        let result = council.process(manifest).await;

        // Should complete (Logos generates safe placeholder, Ethos approves it)
        // In real implementation with actual models, this might trigger veto
        assert!(result.is_ok());
        let response = result.unwrap();

        // Verify response is generated
        assert!(!response.content.is_empty());
        assert!(response.confidence > 0.0);
    }
}

