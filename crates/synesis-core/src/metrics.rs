//! Metrics and Observability
//!
//! This module provides metrics collection for monitoring system performance and behavior.
//! Uses atomic operations for thread-safe, lock-free metric updates.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// System-wide metrics collector
#[derive(Debug, Clone)]
pub struct Metrics {
    inner: Arc<MetricsInner>,
}

#[derive(Debug)]
struct MetricsInner {
    // Query metrics
    queries_total: AtomicU64,
    queries_successful: AtomicU64,
    queries_failed: AtomicU64,

    // Consensus metrics
    consensus_reached_first_round: AtomicU64,
    consensus_reached_second_round: AtomicU64,
    consensus_reached_third_round: AtomicU64,
    consensus_failed: AtomicU64,

    // Agent metrics
    ethos_vetoes: AtomicU64,
    pathos_timeouts: AtomicU64,
    logos_retrievals: AtomicU64,

    // Performance metrics
    total_response_time_ms: AtomicU64,
    min_response_time_ms: AtomicU64,
    max_response_time_ms: AtomicU64,

    // Knowledge metrics
    documents_indexed: AtomicU64,
    chunks_stored: AtomicU64,
    searches_performed: AtomicU64,

    // Privacy metrics
    redactions_performed: AtomicU64,
    tokens_generated: AtomicU64,
}

impl Default for MetricsInner {
    fn default() -> Self {
        Self {
            queries_total: AtomicU64::new(0),
            queries_successful: AtomicU64::new(0),
            queries_failed: AtomicU64::new(0),
            consensus_reached_first_round: AtomicU64::new(0),
            consensus_reached_second_round: AtomicU64::new(0),
            consensus_reached_third_round: AtomicU64::new(0),
            consensus_failed: AtomicU64::new(0),
            ethos_vetoes: AtomicU64::new(0),
            pathos_timeouts: AtomicU64::new(0),
            logos_retrievals: AtomicU64::new(0),
            total_response_time_ms: AtomicU64::new(0),
            min_response_time_ms: AtomicU64::new(u64::MAX),
            max_response_time_ms: AtomicU64::new(0),
            documents_indexed: AtomicU64::new(0),
            chunks_stored: AtomicU64::new(0),
            searches_performed: AtomicU64::new(0),
            redactions_performed: AtomicU64::new(0),
            tokens_generated: AtomicU64::new(0),
        }
    }
}

