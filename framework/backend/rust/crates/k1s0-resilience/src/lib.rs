//! k1s0 レジリエンスパターン
//!
//! 依存先呼び出しのガードレールを提供する。
//!
//! # 主な機能
//!
//! - **タイムアウト**: 上限/下限バリデーション付きタイムアウト
//! - **同時実行制限**: セマフォベースの同時実行数制限
//! - **バルクヘッド**: 依存先ごとの独立したリソースプール
//! - **サーキットブレーカ**: 障害検知と呼び出し遮断（既定OFF）
//!
//! # 設計原則
//!
//! 1. **タイムアウト必須**: 無制限待機を防ぐ
//! 2. **同時実行制限**: リソース枯渇を防ぐ
//! 3. **障害隔離**: バルクヘッドで障害の波及を防ぐ
//! 4. **サーキットブレーカ**: 必要時のみ有効化（既定OFF）
//!
//! # 使用例
//!
//! ## タイムアウト
//!
//! ```rust
//! use k1s0_resilience::{TimeoutConfig, TimeoutGuard, ResilienceError};
//!
//! # async fn example() -> Result<(), ResilienceError> {
//! let guard = TimeoutGuard::new(TimeoutConfig::new(5000))?;
//!
//! let result = guard.execute(async {
//!     // 処理
//!     Ok::<_, ResilienceError>(42)
//! }).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## 同時実行制限
//!
//! ```rust
//! use k1s0_resilience::{ConcurrencyConfig, ConcurrencyLimiter, ResilienceError};
//!
//! # async fn example() -> Result<(), ResilienceError> {
//! let limiter = ConcurrencyLimiter::new(ConcurrencyConfig::new(10));
//!
//! let result = limiter.execute(async {
//!     // 処理
//!     Ok::<_, ResilienceError>(42)
//! }).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## バルクヘッド
//!
//! ```rust
//! use k1s0_resilience::{BulkheadConfig, Bulkhead, ResilienceError};
//!
//! # async fn example() -> Result<(), ResilienceError> {
//! let bulkhead = Bulkhead::new(
//!     BulkheadConfig::new(100)
//!         .with_service_limit("auth-service", 10)
//!         .with_service_limit("config-service", 20)
//! );
//!
//! let result = bulkhead.execute("auth-service", async {
//!     // 処理
//!     Ok::<_, ResilienceError>(42)
//! }).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## サーキットブレーカ（既定OFF）
//!
//! ```rust
//! use k1s0_resilience::{CircuitBreakerConfig, CircuitBreaker, ResilienceError};
//!
//! # async fn example() -> Result<(), ResilienceError> {
//! // 既定は無効
//! let cb = CircuitBreaker::disabled();
//! assert!(cb.allow_request());
//!
//! // 有効にする場合
//! let cb = CircuitBreaker::new(
//!     CircuitBreakerConfig::enabled()
//!         .failure_threshold(5)
//!         .success_threshold(3)
//!         .reset_timeout_secs(30)
//!         .build()
//! );
//!
//! let result = cb.execute(async {
//!     // 処理
//!     Ok::<_, ResilienceError>(42)
//! }).await?;
//! # Ok(())
//! # }
//! ```

pub mod bulkhead;
pub mod circuit_breaker;
pub mod concurrency;
pub mod error;
pub mod timeout;

// Re-exports
pub use bulkhead::{Bulkhead, BulkheadConfig};
pub use circuit_breaker::{
    CircuitBreaker, CircuitBreakerConfig, CircuitBreakerConfigBuilder, CircuitBreakerMetrics,
    CircuitState, FailurePredicate,
};
pub use concurrency::{ConcurrencyConfig, ConcurrencyLimiter, ConcurrencyMetrics};
pub use error::ResilienceError;
pub use timeout::{TimeoutConfig, TimeoutGuard, DEFAULT_TIMEOUT_MS, MAX_TIMEOUT_MS, MIN_TIMEOUT_MS};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_timeout_basic() {
        let guard = TimeoutGuard::default_timeout();
        let result = guard
            .execute(async { Ok::<_, ResilienceError>(42) })
            .await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_concurrency_basic() {
        let limiter = ConcurrencyLimiter::default_config();
        let result = limiter
            .execute(async { Ok::<_, ResilienceError>(42) })
            .await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_bulkhead_basic() {
        let bulkhead = Bulkhead::default_config();
        let result = bulkhead
            .execute("test-service", async { Ok::<_, ResilienceError>(42) })
            .await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_circuit_breaker_disabled() {
        let cb = CircuitBreaker::disabled();
        assert!(cb.allow_request());

        let result = cb
            .execute(async { Ok::<_, ResilienceError>(42) })
            .await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_circuit_breaker_enabled() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig::enabled().build());
        assert!(cb.allow_request());
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_resilience_error() {
        let err = ResilienceError::timeout(1000);
        assert_eq!(err.error_code(), "RESILIENCE_TIMEOUT");
        assert!(err.is_retryable());
    }

    #[test]
    fn test_timeout_config_validation() {
        // 正常範囲
        let config = TimeoutConfig::new(10_000);
        assert!(config.validate().is_ok());

        // 下限未満
        let config = TimeoutConfig::new(50);
        assert!(config.validate().is_err());

        // 上限超過
        let config = TimeoutConfig::new(400_000);
        assert!(config.validate().is_err());
    }
}
