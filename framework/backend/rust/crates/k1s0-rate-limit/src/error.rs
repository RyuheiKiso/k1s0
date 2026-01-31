//! レートリミットエラー定義
//!
//! レートリミット操作で発生するエラーを統一的に表現する。

use std::time::Duration;
use thiserror::Error;

/// レートリミット操作のエラー型
#[derive(Debug, Error)]
pub enum RateLimitError {
    /// レートリミット超過
    #[error("rate limit exceeded, retry after {retry_after:?}")]
    Exceeded {
        /// リトライまでの待機時間
        retry_after: Duration,
    },

    /// 設定エラー
    #[error("rate limiter configuration error: {message}")]
    Config {
        /// エラーメッセージ
        message: String,
    },
}

impl RateLimitError {
    /// レートリミット超過エラーを生成する
    #[must_use]
    pub fn exceeded(retry_after: Duration) -> Self {
        Self::Exceeded { retry_after }
    }

    /// 設定エラーを生成する
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// リトライ可能なエラーかどうかを返す
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Exceeded { .. })
    }

    /// エラーコードを返す
    #[must_use]
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Exceeded { .. } => "RATE_LIMIT_EXCEEDED",
            Self::Config { .. } => "RATE_LIMIT_CONFIG_ERROR",
        }
    }

    /// gRPC ステータスコードに変換する
    #[must_use]
    pub fn to_grpc_status_code(&self) -> i32 {
        match self {
            Self::Exceeded { .. } => 8,  // RESOURCE_EXHAUSTED
            Self::Config { .. } => 13,   // INTERNAL
        }
    }

    /// HTTP ステータスコードに変換する
    #[must_use]
    pub fn to_http_status_code(&self) -> u16 {
        match self {
            Self::Exceeded { .. } => 429,
            Self::Config { .. } => 500,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exceeded_error() {
        let err = RateLimitError::exceeded(Duration::from_secs(1));
        assert!(err.is_retryable());
        assert_eq!(err.error_code(), "RATE_LIMIT_EXCEEDED");
        assert_eq!(err.to_http_status_code(), 429);
        assert_eq!(err.to_grpc_status_code(), 8);
        assert!(err.to_string().contains("retry after"));
    }

    #[test]
    fn test_config_error() {
        let err = RateLimitError::config("invalid capacity");
        assert!(!err.is_retryable());
        assert_eq!(err.error_code(), "RATE_LIMIT_CONFIG_ERROR");
        assert_eq!(err.to_http_status_code(), 500);
        assert_eq!(err.to_grpc_status_code(), 13);
        assert!(err.to_string().contains("invalid capacity"));
    }
}
