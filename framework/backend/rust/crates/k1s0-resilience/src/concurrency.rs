//! 同時実行制限
//!
//! 依存先呼び出しの同時実行数を制限する。

use crate::error::ResilienceError;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

/// デフォルトの同時実行数上限
pub const DEFAULT_MAX_CONCURRENT: u32 = 100;

/// デフォルトの待機タイムアウト（ミリ秒）
pub const DEFAULT_ACQUIRE_TIMEOUT_MS: u64 = 5_000;

/// 同時実行制限設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcurrencyConfig {
    /// 最大同時実行数
    #[serde(default = "default_max_concurrent")]
    max_concurrent: u32,
    /// 許可取得タイムアウト（ミリ秒）
    #[serde(default = "default_acquire_timeout_ms")]
    acquire_timeout_ms: u64,
    /// 待機時間のメトリクス出力有効
    #[serde(default = "default_true")]
    metrics_enabled: bool,
}

fn default_max_concurrent() -> u32 {
    DEFAULT_MAX_CONCURRENT
}

fn default_acquire_timeout_ms() -> u64 {
    DEFAULT_ACQUIRE_TIMEOUT_MS
}

fn default_true() -> bool {
    true
}

impl ConcurrencyConfig {
    /// 新しい設定を作成
    pub fn new(max_concurrent: u32) -> Self {
        Self {
            max_concurrent,
            acquire_timeout_ms: DEFAULT_ACQUIRE_TIMEOUT_MS,
            metrics_enabled: true,
        }
    }

    /// 待機タイムアウトを設定
    pub fn with_acquire_timeout_ms(mut self, ms: u64) -> Self {
        self.acquire_timeout_ms = ms;
        self
    }

    /// メトリクス出力を設定
    pub fn with_metrics(mut self, enabled: bool) -> Self {
        self.metrics_enabled = enabled;
        self
    }

    /// 最大同時実行数を取得
    pub fn max_concurrent(&self) -> u32 {
        self.max_concurrent
    }

    /// 待機タイムアウトを取得
    pub fn acquire_timeout(&self) -> Duration {
        Duration::from_millis(self.acquire_timeout_ms)
    }

    /// メトリクス出力が有効かどうか
    pub fn metrics_enabled(&self) -> bool {
        self.metrics_enabled
    }
}

impl Default for ConcurrencyConfig {
    fn default() -> Self {
        Self {
            max_concurrent: DEFAULT_MAX_CONCURRENT,
            acquire_timeout_ms: DEFAULT_ACQUIRE_TIMEOUT_MS,
            metrics_enabled: true,
        }
    }
}

/// 同時実行リミッタ
///
/// セマフォベースの同時実行数制限。
#[derive(Debug)]
pub struct ConcurrencyLimiter {
    semaphore: Arc<Semaphore>,
    config: ConcurrencyConfig,
    metrics: ConcurrencyMetrics,
}

impl ConcurrencyLimiter {
    /// 新しいリミッタを作成
    pub fn new(config: ConcurrencyConfig) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(config.max_concurrent as usize)),
            metrics: ConcurrencyMetrics::new(),
            config,
        }
    }

    /// デフォルト設定でリミッタを作成
    pub fn default_config() -> Self {
        Self::new(ConcurrencyConfig::default())
    }

    /// 許可を取得して処理を実行
    pub async fn execute<F, T>(&self, f: F) -> Result<T, ResilienceError>
    where
        F: std::future::Future<Output = Result<T, ResilienceError>>,
    {
        // 許可の取得
        let permit = match tokio::time::timeout(
            self.config.acquire_timeout(),
            self.semaphore.clone().acquire_owned(),
        )
        .await
        {
            Ok(Ok(permit)) => permit,
            Ok(Err(_)) => {
                return Err(ResilienceError::concurrency(
                    "semaphore closed unexpectedly",
                ));
            }
            Err(_) => {
                self.metrics.increment_rejected();
                return Err(ResilienceError::concurrency_limit(
                    self.config.max_concurrent,
                ));
            }
        };

        // 処理の実行
        self.metrics.increment_active();
        let result = f.await;
        self.metrics.decrement_active();

        // 許可の解放（drop で自動）
        drop(permit);

        result
    }

    /// 利用可能な許可数を取得
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }

    /// アクティブな実行数を取得
    pub fn active_count(&self) -> u64 {
        self.metrics.active()
    }

    /// 拒否された数を取得
    pub fn rejected_count(&self) -> u64 {
        self.metrics.rejected()
    }

    /// 設定を取得
    pub fn config(&self) -> &ConcurrencyConfig {
        &self.config
    }

    /// メトリクスを取得
    pub fn metrics(&self) -> &ConcurrencyMetrics {
        &self.metrics
    }
}

