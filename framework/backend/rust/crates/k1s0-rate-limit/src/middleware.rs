//! レートリミット ミドルウェアサポート
//!
//! tonic (gRPC) および axum (REST) のミドルウェアに組み込むための
//! レートリミットインターセプターを提供する。
//!
//! 注意: tonic/axum への直接依存は持たない。利用者が自身のミドルウェアに
//! `check()` メソッドを組み込む形で使用する。

use std::sync::Arc;
use std::time::Duration;

use crate::error::RateLimitError;
use crate::token_bucket::RateLimiter;

/// レートリミット インターセプター
///
/// gRPC/REST ミドルウェアから呼び出し可能なレートリミットチェックを提供する。
///
/// # 使用例
///
/// ```rust
/// use k1s0_rate_limit::{RateLimitInterceptor, TokenBucket};
/// use std::sync::Arc;
///
/// # #[tokio::main]
/// # async fn main() {
/// let limiter = Arc::new(TokenBucket::new(100, 10.0));
/// let interceptor = RateLimitInterceptor::new(limiter);
///
/// match interceptor.check() {
///     Ok(()) => { /* リクエスト処理を続行 */ }
///     Err(e) => {
///         let status = e.to_http_status_code(); // 429
///         let retry = interceptor.retry_after();
///         // Retry-After ヘッダーを設定して応答
///     }
/// }
/// # }
/// ```
pub struct RateLimitInterceptor {
    /// 内部のレートリミッター
    limiter: Arc<dyn RateLimiter>,
}

impl RateLimitInterceptor {
    /// 新しいインターセプターを生成する
    #[must_use]
    pub fn new(limiter: Arc<dyn RateLimiter>) -> Self {
        Self { limiter }
    }

    /// レートリミットをチェックする
    ///
    /// # Errors
    ///
    /// レートリミットを超過した場合に `RateLimitError::Exceeded` を返す。
    pub fn check(&self) -> Result<(), RateLimitError> {
        self.limiter.try_acquire()
    }

    /// リトライまでの待機時間を返す
    #[must_use]
    pub fn retry_after(&self) -> Duration {
        self.limiter.time_until_available()
    }

    /// 現在利用可能なトークン数を返す
    #[must_use]
    pub fn available_tokens(&self) -> u64 {
        self.limiter.available_tokens()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token_bucket::TokenBucket;

    #[tokio::test]
    async fn test_interceptor_check_ok() {
        let limiter = Arc::new(TokenBucket::new(10, 10.0));
        let interceptor = RateLimitInterceptor::new(limiter);
        assert!(interceptor.check().is_ok());
    }

    #[tokio::test]
    async fn test_interceptor_check_exceeded() {
        let limiter = Arc::new(TokenBucket::new(1, 1.0));
        let interceptor = RateLimitInterceptor::new(limiter);
        assert!(interceptor.check().is_ok());
        assert!(interceptor.check().is_err());
    }

    #[tokio::test]
    async fn test_interceptor_retry_after() {
        let limiter = Arc::new(TokenBucket::new(1, 1.0));
        let interceptor = RateLimitInterceptor::new(limiter);
        interceptor.check().unwrap();
        let wait = interceptor.retry_after();
        assert!(wait > Duration::ZERO);
    }

    #[tokio::test]
    async fn test_interceptor_available_tokens() {
        let limiter = Arc::new(TokenBucket::new(5, 1.0));
        let interceptor = RateLimitInterceptor::new(limiter);
        assert_eq!(interceptor.available_tokens(), 5);
        interceptor.check().unwrap();
        assert_eq!(interceptor.available_tokens(), 4);
    }
}
