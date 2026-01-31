//! トークンバケット レートリミッター
//!
//! ロックフリーな `AtomicU64` ベースのトークンバケットアルゴリズムを提供する。
//! サブトークン精度のために内部では `tokens * 1_000_000` の単位で管理する。

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use crate::error::RateLimitError;
use crate::metrics::RateLimitMetrics;

/// トークン精度の乗数（マイクロトークン単位）
const PRECISION: u64 = 1_000_000;

/// レートリミッターのトレイト
///
/// トークンバケットとスライディングウィンドウの共通インターフェース。
pub trait RateLimiter: Send + Sync {
    /// トークンの取得を試みる
    ///
    /// # Errors
    ///
    /// レートリミットを超過した場合に `RateLimitError::Exceeded` を返す。
    fn try_acquire(&self) -> Result<(), RateLimitError>;

    /// 次のトークンが利用可能になるまでの時間を返す
    fn time_until_available(&self) -> Duration;

    /// 現在利用可能なトークン数を返す
    fn available_tokens(&self) -> u64;
}

/// トークンバケット レートリミッター
///
/// 指定容量のバケットから一定レートでトークンを補充する。
/// CASループにより高い並行性能を実現する。
pub struct TokenBucket {
    /// バケット容量
    capacity: u64,
    /// 現在のトークン数（マイクロトークン単位）
    tokens: AtomicU64,
    /// 1秒あたりの補充トークン数
    refill_rate: f64,
    /// 最終補充時刻（`Instant::now()` からのナノ秒）
    last_refill_nanos: AtomicU64,
    /// 基準時刻
    epoch: tokio::time::Instant,
    /// メトリクス
    metrics: RateLimitMetrics,
}

impl TokenBucket {
    /// 新しいトークンバケットを生成する
    ///
    /// # 引数
    ///
    /// * `capacity` - バケットの最大トークン数
    /// * `refill_rate` - 1秒あたりの補充トークン数
    #[must_use]
    pub fn new(capacity: u64, refill_rate: f64) -> Self {
        let now = tokio::time::Instant::now();
        Self {
            capacity,
            tokens: AtomicU64::new(capacity * PRECISION),
            refill_rate,
            last_refill_nanos: AtomicU64::new(0),
            epoch: now,
            metrics: RateLimitMetrics::new(),
        }
    }

    /// メトリクスへの参照を返す
    #[must_use]
    pub fn metrics(&self) -> &RateLimitMetrics {
        &self.metrics
    }

    /// トークンを補充する
    fn refill(&self) {
        #[allow(clippy::cast_possible_truncation)]
        let now_nanos = self.epoch.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64;
        let prev_nanos = self.last_refill_nanos.load(Ordering::SeqCst);

        if now_nanos <= prev_nanos {
            return;
        }

        #[allow(clippy::cast_precision_loss)]
        let elapsed_secs = (now_nanos - prev_nanos) as f64 / 1_000_000_000.0;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_precision_loss)]
        let new_tokens = (elapsed_secs * self.refill_rate * PRECISION as f64) as u64;

        if new_tokens == 0 {
            return;
        }

        // CASループで last_refill_nanos を更新
        if self
            .last_refill_nanos
            .compare_exchange(prev_nanos, now_nanos, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            let max_tokens = self.capacity * PRECISION;
            // CASループでトークンを追加
            loop {
                let current = self.tokens.load(Ordering::SeqCst);
                let updated = (current + new_tokens).min(max_tokens);
                if self
                    .tokens
                    .compare_exchange(current, updated, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok()
                {
                    break;
                }
            }
        }
    }
}

impl RateLimiter for TokenBucket {
    fn try_acquire(&self) -> Result<(), RateLimitError> {
        self.refill();

        loop {
            let current = self.tokens.load(Ordering::SeqCst);
            if current < PRECISION {
                self.metrics.increment_rejected();
                let wait = Duration::from_secs_f64(1.0 / self.refill_rate);
                return Err(RateLimitError::exceeded(wait));
            }
            if self
                .tokens
                .compare_exchange(
                    current,
                    current - PRECISION,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                )
                .is_ok()
            {
                self.metrics.increment_allowed();
                return Ok(());
            }
        }
    }

