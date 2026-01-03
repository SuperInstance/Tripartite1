# Pathos Agent - Worker Onboarding

> **Agent Type**: Intent Extraction
> **Domain**: Understanding what the user actually wants
> **Primary Question**: "What is the true goal here?"

---

## Your Role

You are building the **Pathos Agent**, the front-line translator between human intent and machine action. Your implementation must:

1. Parse ambiguous human prompts into clear, structured intent
2. Identify implicit constraints the user hasn't stated
3. Detect the user's expertise level and communication preferences
4. Output a valid A2A Manifest that guides the other agents

## Why Pathos Matters

Pathos is the **efficiency guardian**. By accurately decoding intent:
- Logos (expensive reasoning) only runs when truly needed
- Cloud escalations are minimized (saving cost)
- Users don't have to repeat themselves

**Bad Pathos**: "Tell me about Python" → Logos generates 10,000 token essay
**Good Pathos**: "Tell me about Python" → Detects: user is a beginner, wants quick overview, not a comprehensive guide → Logos generates 200 token intro

---

## Technical Specification

### Input
- Raw user prompt (text, potentially with images)
- Project context (file structure, recent interactions)
- Hardware manifest (what the local device can do)

### Output: A2A Manifest

```typescript
interface A2AManifest {
  id: string;                      // UUID
  timestamp: number;               // Unix milliseconds
  
  intent: {
    telos: string;                 // The actual goal in clear language
    query_type: 'generate' | 'analyze' | 'transform' | 'verify' | 'explain';
    constraints: string[];         // Explicit + inferred limits
    priority: 'speed' | 'quality' | 'cost';
  };
  
  persona: {
    expertise_level: 'novice' | 'intermediate' | 'expert';
    communication_style: 'formal' | 'casual' | 'technical';
    known_preferences: string[];   // From interaction history
  };
  
  context_hints: {
    relevant_files: string[];      // Files Logos should check
    related_queries: string[];     // Similar past questions
    domain: string;                // e.g., "web development", "data science"
  };
  
  verification_scope: {
    check_facts: boolean;
    check_hardware: boolean;
    check_safety: boolean;
  };
}
```

---

## Implementation Guide

### Core Function

```rust
// cli/src/agents/pathos.rs

pub struct PathosAgent {
    model: LocalModel,           // phi-3-mini-4k or similar
    persona_store: PersonaStore, // SQLite table of user patterns
    context_loader: ContextLoader,
}

impl PathosAgent {
    pub async fn process(
        &self,
        prompt: &str,
        project_ctx: &ProjectContext,
    ) -> Result<PathosResult> {
        // 1. Load user history for persona detection
        let recent_interactions = self.persona_store.get_recent(10)?;
        
        // 2. Build context window
        let context = self.build_context(prompt, &recent_interactions, project_ctx);
        
        // 3. Run intent extraction
        let system_prompt = self.get_system_prompt();
        let response = self.model.generate(&system_prompt, &context).await?;
        
        // 4. Parse and validate manifest
        let manifest: A2AManifest = serde_json::from_str(&response)
            .map_err(|e| self.handle_parse_error(e, &response))?;
        
        // 5. Calculate confidence
        let confidence = self.calculate_confidence(&manifest, prompt);
        
        Ok(PathosResult { manifest, confidence })
    }
    
    fn get_system_prompt(&self) -> String {
        r#"
You are Pathos, the Intent Agent in the SuperInstance system.

Your mission: Transform messy human prompts into crystal-clear intent.

## Your Process

1. **Identify the Telos** (true goal)
   - What does the user actually want to achieve?
   - Read between the lines - what aren't they saying?
   
2. **Detect Constraints**
   - Explicit: "in Python", "under 100 lines", "for a beginner"
   - Implicit: Context suggests they're in a hurry, on mobile, etc.
   
3. **Profile the Persona**
   - Expertise: Are they asking what an expert or novice would ask?
   - Style: Do they want formal documentation or casual explanation?
   
4. **Set Verification Scope**
   - check_facts: True if claims will be made
   - check_hardware: True if code/actions will run locally
   - check_safety: True if code execution or system changes

## Output Format

Respond ONLY with a valid JSON object matching the A2AManifest schema.
No explanations, no markdown, just JSON.

## Examples

Input: "how do i center a div"
Output: {
  "intent": {
    "telos": "Center a div element using CSS",
    "query_type": "explain",
    "constraints": ["CSS-based solution", "common/modern approach"],
    "priority": "speed"
  },
  "persona": {
    "expertise_level": "novice",
    "communication_style": "casual"
  },
  "verification_scope": {
    "check_facts": false,
    "check_hardware": false,
    "check_safety": false
  }
}

Input: "Review my authentication implementation for security vulnerabilities"
Output: {
  "intent": {
    "telos": "Security audit of authentication code",
    "query_type": "analyze",
    "constraints": ["focus on security", "actionable findings"],
    "priority": "quality"
  },
  "persona": {
    "expertise_level": "intermediate",
    "communication_style": "technical"
  },
  "verification_scope": {
    "check_facts": true,
    "check_hardware": false,
    "check_safety": true
  }
}
        "#.to_string()
    }
}
```

