# Thread Safety Patterns in SuperInstance AI

This document defines the standardized thread safety patterns used across the SuperInstance AI codebase. All new code should follow these patterns to ensure consistency, maintainability, and correctness.

## Table of Contents

1. [Overview](#overview)
2. [Pattern 1: Arc<tokio::sync::Mutex<T>> for Async Code](#pattern-1-arctokiosynce_mutext-for-async-code)
3. [Pattern 2: Arc<AtomicBool> for Thread-Safe Flags](#pattern-2-arcatomicbool-for-thread-safe-flags)
4. [Pattern 3: Arc<Vec<T>> for Immutable Collections](#pattern-3-arcvect-for-immutable-collections)
5. [Pattern 4: Arc<AtomicU64> for Lock-Free Metrics](#pattern-4-arcatomicu64-for-lock-free-metrics)
6. [Anti-Patterns to Avoid](#anti-patterns-to-avoid)
7. [Decision Tree](#decision-tree)
8. [Testing Guidelines](#testing-guidelines)
9. [Common Pitfalls](#common-pitfalls)

---

## Overview

SuperInstance AI uses multiple threads for parallel agent execution and async/await for I/O operations. Thread safety is critical for correctness and performance.

### Key Principles

1. **Never hold `MutexGuard` across `await` points** - This causes deadlocks and undefined behavior
2. **Use `tokio::sync::Mutex` in async code, NOT `std::sync::Mutex`** - Tokio's mutex is async-aware
3. **Prefer atomic operations for simple counters** - Lock-free is faster and avoids deadlocks
4. **Use `Arc<T>` for shared state across threads, NOT `Rc<T>`** - Rc is not thread-safe
5. **Prefer immutable shared collections** - Read-only access needs no locking

### When to Use Each Pattern

| Pattern | Use Case | Example |
|---------|----------|---------|
| `Arc<tokio::sync::Mutex<T>>` | Mutable shared state in async code | Knowledge vault, embedder, model storage |
| `Arc<AtomicBool>` | Simple boolean flags across threads | Agent ready state, shutdown flags |
| `Arc<AtomicU64>` | Counters and metrics | Query counts, latency tracking |
| `Arc<Vec<T>>` | Immutable shared collections | Veto patterns, agent configurations |
| `Arc<HashMap<K,V>>` | Read-only shared maps | Configuration lookups |

---

## Pattern 1: Arc<tokio::sync::Mutex<T>> for Async Code

**Use Case**: Sharing mutable state across async tasks

**When to Use**:
- You need to share mutable state across multiple async tasks
- The shared state will be accessed and modified
- You're using async/await and Tokio runtime

**Example Usage**:

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

// Create the shared state
let vault = Arc::new(tokio::sync::Mutex::new(KnowledgeVault::open(&path, 384)?));
let embedder = Arc::new(tokio::sync::Mutex::new(PlaceholderEmbedder::new(384)));

// Spawn multiple tasks that share the state
let vault_clone = Arc::clone(&vault);
let task1 = tokio::spawn(async move {
    // Lock the mutex
    let vault_lock = vault_clone.lock().await;
    // Perform synchronous operations
    let result = vault_lock.search(&query).await?;
    // CRITICAL: Drop the lock BEFORE await
    drop(vault_lock);
    // Now safe to await
    some_async_operation().await
});

// In another task
let vault_clone2 = Arc::clone(&vault);
let task2 = tokio::spawn(async move {
    let vault_lock = vault_clone2.lock().await;
    let result = vault_lock.index(&doc).await?;
    drop(vault_lock); // Drop before await
    more_async_work().await
});
```

**Critical Rules**:
1. ALWAYS use `tokio::sync::Mutex`, NEVER `std::sync::Mutex` in async code
2. ALWAYS drop the lock before any `.await` point
3. Use `Arc::clone()` to create references for each task
4. Keep critical sections as short as possible

**Why Not std::sync::Mutex?**:
- `std::sync::Mutex` causes deadlock if held across `.await`
- `tokio::sync::Mutex` is designed for async/await
- Compilation error if you try to `.await` while holding the lock

**Real Examples in Codebase**:
- `crates/synesis-cli/src/commands/knowledge.rs:312` - Knowledge vault sharing
- `crates/synesis-cli/src/commands/knowledge.rs:315` - Embedder sharing

---

## Pattern 2: Arc<AtomicBool> for Thread-Safe Flags

**Use Case**: Simple boolean flags that need to be shared across threads

**When to Use**:
- You need a boolean flag visible across multiple threads
- The flag is read/written frequently
- You don't need complex logic around the flag
- Performance is critical (lock-free)

**Example Usage**:

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone)]
pub struct PathosAgent {
    config: AgentConfig,
    ready: Arc<AtomicBool>,
}

impl PathosAgent {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            config,
            ready: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn initialize(&mut self) -> CoreResult<()> {
        // Load model...
        self.ready.store(true, Ordering::SeqCst);
        Ok(())
    }

    pub fn is_ready(&self) -> bool {
        self.ready.load(Ordering::SeqCst)
    }
}
```

**Memory Ordering Guide**:
- `Ordering::SeqCst` - Sequentially consistent, strongest guarantee. Use for initialization flags.
- `Ordering::Release` / `Ordering::Acquire` - Pair for write/read synchronization
- `Ordering::Relaxed` - Weakest, no synchronization. Only for counters where order doesn't matter.

**Why Use AtomicBool Instead of Mutex<bool>?**:
- Lock-free (faster)
- No deadlock risk
- Smaller memory footprint
- Cannot be held across await (not a lock)

**Real Examples in Codebase**:
- `crates/synesis-core/src/agents/pathos.rs:27` - Agent ready flag
- `crates/synesis-core/src/agents/logos.rs:21` - Agent ready flag
- `crates/synesis-core/src/agents/ethos.rs:33` - Agent ready flag

---

## Pattern 3: Arc<Vec<T>> for Immutable Collections

**Use Case**: Sharing read-only collections across threads

**When to Use**:
- You have a collection that never changes after creation
- Multiple threads need to read from it
- Performance is critical (no locking overhead)

**Example Usage**:

```rust
use std::sync::Arc;

#[derive(Clone)]
pub struct EthosAgent {
    config: AgentConfig,
    ready: Arc<AtomicBool>,
    // Immutable collection of veto patterns
    veto_patterns: Arc<Vec<VetoPattern>>,
}

impl EthosAgent {
    pub fn new(config: AgentConfig) -> Self {
        // Create patterns once
        let veto_patterns = vec![
            VetoPattern {
                pattern: Regex::new(r"rm\s+-rf\s+/").expect("invalid regex"),
                description: "Recursive root deletion",
                category: VetoCategory::FileSystem,
            },
            // ... more patterns
        ];

        Self {
            config,
            ready: Arc::new(AtomicBool::new(false)),
            // Wrap in Arc for cheap cloning
            veto_patterns: Arc::new(veto_patterns),
        }
    }

    fn check_patterns(&self, solution: &str) -> Vec<Constraint> {
        let mut constraints = Vec::new();

        // Read from immutable collection - no lock needed!
        for veto_pattern in self.veto_patterns.iter() {
            if veto_pattern.pattern.is_match(solution) {
                constraints.push(/* ... */);
            }
        }

        constraints
    }
}
```

**Why Use Arc<Vec<T>> Instead of Arc<Mutex<Vec<T>>>?**:
- No locking overhead (faster)
- Immutable = no race conditions
- Can't accidentally modify
- Cheaper to clone (just pointer increment)

**When NOT to Use This Pattern**:
- If the collection needs to be modified after creation
- If different threads need different versions (use `Arc<RwLock<Vec<T>>>`)

**Real Examples in Codebase**:
- `crates/synesis-core/src/agents/ethos.rs:35` - Veto patterns collection

---

## Pattern 4: Arc<AtomicU64> for Lock-Free Metrics

**Use Case**: High-performance counters and metrics

**When to Use**:
- You need to track counts across multiple threads
- Performance is critical (high frequency updates)
- You only need simple increment/add operations
- You don't need to read the exact value mid-update

**Example Usage**:

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Metrics {
    queries_total: Arc<AtomicU64>,
    queries_successful: Arc<AtomicU64>,
    queries_failed: Arc<AtomicU64>,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            queries_total: Arc::new(AtomicU64::new(0)),
            queries_successful: Arc::new(AtomicU64::new(0)),
            queries_failed: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn record_query(&self) {
        // Lock-free increment
        self.queries_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_success(&self) {
        self.queries_successful.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_failure(&self) {
        self.queries_failed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_stats(&self) -> (u64, u64, u64) {
        (
            self.queries_total.load(Ordering::Relaxed),
            self.queries_successful.load(Ordering::Relaxed),
            self.queries_failed.load(Ordering::Relaxed),
        )
    }
}
```

**Atomic Operations**:
- `fetch_add(n, ordering)` - Add and return old value
- `fetch_sub(n, ordering)` - Subtract and return old value
- `fetch_max(n, ordering)` - Set to max of current and n
- `fetch_min(n, ordering)` - Set to min of current and n
- `swap(n, ordering)` - Swap with new value, return old
- `compare_exchange(old, new, ...)` - Conditional swap

**Memory Ordering for Metrics**:
- Use `Ordering::Relaxed` for counters where exact ordering doesn't matter
- Use `Ordering::Release` on write, `Ordering::Acquire` on read for synchronization
- Use `Ordering::SeqCst` when in doubt (slower but safest)

**Why Not Use Mutex<u64>?**:
- Lock-free (no contention)
- Faster under high load
- No deadlock risk
- Can't accidentally hold across await

**Real Examples in Codebase**:
- `crates/synesis-core/src/metrics.rs` - All metric counters

---

## Anti-Patterns to Avoid

### Anti-Pattern 1: std::sync::Mutex in Async Code

```rust
// ❌ WRONG: Will cause deadlock or compilation error
let vault = Arc::new(std::sync::Mutex::new(KnowledgeVault::open(...)?));

async fn bad_example(vault: Arc<std::sync::Mutex<KnowledgeVault>>) {
    let lock = vault.lock().unwrap();
    let result = lock.search(&query).await?; // ERROR: await while holding lock!
    drop(lock);
}

// ✅ CORRECT: Use tokio::sync::Mutex
let vault = Arc::new(tokio::sync::Mutex::new(KnowledgeVault::open(...)?));

async fn good_example(vault: Arc<tokio::sync::Mutex<KnowledgeVault>>) {
    let lock = vault.lock().await;
    let result = sync_operation(&lock);
    drop(lock); // Drop before await
    async_operation().await; // Safe now
}
```

### Anti-Pattern 2: Rc<T> Instead of Arc<T>

```rust
// ❌ WRONG: Rc is not thread-safe
use std::rc::Rc;

let data = Rc::new(vec![1, 2, 3]);
let handle = std::thread::spawn(move || {
    // Compilation error: Rc cannot be sent between threads
    println!("{:?}", *data);
});

// ✅ CORRECT: Use Arc for thread safety
use std::sync::Arc;

let data = Arc::new(vec![1, 2, 3]);
let data_clone = Arc::clone(&data);
let handle = std::thread::spawn(move || {
    println!("{:?}", *data_clone);
});
```

### Anti-Pattern 3: Holding Lock Across Await

```rust
// ❌ WRONG: Deadlock!
async fn bad_example(mutex: Arc<tokio::sync::Mutex<Vec<String>>>) {
    let mut lock = mutex.lock().await;
    lock.push("test".to_string());
    some_async_function().await; // ERROR: Still holding lock!
    lock.push("done".to_string());
    drop(lock);
}

// ✅ CORRECT: Release lock before await
async fn good_example(mutex: Arc<tokio::sync::Mutex<Vec<String>>>) {
    {
        let mut lock = mutex.lock().await;
        lock.push("test".to_string());
    } // Lock dropped here

    some_async_function().await; // Safe: no lock held

    {
        let mut lock = mutex.lock().await;
        lock.push("done".to_string());
    } // Lock dropped here
}
```

### Anti-Pattern 4: Using Mutex for Simple Flags

```rust
// ❌ WRONG: Unnecessary overhead
use std::sync::{Arc, Mutex};

pub struct Agent {
    ready: Arc<Mutex<bool>>,
}

// ✅ CORRECT: Use AtomicBool for simple flags
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct Agent {
    ready: Arc<AtomicBool>,
}
```

### Anti-Pattern 5: Mutable Static Variables

```rust
// ❌ WRONG: Mutable static is unsafe
static mut COUNTER: u64 = 0;

unsafe fn increment() {
    COUNTER += 1; // Undefined behavior with concurrent access!
}

// ✅ CORRECT: Use atomic static
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn increment() {
    COUNTER.fetch_add(1, Ordering::Relaxed);
}
```

---

## Decision Tree

```
Need to share data across threads?
    │
    ├─ No: Use normal ownership, Rust's type system will prevent issues
    │
    └─ Yes: Continue
        │
        ├─ Is the data mutable after creation?
        │   │
        │   ├─ No (immutable): Use Arc<T> (no lock needed)
        │   │
        │   └─ Yes (mutable): Continue
        │       │
        │       ├─ Simple boolean flag?
        │       │   └─ Yes: Use Arc<AtomicBool>
        │       │
        │       ├─ Simple counter/numeric value?
        │       │   └─ Yes: Use Arc<AtomicU64> / Arc<AtomicI64> / Arc<AtomicUsize>
        │       │
        │       ├─ Collection that never changes?
        │       │   └─ Yes: Use Arc<Vec<T>> or Arc<HashMap<K,V>>
        │       │
        │       ├─ Used in async/await code?
        │       │   └─ Yes: Use Arc<tokio::sync::Mutex<T>>
        │       │
        │       └─ Used in sync code only?
        │           └─ Yes: Use Arc<std::sync::Mutex<T>>
        │
        └─ Performance critical?
            │
            ├─ Yes: Prefer atomics or lock-free patterns
            │
            └─ No: Use mutex for simplicity
```

---

## Testing Guidelines

### Unit Tests for Thread Safety

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_concurrent_access() {
        let shared = Arc::new(tokio::sync::Mutex::new(vec![1, 2, 3]));
        let mut handles = vec![];

        // Spawn multiple tasks
        for i in 0..10 {
            let shared_clone = Arc::clone(&shared);
            let handle = tokio::spawn(async move {
                let mut lock = shared_clone.lock().await;
                lock.push(i);
            });
            handles.push(handle);
        }

        // Wait for all tasks
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify no data corruption
        let lock = shared.lock().await;
        assert_eq!(lock.len(), 13); // Original 3 + 10 new
    }

    #[tokio::test]
    async fn test_atomic_operations() {
        use std::sync::atomic::{AtomicU64, Ordering};

        let counter = Arc::new(AtomicU64::new(0));
        let mut handles = vec![];

        // Spawn 100 threads incrementing
        for _ in 0..100 {
            let counter_clone = Arc::clone(&counter);
            let handle = tokio::spawn(async move {
                for _ in 0..1000 {
                    counter_clone.fetch_add(1, Ordering::Relaxed);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        // Should be exactly 100,000
        assert_eq!(counter.load(Ordering::SeqCst), 100_000);
    }

    #[tokio::test]
    async fn test_arc_clone_behavior() {
        use std::sync::Arc;

        let data = Arc::new(vec![1, 2, 3]);

        // Clone creates a new reference, not a copy
        let data_clone = Arc::clone(&data);

        // Both point to same data
        assert!(Arc::ptr_eq(&data, &data_clone));

        // Weak references don't prevent deallocation
        let weak = Arc::downgrade(&data);
        assert!(weak.upgrade().is_some());

        drop(data);
        drop(data_clone);

        // Now no strong references, weak upgrade fails
        assert!(weak.upgrade().is_none());
    }
}
```

### Integration Tests for Concurrent Scenarios

```rust
#[tokio::test]
async fn test_parallel_agent_execution() {
    let pathos = PathosAgent::new(config.clone());
    let logos = LogosAgent::new(config.clone());
    let ethos = EthosAgent::new(config);

    let manifest = A2AManifest::new("Test query".to_string());

    // Run agents in parallel
    let (pathos_result, logos_result, ethos_result) = tokio::join!(
        pathos.process(AgentInput::new(manifest.clone())),
        logos.process(AgentInput::new(manifest.clone())),
        ethos.prefetch(&AgentInput::new(manifest))
    );

    // Verify all completed successfully
    assert!(pathos_result.is_ok());
    assert!(logos_result.is_ok());
    assert!(ethos_result.is_ok());
}
```

---

## Common Pitfalls

### Pitfall 1: Forgetting to Clone Arc

```rust
// ❌ WRONG: Moves Arc into first task, can't use in second
let shared = Arc::new(Mutex::new(vec![1, 2, 3]));
let task1 = tokio::spawn(async move {
    let lock = shared.lock().await;
    // ...
});
// Compilation error: use of moved value

// ✅ CORRECT: Clone Arc for each task
let shared = Arc::new(Mutex::new(vec![1, 2, 3]));
let shared_clone1 = Arc::clone(&shared);
let task1 = tokio::spawn(async move {
    let lock = shared_clone1.lock().await;
    // ...
});
let shared_clone2 = Arc::clone(&shared);
let task2 = tokio::spawn(async move {
    let lock = shared_clone2.lock().await;
    // ...
});
```

### Pitfall 2: Wrong Memory Ordering

```rust
// ❌ WRONG: Relaxed ordering for initialization flag
static INITIALIZED: AtomicBool = AtomicBool::new(false);

fn init_and_use() {
    // Thread 1
    INITIALIZED.store(true, Ordering::Relaxed);
    // Might be reordered! Other thread might see
    // initialized=true before data is actually ready
}

// ✅ CORRECT: Use SeqCst for initialization flags
static INITIALIZED: AtomicBool = AtomicBool::new(false);

fn init_and_use() {
    // Thread 1
    // All writes before here visible to other thread
    INITIALIZED.store(true, Ordering::SeqCst);
}
```

### Pitfall 3: Priority Inversion with Mutex

```rust
// ❌ WRONG: Long critical section
async fn bad_example(mutex: Arc<Mutex<Vec<String>>>) {
    let lock = mutex.lock().await;
    // ... 100ms of work ...
    expensive_computation(); // Holding lock the whole time!
    drop(lock);
}

// ✅ CORRECT: Keep critical sections short
async fn good_example(mutex: Arc<Mutex<Vec<String>>>) {
    // Copy data, release lock
    let data = {
        let lock = mutex.lock().await;
        lock.clone()
    }; // Lock released

    // Do expensive work outside lock
    expensive_computation(&data).await;
}
```

### Pitfall 4: Poisoned Mutex

```rust
// ❌ WRONG: Unwrap on lock can panic
let lock = mutex.lock().unwrap();
// If another thread panicked while holding this lock,
// it's poisoned and unwrap will panic too

// ✅ CORRECT: Handle poisoned mutex
let lock = match mutex.lock() {
    Ok(guard) => guard,
    Err(poisoned) => {
        // Recover from panic, get the guard anyway
        poisoned.into_inner()
    }
};
```

### Pitfall 5: Recursive Locking

```rust
// ❌ WRONG: tokio::sync::Mutex is not recursive
async fn bad_example(mutex: Arc<Mutex<Data>>) {
    let lock1 = mutex.lock().await;
    let lock2 = mutex.lock().await; // DEADLOCK! Same thread waiting twice
}

// ✅ CORRECT: Restructure code or use RwLock if needed
async fn good_example(mutex: Arc<Mutex<Data>>) {
    // Do all work in one lock acquisition
    let lock = mutex.lock().await;
    // ... all work ...
    drop(lock);
}
```

---

## Quick Reference

### Import Statements

```rust
// For async mutex (always use in async code)
use tokio::sync::Mutex;

// For atomic operations
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

// For Arc
use std::sync::Arc;

// For standard mutex (only in sync code, not with async)
use std::sync::Mutex;
```

### Common Operations

```rust
// Create Arc-wrapped value
let data = Arc::new(Mutex::new(vec![1, 2, 3]));

// Clone Arc (cheap, just increments counter)
let data_clone = Arc::clone(&data);

// Lock mutex (in async code)
let lock = data.lock().await;

// Atomic operations
counter.fetch_add(1, Ordering::Relaxed);
flag.store(true, Ordering::SeqCst);
let value = flag.load(Ordering::SeqCst);
```

---

## Further Reading

- [Tokio Mutex Documentation](https://docs.rs/tokio/latest/tokio/sync/struct.Mutex.html)
- [std::sync::atomic Documentation](https://doc.rust-lang.org/std/sync/atomic/index.html)
- [Rust Async Book: Concurrency](https://rust-lang.github.io/async-book/06_futures/02_future.html)
- [Rustonomicon: Atomics](https://doc.rust-lang.org/nomicon/atomics.html)

---

*Last Updated: 2026-01-02*
*Version: 1.0*
*Status: Active - All new code must follow these patterns*
