//! キャッシュメトリクス
//!
//! キャッシュ操作の計測とパフォーマンス監視。

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// キャッシュ操作の種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheOperation {
    /// GET 操作
    Get,
    /// SET 操作
    Set,
    /// DELETE 操作
    Delete,
    /// EXISTS 操作
    Exists,
    /// INCR 操作
    Incr,
    /// DECR 操作
    Decr,
    /// HGET 操作
    HGet,
    /// HSET 操作
    HSet,
    /// HDEL 操作
    HDel,
    /// HGETALL 操作
    HGetAll,
    /// LPUSH 操作
    LPush,
    /// RPUSH 操作
    RPush,
    /// LPOP 操作
    LPop,
    /// RPOP 操作
    RPop,
    /// LRANGE 操作
    LRange,
    /// SADD 操作
    SAdd,
    /// SREM 操作
    SRem,
    /// SMEMBERS 操作
    SMembers,
}

impl CacheOperation {
    /// 操作名を取得
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Set => "SET",
            Self::Delete => "DELETE",
            Self::Exists => "EXISTS",
            Self::Incr => "INCR",
            Self::Decr => "DECR",
            Self::HGet => "HGET",
            Self::HSet => "HSET",
            Self::HDel => "HDEL",
            Self::HGetAll => "HGETALL",
            Self::LPush => "LPUSH",
            Self::RPush => "RPUSH",
            Self::LPop => "LPOP",
            Self::RPop => "RPOP",
            Self::LRange => "LRANGE",
            Self::SAdd => "SADD",
            Self::SRem => "SREM",
            Self::SMembers => "SMEMBERS",
        }
    }
}

/// キャッシュメトリクス
#[derive(Debug)]
pub struct CacheMetrics {
    /// ヒット数
    hits: AtomicU64,
    /// ミス数
    misses: AtomicU64,
    /// 操作数
    operations: AtomicU64,
    /// エラー数
    errors: AtomicU64,
    /// 作成時刻
    created_at: Instant,
}

impl Default for CacheMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl CacheMetrics {
    /// 新しいメトリクスを作成
    pub fn new() -> Self {
        Self {
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            operations: AtomicU64::new(0),
            errors: AtomicU64::new(0),
            created_at: Instant::now(),
        }
    }

    /// ヒットを記録
    pub fn record_hit(&self, _operation: CacheOperation) {
        self.hits.fetch_add(1, Ordering::Relaxed);
        self.operations.fetch_add(1, Ordering::Relaxed);
    }

    /// ミスを記録
    pub fn record_miss(&self, _operation: CacheOperation) {
        self.misses.fetch_add(1, Ordering::Relaxed);
        self.operations.fetch_add(1, Ordering::Relaxed);
    }

    /// 操作を記録
    pub fn record_operation(&self, _operation: CacheOperation) {
        self.operations.fetch_add(1, Ordering::Relaxed);
    }

    /// 複数の操作を記録
    pub fn record_operations(&self, _operation: CacheOperation, count: u64) {
        self.operations.fetch_add(count, Ordering::Relaxed);
    }

    /// エラーを記録
    pub fn record_error(&self, _operation: CacheOperation) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    /// ヒット数を取得
    pub fn hits(&self) -> u64 {
        self.hits.load(Ordering::Relaxed)
    }

    /// ミス数を取得
    pub fn misses(&self) -> u64 {
        self.misses.load(Ordering::Relaxed)
    }

    /// 操作数を取得
    pub fn operations(&self) -> u64 {
        self.operations.load(Ordering::Relaxed)
    }

    /// エラー数を取得
    pub fn errors(&self) -> u64 {
        self.errors.load(Ordering::Relaxed)
    }

    /// ヒット率を取得（0.0 - 1.0）
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits() as f64;
        let total = hits + self.misses() as f64;
        if total == 0.0 {
            0.0
        } else {
            hits / total
        }
    }

    /// ミス率を取得（0.0 - 1.0）
    pub fn miss_rate(&self) -> f64 {
        1.0 - self.hit_rate()
    }

    /// 稼働時間を取得
    pub fn uptime(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// 秒あたりの操作数を取得
    pub fn operations_per_second(&self) -> f64 {
        let operations = self.operations() as f64;
        let seconds = self.uptime().as_secs_f64();
        if seconds == 0.0 {
            0.0
        } else {
            operations / seconds
        }
    }

    /// メトリクスをリセット
    pub fn reset(&self) {
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
        self.operations.store(0, Ordering::Relaxed);
        self.errors.store(0, Ordering::Relaxed);
    }

    /// メトリクスのスナップショットを取得
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            hits: self.hits(),
            misses: self.misses(),
            operations: self.operations(),
            errors: self.errors(),
            hit_rate: self.hit_rate(),
            uptime_secs: self.uptime().as_secs(),
            ops_per_second: self.operations_per_second(),
        }
    }
}

