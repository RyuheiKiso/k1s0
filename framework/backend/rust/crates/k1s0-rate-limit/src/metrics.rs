//! レートリミットメトリクス
//!
//! アトミック操作によるロックフリーなカウンタを提供する。

use std::sync::atomic::{AtomicU64, Ordering};

/// レートリミットのメトリクス
///
/// 許可・拒否・合計のリクエスト数をアトミックに追跡する。
#[derive(Debug)]
pub struct RateLimitMetrics {
    /// 許可されたリクエスト数
    allowed: AtomicU64,
    /// 拒否されたリクエスト数
    rejected: AtomicU64,
    /// 合計リクエスト数
    total: AtomicU64,
}

impl RateLimitMetrics {
    /// 新しいメトリクスインスタンスを生成する
    #[must_use]
    pub fn new() -> Self {
        Self {
            allowed: AtomicU64::new(0),
            rejected: AtomicU64::new(0),
            total: AtomicU64::new(0),
        }
    }

    /// 許可カウンタをインクリメントする
    pub fn increment_allowed(&self) {
        self.allowed.fetch_add(1, Ordering::Relaxed);
        self.total.fetch_add(1, Ordering::Relaxed);
    }

    /// 拒否カウンタをインクリメントする
    pub fn increment_rejected(&self) {
        self.rejected.fetch_add(1, Ordering::Relaxed);
        self.total.fetch_add(1, Ordering::Relaxed);
    }

    /// 許可されたリクエスト数を返す
    #[must_use]
    pub fn allowed(&self) -> u64 {
        self.allowed.load(Ordering::Relaxed)
    }

    /// 拒否されたリクエスト数を返す
    #[must_use]
    pub fn rejected(&self) -> u64 {
        self.rejected.load(Ordering::Relaxed)
    }

    /// 合計リクエスト数を返す
    #[must_use]
    pub fn total(&self) -> u64 {
        self.total.load(Ordering::Relaxed)
    }

    /// メトリクスをリセットする
    pub fn reset(&self) {
        self.allowed.store(0, Ordering::Relaxed);
        self.rejected.store(0, Ordering::Relaxed);
        self.total.store(0, Ordering::Relaxed);
    }
}

impl Default for RateLimitMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_values() {
        let metrics = RateLimitMetrics::new();
        assert_eq!(metrics.allowed(), 0);
        assert_eq!(metrics.rejected(), 0);
        assert_eq!(metrics.total(), 0);
    }

    #[test]
    fn test_increment_allowed() {
        let metrics = RateLimitMetrics::new();
        metrics.increment_allowed();
        metrics.increment_allowed();
        assert_eq!(metrics.allowed(), 2);
        assert_eq!(metrics.rejected(), 0);
        assert_eq!(metrics.total(), 2);
    }

    #[test]
    fn test_increment_rejected() {
        let metrics = RateLimitMetrics::new();
        metrics.increment_rejected();
        assert_eq!(metrics.allowed(), 0);
        assert_eq!(metrics.rejected(), 1);
        assert_eq!(metrics.total(), 1);
    }

    #[test]
    fn test_mixed_increments() {
        let metrics = RateLimitMetrics::new();
        metrics.increment_allowed();
        metrics.increment_rejected();
        metrics.increment_allowed();
        assert_eq!(metrics.allowed(), 2);
        assert_eq!(metrics.rejected(), 1);
        assert_eq!(metrics.total(), 3);
    }

    #[test]
    fn test_reset() {
        let metrics = RateLimitMetrics::new();
        metrics.increment_allowed();
        metrics.increment_rejected();
        metrics.reset();
        assert_eq!(metrics.allowed(), 0);
        assert_eq!(metrics.rejected(), 0);
        assert_eq!(metrics.total(), 0);
    }
}
