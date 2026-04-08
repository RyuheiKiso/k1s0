//! Auth サービスのドメインエラー型。
//!
//! 文字列マッチングではなく、型安全な分類で HTTP ステータスコードを決定する。

use k1s0_server_common::error::{ErrorCode, ServiceError};

/// Auth ドメイン固有のエラー型。
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    /// トークンが無効または期限切れ
    #[error("invalid or expired token: {0}")]
    InvalidToken(String),

    /// 認証情報が不足している
    #[error("missing credentials: {0}")]
    MissingCredentials(String),

    /// 権限が不足している
    #[error("permission denied: {0}")]
    PermissionDenied(String),

    /// ユーザーが見つからない
    #[error("user not found: {0}")]
    NotFound(String),

    /// バリデーションエラー
    #[error("validation failed: {0}")]
    ValidationFailed(String),

    /// 内部エラー
    #[error("internal error: {0}")]
    Internal(String),
}

/// `AuthError` から `ServiceError` への変換実装
impl From<AuthError> for ServiceError {
    fn from(err: AuthError) -> Self {
        match err {
            AuthError::InvalidToken(msg) => ServiceError::Unauthorized {
                code: ErrorCode::new("SYS_AUTH_INVALID_TOKEN"),
                message: msg,
            },
            AuthError::MissingCredentials(msg) => ServiceError::Unauthorized {
                code: ErrorCode::new("SYS_AUTH_MISSING_CLAIMS"),
                message: msg,
            },
            AuthError::PermissionDenied(msg) => ServiceError::Forbidden {
                code: ErrorCode::new("SYS_AUTH_PERMISSION_DENIED"),
                message: msg,
            },
            AuthError::NotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_AUTH_NOT_FOUND"),
                message: msg,
            },
            AuthError::ValidationFailed(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_AUTH_VALIDATION_FAILED"),
                message: msg,
                details: vec![],
            },
            AuthError::Internal(msg) => ServiceError::Internal {
                code: ErrorCode::new("SYS_AUTH_INTERNAL_ERROR"),
                message: msg,
            },
        }
    }
}
