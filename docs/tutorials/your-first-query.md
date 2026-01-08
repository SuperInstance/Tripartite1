# Your First Query: Understanding the Tripartite System

**Time**: 15 minutes
**Difficulty**: Beginner
**Prerequisites**:
- Completed [Getting Started](getting-started.md)
- SuperInstance AI initialized

---

## What You'll Learn

By the end of this tutorial, you will understand:
- ✅ What happens during a query
- ✅ How the three-agent system works
- ✅ How to interpret query results
- ✅ How consensus is reached

---

## The Tripartite System

SuperInstance uses a **three-agent consensus system**:

```
┌─────────────────────────────────────────┐
│           Your Query                    │
└─────────────────┬───────────────────────┘
                  │
        ┌─────────┴──────────┐
        │                    │
    ┌───▼────┐          ┌────▼───┐
    │ Pathos │          │  Query │
    │(Intent)│          │  Input │
    └───┬────┘          └────────┘
        │
    ┌───┴──────────────┬─────────────┐
    │                  │             │
┌───▼────┐      ┌─────▼───┐   ┌────▼────┐
│ Pathos │      │  Logos  │   │  Ethos  │
│Intent  │      │  Logic  │   │  Truth  │
└───┬────┘      └─────┬───┘   └────┬────┘
    │                 │             │
    └────────┬────────┴─────────────┘
             │
      ┌──────▼──────┐
      │  Consensus  │
      │   Engine    │
      └──────┬──────┘
             │
      ┌──────▼─────────┐
      │  Your Response │
      └────────────────┘
```

### The Three Agents

| Agent | Role | Primary Question |
|-------|------|------------------|
| **Pathos** | Intent Extraction | "What does the user actually want?" |
| **Logos** | Logical Reasoning | "How do we accomplish this?" |
| **Ethos** | Truth Verification | "Is this safe, accurate, and feasible?" |

---

## Running a Query

Let's run a query and watch what happens:

```bash
synesis ask "Explain how vector databases work"
```

### What You'll See

```text
🤔 Pathos (Intent): User wants technical explanation of vector database technology
🧠 Logos (Logic): Retrieving knowledge about vector databases, embeddings, similarity search...
✅ Ethos (Truth): Verifying technical accuracy and completeness...

✅ Consensus reached (0.91 confidence)

Vector databases are specialized databases designed to store and query
high-dimensional vectors efficiently...

[Detailed explanation continues...]

---
Agents: 3/3 agreed | Rounds: 1 | Confidence: 91% | Time: 2.8s
```

### Breakdown: What Just Happened

#### 1. Pathos - Intent Extraction (🤔)

```
🤔 Pathos (Intent): User wants technical explanation of vector database technology
```

**Pathos analyzed:**
- Your query: "Explain how vector databases work"
- Your intent: Educational/technical explanation
- Expertise level: Technical (uses technical terms)
- Context: Learning about databases

**Pathos output** is passed to other agents to frame their work.

#### 2. Logos - Logical Reasoning (🧠)

```
🧠 Logos (Logic): Retrieving knowledge about vector databases, embeddings, similarity search...
```

**Logos did:**
- Retrieved relevant knowledge from:
  - Its training data
  - Your knowledge vault (if indexed)
  - Technical documentation
- Synthesized a comprehensive explanation
- Structured the answer logically:
  1. What are vector databases?
  2. How do they work?
  3. Key concepts (embeddings, similarity search)
  4. Use cases

#### 3. Ethos - Truth Verification (✅)

```
✅ Ethos (Truth): Verifying technical accuracy and completeness...
```

**Ethos checked:**
- **Accuracy**: Are the technical details correct?
- **Completeness**: Is anything important missing?
- **Safety**: Is this safe to explain? (no harmful content)
- **Feasibility**: Is this within our capabilities? (yes)

#### 4. Consensus Engine

```
✅ Consensus reached (0.91 confidence)
```

The consensus engine:
1. **Collected outputs** from all three agents
2. **Evaluated agreement**: All agents agreed (3/3)
3. **Calculated confidence**: 91% agreement
4. **Returned response**: Since confidence ≥ threshold (85%), response delivered

---

## Understanding Confidence Scores

Confidence indicates how much the agents agreed:

| Confidence | Meaning | Action |
|------------|---------|--------|
| **≥ 90%** | Strong agreement | Immediate response |
| **85-90%** | Good agreement | Response delivered |
| **70-85%** | Moderate agreement | Revision round (if within limits) |
| **< 70%** | Low agreement | Arbiter escalation or clarification request |

### Example: Low Consensus

```bash
synesis ask "How do I hack into a server?"
```

```text
🤔 Pathos: Potentially malicious intent detected
🧠 Logos: Query unclear - need clarification
✅ Ethos: VETO - Request violates safety guidelines

❌ Consensus NOT reached (agent veto)
❌ Ethos vetoed: Cannot assist with harmful activities

Would you like help with:
- Authorized server administration?
- Security testing on your own systems?
- Learning about server security?
```

---

## Consensus Rounds

If initial consensus is low, the system enters revision rounds:

### Round 1
- Each agent provides initial response
- Consensus: 72% (below threshold)

