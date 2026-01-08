//! Ethos Agent - Verification & Truth
//!
//! Ethos is the final check. It verifies the response from Logos for:
//! - Factual accuracy (when verifiable)
//! - Safety (no harmful content)
//! - Feasibility (can the suggestion actually work?)
//! - Hardware constraints (can the system handle this?)
//! - Code quality (for generated code)
//!
//! Ethos has VETO power - if it flags something, the response doesn't go through.
//!
//! Model: phi-3-mini-4k (fast verification, good at following instructions)

use async_trait::async_trait;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, instrument, warn};

use super::{
    Agent, AgentConfig, AgentInput, AgentOutput, ConsensusVote, Constraint, ConstraintType,
    Severity,
};
use crate::consensus::Verdict;
use crate::manifest::A2AManifest;
use crate::{CoreError, CoreResult};

/// Ethos agent for verification
#[derive(Clone)]
pub struct EthosAgent {
    config: AgentConfig,
    ready: Arc<std::sync::atomic::AtomicBool>,
    // Dangerous patterns for veto scenarios (immutable collection)
    veto_patterns: Arc<Vec<VetoPattern>>,
}

/// A dangerous pattern that triggers automatic veto
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct VetoPattern {
    pattern: Regex,
    description: &'static str,
    category: VetoCategory,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
enum VetoCategory {
    FileSystem,
    Credential,
    Network,
    System,
    Thermal,
}

/// Verification result from Ethos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthosVerdict {
    /// Final verdict
    pub verdict: Verdict,

    /// Feedback for revision rounds
    pub feedback: String,

    /// Constraints violated
    pub constraints_violated: Vec<Constraint>,

    /// Overall confidence score (0.0-1.0)
    pub confidence: f32,
}

/// Prefetch data for Ethos verification (computed in parallel with Logos)
#[derive(Debug, Clone)]
pub struct EthosPrefetchData {
    /// Pre-computed safety pattern checks
    pub safety_constraints: Vec<Constraint>,
    /// Pre-fetched hardware limits
    pub hardware_constraints: Vec<Constraint>,
    /// Whether the solution contains code
    pub contains_code: bool,
    /// Whether secrets were detected
    pub contains_secrets: bool,
}

