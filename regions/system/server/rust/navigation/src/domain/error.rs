//! Navigation サービスのドメインエラー型。
//!
//! 文字列マッチングではなく、型安全な分類で HTTP ステータスコードを決定する。

use k1s0_server_common::error::{ErrorCode, ServiceError};

/// Navigation ドメイン固有のエラー型。
#[derive(Debug, thiserror::Error)]
pub enum NavigationError {
    /// ナビゲーション項目が見つからない
    #[error("navigation item '{0}' not found")]
    NotFound(String),

    /// バリデーションエラー
    #[error("validation failed: {0}")]
    ValidationFailed(String),

    /// 内部エラー
    #[error("internal error: {0}")]
    Internal(String),
}

/// NavigationError から ServiceError への変換実装
impl From<NavigationError> for ServiceError {
    fn from(err: NavigationError) -> Self {
        match err {
            NavigationError::NotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_NAV_NOT_FOUND"),
                message: msg,
            },
            NavigationError::ValidationFailed(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_NAV_VALIDATION_FAILED"),
                message: msg,
                details: vec![],
            },
            NavigationError::Internal(msg) => ServiceError::Internal {
                code: ErrorCode::new("SYS_NAV_INTERNAL_ERROR"),
                message: msg,
            },
        }
    }
}
