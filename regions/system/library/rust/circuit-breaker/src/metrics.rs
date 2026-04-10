use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};

use crate::breaker::CircuitBreakerState;

#[cfg(feature = "metrics")]
use opentelemetry::metrics::{Counter, UpDownCounter};

#[derive(Debug, Clone)]
pub struct CircuitBreakerMetrics {
    pub failure_count: u32,
    pub success_count: u32,
    pub state: String,
}

#[derive(Debug)]
pub(crate) struct CircuitBreakerMetricsRecorder {
    failure_count: AtomicU64,
    success_count: AtomicU64,
    state_code: AtomicI64,
    #[cfg(feature = "metrics")]
    otel_failure_counter: Counter<u64>,
    #[cfg(feature = "metrics")]
    otel_success_counter: Counter<u64>,
    #[cfg(feature = "metrics")]
    otel_state_gauge: UpDownCounter<i64>,
}

impl Default for CircuitBreakerMetricsRecorder {
    fn default() -> Self {
        Self::new()
    }
}

impl CircuitBreakerMetricsRecorder {
    pub fn new() -> Self {
        #[cfg(feature = "metrics")]
        let meter = opentelemetry::global::meter("k1s0.circuit-breaker");

        Self {
            failure_count: AtomicU64::new(0),
            success_count: AtomicU64::new(0),
            state_code: AtomicI64::new(state_to_code(CircuitBreakerState::Closed)),
            #[cfg(feature = "metrics")]
            otel_failure_counter: meter.u64_counter("circuit_breaker_failures_total").build(),
            #[cfg(feature = "metrics")]
            otel_success_counter: meter.u64_counter("circuit_breaker_successes_total").build(),
            #[cfg(feature = "metrics")]
            otel_state_gauge: meter.i64_up_down_counter("circuit_breaker_state").build(),
        }
    }

    pub fn record_failure(&self) {
        self.failure_count.fetch_add(1, Ordering::Relaxed);
        #[cfg(feature = "metrics")]
        self.otel_failure_counter.add(1, &[]);
    }

    pub fn record_success(&self) {
        self.success_count.fetch_add(1, Ordering::Relaxed);
        #[cfg(feature = "metrics")]
        self.otel_success_counter.add(1, &[]);
    }

    pub fn set_state(&self, state: CircuitBreakerState) {
        let next = state_to_code(state);
        let _prev = self.state_code.swap(next, Ordering::Relaxed);

        #[cfg(feature = "metrics")]
        if _prev != next {
            self.otel_state_gauge.add(next - _prev, &[]);
        }
    }

    pub fn snapshot(&self) -> CircuitBreakerMetrics {
        // HIGH-001 監査対応: u64→u32 の切り捨て変換をセーフキャストに変更する
        // カウンタが u32::MAX（約43億）を超える場合は上限値に固定する
        CircuitBreakerMetrics {
            failure_count: u32::try_from(self.failure_count.load(Ordering::Relaxed))
                .unwrap_or(u32::MAX),
            success_count: u32::try_from(self.success_count.load(Ordering::Relaxed))
                .unwrap_or(u32::MAX),
            state: code_to_state(self.state_code.load(Ordering::Relaxed)).to_string(),
        }
    }
}

fn state_to_code(state: CircuitBreakerState) -> i64 {
    match state {
        CircuitBreakerState::Closed => 0,
        CircuitBreakerState::Open => 1,
        CircuitBreakerState::HalfOpen => 2,
    }
}

fn code_to_state(code: i64) -> &'static str {
    match code {
        1 => "Open",
        2 => "HalfOpen",
        _ => "Closed",
    }
}
