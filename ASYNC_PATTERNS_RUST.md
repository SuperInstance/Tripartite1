# Async/Await Patterns in SuperInstance AI

**Research Document**: Best practices and patterns for async Rust in this codebase
**Last Updated**: 2026-01-02
**Context**: Phase 1 - Local Kernel Implementation

---

## Table of Contents

1. [Overview](#overview)
2. [Core Principles](#core-principles)
3. [Patterns Used](#patterns-used)
4. [Anti-Patterns to Avoid](#anti-patterns-to-avoid)
5. [Specific Cases](#specific-cases)
6. [Best Practices](#best-practices)
7. [References](#references)

---

## Overview

SuperInstance AI uses Tokio as its async runtime throughout the codebase. All agents, database operations, and I/O-bound tasks use async/await for efficient concurrency.

### Why Async?

- **Concurrent Agent Execution**: Multiple agents can process queries simultaneously
- **Non-Blocking I/O**: Database and file operations don't block the thread
- **Efficient Resource Usage**: Single-threaded can handle many concurrent operations
- **Future-Proof**: Easy to add streaming and real-time features

---

## Core Principles

### 1. Runtime: Tokio

All async code uses `tokio` runtime:

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Async code here
    Ok(())
}
```

**Version**: 1.x (latest: 1.48.0)
**Features**: `full` (macros, rt-multi-thread, sync, time, io, net)

---

### 2. Send + Sync Boundaries

All async types must be `Send + Sync` to be safely shared across tasks:

```rust
pub trait Agent: Send + Sync {
    async fn process(&self, input: AgentInput) -> CoreResult<AgentOutput>;
}
```

**Why?** Tokio's multi-threaded executor can move tasks between threads.

---

## Patterns Used

### Pattern 1: Arc<Mutex<T>> for Shared State

**Problem**: Need to share mutable state across async tasks.

**Solution**: Use `Arc<Mutex<T>>` at the **application level**, not inside library structs.

```rust
// ✅ CORRECT: Application level
struct AppState {
    vault: Arc<Mutex<KnowledgeVault>>,
    models: Arc<Mutex<ModelRegistry>>,
}

// Usage
let mut guard = vault.lock().await;
guard.add_document(doc).await?;
drop(guard); // Release lock before await
```

**Why Application Level?**
- Library structs cannot hold `Arc<Mutex<T>>` without knowing runtime
- Application manages lock scope and lifetime
- Avoids holding locks across await points (see Anti-Pattern 1)

---

### Pattern 2: Scoped Locking

**Problem**: Need to hold lock temporarily, then do async work.

**Solution**: Use block scope to ensure lock is released before await:

```rust
// ✅ CORRECT: Scoped locking
{
    let mut guard = vault.lock().await;
    guard.add_document(doc)?;
} // Lock released here

async_function().await; // Safe: no lock held

// ❌ WRONG: Holding lock across await
let guard = vault.lock().await;
async_function().await; // DEADLOCK: guard still held!
drop(guard);
```

---

### Pattern 3: Channel-Based Communication

**Problem**: Async callback needs to modify shared state (FileWatcher issue).

**Solution**: Use channels to send messages to a dedicated task:

```rust
// ⚠️ FUTURE: Not yet implemented, but recommended pattern
use tokio::sync::mpsc;

let (tx, mut rx) = mpsc::channel(100);

// FileWatcher task
tokio::spawn(async move {
    while let Some(event) = rx.recv().await {
        // Process file event
    }
});

// Sender
tx.send(event).await?;
```

**Use Case**: FileWatcher auto-indexing (currently disabled due to lifetime issues).

---

### Pattern 4: Agent Trait with async_trait

**Problem**: Trait methods cannot be async (prior to Rust 1.75).

**Solution**: Use `async_trait` macro:

```rust
use async_trait::async_trait;

#[async_trait]
pub trait Agent: Send + Sync {
    async fn process(&self, input: AgentInput) -> CoreResult<AgentOutput>;
}
```

**Note**: Rust 1.75+ has native async traits, but `async_trait` is still widely used for compatibility.

---

## Anti-Patterns to Avoid

### Anti-Pattern 1: Holding MutexGuard Across Await

```rust
// ❌ WRONG: Will cause deadlock or panic
async fn bad_example(vault: Arc<Mutex<Vault>>) {
    let guard = vault.lock().await;
    let result = async_function().await; // Holding guard across await!
    drop(guard);
}

// ✅ CORRECT: Release lock before await
async fn good_example(vault: Arc<Mutex<Vault>>) {
    {
        let guard = vault.lock().await;
        // Do synchronous work only
    }
    let result = async_function().await; // Lock released
}
```

**Why?**
- `MutexGuard` is not `Send` (uses `RefCell` internally with rusqlite)
- Tokio may move task to another thread while guard is held
- Causes panic or undefined behavior

---

### Anti-Pattern 2: Blocking in Async Context

```rust
// ❌ WRONG: Blocks the executor
async fn bad_example() {
    std::thread::sleep(Duration::from_secs(1)); // Blocking!
}

// ✅ CORRECT: Use async sleep
async fn good_example() {
    tokio::time::sleep(Duration::from_secs(1)).await;
}
```

**Why?** Blocking calls block the entire thread, preventing other tasks from running.

---

### Anti-Pattern 3: References Across Await

```rust
// ❌ WRONG: Reference may become invalid
async fn bad_example(data: &Vec<u8>) {
    let slice = &data[0..10];
    async_function().await; // 'data' might be dropped!
    println!("{:?}", slice);
}

// ✅ CORRECT: Clone or owned data
async fn good_example(data: Vec<u8>) {
    let slice = data[0..10].to_owned();
    async_function().await;
    println!("{:?}", slice);
}
```

**Why?** Async functions may yield, allowing references to become invalid.

---

## Specific Cases

### Case 1: DocumentIndexer Lifetime Issue

**Problem**: `DocumentIndexer` holds `&'a KnowledgeVault`, incompatible with async callbacks.

```rust
// ❌ CURRENT DESIGN (doesn't work with async)
pub struct DocumentIndexer<'a> {
    vault: &'a KnowledgeVault,  // Lifetime bound
}

// Async callback cannot hold this reference
async fn on_file_change(path: PathBuf) {
    indexer.index(path).await?;  // Borrows indexer across await!
}
```

**Proposed Solution 1**: Arc<Mutex<KnowledgeVault>>
```rust
// ✅ RECOMMENDED: Owned state
pub struct DocumentIndexer {
    vault: Arc<Mutex<KnowledgeVault>>,
}

async fn on_file_change(path: PathBuf) {
    let mut guard = indexer.vault.lock().await;
    guard.index_document(&path)?;
    drop(guard);
}
```

**Proposed Solution 2**: Channel-based
```rust
// ✅ ALTERNATIVE: Message passing
pub struct DocumentIndexer {
    tx: mpsc::Sender<IndexRequest>,
}

async fn on_file_change(path: PathBuf) {
    tx.send(IndexRequest::Index(path)).await?;
}
```

**Status**: Not yet fixed (Phase 2).

---

### Case 2: ConsensusEngine Sequential Agent Execution

**Current**: Agents run sequentially (Pathos → Logos → Ethos).

```rust
async fn run(&mut self, prompt: &str) -> CoreResult<ConsensusOutcome> {
    let pathos_response = self.pathos.process(input).await?;
    let logos_response = self.logos.process(input).await?;
    let ethos_response = self.ethos.process(input).await?;
    // ...
}
```

**Optimization**: Run Pathos and Logos in parallel (they're independent):

```rust
// ✅ FUTURE: Parallel execution
async fn run(&mut self, prompt: &str) -> CoreResult<ConsensusOutcome> {
    let (pathos_response, logos_response) = tokio::join!(
        self.pathos.process(input.clone()),
        self.logos.process(input.clone())
    )?;

    let ethos_response = self.ethos.process(input).await?;
    // ...
}
```

**Why Not Currently?** Logos depends on Pathos's intent extraction.

---

### Case 3: SQLite in Async Context

**Problem**: `rusqlite` is synchronous (blocking).

**Current Solution**: Use scoped locking:

```rust
{
    let vault = self.vault.lock().await;
    vault.add_document(doc)?;  // Blocking call, but fast (< 10ms)
} // Lock released
```

**Future Solution**: Use `sqlx` (async SQLite):
```rust
// ✅ FUTURE: Async database
use sqlx::sqlite::SqlitePool;

async fn add_document(pool: &SqlitePool, doc: Document) -> Result<()> {
    sqlx::query("INSERT INTO documents ...")
        .execute(pool)
        .await?;
    Ok(())
}
```

**Trade-off**: `sqlx` requires compile-time query checking, more complex setup.

---

## Best Practices

### 1. Use `tokio::time::sleep` for Delays

```rust
// ✅ Use async sleep
tokio::time::sleep(Duration::from_secs(1)).await;

// ❌ Don't use std::thread::sleep
std::thread::sleep(Duration::from_secs(1));  // Blocks thread
```

---

### 2. Timeout Long-Running Operations

```rust
use tokio::time::{timeout, Duration};

let result = timeout(
    Duration::from_secs(30),
    agent.process(input)
).await??;  // Double ? for timeout + operation error
```

---

### 3. Use `join!` for Concurrent Independent Operations

```rust
// ✅ Run in parallel
let (result1, result2) = tokio::join!(
    async_operation_1(),
    async_operation_2()
);

// ❌ Sequential (slower)
let result1 = async_operation_1().await;
let result2 = async_operation_2().await;
```

---

### 4. Use `select!` for Racing Operations

```rust
use tokio::select;

select! {
    result = operation1() => {
        // operation1 finished first
    }
    result = operation2() => {
        // operation2 finished first
    }
}
```

**Use Case**: Timeout vs. completion.

---

### 5. Clone Expensive Data Before Spawn

```rust
// ✅ Clone Arc (cheap)
let data = Arc::new(large_data);
tokio::spawn(async move {
    let data = Arc::clone(&data);
    process(data).await;
});

// ❌ Clone large data (expensive)
tokio::spawn(async move {
    process(large_data.clone()).await;  // Expensive copy
});
```

---

### 6. Use `Instrument` for Tracing

```rust
use tracing::instrument;

#[instrument(skip(self))]
async fn process(&self, input: AgentInput) -> CoreResult<AgentOutput> {
    debug!("Processing input");
    // ...
}
```

**Benefit**: Automatic tracing of async function entry/exit.

---

## Performance Considerations

### 1. Task Spawning Overhead

**Cost**: ~10-50μs per spawn

**Guideline**: Spawn tasks for I/O-bound work, avoid for CPU-bound work.

```rust
// ✅ Good: Spawn for I/O
tokio::spawn(async move {
    fetch_url(url).await;
});

// ❌ Bad: Spawn for CPU work
tokio::spawn(async move {
    heavy_computation();  // Blocks executor
});
```

**For CPU Work**: Use `tokio::task::spawn_blocking`:

```rust
tokio::task::spawn_blocking(|| {
    heavy_computation();  // Runs on blocking thread pool
});
```

---

### 2. Lock Contention

**Problem**: High contention on `Mutex` reduces parallelism.

**Solution**: Use `RwLock` for read-heavy workloads:

```rust
use tokio::sync::RwLock;

let lock = Arc::new(RwLock::new(data));

// Many readers can hold lock concurrently
let reader = lock.read().await;
// ...

// Only one writer
let mut writer = lock.write().await;
// ...
```

---

### 3. Buffer Sizes for Channels

**Default**: 100 (mpsc channels)

**Guideline**:
- Low-volume: 10-50 (less memory)
- High-volume: 100-1000 (more buffering)
- Unbounded: Avoid (can cause OOM)

```rust
let (tx, rx) = mpsc::channel(100);  // Balanced
```

---

## Testing Async Code

### Unit Tests

```rust
#[tokio::test]
async fn test_async_function() {
    let result = async_function().await.unwrap();
    assert_eq!(result, expected);
}
```

---

### Integration Tests

```rust
#[tokio::test]
async fn test_agent_consensus() {
    let mut engine = ConsensusEngine::with_agents(...);
    let outcome = engine.run("test prompt").await.unwrap();
    assert!(outcome.is_consensus());
}
```

---

### Mocking Async Dependencies

Use `mockall` crate for async mocking:

```rust
#[automock]
#[async_trait]
trait Agent {
    async fn process(&self, input: Input) -> Result<Output>;
}
```

---

## Common Pitfalls

### 1. Forgetting `.await`

```rust
// ❌ Forgets .await (returns Future, not value)
let result = async_function();

// ✅ Correct
let result = async_function().await;
```

---

### 2. Forgetting `?` in Async Context

```rust
// ❌ Forgets ? (returns Result, not value)
async fn bad() -> Result<()> {
    let result = may_fail().await;
}

// ✅ Correct
async fn good() -> Result<()> {
    let result = may_fail().await?;
}
```

---

### 3. Mixing Sync and Async

```rust
// ❌ Wrong: Call sync function from async
async fn bad() {
    sync_blocking_function();  // Blocks executor!
}

// ✅ Correct: Use spawn_blocking
async fn good() {
    tokio::task::spawn_blocking(|| {
        sync_blocking_function();
    }).await?;
}
```

---

## Migration Path to Phase 2

### Current (Phase 1)

- Sequential agent execution
- Synchronous SQLite (rusqlite)
- File watcher disabled (lifetime issues)
- No streaming responses

### Phase 2 Goals

- Parallel agent execution (where independent)
- Async SQLite (sqlx or async-rusqlite)
- Channel-based file watching
- Streaming token responses

---

## References

### Official Documentation

- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Async Rust Book](https://rust-lang.github.io/async-book/)
- [Rust Async: What's Blocking?](https://blog.yoshuawuyts.com/async-rust-what-is-blocking/)

### Related Crates

- [tokio](https://docs.rs/tokio) - Async runtime
- [async-trait](https://docs.rs/async-trait) - Async traits
- [tokio-stream](https://docs.rs/tokio-stream) - Stream utilities
- [tracing](https://docs.rs/tracing) - Async instrumentation

### SuperInstance Specific

- [ARCHITECTURE.md](ARCHITECTURE.md) - System architecture
- [INTEGRATION_REPORT.md](INTEGRATION_REPORT.md) - Integration testing
- [BUILD_STATUS.md](status/BUILD_STATUS.md) - Current status

---

*Document Version: 1.0*
*Last Updated: 2026-01-02*
*Author: Integration & Research Agent*
*Status: Phase 1 Complete*