### Round 2 (Revision)
- Agents see each other's work
- Pathos: "User meant security testing, not hacking"
- Logos: Adjusts explanation to focus on defensive security
- Ethos: "Safe to explain defensive concepts"
- Consensus: 88% ✓

### Result

```text
✅ Consensus reached (Round 2/3, 88% confidence)

I can explain server security concepts for defensive purposes:

[Defensive security explanation...]

Note: If you meant penetration testing, please clarify you own the system.
```

---

## Interpreting Query Results

### Full Result Breakdown

```text
🤔 Pathos (Intent): User wants to understand Rust ownership
🧠 Logos (Logic): Explaining ownership with code examples...
✅ Ethos (Truth): Verifying accuracy of Rust concepts...

✅ Consensus reached (0.93 confidence)

Rust's ownership system ensures memory safety without garbage collection.
The key rules are:

1. Each value has an owner
2. There can only be one owner at a time
3. When the owner goes out of scope, the value is dropped

[Detailed explanation with examples...]

---
Query Metadata:
  Confidence: 93%
  Agents: 3/3 agreed
  Rounds: 1
  Time: 2.1s
  Pathos: intent extraction (0.95s)
  Logos: reasoning & synthesis (0.8s)
  Ethos: verification (0.35s)
```

### What Each Metric Means

| Metric | Meaning | Good Range |
|--------|---------|------------|
| **Confidence** | Agent agreement level | 85-100% |
| **Agents** | How many agents agreed | 3/3 ideal |
| **Rounds** | Consensus iterations | 1-3 ideal |
| **Time** | Total processing | 2-5s typical |
| **Agent Times** | Per-agent breakdown | Varies by complexity |

---

## Advanced: Seeing Agent Details

Enable verbose output to see more details:

```bash
synesis ask "Explain closures in Rust" --verbose
```

```text
=== PATHOS (Intent Agent) ===
Input: "Explain closures in Rust"
Intent: Educational request for programming concept
Expertise Level: Intermediate
Technical Domain: Rust programming
Framing: Provide clear explanation with examples

Processing Time: 0.87s

=== LOGOS (Logic Agent) ===
Knowledge Retrieved:
- Rust documentation on closures
- Local code examples (3 found)
- General programming concepts

Reasoning Process:
1. Define what closures are
2. Explain closure syntax
3. Show capture mechanisms
4. Provide practical examples
5. Compare with other languages

Processing Time: 1.2s

=== ETHOS (Truth Agent) ===
Verification Checks:
✅ Technical accuracy: All Rust syntax correct
✅ Completeness: Covers key concepts
✅ Safety: No harmful content
✅ Feasibility: Within capabilities

Processing Time: 0.42s

=== CONSENSUS ENGINE ===
Agent Votes:
- Pathos: 0.94 (confidence)
- Logos: 0.91 (confidence)
- Ethos: 0.96 (confidence)

Weighted Consensus: 0.93
Threshold: 0.85
Result: PASS ✓

=== RESPONSE ===
[Full response about closures...]

---
Total Time: 2.49s
```

---

## Troubleshooting

### Issue: "Consensus not reached after 3 rounds"

**Cause**: Agents couldn't agree on interpretation or answer

**Solutions**:
1. **Rephrase your query** - Be more specific
2. **Provide context** - Add more details
3. **Split complex queries** - Break into multiple questions

```bash
# Too vague
synesis ask "How do I fix it?"

# Better
synesis ask "How do I fix a Rust borrow error in closures?"

# Even better (with context)
synesis ask "I have a closure that captures a reference but gets 'borrowed value does not live long enough'. How do I fix this?"
```

### Issue: "Low confidence (< 85%)"

**Cause**: Agents uncertain about interpretation or accuracy

**Solutions**:
1. Add more context to your query
2. Check if query is ambiguous
3. Consider if topic is outside system's knowledge

### Issue: "Slow response (> 10s)"

**Causes**:
- First query (model loading)
- Complex topic requiring reasoning
- Large knowledge vault search

**Solutions**:
- First query is always slower (caching after)
- Use `--no-rag` to skip knowledge search for simple queries
- Consider upgrading hardware

---

## What's Next?

Congratulations! You now understand how SuperInstance processes queries.

### Continue Learning

1. **[Tripartite Consensus](tripartite-consensus.md)** - Deep dive into agent system
2. **[Knowledge Vault](knowledge-vault.md)** - Add your own documents
3. **[Privacy Basics](privacy-basics.md)** - Understand privacy features
4. **[Configuration Guide](../guides/configuration.md)** - Customize behavior

### Try These Queries

Test your understanding with these queries:

```bash
# Simple factual query
synesis ask "What is the capital of Japan?"

# Technical explanation
synesis ask "How does HashMap work in Rust?"

# Code-related (if you have code indexed)
synesis ask "How does the authentication work in my codebase?"

# Complex comparison
synesis ask "Compare Rust and Go memory management"

# Creative task
synesis ask "Write a haiku about artificial intelligence"
```

---

## Need Help?

- **Confused by consensus?** See [Glossary](../reference/glossary.md)
- **Having issues?** Check [Troubleshooting](../reference/troubleshooting.md)
- **Want to customize?** Read [Configuration Guide](../guides/configuration.md)

---

**Tutorial Version**: v0.2.0
**Last Updated**: 2026-01-07
