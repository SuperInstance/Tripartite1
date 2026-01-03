# Ethos Agent - Worker Onboarding

> **Agent Type**: Truth Verification
> **Domain**: Ground truth, safety, and hardware constraints
> **Primary Question**: "Is this safe, accurate, and feasible?"

---

## Your Role

You are building the **Ethos Agent**, the guardian of the system. Your implementation must:

1. Fact-check claims against known sources
2. Verify solutions against hardware constraints
3. Flag safety concerns before execution
4. Provide actionable feedback when rejecting solutions

## Why Ethos Matters

Ethos is the **trust guardian**. Without Ethos:
- Hallucinations reach users
- Code breaks hardware
- Dangerous operations execute
- The system loses credibility

**Ethos has veto power**. If Ethos says no, the response doesn't ship.

---

## Technical Specification

### Input
- Solution from Logos
- A2A Manifest from Pathos
- Hardware manifest
- Access to fact-checking sources

### Output: EthosResult

```typescript
interface EthosResult {
  verdict: 'approved' | 'needs_revision' | 'veto';
  confidence: number;            // 0-1
  constraints: Constraint[];     // Issues found
  feedback: string;              // Guidance for retry
}

interface Constraint {
  type: 'fact' | 'hardware' | 'safety' | 'quality';
  severity: 'warning' | 'error' | 'critical';
  description: string;
  source?: string;               // Evidence for the constraint
  suggestion?: string;           // How to fix it
}
```

---

## Implementation Guide

### Core Function

```rust
// cli/src/agents/ethos.rs

pub struct EthosAgent {
    model: LocalModel,           // phi-3-mini-4k (fast verification)
    hardware: HardwareManifest,
    fact_checker: FactChecker,
    safety_scanner: SafetyScanner,
}

impl EthosAgent {
    pub async fn verify(
        &self,
        solution: &str,
        manifest: &A2AManifest,
    ) -> Result<EthosResult> {
        let mut constraints = Vec::new();
        
        // 1. Hardware verification (if needed)
        if manifest.verification_scope.check_hardware {
            constraints.extend(self.check_hardware(solution).await?);
        }
        
        // 2. Fact verification (if needed)
        if manifest.verification_scope.check_facts {
            constraints.extend(self.check_facts(solution).await?);
        }
        
        // 3. Safety verification (always)
        if manifest.verification_scope.check_safety {
            constraints.extend(self.check_safety(solution).await?);
        }
        
        // 4. Quality check (code-specific)
        if manifest.intent.query_type == "generate" {
            constraints.extend(self.check_code_quality(solution).await?);
        }
        
        // 5. Determine verdict
        let verdict = self.determine_verdict(&constraints);
        
        // 6. Generate feedback
        let feedback = self.generate_feedback(&constraints, &verdict);
        
        // 7. Calculate confidence
        let confidence = self.calculate_confidence(&constraints);
        
        Ok(EthosResult {
            verdict,
            confidence,
            constraints,
            feedback,
        })
    }
    
    fn determine_verdict(&self, constraints: &[Constraint]) -> Verdict {
        // Any critical constraint = veto
        if constraints.iter().any(|c| c.severity == Severity::Critical) {
            return Verdict::Veto;
        }
        
        // Any error = needs revision
        if constraints.iter().any(|c| c.severity == Severity::Error) {
            return Verdict::NeedsRevision;
        }
        
        // Only warnings = approved (with notes)
        Verdict::Approved
    }
}
```

### Hardware Verification

```rust
impl EthosAgent {
    async fn check_hardware(&self, solution: &str) -> Result<Vec<Constraint>> {
        let mut constraints = Vec::new();
        
        // Check memory requirements
        if let Some(mem_req) = self.extract_memory_requirement(solution) {
            if mem_req > self.hardware.limits.max_vram_mb {
                constraints.push(Constraint {
                    type_: ConstraintType::Hardware,
                    severity: Severity::Error,
                    description: format!(
                        "Solution requires {}MB VRAM but device only has {}MB",
                        mem_req, self.hardware.limits.max_vram_mb
                    ),
                    suggestion: Some("Use a smaller model or enable quantization".to_string()),
                });
            }
        }
        
        // Check power mode compatibility
        if solution.contains("max_power") || solution.contains("POWER_MODE=MAX") {
            if self.hardware.limits.max_power_watts < 25 {
                constraints.push(Constraint {
                    type_: ConstraintType::Hardware,
                    severity: Severity::Warning,
                    description: "Max power mode requested but device has low power limit".to_string(),
                    suggestion: Some("Consider using 15W power mode instead".to_string()),
                });
            }
        }
        
        // Check thermal implications
        let current_temp = self.get_current_temp().await?;
        if current_temp > 75.0 && solution.contains("intensive") {
            constraints.push(Constraint {
                type_: ConstraintType::Hardware,
                severity: Severity::Warning,
                description: format!("Device is warm ({}°C), intensive task may throttle", current_temp),
                suggestion: Some("Consider waiting for cooldown or reducing workload".to_string()),
            });
        }
        
        Ok(constraints)
    }
}
```

### Fact Checking

