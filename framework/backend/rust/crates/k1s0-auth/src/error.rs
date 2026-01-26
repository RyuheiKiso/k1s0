//! 認証エラー型

use thiserror::Error;

/// 認証エラー
#[derive(Debug, Error)]
pub enum AuthError {
    /// 無効なトークン
    #[error("invalid token: {0}")]
    InvalidToken(String),

    /// 無効な鍵
    #[error("invalid key: {0}")]
    InvalidKey(String),

    /// 設定エラー
    #[error("configuration error: {0}")]
    Configuration(String),

    /// ネットワークエラー
    #[error("network error: {0}")]
    NetworkError(String),

    /// 認証失敗
    #[error("authentication failed: {0}")]
    AuthenticationFailed(String),

    /// 認可失敗
    #[error("authorization failed: {0}")]
    AuthorizationFailed(String),

    /// トークン期限切れ
    #[error("token expired")]
    TokenExpired,

    /// 内部エラー
    #[error("internal error: {0}")]
    Internal(String),
}

impl AuthError {
    /// gRPCステータスコードを取得
    pub fn to_grpc_code(&self) -> i32 {
        match self {
            Self::InvalidToken(_) => 16,         // UNAUTHENTICATED
            Self::InvalidKey(_) => 13,           // INTERNAL
            Self::Configuration(_) => 13,        // INTERNAL
            Self::NetworkError(_) => 14,         // UNAVAILABLE
            Self::AuthenticationFailed(_) => 16, // UNAUTHENTICATED
            Self::AuthorizationFailed(_) => 7,   // PERMISSION_DENIED
            Self::TokenExpired => 16,            // UNAUTHENTICATED
            Self::Internal(_) => 13,             // INTERNAL
        }
    }

    /// HTTPステータスコードを取得
    pub fn to_http_status(&self) -> u16 {
        match self {
            Self::InvalidToken(_) => 401,
            Self::InvalidKey(_) => 500,
            Self::Configuration(_) => 500,
            Self::NetworkError(_) => 503,
            Self::AuthenticationFailed(_) => 401,
            Self::AuthorizationFailed(_) => 403,
            Self::TokenExpired => 401,
            Self::Internal(_) => 500,
        }
    }

    /// エラーコードを取得
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::InvalidToken(_) => "INVALID_TOKEN",
            Self::InvalidKey(_) => "INVALID_KEY",
            Self::Configuration(_) => "CONFIGURATION_ERROR",
            Self::NetworkError(_) => "NETWORK_ERROR",
            Self::AuthenticationFailed(_) => "AUTHENTICATION_FAILED",
            Self::AuthorizationFailed(_) => "AUTHORIZATION_FAILED",
            Self::TokenExpired => "TOKEN_EXPIRED",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }

    /// リトライ可能かどうか
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::NetworkError(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        let err = AuthError::InvalidToken("test".to_string());
        assert_eq!(err.error_code(), "INVALID_TOKEN");
        assert_eq!(err.to_http_status(), 401);
        assert_eq!(err.to_grpc_code(), 16);
    }

    #[test]
    fn test_retryable() {
        assert!(AuthError::NetworkError("timeout".to_string()).is_retryable());
        assert!(!AuthError::InvalidToken("bad".to_string()).is_retryable());
    }
}
