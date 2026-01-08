//! Query Router
//!
//! Determines whether a query should be processed locally or escalated to cloud.
//! Uses various heuristics and can be trained over time.

use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

use crate::manifest::A2AManifest;

/// Routing decision
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoutingDecision {
    /// Process entirely on local hardware
    Local,
    /// Escalate to cloud processing
    Cloud,
    /// Start local, may escalate if needed
    Hybrid,
}

/// Reasons for routing decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingReason {
    pub decision: RoutingDecision,
    pub confidence: f32,
    pub factors: Vec<String>,
}

/// Router configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    /// Maximum tokens to process locally
    pub max_local_tokens: u32,
    /// Complexity threshold for cloud escalation (1-10)
    pub complexity_threshold: u8,
    /// Force local-only mode
    pub force_local: bool,
    /// Force cloud mode
    pub force_cloud: bool,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            max_local_tokens: 4096,
            complexity_threshold: 7,
            force_local: false,
            force_cloud: false,
        }
    }
}

/// Query router
pub struct Router {
    config: RouterConfig,
}

impl Router {
    /// Create a new router
    pub fn new(config: RouterConfig) -> Self {
        Self { config }
    }

    /// Route a query
    #[instrument(skip(self))]
    pub fn route(&self, manifest: &A2AManifest) -> RoutingReason {
        debug!("Routing query: {}", manifest.id);

        // Check force flags
        if self.config.force_local {
            return RoutingReason {
                decision: RoutingDecision::Local,
                confidence: 1.0,
                factors: vec!["Force local mode enabled".to_string()],
            };
        }

        if self.config.force_cloud {
            return RoutingReason {
                decision: RoutingDecision::Cloud,
                confidence: 1.0,
                factors: vec!["Force cloud mode enabled".to_string()],
            };
        }

        let mut factors = vec![];
        let mut cloud_score = 0.0f32;

        // Factor 1: Query length
        let query_tokens = estimate_tokens(&manifest.query);
        if query_tokens > self.config.max_local_tokens {
            cloud_score += 0.4;
            factors.push(format!(
                "Query length ({} tokens) exceeds local limit ({})",
                query_tokens, self.config.max_local_tokens
            ));
        }

        // Factor 2: Complexity from intent
        if let Some(complexity) = manifest.get_metadata("complexity") {
            if let Some(c) = complexity.as_u64() {
                if c as u8 >= self.config.complexity_threshold {
                    cloud_score += 0.3;
                    factors.push(format!(
                        "High complexity ({}) exceeds threshold ({})",
                        c, self.config.complexity_threshold
                    ));
                }
            }
        }

        // Factor 3: Requires knowledge retrieval
        if manifest.flags.needs_knowledge {
            // Knowledge retrieval is local, but complex queries may need cloud
            cloud_score += 0.1;
            factors.push("Query requires knowledge retrieval".to_string());
        }

        // Factor 4: Multi-turn context
        if manifest.history.len() > 5 {
            cloud_score += 0.2;
            factors.push(format!(
                "Long conversation history ({} turns)",
                manifest.history.len()
            ));
        }

        // Factor 5: Specific task types that benefit from cloud
        let cloud_keywords = [
            "analyze",
            "research",
            "compare",
            "comprehensive",
            "detailed",
        ];
        let query_lower = manifest.query.to_lowercase();
        for keyword in cloud_keywords {
            if query_lower.contains(keyword) {
                cloud_score += 0.1;
                factors.push(format!(
                    "Query contains cloud-beneficial keyword: {}",
                    keyword
                ));
                break;
            }
        }

        // Make decision
        let decision = if cloud_score >= 0.5 {
            RoutingDecision::Cloud
        } else if cloud_score >= 0.2 {
            RoutingDecision::Hybrid
        } else {
            RoutingDecision::Local
        };

        if factors.is_empty() {
            factors.push("Default: simple query suitable for local processing".to_string());
        }

        RoutingReason {
            decision,
            confidence: if cloud_score >= 0.5 {
                cloud_score.min(1.0)
            } else {
                1.0 - cloud_score
            },
            factors,
        }
    }

    /// Check if escalation is recommended during processing
    pub fn should_escalate(&self, manifest: &A2AManifest, current_tokens: u32) -> bool {
        if self.config.force_local {
            return false;
        }

        // Escalate if we're running out of local capacity
        if current_tokens > self.config.max_local_tokens * 3 / 4 {
            return true;
        }

        // Escalate if consensus is failing
        if manifest.round >= 2 {
            return true;
        }

        false
    }
}

/// Estimate token count (rough approximation)
fn estimate_tokens(text: &str) -> u32 {
    // Rough estimate: ~4 characters per token for English
    // Cap at reasonable maximum (1M tokens) to prevent overflow
    (text.len() / 4).min(1_000_000) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_query_routes_local() {
        let router = Router::new(RouterConfig::default());
        let manifest = A2AManifest::new("Hello, how are you?".to_string());

        let result = router.route(&manifest);
        assert_eq!(result.decision, RoutingDecision::Local);
    }

    #[test]
    fn test_long_query_routes_cloud() {
        let router = Router::new(RouterConfig {
            max_local_tokens: 100,
            ..Default::default()
        });

        let long_query = "a ".repeat(1000); // ~500 tokens
        let manifest = A2AManifest::new(long_query);

        let result = router.route(&manifest);
        assert!(matches!(
            result.decision,
            RoutingDecision::Cloud | RoutingDecision::Hybrid
        ));
    }

    #[test]
    fn test_force_local() {
        let router = Router::new(RouterConfig {
            force_local: true,
            ..Default::default()
        });

        let manifest = A2AManifest::new("Analyze this comprehensive research".to_string());

        let result = router.route(&manifest);
        assert_eq!(result.decision, RoutingDecision::Local);
    }

    #[test]
    fn test_force_cloud() {
        let router = Router::new(RouterConfig {
            force_cloud: true,
            ..Default::default()
        });

        let manifest = A2AManifest::new("Hi".to_string());

        let result = router.route(&manifest);
        assert_eq!(result.decision, RoutingDecision::Cloud);
    }
}