```rust
impl EthosAgent {
    async fn check_facts(&self, solution: &str) -> Result<Vec<Constraint>> {
        let mut constraints = Vec::new();
        
        // Extract claims from solution
        let claims = self.extract_claims(solution).await?;
        
        for claim in claims {
            // Check against local knowledge base first
            if let Some(local_verdict) = self.fact_checker.check_local(&claim).await? {
                if !local_verdict.verified {
                    constraints.push(Constraint {
                        type_: ConstraintType::Fact,
                        severity: Severity::Warning,
                        description: format!("Unverified claim: {}", claim.text),
                        source: local_verdict.conflicting_source,
                        suggestion: Some("Consider adding a source or hedging language".to_string()),
                    });
                }
            }
            
            // For critical claims, check external sources
            if claim.importance == Importance::High {
                if let Some(external) = self.fact_checker.check_external(&claim).await? {
                    if !external.verified {
                        constraints.push(Constraint {
                            type_: ConstraintType::Fact,
                            severity: Severity::Error,
                            description: format!("Contradicted by external source: {}", claim.text),
                            source: Some(external.source_url),
                            suggestion: Some("Revise based on current information".to_string()),
                        });
                    }
                }
            }
        }
        
        Ok(constraints)
    }
    
    async fn extract_claims(&self, solution: &str) -> Result<Vec<Claim>> {
        // Use model to identify factual claims
        let prompt = format!(r#"
Identify factual claims in this text that could be true or false.
Only include specific, verifiable claims, not opinions.

Text: {}

Respond with JSON array of claims:
[{{"text": "...", "importance": "high|medium|low"}}]
"#, solution);
        
        let response = self.model.generate(&prompt).await?;
        let claims: Vec<Claim> = serde_json::from_str(&response)?;
        
        Ok(claims)
    }
}
```

### Safety Verification

```rust
impl EthosAgent {
    async fn check_safety(&self, solution: &str) -> Result<Vec<Constraint>> {
        let mut constraints = Vec::new();
        
        // File system dangers
        let dangerous_patterns = [
            (r"rm\s+-rf\s+/", "Recursive root deletion"),
            (r"rm\s+-rf\s+\$HOME", "Home directory deletion"),
            (r"chmod\s+777", "Overly permissive file permissions"),
            (r"curl.*\|\s*sh", "Piping URL to shell"),
            (r"eval\(.*input", "Eval with user input"),
        ];
        
        for (pattern, description) in &dangerous_patterns {
            let re = Regex::new(pattern)?;
            if re.is_match(solution) {
                constraints.push(Constraint {
                    type_: ConstraintType::Safety,
                    severity: Severity::Critical, // Critical = veto
                    description: format!("Dangerous pattern detected: {}", description),
                    suggestion: Some("This operation is too risky to execute automatically".to_string()),
                });
            }
        }
        
        // Network dangers
        if solution.contains("0.0.0.0") && solution.contains("bind") {
            constraints.push(Constraint {
                type_: ConstraintType::Safety,
                severity: Severity::Warning,
                description: "Binding to all interfaces exposes service to network".to_string(),
                suggestion: Some("Consider binding to 127.0.0.1 for local-only access".to_string()),
            });
        }
        
        // Credential exposure
        if self.safety_scanner.contains_secrets(solution) {
            constraints.push(Constraint {
                type_: ConstraintType::Safety,
                severity: Severity::Error,
                description: "Solution appears to contain hardcoded credentials".to_string(),
                suggestion: Some("Use environment variables or a secrets manager".to_string()),
            });
        }
        
        Ok(constraints)
    }
}
```

### Feedback Generation

```rust
impl EthosAgent {
    fn generate_feedback(&self, constraints: &[Constraint], verdict: &Verdict) -> String {
        if constraints.is_empty() {
            return "Solution verified. No issues found.".to_string();
        }
        
        let mut feedback = String::new();
        
        match verdict {
            Verdict::Veto => {
                feedback.push_str("BLOCKED: Critical issues prevent this solution.\n\n");
            }
            Verdict::NeedsRevision => {
                feedback.push_str("REVISION NEEDED: Please address the following:\n\n");
            }
            Verdict::Approved => {
                feedback.push_str("APPROVED with notes:\n\n");
            }
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
}
```

---

## Testing Requirements

### Unit Tests

```rust
#[test]
fn test_dangerous_command_veto() {
    let ethos = EthosAgent::new_test();
    let manifest = A2AManifest::default();
    
    let result = ethos.verify("Run: rm -rf /", &manifest).await.unwrap();
    
    assert_eq!(result.verdict, Verdict::Veto);
    assert!(result.constraints.iter().any(|c| c.severity == Severity::Critical));
}

#[test]
fn test_hardware_constraint() {
    let ethos = EthosAgent::new_test_with_hardware(HardwareManifest {
        limits: Limits { max_vram_mb: 8192, .. },
        ..
    });
    
    let manifest = A2AManifest {
        verification_scope: VerificationScope { check_hardware: true, .. },
        ..
    };
    
    let solution = "Load the 70B model (requires 40GB VRAM)";
    let result = ethos.verify(solution, &manifest).await.unwrap();
    
    assert_eq!(result.verdict, Verdict::NeedsRevision);
    assert!(result.constraints.iter().any(|c| c.type_ == ConstraintType::Hardware));
}

#[test]
fn test_clean_solution_approved() {
    let ethos = EthosAgent::new_test();
    let manifest = A2AManifest::default();
    
    let solution = "def hello(): return 'Hello, World!'";
    let result = ethos.verify(solution, &manifest).await.unwrap();
    
    assert_eq!(result.verdict, Verdict::Approved);
    assert!(result.constraints.is_empty());
}
```

---

## Performance Targets

| Metric | Target |
|--------|--------|
| Hardware check | <50ms |
| Fact extraction | <300ms |
| Safety scan | <100ms |
| Total verification | <500ms |

---

## Veto Scenarios (Always Block)

1. **Recursive deletion of system directories**
2. **Credential exposure in output**
3. **Code execution from untrusted URLs**
4. **Modification of system files without confirmation**
5. **Operations exceeding hardware thermal limits**

---

*See also: [PATHOS_AGENT.md](./PATHOS_AGENT.md), [LOGOS_AGENT.md](./LOGOS_AGENT.md)*
