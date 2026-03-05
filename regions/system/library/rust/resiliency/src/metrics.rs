use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Default)]
pub struct ResiliencyMetrics {
    retry_attempts: AtomicU64,
    circuit_open_events: AtomicU64,
    bulkhead_rejections: AtomicU64,
    timeout_events: AtomicU64,
}

impl ResiliencyMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_retry_attempt(&self) {
        self.retry_attempts.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_circuit_open(&self) {
        self.circuit_open_events.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_bulkhead_rejection(&self) {
        self.bulkhead_rejections.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_timeout(&self) {
        self.timeout_events.fetch_add(1, Ordering::Relaxed);
    }

    pub fn retry_attempts(&self) -> u64 {
        self.retry_attempts.load(Ordering::Relaxed)
    }

    pub fn circuit_open_events(&self) -> u64 {
        self.circuit_open_events.load(Ordering::Relaxed)
    }

    pub fn bulkhead_rejections(&self) -> u64 {
        self.bulkhead_rejections.load(Ordering::Relaxed)
    }

    pub fn timeout_events(&self) -> u64 {
        self.timeout_events.load(Ordering::Relaxed)
    }
}
