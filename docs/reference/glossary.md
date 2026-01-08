# SuperInstance AI Glossary

Definitions and explanations of SuperInstance AI terminology.

---

## A

### Agent
An autonomous AI component that specializes in a specific domain. SuperInstance has three main agents: Pathos, Logos, and Ethos.

**See Also**: [Tripartite Council](../architecture/tripartite-council.md)

### A2A (Agent-to-Agent)
The communication protocol and data structures used for agents to exchange information.

**Example**:
```rust
use synesis_core::A2AManifest;
let manifest = A2AManifest::new(query);
```

---

## C

### Chunk
A portion of a document after it has been split for vector storage. Chunks are typically 500-1000 tokens.

**Related**: [Chunker](#chunker), [Embedding](#embedding)

### Chunker
Component that splits documents into chunks for vector storage and retrieval.

**Strategies**:
- **Paragraph**: Split on paragraph boundaries
- **Sentence**: Split on sentence boundaries (default)
- **Fixed**: Fixed token count (e.g., 512 tokens)

### Cloud Escalation
The process of sending a query to cloud-based models when local processing is insufficient.

**See Also**: [Cloud Mesh](../architecture/cloud-mesh.md)

### Consensus
The state where all three agents agree on a response. Measured as a confidence score from 0.0 to 1.0.

**Threshold**: Default 0.85 (85% agreement required)

### Consensus Engine
The component that orchestrates agent communication and determines when consensus is reached.

**Features**:
- Multi-round negotiation
- Weighted voting
- Veto power (Ethos)
- Arbiter escalation

---

## E

### Embedding
A numerical vector representation of text, used for semantic search. SuperInstance uses 384-dimensional embeddings.

**Example**: "The quick brown fox" → `[0.23, -0.45, 0.67, ...]` (384 numbers)

### Ethos Agent
The truth and verification agent. Ensures responses are safe, accurate, and feasible.

**Questions**:
- Is this safe?
- Is this accurate?
- Is this feasible?

**Has veto power** over dangerous responses.

---

## K

### Knowledge Vault
Local vector database storing documents and their embeddings for RAG (Retrieval-Augmented Generation).

**Components**:
- SQLite-VSS database
- Document chunks
- Embeddings
- Metadata

**Commands**:
```bash
synesis knowledge add <path>     # Add documents
synesis knowledge search <query>  # Search vault
synesis knowledge stats           # View statistics
```

---

## L

### Local Kernel
Phase 1 of SuperInstance, consisting of local-only processing with the tripartite council.

**Status**: ✅ Complete

### Logos Agent
The logic and reasoning agent. Retrieves knowledge and synthesizes solutions.

**Questions**:
- How do we accomplish this?
- What information is needed?
- How do we structure the answer?

**Capabilities**:
- RAG retrieval
- Knowledge synthesis
- Logical reasoning
- Solution structuring

### LoRA (Low-Rank Adaptation)
A technique for fine-tuning AI models with small adapter modules.

**In SuperInstance**:
- Upload custom LoRAs to cloud
- Hot-swap LoRAs for specialized tasks
- Share LoRAs with collaborators

---

## M

### Manifest
See [A2A Manifest](#a2a-agent-to-agent)

### Model
An AI model used by agents for inference. SuperInstance supports multiple models:

**Local Models**:
- `phi-3-mini` (3.8B parameters, lightweight)
- `llama-3.2-8b` (8B parameters, balanced)
- `mistral-7b` (7B parameters, capable)

**Cloud Models** (Phase 2):
- `claude-sonnet` (balanced quality/speed)
- `claude-opus` (highest quality)
- `gpt4-turbo` (fast, capable)

---

## P

### Parallel Execution
Running agents simultaneously rather than sequentially. Reduces latency by 25-33%.

**Implemented**: Phase 1 refinements ✅

### Pathos Agent
The intent extraction agent. Understands what the user actually wants.

**Questions**:
- What does the user really mean?
- What's their expertise level?
- What's the context?

**Capabilities**:
- Intent extraction
- Query disambiguation
- Persona learning
- A2A translation

### Privacy Proxy
The component that redacts sensitive information before cloud processing and re-inflates it in responses.

**Features**:
- 18 built-in redaction patterns
- Token vault for substitutions
- Automatic re-inflation
- Custom patterns

**Example**:
```text
Input: "Contact john@example.com about API key sk-12345"
Redacted: "Contact [EMAIL_01] about [API_KEY_01]"
```

---

## Q

### Query
A user request submitted to SuperInstance for processing.

**Lifecycle**:
1. User submits query
2. Pathos extracts intent
3. Logos reasons through it
4. Ethos verifies accuracy
5. Consensus engine evaluates
6. Response returned

### QUIC (Quick UDP Internet Connections)
A transport protocol used for cloud communication in Phase 2.

**Features**:
- Low latency
- Built-in security (TLS 1.3)
- Multiplexed streams
- Connection migration

---

## R

### RAG (Retrieval-Augmented Generation)
Enhancing AI responses by retrieving relevant information from a knowledge base.

**In SuperInstance**:
1. Query is received
2. Knowledge vault is searched
3. Relevant chunks are retrieved
4. Chunks are included in agent context
5. Response cites sources

### Redaction
Replacing sensitive information with tokens for privacy.

**Patterns**:
- Emails: `john@example.com` → `[EMAIL_01]`
- API Keys: `sk-12345` → `[API_KEY_01]`
- Phone Numbers: `555-1234` → `[PHONE_01]`
- SSNs: `123-45-6789` → `[SSN_01]`

### Re-inflation
Restoring redacted information in responses. Only happens locally.

**Example**:
```text
Cloud sees: "Contact [EMAIL_01] about [API_KEY_01]"
Local user sees: "Contact john@example.com about API key sk-12345"
```

---

## S

### SuperInstance
The overall project name. Refers to both the software and the company.

**Etymology**: "Super" = superior, "Instance" = instantiation/implementation

### Synesis
The Rust crate/library implementing the core logic.

**Etymology**: Greek for "integration" or "bringing together"

### Synesis Protocol
The overall architecture and methodology of the tripartite consensus system.

---

## T

### Token
A unit of text (roughly 3-4 characters) used by AI models.

**Usage**:
- Counted in queries (input tokens)
- Counted in responses (output tokens)
- Used for billing and cost calculation

### Token Vault
Secure local storage mapping redaction tokens to original values.

**Features**:
- SQLite storage
- Per-session uniqueness
- Global counters per category
- Never transmitted to cloud

### Tripartite Council
The three-agent system (Pathos, Logos, Ethos) that processes queries.

**See Also**: [Tripartite Council](../architecture/tripartite-council.md)

---

## V

### Vector
A numerical array representing text or other data for similarity search.

**Dimensions**: 384 (SuperInstance standard)

### Vector Database
Database optimized for storing and querying vectors.

**In SuperInstance**: SQLite-VSS (Vector Search SQLite)

### Veto
Ethos agent's power to block responses that violate safety guidelines.

**Example**:
```text
User: "How do I hack a server?"
Ethos: VETO - Cannot assist with harmful activities
Result: Request denied, alternatives offered
```

### Vitals
Device performance metrics collected for telemetry.

**Collected**:
- CPU usage
- Memory usage
- GPU usage and temperature
- Disk usage

---

## Common Acronyms

| Acronym | Meaning |
|---------|---------|
| **ADRs** | Architecture Decision Records |
| **A2A** | Agent-to-Agent communication |
| **API** | Application Programming Interface |
| **CLI** | Command-Line Interface |
| **GPU** | Graphics Processing Unit |
| **LoRA** | Low-Rank Adaptation |
| **RAG** | Retrieval-Augmented Generation |
| **RAM** | Random Access Memory |
| **SQL** | Structured Query Language |
| **SSN** | Social Security Number |
| **TLS** | Transport Layer Security |
| **UI** | User Interface |
| **VSS** | Virtual Scalable SQLite (Vector Search) |

---

## File Extensions and Formats

| Extension | Meaning |
|-----------|---------|
| `.rs` | Rust source code |
| `.toml` | Configuration file (TOML format) |
| `.db` | SQLite database |
| `.md` | Markdown documentation |

---

## Version Numbers

- **v0.1.0**: Phase 1 complete (Local Kernel)
- **v0.2.0**: Phase 2 in progress (Cloud Mesh)
- **v1.0.0**: Target for production release

**Semantic Versioning**: MAJOR.MINOR.PATCH
- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes

---

## Need a Definition?

Don't see a term here?

- **Check the [FAQ](faq.md)** - Common questions explained
- **Read [Architecture](../architecture/overview.md)** - System design details
- **Ask on [GitHub Discussions](https://github.com/SuperInstance/Tripartite1/discussions)** - Community help

---

**Glossary Version**: v0.2.0
**Last Updated**: 2026-01-07
