//! App Registry サービスのドメインエラー型。
//!
//! 文字列マッチングではなく、型安全な分類で HTTP ステータスコードを決定する。

use k1s0_server_common::error::{ErrorCode, ServiceError};

/// `AppRegistry` ドメイン固有のエラー型。
#[derive(Debug, thiserror::Error)]
pub enum AppRegistryError {
    /// アプリが見つからない
    #[error("app '{0}' not found")]
    NotFound(String),

    /// アプリが既に存在する
    #[error("app already exists: {0}")]
    AlreadyExists(String),

    /// バージョン競合
    #[error("version conflict: {0}")]
    VersionConflict(String),

    /// バリデーションエラー
    #[error("validation failed: {0}")]
    ValidationFailed(String),

    /// 内部エラー
    #[error("internal error: {0}")]
    Internal(String),
}

/// `AppRegistryError` から `ServiceError` への変換実装
impl From<AppRegistryError> for ServiceError {
    fn from(err: AppRegistryError) -> Self {
        match err {
            AppRegistryError::NotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_APPREG_NOT_FOUND"),
                message: msg,
            },
            AppRegistryError::AlreadyExists(msg) => ServiceError::Conflict {
                code: ErrorCode::new("SYS_APPREG_ALREADY_EXISTS"),
                message: msg,
                details: vec![],
            },
            AppRegistryError::VersionConflict(msg) => ServiceError::Conflict {
                code: ErrorCode::new("SYS_APPREG_VERSION_CONFLICT"),
                message: msg,
                details: vec![],
            },
            AppRegistryError::ValidationFailed(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_APPREG_VALIDATION_FAILED"),
                message: msg,
                details: vec![],
            },
            AppRegistryError::Internal(msg) => ServiceError::Internal {
                code: ErrorCode::new("SYS_APPREG_INTERNAL_ERROR"),
                message: msg,
            },
        }
    }
}
