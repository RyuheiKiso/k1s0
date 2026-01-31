//! スライディングウィンドウ レートリミッター
//!
//! 固定時間ウィンドウ内のリクエスト数を制限する。
//! `Mutex<VecDeque>` でタイムスタンプを管理し、期限切れエントリを自動削除する。

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::Duration;

use crate::error::RateLimitError;
use crate::metrics::RateLimitMetrics;
use crate::token_bucket::RateLimiter;

/// スライディングウィンドウ レートリミッター
///
/// ウィンドウ期間内のリクエスト数を追跡し、上限を超えた場合に拒否する。
pub struct SlidingWindow {
    /// ウィンドウサイズ
    window_size: Duration,
    /// ウィンドウ内の最大リクエスト数
    max_requests: u64,
    /// リクエストのタイムスタンプ（基準時刻からの経過時間）
    timestamps: Mutex<VecDeque<tokio::time::Instant>>,
    /// メトリクス
    metrics: RateLimitMetrics,
}

impl SlidingWindow {
    /// 新しいスライディングウィンドウを生成する
    ///
    /// # 引数
    ///
    /// * `window_size` - ウィンドウの期間
    /// * `max_requests` - ウィンドウ内の最大リクエスト数
    #[must_use]
    pub fn new(window_size: Duration, max_requests: u64) -> Self {
        Self {
            window_size,
            max_requests,
            timestamps: Mutex::new(VecDeque::new()),
            metrics: RateLimitMetrics::new(),
        }
    }

    /// メトリクスへの参照を返す
    #[must_use]
    pub fn metrics(&self) -> &RateLimitMetrics {
        &self.metrics
    }

    /// 期限切れのタイムスタンプを削除する
    fn cleanup(timestamps: &mut VecDeque<tokio::time::Instant>, cutoff: tokio::time::Instant) {
        while timestamps.front().is_some_and(|&t| t < cutoff) {
            timestamps.pop_front();
        }
    }
}

impl RateLimiter for SlidingWindow {
    fn try_acquire(&self) -> Result<(), RateLimitError> {
        let now = tokio::time::Instant::now();
        let cutoff = now - self.window_size;

        let mut timestamps = self.timestamps.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        Self::cleanup(&mut timestamps, cutoff);

        #[allow(clippy::cast_possible_truncation)]
        if timestamps.len() as u64 >= self.max_requests {
            self.metrics.increment_rejected();
            // 最古のエントリが期限切れになるまでの待機時間を計算
            let retry_after = timestamps
                .front()
                .map_or(self.window_size, |&oldest| {
                    let expires = oldest + self.window_size;
                    if expires > now {
                        expires - now
                    } else {
                        Duration::ZERO
                    }
                });
            return Err(RateLimitError::exceeded(retry_after));
        }

        timestamps.push_back(now);
        self.metrics.increment_allowed();
        Ok(())
    }

    fn time_until_available(&self) -> Duration {
        let now = tokio::time::Instant::now();
        let cutoff = now - self.window_size;

        let mut timestamps = self.timestamps.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        Self::cleanup(&mut timestamps, cutoff);

        #[allow(clippy::cast_possible_truncation)]
        if (timestamps.len() as u64) < self.max_requests {
            return Duration::ZERO;
        }

        timestamps
            .front()
            .map_or(Duration::ZERO, |&oldest| {
                let expires = oldest + self.window_size;
                if expires > now { expires - now } else { Duration::ZERO }
            })
    }

    fn available_tokens(&self) -> u64 {
        let now = tokio::time::Instant::now();
        let cutoff = now - self.window_size;

        let mut timestamps = self.timestamps.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        Self::cleanup(&mut timestamps, cutoff);

        #[allow(clippy::cast_possible_truncation)]
        let used = timestamps.len() as u64;
        self.max_requests.saturating_sub(used)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test(start_paused = true)]
    async fn test_within_limit() {
        let sw = SlidingWindow::new(Duration::from_secs(1), 5);
        for _ in 0..5 {
            assert!(sw.try_acquire().is_ok());
        }
        assert_eq!(sw.metrics().allowed(), 5);
    }

    #[tokio::test(start_paused = true)]
    async fn test_exceeds_limit() {
        let sw = SlidingWindow::new(Duration::from_secs(1), 3);
        for _ in 0..3 {
            assert!(sw.try_acquire().is_ok());
        }
        let err = sw.try_acquire().unwrap_err();
        assert!(err.is_retryable());
        assert_eq!(sw.metrics().rejected(), 1);
    }

    #[tokio::test(start_paused = true)]
    async fn test_window_expiry() {
        let sw = SlidingWindow::new(Duration::from_secs(1), 2);

        assert!(sw.try_acquire().is_ok());
        assert!(sw.try_acquire().is_ok());
        assert!(sw.try_acquire().is_err());

        // ウィンドウを超えるまで待機
        tokio::time::advance(Duration::from_millis(1001)).await;

        // 古いエントリが期限切れ → 再度取得可能
        assert!(sw.try_acquire().is_ok());
        assert_eq!(sw.available_tokens(), 1);
    }

    #[tokio::test(start_paused = true)]
    async fn test_time_until_available() {
        let sw = SlidingWindow::new(Duration::from_secs(1), 1);
        assert_eq!(sw.time_until_available(), Duration::ZERO);

        sw.try_acquire().unwrap();
        let wait = sw.time_until_available();
        assert!(wait > Duration::ZERO);
        assert!(wait <= Duration::from_secs(1));
    }

    #[tokio::test(start_paused = true)]
    async fn test_available_tokens() {
        let sw = SlidingWindow::new(Duration::from_secs(1), 10);
        assert_eq!(sw.available_tokens(), 10);

        sw.try_acquire().unwrap();
        assert_eq!(sw.available_tokens(), 9);
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let sw = Arc::new(SlidingWindow::new(Duration::from_secs(1), 50));
        let mut handles = Vec::new();

        for _ in 0..10 {
            let s = Arc::clone(&sw);
            handles.push(tokio::spawn(async move {
                let mut ok_count = 0u64;
                for _ in 0..10 {
                    if s.try_acquire().is_ok() {
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

        assert!(total_ok <= 50, "total_ok={total_ok} exceeds max_requests");
        assert_eq!(
            sw.metrics().allowed() + sw.metrics().rejected(),
            sw.metrics().total()
        );
    }
}