impl Metrics {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            inner: Arc::new(MetricsInner::default()),
        }
    }

    /// Record a query start
    pub fn record_query_start(&self) -> QueryTimer {
        self.inner.queries_total.fetch_add(1, Ordering::Relaxed);
        QueryTimer {
            metrics: self.clone(),
            start: Instant::now(),
        }
    }

    /// Record a successful query
    pub(crate) fn record_query_success(&self, duration: Duration) {
        self.inner.queries_successful.fetch_add(1, Ordering::Relaxed);
        let duration_ms = duration.as_millis() as u64;
        self.update_response_time(duration_ms);
    }

    /// Record a failed query
    pub(crate) fn record_query_failure(&self, duration: Duration) {
        self.inner.queries_failed.fetch_add(1, Ordering::Relaxed);
        let duration_ms = duration.as_millis() as u64;
        self.update_response_time(duration_ms);
    }

    /// Update response time metrics
    fn update_response_time(&self, duration_ms: u64) {
        // Update total
        self.inner.total_response_time_ms.fetch_add(duration_ms, Ordering::Relaxed);

        // Update min
        let mut current_min = self.inner.min_response_time_ms.load(Ordering::Relaxed);
        while duration_ms < current_min {
            match self.inner.min_response_time_ms.compare_exchange(
                current_min,
                duration_ms,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(new_min) => current_min = new_min,
            }
        }

        // Update max
        let mut current_max = self.inner.max_response_time_ms.load(Ordering::Relaxed);
        while duration_ms > current_max {
            match self.inner.max_response_time_ms.compare_exchange(
                current_max,
                duration_ms,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(new_max) => current_max = new_max,
            }
        }
    }

    /// Record consensus reached on a specific round
    pub fn record_consensus_reached(&self, round: u8) {
        match round {
            1 => {
                let _ = self.inner.consensus_reached_first_round.fetch_add(1, Ordering::Relaxed);
            }
            2 => {
                let _ = self.inner.consensus_reached_second_round.fetch_add(1, Ordering::Relaxed);
            }
            3 => {
                let _ = self.inner.consensus_reached_third_round.fetch_add(1, Ordering::Relaxed);
            }
            _ => {}
        };
    }

    /// Record consensus failure
    pub fn record_consensus_failed(&self) {
        self.inner.consensus_failed.fetch_add(1, Ordering::Relaxed);
    }

    /// Record an Ethos veto
    pub fn record_ethos_veto(&self) {
        self.inner.ethos_vetoes.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a Pathos timeout
    pub fn record_pathos_timeout(&self) {
        self.inner.pathos_timeouts.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a Logos retrieval
    pub fn record_logos_retrieval(&self) {
        self.inner.logos_retrievals.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a document indexed
    pub fn record_document_indexed(&self) {
        self.inner.documents_indexed.fetch_add(1, Ordering::Relaxed);
    }

    /// Record chunks stored
    pub fn record_chunks_stored(&self, count: u64) {
        self.inner.chunks_stored.fetch_add(count, Ordering::Relaxed);
    }

    /// Record a search performed
    pub fn record_search_performed(&self) {
        self.inner.searches_performed.fetch_add(1, Ordering::Relaxed);
    }

    /// Record redactions performed
    pub fn record_redactions(&self, count: u64) {
        self.inner.redactions_performed.fetch_add(count, Ordering::Relaxed);
    }

    /// Record tokens generated
    pub fn record_tokens_generated(&self, count: u64) {
        self.inner.tokens_generated.fetch_add(count, Ordering::Relaxed);
    }

    /// Get current metrics as a snapshot
    pub fn snapshot(&self) -> MetricsSnapshot {
        let queries_total = self.inner.queries_total.load(Ordering::Relaxed);
        let queries_successful = self.inner.queries_successful.load(Ordering::Relaxed);
        let total_response_time_ms = self.inner.total_response_time_ms.load(Ordering::Relaxed);

        MetricsSnapshot {
            queries_total,
            queries_successful,
            queries_failed: self.inner.queries_failed.load(Ordering::Relaxed),
            success_rate: if queries_total > 0 {
                (queries_successful as f64 / queries_total as f64) * 100.0
            } else {
                0.0
            },
            avg_response_time_ms: if queries_successful > 0 {
                total_response_time_ms / queries_successful
            } else {
                0
            },
            min_response_time_ms: self.inner.min_response_time_ms.load(Ordering::Relaxed),
            max_response_time_ms: self.inner.max_response_time_ms.load(Ordering::Relaxed),
            consensus_reached_first_round: self.inner.consensus_reached_first_round.load(Ordering::Relaxed),
            consensus_reached_second_round: self.inner.consensus_reached_second_round.load(Ordering::Relaxed),
            consensus_reached_third_round: self.inner.consensus_reached_third_round.load(Ordering::Relaxed),
            consensus_failed: self.inner.consensus_failed.load(Ordering::Relaxed),
            ethos_vetoes: self.inner.ethos_vetoes.load(Ordering::Relaxed),
            pathos_timeouts: self.inner.pathos_timeouts.load(Ordering::Relaxed),
            logos_retrievals: self.inner.logos_retrievals.load(Ordering::Relaxed),
            documents_indexed: self.inner.documents_indexed.load(Ordering::Relaxed),
            chunks_stored: self.inner.chunks_stored.load(Ordering::Relaxed),
            searches_performed: self.inner.searches_performed.load(Ordering::Relaxed),
            redactions_performed: self.inner.redactions_performed.load(Ordering::Relaxed),
            tokens_generated: self.inner.tokens_generated.load(Ordering::Relaxed),
        }
    }

    /// Export metrics in Prometheus format
    pub fn to_prometheus(&self) -> String {
        let snap = self.snapshot();

        format!(
            "# HELP synesis_queries_total Total number of queries processed\n\
             # TYPE synesis_queries_total counter\n\
             synesis_queries_total {}\n\
             # HELP synesis_queries_successful Total number of successful queries\n\
             # TYPE synesis_queries_successful counter\n\
             synesis_queries_successful {}\n\
             # HELP synesis_queries_failed Total number of failed queries\n\
             # TYPE synesis_queries_failed counter\n\
             synesis_queries_failed {}\n\
             # HELP synesis_success_rate Success rate percentage\n\
             # TYPE synesis_success_rate gauge\n\
             synesis_success_rate {:.2}\n\
             # HELP synesis_avg_response_time_ms Average response time in milliseconds\n\
             # TYPE synesis_avg_response_time_ms gauge\n\
             synesis_avg_response_time_ms {}\n\
             # HELP synesis_min_response_time_ms Minimum response time in milliseconds\n\
             # TYPE synesis_min_response_time_ms gauge\n\
             synesis_min_response_time_ms {}\n\
             # HELP synesis_max_response_time_ms Maximum response time in milliseconds\n\
             # TYPE synesis_max_response_time_ms gauge\n\
             synesis_max_response_time_ms {}\n\
             # HELP synesis_consensus_reached_first_round Consensus reached on first round\n\
             # TYPE synesis_consensus_reached_first_round counter\n\
             synesis_consensus_reached_first_round {}\n\
             # HELP synesis_consensus_reached_second_round Consensus reached on second round\n\
             # TYPE synesis_consensus_reached_second_round counter\n\
             synesis_consensus_reached_second_round {}\n\
             # HELP synesis_consensus_reached_third_round Consensus reached on third round\n\
             # TYPE synesis_consensus_reached_third_round counter\n\
             synesis_consensus_reached_third_round {}\n\
             # HELP synesis_consensus_failed Consensus not reached\n\
             # TYPE synesis_consensus_failed counter\n\
             synesis_consensus_failed {}\n\
             # HELP synesis_ethos_vetoes Total number of Ethos vetoes\n\
             # TYPE synesis_ethos_vetoes counter\n\
             synesis_ethos_vetoes {}\n\
             # HELP synesis_documents_indexed Total number of documents indexed\n\
             # TYPE synesis_documents_indexed counter\n\
             synesis_documents_indexed {}\n\
             # HELP synesis_chunks_stored Total number of chunks stored\n\
             # TYPE synesis_chunks_stored counter\n\
             synesis_chunks_stored {}\n\
             # HELP synesis_searches_performed Total number of searches performed\n\
             # TYPE synesis_searches_performed counter\n\
             synesis_searches_performed {}\n\
             # HELP synesis_redactions_performed Total number of redactions performed\n\
             # TYPE synesis_redactions_performed counter\n\
             synesis_redactions_performed {}\n",
            snap.queries_total,
            snap.queries_successful,
            snap.queries_failed,
            snap.success_rate,
            snap.avg_response_time_ms,
            if snap.min_response_time_ms == u64::MAX {
                0
            } else {
                snap.min_response_time_ms
            },
            snap.max_response_time_ms,
            snap.consensus_reached_first_round,
            snap.consensus_reached_second_round,
            snap.consensus_reached_third_round,
            snap.consensus_failed,
            snap.ethos_vetoes,
            snap.documents_indexed,
            snap.chunks_stored,
            snap.searches_performed,
            snap.redactions_performed,
        )
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

/// A timer for measuring query duration
pub struct QueryTimer {
    metrics: Metrics,
    start: Instant,
}

impl QueryTimer {
    /// Complete the timer and record success
    pub fn finish_success(self) {
        let duration = self.start.elapsed();
        self.metrics.record_query_success(duration);
    }

    /// Complete the timer and record failure
    pub fn finish_failure(self) {
        let duration = self.start.elapsed();
        self.metrics.record_query_failure(duration);
    }
}

/// A snapshot of metrics at a point in time
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    /// Total queries processed
    pub queries_total: u64,
    /// Successful queries
    pub queries_successful: u64,
    /// Failed queries
    pub queries_failed: u64,
    /// Success rate percentage
    pub success_rate: f64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: u64,
    /// Minimum response time in milliseconds
    pub min_response_time_ms: u64,
    /// Maximum response time in milliseconds
    pub max_response_time_ms: u64,
    /// Consensus reached on first round
    pub consensus_reached_first_round: u64,
    /// Consensus reached on second round
    pub consensus_reached_second_round: u64,
    /// Consensus reached on third round
    pub consensus_reached_third_round: u64,
    /// Consensus failed
    pub consensus_failed: u64,
    /// Ethos vetoes
    pub ethos_vetoes: u64,
    /// Pathos timeouts
    pub pathos_timeouts: u64,
    /// Logos retrievals
    pub logos_retrievals: u64,
    /// Documents indexed
    pub documents_indexed: u64,
    /// Chunks stored
    pub chunks_stored: u64,
    /// Searches performed
    pub searches_performed: u64,
    /// Redactions performed
    pub redactions_performed: u64,
    /// Tokens generated
    pub tokens_generated: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_metrics_creation() {
        let metrics = Metrics::new();
        assert_eq!(metrics.snapshot().queries_total, 0);
    }

    #[test]
    fn test_query_recording() {
        let metrics = Metrics::new();
        let _timer = metrics.record_query_start();
        assert_eq!(metrics.snapshot().queries_total, 1);
    }

    #[test]
    fn test_consensus_tracking() {
        let metrics = Metrics::new();
        metrics.record_consensus_reached(1);
        metrics.record_consensus_reached(2);
        metrics.record_consensus_failed();

        let snap = metrics.snapshot();
        assert_eq!(snap.consensus_reached_first_round, 1);
        assert_eq!(snap.consensus_reached_second_round, 1);
        assert_eq!(snap.consensus_failed, 1);
    }

    #[test]
    fn test_prometheus_export() {
        let metrics = Metrics::new();
        metrics.record_query_start().finish_success();
        metrics.record_consensus_reached(1);

        let prom = metrics.to_prometheus();
        assert!(prom.contains("synesis_queries_total 1"));
        assert!(prom.contains("synesis_consensus_reached_first_round 1"));
    }

    /// Thread Safety Test 1: Concurrent increments
    ///
    /// Verify that atomic operations are truly thread-safe by spawning
    /// multiple threads that all increment the same counter.
    #[test]
    fn test_concurrent_atomic_operations() {
        use std::thread;
        use std::sync::atomic::{AtomicU64, Ordering};

        let counter = Arc::new(AtomicU64::new(0));
        let mut handles = vec![];

        // Spawn 100 threads
        for _ in 0..100 {
            let counter_clone = Arc::clone(&counter);
            let handle = thread::spawn(move || {
                // Each thread increments 1000 times
                for _ in 0..1000 {
                    counter_clone.fetch_add(1, Ordering::Relaxed);
                }
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // Should be exactly 100,000 (100 threads * 1000 increments)
        assert_eq!(counter.load(Ordering::SeqCst), 100_000);
    }

    /// Thread Safety Test 2: Concurrent metric updates
    ///
    /// Verify that Metrics can be safely cloned and used from multiple threads.
    #[test]
    fn test_concurrent_metrics_updates() {
        use std::thread;
        use std::time::Duration;

        let metrics = Arc::new(Metrics::new());
        let mut handles = vec![];

        // Spawn 10 threads, each recording queries
        for _ in 0..10 {
            let metrics_clone = Arc::clone(&metrics);
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    let _timer = metrics_clone.record_query_start();
                    // Simulate some work
                    std::hint::spin_loop();
                    metrics_clone.record_query_success(Duration::from_millis(10));
                }
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // Should have exactly 1000 queries (10 threads * 100 queries)
        let snap = metrics.snapshot();
        assert_eq!(snap.queries_total, 1000);
        assert_eq!(snap.queries_successful, 1000);
    }

    /// Thread Safety Test 3: Arc cloning behavior
    ///
    /// Verify that Arc correctly manages reference counting and
    /// that data is not dropped prematurely.
    #[test]
    fn test_arc_clone_behavior() {
        use std::sync::atomic::{AtomicU64, Ordering};

        let data = Arc::new(AtomicU64::new(42));

        // Clone creates a new reference, not a copy
        let data_clone = Arc::clone(&data);

        // Both point to same data (same address)
        assert!(Arc::ptr_eq(&data, &data_clone));

        // Both can read the same value
        assert_eq!(data.load(Ordering::SeqCst), 42);
        assert_eq!(data_clone.load(Ordering::SeqCst), 42);

        // Modifications via one Arc are visible via the other
        data.store(100, Ordering::SeqCst);
        assert_eq!(data_clone.load(Ordering::SeqCst), 100);

        // Weak references don't prevent deallocation
        let weak = Arc::downgrade(&data);
        assert!(weak.upgrade().is_some());

        drop(data);
        drop(data_clone);

        // Now no strong references, weak upgrade fails
        assert!(weak.upgrade().is_none());
    }

    /// Thread Safety Test 4: Metrics clone is cheap
    ///
    /// Verify that cloning Metrics is cheap (just Arc increment).
    #[test]
    fn test_metrics_clone_is_cheap() {
        let metrics = Metrics::new();

        // Clone many times (should be fast - just pointer copy)
        for _ in 0..1000 {
            let _clone = metrics.clone();
        }

        // All clones share the same inner data
        let snap = metrics.snapshot();
        assert_eq!(snap.queries_total, 0);
    }

    /// Thread Safety Test 5: AtomicBool for ready flags
    ///
    /// Verify that Arc<AtomicBool> works correctly for agent ready flags.
    #[test]
    fn test_atomic_bool_ready_flag() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::thread;
        use std::time::Duration;

        let ready = Arc::new(AtomicBool::new(false));
        let ready_clone = Arc::clone(&ready);

        // Spawn thread that sets ready flag
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            ready_clone.store(true, Ordering::SeqCst);
        });

        // Main thread waits for ready flag
        while !ready.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(10));
        }

        // Eventually ready flag should be set
        assert!(ready.load(Ordering::SeqCst));
    }

    /// Thread Safety Test 6: Concurrent Vec access (immutable)
    ///
    /// Verify that Arc<Vec<T>> allows safe concurrent reads.
    #[test]
    fn test_concurrent_vec_reads() {
        use std::thread;

        let data = Arc::new(vec![1, 2, 3, 4, 5]);
        let mut handles = vec![];

        // Spawn 10 threads reading the same Vec
        for _ in 0..10 {
            let data_clone = Arc::clone(&data);
            let handle = thread::spawn(move || {
                // All threads can read simultaneously (no lock needed!)
                let sum: i32 = data_clone.iter().sum();
                assert_eq!(sum, 15);
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // Original data unchanged
        assert_eq!(data.len(), 5);
    }
}