impl EthosAgent {
    /// Create a new Ethos agent
    pub fn new(config: AgentConfig) -> Self {
        // Initialize veto patterns - these trigger automatic blocking
        // NOTE: These regex patterns are tested and validated. If modifying,
        // ensure all regexes compile correctly or agent initialization will panic.
        let veto_patterns = vec![
            // File system dangers
            VetoPattern {
                pattern: Regex::new(r"rm\s+-rf\s+/")
                    .expect("Regex compilation failed - this is a bug in the hardcoded pattern"),
                description: "Recursive root deletion",
                category: VetoCategory::FileSystem,
            },
            VetoPattern {
                pattern: Regex::new(r"rm\s+-rf\s+\$HOME")
                    .expect("Regex compilation failed - this is a bug in the hardcoded pattern"),
                description: "Home directory deletion",
                category: VetoCategory::FileSystem,
            },
            VetoPattern {
                pattern: Regex::new(r"rm\s+-rf\s+~")
                    .expect("Regex compilation failed - this is a bug in the hardcoded pattern"),
                description: "Home directory deletion",
                category: VetoCategory::FileSystem,
            },
            VetoPattern {
                pattern: Regex::new(r"dd\s+if=/dev/zero")
                    .expect("Regex compilation failed - this is a bug in the hardcoded pattern"),
                description: "Disk destruction command",
                category: VetoCategory::FileSystem,
            },
            VetoPattern {
                pattern: Regex::new(r":\(.*\)\{\s*:\|\:&\s*;\s*\}")
                    .expect("Regex compilation failed - this is a bug in the hardcoded pattern"),
                description: "Fork bomb pattern",
                category: VetoCategory::System,
            },
            // Network dangers
            VetoPattern {
                pattern: Regex::new(r"curl.*\|\s*(sh|bash)")
                    .expect("Regex compilation failed - this is a bug in the hardcoded pattern"),
                description: "Piping URL directly to shell",
                category: VetoCategory::Network,
            },
            VetoPattern {
                pattern: Regex::new(r"wget.*\|\s*(sh|bash)")
                    .expect("Regex compilation failed - this is a bug in the hardcoded pattern"),
                description: "Piping URL directly to shell",
                category: VetoCategory::Network,
            },
            VetoPattern {
                pattern: Regex::new(r"eval\s*\(\s*.*curl")
                    .expect("Regex compilation failed - this is a bug in the hardcoded pattern"),
                description: "Eval with remote content",
                category: VetoCategory::Network,
            },
            // System file modification
            VetoPattern {
                pattern: Regex::new(r"echo.*>.*(/etc/|/usr/bin/|/usr/sbin/|/boot/|/kernel)")
                    .expect("Regex compilation failed - this is a bug in the hardcoded pattern"),
                description: "System file modification",
                category: VetoCategory::System,
            },
            VetoPattern {
                pattern: Regex::new(r"chmod\s+777\s+(/etc/|/usr/|/var/|/boot/)")
                    .expect("Regex compilation failed - this is a bug in the hardcoded pattern"),
                description: "Overly permissive system file permissions",
                category: VetoCategory::System,
            },
            // Credential exposure patterns
            VetoPattern {
                pattern: Regex::new(r#"(password|api_key|secret)\s*=\s*['"][^'"]+['"]"#)
                    .expect("Regex compilation failed - this is a bug in the hardcoded pattern"),
                description: "Hardcoded credentials",
                category: VetoCategory::Credential,
            },
            VetoPattern {
                pattern: Regex::new(r"(Bearer|Token)\s+[A-Za-z0-9]{20,}")
                    .expect("Regex compilation failed - this is a bug in the hardcoded pattern"),
                description: "Token exposure",
                category: VetoCategory::Credential,
            },
        ];

        Self {
            config,
            ready: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            veto_patterns: Arc::new(veto_patterns),
        }
    }

    /// Initialize the agent (load model)
    pub async fn initialize(&mut self) -> CoreResult<()> {
        info!("Initializing Ethos agent with model: {}", self.config.model);

        // TODO: Load phi-3-mini-4k model via synesis-models
        // For now, mark as ready with placeholder model
        debug!("Model loading placeholder - phi-3-mini-4k will be integrated");

        self.ready.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    /// Prefetch verification data (can run in parallel with Logos)
    /// This pre-computes expensive checks before the Logos solution is ready
    #[instrument(skip(self, input))]
    pub async fn prefetch(&self, input: &AgentInput) -> CoreResult<EthosPrefetchData> {
        let manifest = &input.manifest;
        let query = &manifest.query;

        debug!("Prefetching Ethos verification data");

        // Pre-check safety patterns on the query (before we have the solution)
        let safety_constraints = self.check_safety_patterns(query).await?;

        // Pre-fetch hardware constraints
        let hardware_constraints = if self.should_check_hardware(manifest) {
            self.check_hardware_constraints(query).await?
        } else {
            vec![]
        };

        // Pre-detect content characteristics
        let contains_code = self.contains_code(query);
        let contains_secrets = self.contains_secrets(query);

        Ok(EthosPrefetchData {
            safety_constraints,
            hardware_constraints,
            contains_code,
            contains_secrets,
        })
    }

    /// Main verification entry point
    #[instrument(skip(self, input))]
    pub async fn verify(&self, input: &AgentInput) -> CoreResult<EthosVerdict> {
        let start = std::time::Instant::now();
        let manifest = &input.manifest;

        // Get Logos solution to verify
        let solution = manifest
            .logos_response
            .as_deref()
            .ok_or_else(|| CoreError::AgentError("No Logos response to verify".to_string()))?;

        debug!("Starting verification for solution");

        let mut constraints = Vec::new();

        // 1. Safety verification (always run - VETO scenarios)
        constraints.extend(self.check_safety_patterns(solution).await?);

        // Check for veto conditions early
        if constraints.iter().any(|c| c.severity == Severity::Critical) {
            let critical_count = constraints
                .iter()
                .filter(|c| c.severity == Severity::Critical)
                .count();
            let feedback = self.generate_feedback(&constraints, Verdict::Veto);

            warn!("Ethos VETOED response: {} critical issues", critical_count);

            return Ok(EthosVerdict {
                verdict: Verdict::Veto,
                feedback,
                confidence: 0.0,
                constraints_violated: constraints,
            });
        }

        // 2. Hardware constraint validation (if applicable)
        if self.should_check_hardware(manifest) {
            constraints.extend(self.check_hardware_constraints(solution).await?);
        }

        // 3. Fact-checking (placeholder for model-based verification)
        if self.should_check_facts(manifest) {
            constraints.extend(self.check_facts(solution).await?);
        }

        // 4. Code quality checks (if code present)
        if self.contains_code(solution) {
            constraints.extend(self.check_code_quality(solution).await?);
        }

        // 5. Thermal limit checks
        constraints.extend(self.check_thermal_limits(solution).await?);

        // Determine final verdict
        let verdict = self.determine_verdict(&constraints);
        let confidence = self.calculate_confidence(&constraints);
        let feedback = self.generate_feedback(&constraints, verdict);

        let elapsed = start.elapsed();
        debug!(
            "Verification completed in {:?} with verdict: {:?}",
            elapsed, verdict
        );

        Ok(EthosVerdict {
            verdict,
            feedback,
            confidence,
            constraints_violated: constraints,
        })
    }

    /// Check for dangerous safety patterns (VETO scenarios)
    async fn check_safety_patterns(&self, solution: &str) -> CoreResult<Vec<Constraint>> {
        let mut constraints = Vec::new();

        for veto_pattern in self.veto_patterns.iter() {
            if veto_pattern.pattern.is_match(solution) {
                warn!("Detected veto pattern: {}", veto_pattern.description);

                constraints.push(Constraint {
                    constraint_type: ConstraintType::Safety,
                    severity: Severity::Critical, // Critical = veto
                    description: format!(
                        "Dangerous pattern detected: {}",
                        veto_pattern.description
                    ),
                    source: Some("safety-pattern-scanner".to_string()),
                    suggestion: Some("This operation cannot be executed automatically".to_string()),
                });
            }
        }

        // Additional credential exposure check
        if self.contains_secrets(solution) {
            constraints.push(Constraint {
                constraint_type: ConstraintType::Safety,
                severity: Severity::Error,
                description: "Solution appears to contain hardcoded credentials or secrets"
                    .to_string(),
                source: Some("secret-scanner".to_string()),
                suggestion: Some("Use environment variables or a secrets manager".to_string()),
            });
        }

        Ok(constraints)
    }

    /// Check hardware constraints
    async fn check_hardware_constraints(&self, solution: &str) -> CoreResult<Vec<Constraint>> {
        let mut constraints = Vec::new();

        // Extract memory requirements from solution
        if let Some(mem_req) = self.extract_memory_requirement(solution) {
            // TODO: Get actual hardware limits from hardware manifest
            let max_vram_mb = 8192; // Placeholder - should come from hardware manifest

            if mem_req > max_vram_mb {
                constraints.push(Constraint {
                    constraint_type: ConstraintType::Hardware,
                    severity: Severity::Error,
                    description: format!(
                        "Solution requires {}MB VRAM but device only has {}MB",
                        mem_req, max_vram_mb
                    ),
                    source: Some("hardware-check".to_string()),
                    suggestion: Some("Use a smaller model or enable quantization".to_string()),
                });
            }
        }

        // Check power mode references
        if solution.contains("max_power") || solution.contains("POWER_MODE=MAX") {
            // TODO: Get actual power limits from hardware manifest
            let max_power_watts = 30; // Placeholder

            if max_power_watts < 25 {
                constraints.push(Constraint {
                    constraint_type: ConstraintType::Hardware,
                    severity: Severity::Warning,
                    description: "Max power mode requested but device has low power limit"
                        .to_string(),
                    source: Some("power-check".to_string()),
                    suggestion: Some("Consider using 15W power mode instead".to_string()),
                });
            }
        }

        Ok(constraints)
    }

    /// Check facts using model-based verification (placeholder)
    async fn check_facts(&self, solution: &str) -> CoreResult<Vec<Constraint>> {
        let mut constraints = Vec::new();

        // TODO: Use phi-3-mini model to extract and verify factual claims
        // For now, check for obvious overconfident statements
        let overconfident_patterns = [
            "will definitely succeed",
            "guaranteed to work",
            "100% success rate",
            "impossible to fail",
            "always works",
        ];

        let solution_lower = solution.to_lowercase();
        for pattern in &overconfident_patterns {
            if solution_lower.contains(pattern) {
                constraints.push(Constraint {
                    constraint_type: ConstraintType::Fact,
                    severity: Severity::Warning,
                    description: format!("Overconfident claim: '{}'", pattern),
                    source: Some("fact-checker".to_string()),
                    suggestion: Some(
                        "Use more cautious language (e.g., 'should work', 'likely to succeed')"
                            .to_string(),
                    ),
                });
            }
        }

        debug!("Fact-checked {} claims", constraints.len());
        Ok(constraints)
    }

    /// Check code quality (if code present)
    async fn check_code_quality(&self, solution: &str) -> CoreResult<Vec<Constraint>> {
        let mut constraints = Vec::new();

        // Check for common code quality issues
        if solution.contains("TODO") || solution.contains("FIXME") {
            constraints.push(Constraint {
                constraint_type: ConstraintType::Quality,
                severity: Severity::Warning,
                description: "Solution contains TODO/FIXME markers".to_string(),
                source: Some("code-quality".to_string()),
                suggestion: Some("Resolve TODO items before providing solution".to_string()),
            });
        }

        // Check for hardcoded paths
        if solution.contains("/home/") || solution.contains("C:\\Users\\") {
            constraints.push(Constraint {
                constraint_type: ConstraintType::Quality,
                severity: Severity::Warning,
                description: "Solution contains user-specific hardcoded paths".to_string(),
                source: Some("code-quality".to_string()),
                suggestion: Some(
                    "Use environment variables or $HOME for cross-user compatibility".to_string(),
                ),
            });
        }

        // Check for error handling (basic)
        if solution.contains("unwrap()") && !solution.contains("match") && !solution.contains("?") {
            constraints.push(Constraint {
                constraint_type: ConstraintType::Quality,
                severity: Severity::Warning,
                description: "Solution contains .unwrap() calls without proper error handling"
                    .to_string(),
                source: Some("code-quality".to_string()),
                suggestion: Some(
                    "Replace .unwrap() with proper error handling using ? or match".to_string(),
                ),
            });
        }

        debug!("Code quality check found {} issues", constraints.len());
        Ok(constraints)
    }

    /// Check thermal limits
    async fn check_thermal_limits(&self, solution: &str) -> CoreResult<Vec<Constraint>> {
        let mut constraints = Vec::new();

        // TODO: Get actual temperature from hardware manifest
        let current_temp = 45.0; // Placeholder temperature in Celsius

        // Check if solution suggests intensive operations
        let intensive_keywords = ["intensive", "max performance", "full load", "benchmark"];
        let is_intensive = intensive_keywords
            .iter()
            .any(|kw| solution.to_lowercase().contains(kw));

        if is_intensive && current_temp > 75.0 {
            constraints.push(Constraint {
                constraint_type: ConstraintType::Hardware,
                severity: Severity::Error,
                description: format!(
                    "Device is warm ({}°C), intensive task may cause thermal throttling",
                    current_temp
                ),
                source: Some("thermal-check".to_string()),
                suggestion: Some("Consider waiting for cooldown or reducing workload".to_string()),
            });
        } else if is_intensive && current_temp > 60.0 {
            constraints.push(Constraint {
                constraint_type: ConstraintType::Hardware,
                severity: Severity::Warning,
                description: format!("Device temperature is elevated ({}°C)", current_temp),
                source: Some("thermal-check".to_string()),
                suggestion: Some("Monitor temperature during execution".to_string()),
            });
        }

        Ok(constraints)
    }

    /// Determine verdict based on constraints
    fn determine_verdict(&self, constraints: &[Constraint]) -> Verdict {
        // Any critical constraint = veto
        if constraints.iter().any(|c| c.severity == Severity::Critical) {
            return Verdict::Veto;
        }

        // Any error = needs revision
        if constraints.iter().any(|c| c.severity == Severity::Error) {
            return Verdict::NeedsRevision;
        }

        // Only warnings = approved with notes
        Verdict::Approved
    }

    /// Calculate confidence based on constraints
    fn calculate_confidence(&self, constraints: &[Constraint]) -> f32 {
        let mut confidence = 1.0f32;

        for constraint in constraints {
            match constraint.severity {
                Severity::Critical => confidence = 0.0,
                Severity::Error => confidence -= 0.3,
                Severity::Warning => confidence -= 0.1,
            }
        }

        confidence.max(0.0)
    }

    /// Generate human-readable feedback
    fn generate_feedback(&self, constraints: &[Constraint], verdict: Verdict) -> String {
        if constraints.is_empty() {
            return "Solution verified. No issues found.".to_string();
        }

        let mut feedback = String::new();

        match verdict {
            Verdict::Veto => {
                feedback.push_str("BLOCKED: Critical issues prevent this solution.\n\n");
            },
            Verdict::NeedsRevision => {
                feedback.push_str("REVISION NEEDED: Please address the following:\n\n");
            },
            Verdict::Approved => {
                feedback.push_str("APPROVED with notes:\n\n");
            },
        }

        for (i, constraint) in constraints.iter().enumerate() {
            feedback.push_str(&format!(
                "{}. [{}] {}\n",
                i + 1,
                match constraint.severity {
                    Severity::Critical => "CRITICAL",
                    Severity::Error => "ERROR",
                    Severity::Warning => "WARNING",
                },
                constraint.description
            ));

            if let Some(ref suggestion) = constraint.suggestion {
                feedback.push_str(&format!("   Suggestion: {}\n", suggestion));
            }
            feedback.push('\n');
        }

        feedback
    }

    // Helper methods

    fn should_check_hardware(&self, _manifest: &A2AManifest) -> bool {
        // TODO: Check manifest flags for hardware check requirement
        true // Always check for now
    }

    fn should_check_facts(&self, _manifest: &A2AManifest) -> bool {
        // TODO: Check manifest flags for fact-checking requirement
        true // Always check for now
    }

    fn contains_code(&self, solution: &str) -> bool {
        // Basic heuristic for detecting code
        solution.contains("```")
            || solution.contains("def ")
            || solution.contains("fn ")
            || solution.contains("function ")
            || solution.contains("class ")
            || solution.contains("import ")
            || solution.contains("use ")
            || solution.contains("package ")
    }

    fn contains_secrets(&self, solution: &str) -> bool {
        // Check for common secret patterns
        let secret_patterns = [
            "api_key",
            "apikey",
            "api-key",
            "secret",
            "password",
            "token",
            "private_key",
            "private-key",
        ];

        let solution_lower = solution.to_lowercase();
        for pattern in &secret_patterns {
            if solution_lower.contains(pattern) {
                // Look for assignment or colon after the pattern
                let pattern_with_equals = format!("{}\\s*[=:]\\s*['\"]?[^\\s'\"]+", pattern);
                if let Ok(re) = Regex::new(&pattern_with_equals) {
                    if re.is_match(solution) {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn extract_memory_requirement(&self, solution: &str) -> Option<u32> {
        // Look for memory requirements like "requires 16GB", "40GB VRAM", etc.
        let mem_patterns = [
            r"(\d+)\s*GB\s*VRAM",
            r"(\d+)\s*GB\s*memory",
            r"requires\s+(\d+)\s*GB",
        ];

        for pattern in &mem_patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(caps) = re.captures(solution) {
                    if let Some(mb_match) = caps.get(1) {
                        if let Ok(gb) = mb_match.as_str().parse::<u32>() {
                            return Some(gb * 1024); // Convert GB to MB
                        }
                    }
                }
            }
        }

        None
    }
}

impl Default for EthosAgent {
    fn default() -> Self {
        Self::new(AgentConfig {
            model: "phi-3-mini-4k".to_string(),
            ..Default::default()
        })
    }
}

#[async_trait]
impl Agent for EthosAgent {
    fn name(&self) -> &str {
        "Ethos"
    }

    fn role(&self) -> &str {
        "Verification, safety, and truth assessment"
    }

    async fn process(&self, input: AgentInput) -> CoreResult<AgentOutput> {
        if !self.is_ready() {
            return Err(CoreError::AgentError("Ethos not initialized".to_string()));
        }

        let start = std::time::Instant::now();

        // Run verification
        let verdict = self.verify(&input).await?;

        // Create consensus vote based on verdict
        let approve = matches!(verdict.verdict, Verdict::Approved);
        let vote = ConsensusVote::new(self.name(), approve, verdict.confidence)
            .with_reasoning(format!("{:?}", verdict.verdict))
            .with_concerns(verdict.constraints_violated.clone());

        let mut metadata = HashMap::new();
        metadata.insert(
            "verdict".to_string(),
            serde_json::to_value(verdict.verdict).unwrap_or_default(),
        );
        metadata.insert(
            "constraints".to_string(),
            serde_json::to_value(&verdict.constraints_violated).unwrap_or_default(),
        );
        metadata.insert(
            "feedback".to_string(),
            serde_json::Value::String(verdict.feedback.clone()),
        );

        Ok(AgentOutput {
            agent: self.name().to_string(),
            content: verdict.feedback,
            confidence: verdict.confidence,
            reasoning: Some(format!(
                "Verdict: {:?} | Constraints: {} | Confidence: {:.0}%",
                verdict.verdict,
                verdict.constraints_violated.len(),
                verdict.confidence * 100.0
            )),
            tokens_used: 0, // Placeholder
            latency_ms: start.elapsed().as_millis() as u64,
            metadata,
            vote: Some(vote),
        })
    }

    fn is_ready(&self) -> bool {
        self.ready.load(std::sync::atomic::Ordering::SeqCst)
    }

    fn model(&self) -> &str {
        &self.config.model
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dangerous_command_veto() {
        let ethos = EthosAgent::default();
        let mut manifest = A2AManifest::new("Test query".to_string());
        manifest.set_logos_result("Run: rm -rf /".to_string(), 0.9);

        let input = AgentInput::new(manifest);
        let verdict = ethos.verify(&input).await.unwrap();

        assert_eq!(verdict.verdict, Verdict::Veto);
        assert!(verdict
            .constraints_violated
            .iter()
            .any(|c| c.severity == Severity::Critical));
    }

    #[tokio::test]
    async fn test_credential_exposure() {
        let ethos = EthosAgent::default();
        let mut manifest = A2AManifest::new("Test query".to_string());
        manifest.set_logos_result("Connect with password='secret123'".to_string(), 0.9);

        let input = AgentInput::new(manifest);
        let verdict = ethos.verify(&input).await.unwrap();

        // Should be vetoed due to credential pattern
        assert_eq!(verdict.verdict, Verdict::Veto);
    }

    #[tokio::test]
    async fn test_clean_solution_approved() {
        let ethos = EthosAgent::default();
        let mut manifest = A2AManifest::new("Test query".to_string());
        manifest.set_logos_result("def hello(): return 'Hello, World!'".to_string(), 0.9);

        let input = AgentInput::new(manifest);
        let verdict = ethos.verify(&input).await.unwrap();

        assert_eq!(verdict.verdict, Verdict::Approved);
        assert!(
            verdict.constraints_violated.is_empty()
                || verdict
                    .constraints_violated
                    .iter()
                    .all(|c| c.severity == Severity::Warning)
        );
    }

    #[tokio::test]
    async fn test_code_quality_checks() {
        let ethos = EthosAgent::default();
        let mut manifest = A2AManifest::new("Test query".to_string());
        manifest.set_logos_result("fn main() { let x = unwrap(); }".to_string(), 0.9);

        let input = AgentInput::new(manifest);
        let verdict = ethos.verify(&input).await.unwrap();

        // Should have at least a warning for unwrap without error handling
        assert!(!verdict.constraints_violated.is_empty());
    }

    #[tokio::test]
    async fn test_overconfident_claims() {
        let ethos = EthosAgent::default();
        let mut manifest = A2AManifest::new("Test query".to_string());
        manifest.set_logos_result(
            "This will definitely succeed 100% of the time".to_string(),
            0.9,
        );

        let input = AgentInput::new(manifest);
        let verdict = ethos.verify(&input).await.unwrap();

        // Should have warning about overconfident claims
        assert!(verdict
            .constraints_violated
            .iter()
            .any(|c| matches!(c.constraint_type, ConstraintType::Fact)));
    }

    #[tokio::test]
    async fn test_agent_trait() {
        let config = AgentConfig {
            model: "phi-3-mini-4k".to_string(),
            ..Default::default()
        };
        let mut agent = EthosAgent::new(config);
        agent.initialize().await.unwrap();

        assert_eq!(agent.name(), "Ethos");
        assert!(agent.is_ready());
        assert_eq!(agent.model(), "phi-3-mini-4k");
    }
}
