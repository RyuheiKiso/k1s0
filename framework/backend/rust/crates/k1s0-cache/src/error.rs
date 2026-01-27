//! キャッシュエラー
//!
//! キャッシュ操作で発生するエラーを統一的に扱う。

use thiserror::Error;

/// キャッシュエラー
#[derive(Debug, Error)]
pub enum CacheError {
    /// 接続エラー
    #[error("cache connection failed: {message}")]
    Connection {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 接続タイムアウト
    #[error("cache connection timeout after {timeout_ms}ms")]
    ConnectionTimeout { timeout_ms: u64 },

    /// プール枯渇
    #[error("connection pool exhausted: max {max_size} connections")]
    PoolExhausted { max_size: u32 },

    /// 操作タイムアウト
    #[error("cache operation timeout after {timeout_ms}ms")]
    OperationTimeout { timeout_ms: u64 },

    /// キーが見つからない
    #[error("cache key not found: {key}")]
    KeyNotFound { key: String },

    /// シリアライズエラー
    #[error("serialization failed: {message}")]
    Serialization { message: String },

    /// デシリアライズエラー
    #[error("deserialization failed: {message}")]
    Deserialization { message: String },

    /// 設定エラー
    #[error("cache configuration error: {message}")]
    Config { message: String },

    /// 内部エラー
    #[error("internal cache error: {message}")]
    Internal { message: String },
}

impl CacheError {
    /// 接続エラーを作成
    pub fn connection(message: impl Into<String>) -> Self {
        Self::Connection {
            message: message.into(),
            source: None,
        }
    }

    /// 接続エラーを作成（原因付き）
    pub fn connection_with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::Connection {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// 接続タイムアウトエラーを作成
    pub fn connection_timeout(timeout_ms: u64) -> Self {
        Self::ConnectionTimeout { timeout_ms }
    }

    /// プール枯渇エラーを作成
    pub fn pool_exhausted(max_size: u32) -> Self {
        Self::PoolExhausted { max_size }
    }

    /// 操作タイムアウトエラーを作成
    pub fn operation_timeout(timeout_ms: u64) -> Self {
        Self::OperationTimeout { timeout_ms }
    }

    /// キー未発見エラーを作成
    pub fn key_not_found(key: impl Into<String>) -> Self {
        Self::KeyNotFound { key: key.into() }
    }

    /// シリアライズエラーを作成
    pub fn serialization(message: impl Into<String>) -> Self {
        Self::Serialization {
            message: message.into(),
        }
    }

    /// デシリアライズエラーを作成
    pub fn deserialization(message: impl Into<String>) -> Self {
        Self::Deserialization {
            message: message.into(),
        }
    }

    /// 設定エラーを作成
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// 内部エラーを作成
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    /// error_code を取得
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Connection { .. } => "CACHE_CONNECTION_ERROR",
            Self::ConnectionTimeout { .. } => "CACHE_CONNECTION_TIMEOUT",
            Self::PoolExhausted { .. } => "CACHE_POOL_EXHAUSTED",
            Self::OperationTimeout { .. } => "CACHE_OPERATION_TIMEOUT",
            Self::KeyNotFound { .. } => "CACHE_KEY_NOT_FOUND",
            Self::Serialization { .. } => "CACHE_SERIALIZATION_ERROR",
            Self::Deserialization { .. } => "CACHE_DESERIALIZATION_ERROR",
            Self::Config { .. } => "CACHE_CONFIG_ERROR",
            Self::Internal { .. } => "CACHE_INTERNAL_ERROR",
        }
    }

    /// リトライ可能かどうか
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Connection { .. }
                | Self::ConnectionTimeout { .. }
                | Self::PoolExhausted { .. }
                | Self::OperationTimeout { .. }
        )
    }

    /// クライアントエラーかどうか
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            Self::KeyNotFound { .. }
                | Self::Serialization { .. }
                | Self::Deserialization { .. }
                | Self::Config { .. }
        )
    }

    /// HTTP ステータスコードに変換
    pub fn to_http_status_code(&self) -> u16 {
        match self {
            Self::KeyNotFound { .. } => 404,
            Self::ConnectionTimeout { .. } | Self::OperationTimeout { .. } => 504,
            Self::PoolExhausted { .. } | Self::Connection { .. } => 503,
            Self::Serialization { .. } | Self::Deserialization { .. } => 400,
            Self::Config { .. } | Self::Internal { .. } => 500,
        }
    }

    /// gRPC ステータスコードに変換
    pub fn to_grpc_status_code(&self) -> i32 {
        match self {
            Self::KeyNotFound { .. } => 5,          // NOT_FOUND
            Self::ConnectionTimeout { .. } | Self::OperationTimeout { .. } => 4, // DEADLINE_EXCEEDED
            Self::PoolExhausted { .. } => 8,        // RESOURCE_EXHAUSTED
            Self::Connection { .. } => 14,          // UNAVAILABLE
            Self::Serialization { .. } | Self::Deserialization { .. } => 3, // INVALID_ARGUMENT
            Self::Config { .. } | Self::Internal { .. } => 13, // INTERNAL
        }
    }
}

/// キャッシュ操作の結果型
pub type CacheResult<T> = Result<T, CacheError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_error() {
        let err = CacheError::connection("failed to connect");
        assert_eq!(err.error_code(), "CACHE_CONNECTION_ERROR");
        assert!(err.is_retryable());
        assert!(!err.is_client_error());
    }

    #[test]
    fn test_key_not_found_error() {
        let err = CacheError::key_not_found("user:123");
        assert_eq!(err.error_code(), "CACHE_KEY_NOT_FOUND");
        assert!(!err.is_retryable());
        assert!(err.is_client_error());
        assert_eq!(err.to_http_status_code(), 404);
        assert_eq!(err.to_grpc_status_code(), 5);
    }

    #[test]
    fn test_timeout_errors() {
        let conn_timeout = CacheError::connection_timeout(5000);
        assert_eq!(conn_timeout.error_code(), "CACHE_CONNECTION_TIMEOUT");
        assert!(conn_timeout.is_retryable());

        let op_timeout = CacheError::operation_timeout(30000);
        assert_eq!(op_timeout.error_code(), "CACHE_OPERATION_TIMEOUT");
        assert!(op_timeout.is_retryable());
    }

    #[test]
    fn test_serialization_errors() {
        let ser_err = CacheError::serialization("invalid json");
        assert_eq!(ser_err.error_code(), "CACHE_SERIALIZATION_ERROR");
        assert!(ser_err.is_client_error());

        let de_err = CacheError::deserialization("invalid format");
        assert_eq!(de_err.error_code(), "CACHE_DESERIALIZATION_ERROR");
        assert!(de_err.is_client_error());
    }

    #[test]
    fn test_pool_exhausted() {
        let err = CacheError::pool_exhausted(10);
        assert_eq!(err.error_code(), "CACHE_POOL_EXHAUSTED");
        assert!(err.is_retryable());
        assert_eq!(err.to_http_status_code(), 503);
    }
}
