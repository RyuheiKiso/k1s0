//! 分散ロックの Prometheus メトリクス。

use prometheus::{Counter, Histogram, HistogramOpts, Opts};
use std::sync::OnceLock;

static LOCK_ACQUISITIONS: OnceLock<Counter> = OnceLock::new();
static LOCK_WAIT_DURATION: OnceLock<Histogram> = OnceLock::new();
static LOCK_HOLD_DURATION: OnceLock<Histogram> = OnceLock::new();
static LOCK_TIMEOUTS: OnceLock<Counter> = OnceLock::new();

/// ロック取得回数。
///
/// # Panics
///
/// メトリクスの初回作成に失敗した場合にパニックする（通常発生しない）。
#[must_use]
pub fn lock_acquisitions() -> &'static Counter {
    LOCK_ACQUISITIONS.get_or_init(|| {
        Counter::with_opts(Opts::new(
            "k1s0_lock_acquisitions_total",
            "Total number of successful lock acquisitions",
        ))
        .expect("failed to create lock_acquisitions counter")
    })
}

/// ロック待機時間のヒストグラム。
///
/// # Panics
///
/// メトリクスの初回作成に失敗した場合にパニックする（通常発生しない）。
#[must_use]
pub fn lock_wait_duration() -> &'static Histogram {
    LOCK_WAIT_DURATION.get_or_init(|| {
        Histogram::with_opts(HistogramOpts::new(
            "k1s0_lock_wait_duration_seconds",
            "Time spent waiting to acquire a lock",
        ))
        .expect("failed to create lock_wait_duration histogram")
    })
}

/// ロック保持時間のヒストグラム。
///
/// # Panics
///
/// メトリクスの初回作成に失敗した場合にパニックする（通常発生しない）。
#[must_use]
pub fn lock_hold_duration() -> &'static Histogram {
    LOCK_HOLD_DURATION.get_or_init(|| {
        Histogram::with_opts(HistogramOpts::new(
            "k1s0_lock_hold_duration_seconds",
            "Time a lock was held before release",
        ))
        .expect("failed to create lock_hold_duration histogram")
    })
}

/// ロックタイムアウト回数。
///
/// # Panics
///
/// メトリクスの初回作成に失敗した場合にパニックする（通常発生しない）。
#[must_use]
pub fn lock_timeouts() -> &'static Counter {
    LOCK_TIMEOUTS.get_or_init(|| {
        Counter::with_opts(Opts::new(
            "k1s0_lock_timeouts_total",
            "Total number of lock acquisition timeouts",
        ))
        .expect("failed to create lock_timeouts counter")
    })
}
