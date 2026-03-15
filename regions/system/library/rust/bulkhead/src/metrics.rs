use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(feature = "metrics")]
use opentelemetry::metrics::{Counter, UpDownCounter};

#[derive(Debug, Clone)]
pub struct BulkheadMetrics {
    pub rejection_count: u64,
    pub current_concurrent: u64,
}

#[derive(Debug)]
pub(crate) struct BulkheadMetricsRecorder {
    rejection_count: AtomicU64,
    current_concurrent: AtomicU64,
    #[cfg(feature = "metrics")]
    otel_rejection_counter: Counter<u64>,
    #[cfg(feature = "metrics")]
    otel_concurrent_gauge: UpDownCounter<i64>,
}

impl Default for BulkheadMetricsRecorder {
    fn default() -> Self {
        Self::new()
    }
}

impl BulkheadMetricsRecorder {
    pub fn new() -> Self {
        #[cfg(feature = "metrics")]
        let meter = opentelemetry::global::meter("k1s0.bulkhead");

        Self {
            rejection_count: AtomicU64::new(0),
            current_concurrent: AtomicU64::new(0),
            #[cfg(feature = "metrics")]
            otel_rejection_counter: meter.u64_counter("bulkhead_rejections_total").build(),
            #[cfg(feature = "metrics")]
            otel_concurrent_gauge: meter
                .i64_up_down_counter("bulkhead_concurrent_calls")
                .build(),
        }
    }

    pub fn record_rejection(&self) {
        self.rejection_count.fetch_add(1, Ordering::Relaxed);
        #[cfg(feature = "metrics")]
        self.otel_rejection_counter.add(1, &[]);
    }

    pub fn record_acquire(&self) {
        self.current_concurrent.fetch_add(1, Ordering::Relaxed);
        #[cfg(feature = "metrics")]
        self.otel_concurrent_gauge.add(1, &[]);
    }

    pub fn record_release(&self) {
        self.current_concurrent.fetch_sub(1, Ordering::Relaxed);
        #[cfg(feature = "metrics")]
        self.otel_concurrent_gauge.add(-1, &[]);
    }

    pub fn snapshot(&self) -> BulkheadMetrics {
        BulkheadMetrics {
            rejection_count: self.rejection_count.load(Ordering::Relaxed),
            current_concurrent: self.current_concurrent.load(Ordering::Relaxed),
        }
    }
}