    fn time_until_available(&self) -> Duration {
        self.refill();
        let current = self.tokens.load(Ordering::SeqCst);
        if current >= PRECISION {
            Duration::ZERO
        } else {
            let deficit = PRECISION - current;
            #[allow(clippy::cast_precision_loss)]
            let secs = deficit as f64 / (self.refill_rate * PRECISION as f64);
            Duration::from_secs_f64(secs)
        }
    }

    fn available_tokens(&self) -> u64 {
        self.refill();
        self.tokens.load(Ordering::SeqCst) / PRECISION
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_acquire_within_limit() {
        let bucket = TokenBucket::new(10, 10.0);
        for _ in 0..10 {
            assert!(bucket.try_acquire().is_ok());
        }
        assert_eq!(bucket.metrics().allowed(), 10);
    }

    #[tokio::test]
    async fn test_acquire_exceeds_limit() {
        let bucket = TokenBucket::new(5, 10.0);
        for _ in 0..5 {
            assert!(bucket.try_acquire().is_ok());
        }
        let err = bucket.try_acquire().unwrap_err();
        assert!(err.is_retryable());
        assert_eq!(err.error_code(), "RATE_LIMIT_EXCEEDED");
        assert_eq!(bucket.metrics().rejected(), 1);
    }

    #[tokio::test(start_paused = true)]
    async fn test_refill_over_time() {
        let bucket = TokenBucket::new(5, 5.0);

        // 全トークンを消費
        for _ in 0..5 {
            assert!(bucket.try_acquire().is_ok());
        }
        assert!(bucket.try_acquire().is_err());

        // 1秒進める → 5トークン補充
        tokio::time::advance(Duration::from_secs(1)).await;

        assert_eq!(bucket.available_tokens(), 5);
        assert!(bucket.try_acquire().is_ok());
    }

    #[tokio::test(start_paused = true)]
    async fn test_burst_capacity() {
        let bucket = TokenBucket::new(100, 10.0);

        // バースト：一度に100トークン使用可能
        for _ in 0..100 {
            assert!(bucket.try_acquire().is_ok());
        }
        assert!(bucket.try_acquire().is_err());

        // 容量を超えて補充されないことを確認
        tokio::time::advance(Duration::from_secs(20)).await;
        assert_eq!(bucket.available_tokens(), 100);
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let bucket = Arc::new(TokenBucket::new(100, 1000.0));
        let mut handles = Vec::new();

        for _ in 0..10 {
            let b = Arc::clone(&bucket);
            handles.push(tokio::spawn(async move {
                let mut ok_count = 0u64;
                for _ in 0..10 {
                    if b.try_acquire().is_ok() {
                        ok_count += 1;
                    }
                }
                ok_count
            }));
        }

        let mut total_ok = 0u64;
        for h in handles {
            total_ok += h.await.unwrap();
        }

        // 合計は容量以下であるべき（補充もあるので100以上の場合もある）
        assert!(total_ok <= 110, "total_ok={total_ok} is too high");
        assert_eq!(
            bucket.metrics().allowed() + bucket.metrics().rejected(),
            bucket.metrics().total()
        );
    }

    #[tokio::test(start_paused = true)]
    async fn test_time_until_available() {
        let bucket = TokenBucket::new(1, 1.0);
        assert_eq!(bucket.time_until_available(), Duration::ZERO);

        bucket.try_acquire().unwrap();
        let wait = bucket.time_until_available();
        assert!(wait > Duration::ZERO);
    }

    #[tokio::test]
    async fn test_available_tokens_initial() {
        let bucket = TokenBucket::new(50, 10.0);
        assert_eq!(bucket.available_tokens(), 50);
    }
}
