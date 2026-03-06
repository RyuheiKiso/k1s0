use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};

#[cfg(feature = "metrics")]
use opentelemetry::metrics::{Counter, UpDownCounter};

#[derive(Debug)]
pub struct ResiliencyMetrics {
    retry_attempts: AtomicU64,
    circuit_open_events: AtomicU64,
    bulkhead_rejections: AtomicU64,
    timeout_events: AtomicU64,
    circuit_state: AtomicI64,
    #[cfg(feature = "metrics")]
    otel_retry_attempts: Counter<u64>,
    #[cfg(feature = "metrics")]
    otel_circuit_open_events: Counter<u64>,
    #[cfg(feature = "metrics")]
    otel_bulkhead_rejections: Counter<u64>,
    #[cfg(feature = "metrics")]
    otel_timeout_events: Counter<u64>,
    #[cfg(feature = "metrics")]
    otel_circuit_state: UpDownCounter<i64>,
}

impl Default for ResiliencyMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl ResiliencyMetrics {
    pub fn new() -> Self {
        #[cfg(feature = "metrics")]
        let meter = opentelemetry::global::meter("k1s0.resiliency");

        Self {
            retry_attempts: AtomicU64::new(0),
            circuit_open_events: AtomicU64::new(0),
            bulkhead_rejections: AtomicU64::new(0),
            timeout_events: AtomicU64::new(0),
            circuit_state: AtomicI64::new(0),
            #[cfg(feature = "metrics")]
            otel_retry_attempts: meter.u64_counter("resiliency_retry_attempts_total").build(),
            #[cfg(feature = "metrics")]
            otel_circuit_open_events: meter
                .u64_counter("resiliency_circuit_open_events_total")
                .build(),
            #[cfg(feature = "metrics")]
            otel_bulkhead_rejections: meter
                .u64_counter("resiliency_bulkhead_rejections_total")
                .build(),
            #[cfg(feature = "metrics")]
            otel_timeout_events: meter.u64_counter("resiliency_timeout_events_total").build(),
            #[cfg(feature = "metrics")]
            otel_circuit_state: meter
                .i64_up_down_counter("resiliency_circuit_state")
                .build(),
        }
    }

    pub fn record_retry_attempt(&self) {
        self.retry_attempts.fetch_add(1, Ordering::Relaxed);
        #[cfg(feature = "metrics")]
        self.otel_retry_attempts.add(1, &[]);
    }

    pub fn record_circuit_open(&self) {
        self.circuit_open_events.fetch_add(1, Ordering::Relaxed);
        #[cfg(feature = "metrics")]
        self.otel_circuit_open_events.add(1, &[]);
    }

    pub fn record_bulkhead_rejection(&self) {
        self.bulkhead_rejections.fetch_add(1, Ordering::Relaxed);
        #[cfg(feature = "metrics")]
        self.otel_bulkhead_rejections.add(1, &[]);
    }

    pub fn record_timeout(&self) {
        self.timeout_events.fetch_add(1, Ordering::Relaxed);
        #[cfg(feature = "metrics")]
        self.otel_timeout_events.add(1, &[]);
    }

    pub fn set_circuit_closed(&self) {
        self.set_circuit_state(0);
    }

    pub fn set_circuit_open(&self) {
        self.set_circuit_state(1);
    }

    pub fn set_circuit_half_open(&self) {
        self.set_circuit_state(2);
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

    pub fn circuit_state_code(&self) -> i64 {
        self.circuit_state.load(Ordering::Relaxed)
    }

    fn set_circuit_state(&self, next: i64) {
        let _prev = self.circuit_state.swap(next, Ordering::Relaxed);

        #[cfg(feature = "metrics")]
        if _prev != next {
            self.otel_circuit_state.add(next - _prev, &[]);
        }
    }
}
