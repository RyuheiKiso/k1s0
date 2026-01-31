//! Saga の Prometheus メトリクス。

use prometheus::{Counter, Histogram, HistogramOpts, Opts};
use std::sync::OnceLock;

static SAGA_STARTED: OnceLock<Counter> = OnceLock::new();
static SAGA_COMPLETED: OnceLock<Counter> = OnceLock::new();
static SAGA_FAILED: OnceLock<Counter> = OnceLock::new();
static SAGA_DEAD_LETTER: OnceLock<Counter> = OnceLock::new();
static SAGA_DURATION: OnceLock<Histogram> = OnceLock::new();

/// Saga 開始回数。
///
/// # Panics
///
/// メトリクスの初回作成に失敗した場合にパニックする（通常発生しない）。
#[must_use]
pub fn saga_started() -> &'static Counter {
    SAGA_STARTED.get_or_init(|| {
        Counter::with_opts(Opts::new(
            "k1s0_saga_started_total",
            "Total number of sagas started",
        ))
        .expect("failed to create saga_started counter")
    })
}

/// Saga 完了回数。
///
/// # Panics
///
/// メトリクスの初回作成に失敗した場合にパニックする（通常発生しない）。
#[must_use]
pub fn saga_completed() -> &'static Counter {
    SAGA_COMPLETED.get_or_init(|| {
        Counter::with_opts(Opts::new(
            "k1s0_saga_completed_total",
            "Total number of sagas completed successfully",
        ))
        .expect("failed to create saga_completed counter")
    })
}

/// Saga 失敗回数（補償成功含む）。
///
/// # Panics
///
/// メトリクスの初回作成に失敗した場合にパニックする（通常発生しない）。
#[must_use]
pub fn saga_failed() -> &'static Counter {
    SAGA_FAILED.get_or_init(|| {
        Counter::with_opts(Opts::new(
            "k1s0_saga_failed_total",
            "Total number of sagas that failed (including compensated)",
        ))
        .expect("failed to create saga_failed counter")
    })
}

/// デッドレター送信回数。
///
/// # Panics
///
/// メトリクスの初回作成に失敗した場合にパニックする（通常発生しない）。
#[must_use]
pub fn saga_dead_letter() -> &'static Counter {
    SAGA_DEAD_LETTER.get_or_init(|| {
        Counter::with_opts(Opts::new(
            "k1s0_saga_dead_letter_total",
            "Total number of sagas moved to dead letter queue",
        ))
        .expect("failed to create saga_dead_letter counter")
    })
}

/// Saga 実行時間のヒストグラム。
///
/// # Panics
///
/// メトリクスの初回作成に失敗した場合にパニックする（通常発生しない）。
#[must_use]
pub fn saga_duration() -> &'static Histogram {
    SAGA_DURATION.get_or_init(|| {
        Histogram::with_opts(HistogramOpts::new(
            "k1s0_saga_duration_seconds",
            "Duration of saga execution in seconds",
        ))
        .expect("failed to create saga_duration histogram")
    })
}