/// 同時実行メトリクス
#[derive(Debug)]
pub struct ConcurrencyMetrics {
    /// アクティブな実行数
    active: AtomicU64,
    /// 拒否された数
    rejected: AtomicU64,
    /// 総実行数
    total: AtomicU64,
}

impl ConcurrencyMetrics {
    /// 新しいメトリクスを作成
    pub fn new() -> Self {
        Self {
            active: AtomicU64::new(0),
            rejected: AtomicU64::new(0),
            total: AtomicU64::new(0),
        }
    }

    /// アクティブ数をインクリメント
    fn increment_active(&self) {
        self.active.fetch_add(1, Ordering::SeqCst);
        self.total.fetch_add(1, Ordering::SeqCst);
    }

    /// アクティブ数をデクリメント
    fn decrement_active(&self) {
        self.active.fetch_sub(1, Ordering::SeqCst);
    }

    /// 拒否数をインクリメント
    fn increment_rejected(&self) {
        self.rejected.fetch_add(1, Ordering::SeqCst);
    }

    /// アクティブ数を取得
    pub fn active(&self) -> u64 {
        self.active.load(Ordering::SeqCst)
    }

    /// 拒否数を取得
    pub fn rejected(&self) -> u64 {
        self.rejected.load(Ordering::SeqCst)
    }

    /// 総実行数を取得
    pub fn total(&self) -> u64 {
        self.total.load(Ordering::SeqCst)
    }
}

impl Default for ConcurrencyMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concurrency_config_default() {
        let config = ConcurrencyConfig::default();
        assert_eq!(config.max_concurrent(), DEFAULT_MAX_CONCURRENT);
    }

    #[test]
    fn test_concurrency_config_new() {
        let config = ConcurrencyConfig::new(50);
        assert_eq!(config.max_concurrent(), 50);
    }

    #[test]
    fn test_concurrency_limiter_new() {
        let limiter = ConcurrencyLimiter::new(ConcurrencyConfig::new(10));
        assert_eq!(limiter.available_permits(), 10);
        assert_eq!(limiter.active_count(), 0);
    }

    #[tokio::test]
    async fn test_concurrency_limiter_execute() {
        let limiter = ConcurrencyLimiter::new(ConcurrencyConfig::new(10));

        let result = limiter
            .execute(async { Ok::<_, ResilienceError>(42) })
            .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(limiter.metrics().total(), 1);
    }

    #[tokio::test]
    async fn test_concurrency_limiter_limit() {
        let limiter = Arc::new(ConcurrencyLimiter::new(
            ConcurrencyConfig::new(1).with_acquire_timeout_ms(100),
        ));

        // 最初のタスクで許可を保持
        let limiter_clone = limiter.clone();
        let handle = tokio::spawn(async move {
            limiter_clone
                .execute(async {
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    Ok::<_, ResilienceError>(1)
                })
                .await
        });

        // 少し待ってから2つ目のタスクを実行
        tokio::time::sleep(Duration::from_millis(50)).await;

        // 2つ目のタスクはタイムアウト
        let result = limiter
            .execute(async { Ok::<_, ResilienceError>(2) })
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ResilienceError::ConcurrencyLimit { .. }
        ));

        handle.await.unwrap().unwrap();
    }

    #[test]
    fn test_concurrency_metrics() {
        let metrics = ConcurrencyMetrics::new();
        assert_eq!(metrics.active(), 0);
        assert_eq!(metrics.rejected(), 0);
        assert_eq!(metrics.total(), 0);
    }
}
