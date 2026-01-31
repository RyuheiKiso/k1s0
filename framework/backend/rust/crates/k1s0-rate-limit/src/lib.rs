//! k1s0 レートリミット
//!
//! マイクロサービス向けのレートリミットパターンを提供する。
//!
//! # 主な機能
//!
//! - **トークンバケット**: ロックフリーなCASベースのトークンバケットアルゴリズム
//! - **スライディングウィンドウ**: 固定時間ウィンドウ内のリクエスト数制限
//! - **ミドルウェアサポート**: gRPC/REST ミドルウェアへの組み込み用インターセプター
//! - **メトリクス**: アトミックなリクエスト許可/拒否カウンタ
//!
//! # 設計原則
//!
//! 1. **ロックフリー**: トークンバケットは `AtomicU64` CASループで実装
//! 2. **設定駆動**: YAML設定ファイルからのデシリアライズに対応
//! 3. **軽量依存**: tonic/axum への直接依存なし
//! 4. **メトリクス内蔵**: 許可/拒否の追跡を標準搭載
//!
//! # 使用例
//!
//! ## トークンバケット
//!
//! ```rust
//! use k1s0_rate_limit::{TokenBucket, RateLimiter, RateLimitError};
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), RateLimitError> {
//! let limiter = TokenBucket::new(1000, 100.0);
//!
//! match limiter.try_acquire() {
//!     Ok(()) => { /* リクエスト処理 */ }
//!     Err(e) => {
//!         eprintln!("レートリミット超過: {e}");
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## スライディングウィンドウ
//!
//! ```rust
//! use k1s0_rate_limit::{SlidingWindow, RateLimiter};
//! use std::time::Duration;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let limiter = SlidingWindow::new(Duration::from_secs(60), 600);
//!
//! if limiter.try_acquire().is_ok() {
//!     // リクエスト処理
//! }
//! # }
//! ```
//!
//! ## ミドルウェア連携
//!
//! ```rust
//! use k1s0_rate_limit::{RateLimitInterceptor, TokenBucket};
//! use std::sync::Arc;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let limiter = Arc::new(TokenBucket::new(100, 10.0));
//! let interceptor = RateLimitInterceptor::new(limiter);
//!
//! if let Err(e) = interceptor.check() {
//!     let retry_after = interceptor.retry_after();
//!     let http_status = e.to_http_status_code(); // 429
//!     let grpc_code = e.to_grpc_status_code();   // 8 (RESOURCE_EXHAUSTED)
//! }
//! # }
//! ```

pub mod config;
pub mod error;
pub mod metrics;
pub mod middleware;
pub mod sliding_window;
pub mod token_bucket;

// Re-exports
pub use config::RateLimitConfig;
pub use error::RateLimitError;
pub use metrics::RateLimitMetrics;
pub use middleware::RateLimitInterceptor;
pub use sliding_window::SlidingWindow;
pub use token_bucket::{RateLimiter, TokenBucket};

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::time::Duration;

    #[tokio::test]
    async fn test_token_bucket_basic() {
        let limiter = TokenBucket::new(10, 10.0);
        assert!(limiter.try_acquire().is_ok());
        assert_eq!(limiter.available_tokens(), 9);
    }

    #[tokio::test]
    async fn test_sliding_window_basic() {
        let limiter = SlidingWindow::new(Duration::from_secs(1), 10);
        assert!(limiter.try_acquire().is_ok());
        assert_eq!(limiter.available_tokens(), 9);
    }

    #[tokio::test]
    async fn test_interceptor_with_token_bucket() {
        let limiter = Arc::new(TokenBucket::new(5, 5.0));
        let interceptor = RateLimitInterceptor::new(limiter);

        for _ in 0..5 {
            assert!(interceptor.check().is_ok());
        }
        let err = interceptor.check().unwrap_err();
        assert_eq!(err.to_http_status_code(), 429);
    }

    #[tokio::test]
    async fn test_interceptor_with_sliding_window() {
        let limiter = Arc::new(SlidingWindow::new(Duration::from_secs(60), 3));
        let interceptor = RateLimitInterceptor::new(limiter);

        for _ in 0..3 {
            assert!(interceptor.check().is_ok());
        }
        assert!(interceptor.check().is_err());
    }

    #[test]
    fn test_config_default() {
        let config = RateLimitConfig::default();
        assert!(matches!(config, RateLimitConfig::TokenBucket { .. }));
    }

    #[test]
    fn test_error_codes() {
        let err = RateLimitError::exceeded(Duration::from_secs(1));
        assert_eq!(err.error_code(), "RATE_LIMIT_EXCEEDED");
        assert!(err.is_retryable());

        let err = RateLimitError::config("bad config");
        assert_eq!(err.error_code(), "RATE_LIMIT_CONFIG_ERROR");
        assert!(!err.is_retryable());
    }
}