### Confidence Calculation

```rust
fn calculate_confidence(&self, manifest: &A2AManifest, original_prompt: &str) -> f32 {
    let mut confidence = 1.0;
    
    // Penalty for very short prompts (ambiguous)
    if original_prompt.split_whitespace().count() < 5 {
        confidence -= 0.15;
    }
    
    // Penalty for missing constraints when they seem needed
    if manifest.intent.query_type == "generate" && manifest.intent.constraints.is_empty() {
        confidence -= 0.10;
    }
    
    // Bonus for clear domain detection
    if !manifest.context_hints.domain.is_empty() {
        confidence += 0.05;
    }
    
    // Penalty for "catch-all" telos
    if manifest.intent.telos.len() > 200 {
        confidence -= 0.10; // Likely just echoing the prompt
    }
    
    confidence.clamp(0.0, 1.0)
}
```

---

## Testing Requirements

### Unit Tests

```rust
#[test]
fn test_simple_question() {
    let pathos = PathosAgent::new_test();
    let result = pathos.process("What is Rust?", &empty_context()).await.unwrap();
    
    assert_eq!(result.manifest.intent.query_type, "explain");
    assert_eq!(result.manifest.persona.expertise_level, "novice");
    assert!(result.confidence > 0.7);
}

#[test]
fn test_expert_request() {
    let pathos = PathosAgent::new_test();
    let result = pathos.process(
        "Implement a lock-free MPSC queue using compare-and-swap",
        &empty_context()
    ).await.unwrap();
    
    assert_eq!(result.manifest.intent.query_type, "generate");
    assert_eq!(result.manifest.persona.expertise_level, "expert");
    assert!(result.manifest.intent.constraints.iter().any(|c| c.contains("lock-free")));
}

#[test]
fn test_safety_detection() {
    let pathos = PathosAgent::new_test();
    let result = pathos.process(
        "Write a script to delete all files in /tmp",
        &empty_context()
    ).await.unwrap();
    
    assert!(result.manifest.verification_scope.check_safety);
    assert!(result.manifest.verification_scope.check_hardware);
}
```

### Integration Tests

```rust
#[test]
fn test_persona_learning() {
    let pathos = PathosAgent::new_test();
    
    // Simulate user history of expert-level questions
    pathos.persona_store.add_interaction(expert_interaction_1());
    pathos.persona_store.add_interaction(expert_interaction_2());
    
    // New ambiguous question should assume expert
    let result = pathos.process("How do I optimize this?", &empty_context()).await.unwrap();
    
    assert_eq!(result.manifest.persona.expertise_level, "expert");
}
```

---

## Common Pitfalls

### Pitfall 1: Echo-Pathos
**Problem**: Just repeating the prompt as the telos
**Solution**: Force transformation - telos should never be >50% similar to input

### Pitfall 2: Over-Constraining
**Problem**: Adding constraints the user didn't imply
**Solution**: Distinguish "likely constraints" (include) from "possible constraints" (omit)

### Pitfall 3: Static Persona
**Problem**: Not adapting to conversation flow
**Solution**: Re-evaluate persona every 3 turns based on recent messages

### Pitfall 4: JSON Failures
**Problem**: Model sometimes outputs explanation instead of JSON
**Solution**: Strict system prompt + retry logic with "JSON only" reminder

---

## Performance Targets

| Metric | Target |
|--------|--------|
| Processing time | <500ms |
| JSON parse success rate | >99% |
| Intent accuracy (human eval) | >85% |
| Memory footprint | <2GB |

---

## Handoff to Logos

After Pathos completes, the A2A Manifest is passed to Logos. Ensure:
- `telos` is actionable (Logos can start working immediately)
- `context_hints.relevant_files` are valid paths
- `constraints` are specific enough to guide solution

---

*See also: [LOGOS_AGENT.md](./LOGOS_AGENT.md), [ETHOS_AGENT.md](./ETHOS_AGENT.md)*
