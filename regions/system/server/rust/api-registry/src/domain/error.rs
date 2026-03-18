//! API Registry サービスのドメインエラー型。
//!
//! 文字列マッチングではなく、型安全な分類で HTTP ステータスコードを決定する。

use k1s0_server_common::error::{ErrorCode, ServiceError};

/// ApiRegistry ドメイン固有のエラー型。
#[derive(Debug, thiserror::Error)]
pub enum ApiRegistryError {
    /// API が見つからない
    #[error("API '{0}' not found")]
    NotFound(String),

    /// API が既に存在する
    #[error("API already exists: {0}")]
    AlreadyExists(String),

    /// スキーマが無効
    #[error("invalid schema: {0}")]
    SchemaInvalid(String),

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

/// ApiRegistryError から ServiceError への変換実装
impl From<ApiRegistryError> for ServiceError {
    fn from(err: ApiRegistryError) -> Self {
        match err {
            ApiRegistryError::NotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_APIREG_NOT_FOUND"),
                message: msg,
            },
            ApiRegistryError::AlreadyExists(msg) => ServiceError::Conflict {
                code: ErrorCode::new("SYS_APIREG_ALREADY_EXISTS"),
                message: msg,
                details: vec![],
            },
            ApiRegistryError::SchemaInvalid(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_APIREG_SCHEMA_INVALID"),
                message: msg,
                details: vec![],
            },
            ApiRegistryError::VersionConflict(msg) => ServiceError::Conflict {
                code: ErrorCode::new("SYS_APIREG_CONFLICT"),
                message: msg,
                details: vec![],
            },
            ApiRegistryError::ValidationFailed(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_APIREG_VALIDATION_ERROR"),
                message: msg,
                details: vec![],
            },
            ApiRegistryError::Internal(msg) => ServiceError::Internal {
                code: ErrorCode::new("SYS_APIREG_INTERNAL_ERROR"),
                message: msg,
            },
        }
    }
}
