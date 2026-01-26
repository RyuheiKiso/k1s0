//! レジリエンスエラー
//!
//! レジリエンスパターンで発生するエラーを定義する。

use thiserror::Error;

/// レジリエンスエラー
#[derive(Debug, Error)]
pub enum ResilienceError {
    /// 設定エラー
    #[error("configuration error: {message}")]
    Config { message: String },

    /// タイムアウトエラー
    #[error("timeout: operation exceeded {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    /// 接続エラー
    #[error("connection error: {message}")]
    Connection { message: String },

    /// 同時実行制限エラー
    #[error("concurrency limit reached: max {max_concurrent}")]
    ConcurrencyLimit { max_concurrent: u32 },

    /// 同時実行制御エラー
    #[error("concurrency error: {message}")]
    Concurrency { message: String },

    /// サーキットブレーカ開放エラー
    #[error("circuit breaker is open (state: {state})")]
    CircuitOpen { state: String },

    /// 依存先エラー
    #[error("dependency error: {message}")]
    Dependency { message: String },
}

impl ResilienceError {
    /// 設定エラーを作成
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// タイムアウトエラーを作成
    pub fn timeout(timeout_ms: u64) -> Self {
        Self::Timeout { timeout_ms }
    }

    /// 接続エラーを作成
    pub fn connection(message: impl Into<String>) -> Self {
        Self::Connection {
            message: message.into(),
        }
    }

    /// 同時実行制限エラーを作成
    pub fn concurrency_limit(max_concurrent: u32) -> Self {
        Self::ConcurrencyLimit { max_concurrent }
    }

    /// 同時実行制御エラーを作成
    pub fn concurrency(message: impl Into<String>) -> Self {
        Self::Concurrency {
            message: message.into(),
        }
    }

    /// サーキットブレーカ開放エラーを作成
    pub fn circuit_open(state: String) -> Self {
        Self::CircuitOpen { state }
    }

    /// 依存先エラーを作成
    pub fn dependency(message: impl Into<String>) -> Self {
        Self::Dependency {
            message: message.into(),
        }
    }

    /// リトライ可能かどうか
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Timeout { .. } | Self::Connection { .. } | Self::CircuitOpen { .. }
        )
    }

    /// error_code を取得
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Config { .. } => "RESILIENCE_CONFIG_ERROR",
            Self::Timeout { .. } => "RESILIENCE_TIMEOUT",
            Self::Connection { .. } => "RESILIENCE_CONNECTION_ERROR",
            Self::ConcurrencyLimit { .. } => "RESILIENCE_CONCURRENCY_LIMIT",
            Self::Concurrency { .. } => "RESILIENCE_CONCURRENCY_ERROR",
            Self::CircuitOpen { .. } => "RESILIENCE_CIRCUIT_OPEN",
            Self::Dependency { .. } => "RESILIENCE_DEPENDENCY_ERROR",
        }
    }

    /// gRPC ステータスコードに変換
    pub fn to_grpc_status_code(&self) -> i32 {
        match self {
            Self::Config { .. } => 13,          // INTERNAL
            Self::Timeout { .. } => 4,          // DEADLINE_EXCEEDED
            Self::Connection { .. } => 14,      // UNAVAILABLE
            Self::ConcurrencyLimit { .. } => 8, // RESOURCE_EXHAUSTED
            Self::Concurrency { .. } => 13,     // INTERNAL
            Self::CircuitOpen { .. } => 14,     // UNAVAILABLE
            Self::Dependency { .. } => 14,      // UNAVAILABLE
        }
    }

    /// HTTP ステータスコードに変換
    pub fn to_http_status_code(&self) -> u16 {
        match self {
            Self::Config { .. } => 500,          // Internal Server Error
            Self::Timeout { .. } => 504,         // Gateway Timeout
            Self::Connection { .. } => 503,      // Service Unavailable
            Self::ConcurrencyLimit { .. } => 429, // Too Many Requests
            Self::Concurrency { .. } => 500,     // Internal Server Error
            Self::CircuitOpen { .. } => 503,     // Service Unavailable
            Self::Dependency { .. } => 503,      // Service Unavailable
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resilience_error_config() {
        let err = ResilienceError::config("invalid setting");
        assert!(matches!(err, ResilienceError::Config { .. }));
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_resilience_error_timeout() {
        let err = ResilienceError::timeout(5000);
        assert!(matches!(err, ResilienceError::Timeout { timeout_ms: 5000 }));
        assert!(err.is_retryable());
    }

    #[test]
    fn test_resilience_error_connection() {
        let err = ResilienceError::connection("refused");
        assert!(err.is_retryable());
    }

    #[test]
    fn test_resilience_error_concurrency_limit() {
        let err = ResilienceError::concurrency_limit(100);
        assert!(!err.is_retryable());
        assert_eq!(err.to_http_status_code(), 429);
    }

    #[test]
    fn test_resilience_error_circuit_open() {
        let err = ResilienceError::circuit_open("open".to_string());
        assert!(err.is_retryable());
        assert_eq!(err.to_grpc_status_code(), 14);
    }

    #[test]
    fn test_resilience_error_code() {
        assert_eq!(
            ResilienceError::timeout(1000).error_code(),
            "RESILIENCE_TIMEOUT"
        );
        assert_eq!(
            ResilienceError::circuit_open("open".to_string()).error_code(),
            "RESILIENCE_CIRCUIT_OPEN"
        );
    }

    #[test]
    fn test_resilience_error_grpc_status() {
        assert_eq!(ResilienceError::timeout(1000).to_grpc_status_code(), 4);
        assert_eq!(ResilienceError::connection("").to_grpc_status_code(), 14);
    }

    #[test]
    fn test_resilience_error_http_status() {
        assert_eq!(ResilienceError::timeout(1000).to_http_status_code(), 504);
        assert_eq!(ResilienceError::concurrency_limit(100).to_http_status_code(), 429);
    }
}
