//! リーダー選出の Prometheus メトリクス。

use prometheus::{Counter, Gauge, Histogram, HistogramOpts, Opts};
use std::sync::OnceLock;

static ELECTIONS_TOTAL: OnceLock<Counter> = OnceLock::new();
static IS_LEADER: OnceLock<Gauge> = OnceLock::new();
static LEASE_DURATION: OnceLock<Histogram> = OnceLock::new();
static HEARTBEAT_FAILURES: OnceLock<Counter> = OnceLock::new();

/// リーダー選出試行回数。
///
/// # Panics
///
/// メトリクスの初回作成に失敗した場合にパニックする（通常発生しない）。
#[must_use]
pub fn elections_total() -> &'static Counter {
    ELECTIONS_TOTAL.get_or_init(|| {
        Counter::with_opts(Opts::new(
            "k1s0_leader_elections_total",
            "Total number of leader election attempts",
        ))
        .expect("failed to create elections_total counter")
    })
}

/// 自ノードがリーダーかどうか（1.0 = リーダー, 0.0 = フォロワー）。
///
/// # Panics
///
/// メトリクスの初回作成に失敗した場合にパニックする（通常発生しない）。
#[must_use]
pub fn is_leader() -> &'static Gauge {
    IS_LEADER.get_or_init(|| {
        Gauge::with_opts(Opts::new(
            "k1s0_leader_is_leader",
            "Whether this node is currently the leader (1=leader, 0=follower)",
        ))
        .expect("failed to create is_leader gauge")
    })
}

/// リース期間のヒストグラム。
///
/// # Panics
///
/// メトリクスの初回作成に失敗した場合にパニックする（通常発生しない）。
#[must_use]
pub fn lease_duration() -> &'static Histogram {
    LEASE_DURATION.get_or_init(|| {
        Histogram::with_opts(HistogramOpts::new(
            "k1s0_leader_lease_duration_seconds",
            "Duration of leader lease in seconds",
        ))
        .expect("failed to create lease_duration histogram")
    })
}

/// ハートビート失敗回数。
///
/// # Panics
///
/// メトリクスの初回作成に失敗した場合にパニックする（通常発生しない）。
#[must_use]
pub fn heartbeat_failures() -> &'static Counter {
    HEARTBEAT_FAILURES.get_or_init(|| {
        Counter::with_opts(Opts::new(
            "k1s0_leader_heartbeat_failures_total",
            "Total number of heartbeat renewal failures",
        ))
        .expect("failed to create heartbeat_failures counter")
    })
}

/// リーダー選出メトリクスのヘルパー。
pub struct LeaderMetrics {
    _private: (),
}

impl LeaderMetrics {
    /// 新しいメトリクスインスタンスを作成する。
    #[must_use]
    pub fn new() -> Self {
        // メトリクスを初期化
        let _ = elections_total();
        let _ = is_leader();
        let _ = lease_duration();
        let _ = heartbeat_failures();
        Self { _private: () }
    }

    /// 選出結果を記録する。
    pub fn record_election(&self, elected: bool) {
        elections_total().inc();
        if elected {
            is_leader().set(1.0);
        } else {
            is_leader().set(0.0);
        }
    }

    /// リース期間を記録する。
    pub fn observe_lease_duration(&self, duration_secs: f64) {
        lease_duration().observe(duration_secs);
    }
}

impl Default for LeaderMetrics {
    fn default() -> Self {
        Self::new()
    }
}