/// メトリクスのスナップショット
#[derive(Debug, Clone, serde::Serialize)]
pub struct MetricsSnapshot {
    /// ヒット数
    pub hits: u64,
    /// ミス数
    pub misses: u64,
    /// 操作数
    pub operations: u64,
    /// エラー数
    pub errors: u64,
    /// ヒット率
    pub hit_rate: f64,
    /// 稼働時間（秒）
    pub uptime_secs: u64,
    /// 秒あたりの操作数
    pub ops_per_second: f64,
}

/// 操作タイマー
///
/// 操作の所要時間を計測する。
pub struct OperationTimer {
    operation: CacheOperation,
    start: Instant,
}

impl OperationTimer {
    /// タイマーを開始
    pub fn start(operation: CacheOperation) -> Self {
        Self {
            operation,
            start: Instant::now(),
        }
    }

    /// 所要時間を取得
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// 操作種別を取得
    pub fn operation(&self) -> CacheOperation {
        self.operation
    }

    /// タイマーを終了してメトリクスに記録
    pub fn finish(self, metrics: &CacheMetrics, success: bool) -> Duration {
        let elapsed = self.elapsed();
        if success {
            metrics.record_operation(self.operation);
        } else {
            metrics.record_error(self.operation);
        }
        elapsed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_operation() {
        assert_eq!(CacheOperation::Get.as_str(), "GET");
        assert_eq!(CacheOperation::Set.as_str(), "SET");
        assert_eq!(CacheOperation::Delete.as_str(), "DELETE");
    }

    #[test]
    fn test_cache_metrics() {
        let metrics = CacheMetrics::new();

        // Initial state
        assert_eq!(metrics.hits(), 0);
        assert_eq!(metrics.misses(), 0);
        assert_eq!(metrics.operations(), 0);

        // Record hits and misses
        metrics.record_hit(CacheOperation::Get);
        metrics.record_hit(CacheOperation::Get);
        metrics.record_miss(CacheOperation::Get);

        assert_eq!(metrics.hits(), 2);
        assert_eq!(metrics.misses(), 1);
        assert_eq!(metrics.operations(), 3);

        // Hit rate
        let rate = metrics.hit_rate();
        assert!((rate - 0.666666).abs() < 0.01);
    }

    #[test]
    fn test_hit_rate_no_operations() {
        let metrics = CacheMetrics::new();
        assert_eq!(metrics.hit_rate(), 0.0);
    }

    #[test]
    fn test_metrics_reset() {
        let metrics = CacheMetrics::new();

        metrics.record_hit(CacheOperation::Get);
        metrics.record_error(CacheOperation::Set);

        assert!(metrics.hits() > 0);
        assert!(metrics.errors() > 0);

        metrics.reset();

        assert_eq!(metrics.hits(), 0);
        assert_eq!(metrics.misses(), 0);
        assert_eq!(metrics.operations(), 0);
        assert_eq!(metrics.errors(), 0);
    }

    #[test]
    fn test_metrics_snapshot() {
        let metrics = CacheMetrics::new();

        metrics.record_hit(CacheOperation::Get);
        metrics.record_miss(CacheOperation::Get);

        let snapshot = metrics.snapshot();

        assert_eq!(snapshot.hits, 1);
        assert_eq!(snapshot.misses, 1);
        assert_eq!(snapshot.operations, 2);
        assert!((snapshot.hit_rate - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_operation_timer() {
        let timer = OperationTimer::start(CacheOperation::Get);
        assert_eq!(timer.operation(), CacheOperation::Get);

        // Some time passes
        std::thread::sleep(std::time::Duration::from_millis(10));

        assert!(timer.elapsed().as_millis() >= 10);
    }
}
