//! Service Catalog サービスのドメインエラー型。
//!
//! 文字列マッチングではなく、型安全な分類で HTTP ステータスコードを決定する。

use k1s0_server_common::error::{ErrorCode, ServiceError};

/// ServiceCatalog ドメイン固有のエラー型。
#[derive(Debug, thiserror::Error)]
pub enum ServiceCatalogError {
    /// サービスが見つからない
    #[error("service '{0}' not found")]
    NotFound(String),

    /// サービスが既に存在する
    #[error("service already exists: {0}")]
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

/// ServiceCatalogError から ServiceError への変換実装
impl From<ServiceCatalogError> for ServiceError {
    fn from(err: ServiceCatalogError) -> Self {
        match err {
            ServiceCatalogError::NotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_SVCCAT_NOT_FOUND"),
                message: msg,
            },
            ServiceCatalogError::AlreadyExists(msg) => ServiceError::Conflict {
                code: ErrorCode::new("SYS_SVCCAT_ALREADY_EXISTS"),
                message: msg,
                details: vec![],
            },
            ServiceCatalogError::VersionConflict(msg) => ServiceError::Conflict {
                code: ErrorCode::new("SYS_SVCCAT_VERSION_CONFLICT"),
                message: msg,
                details: vec![],
            },
            ServiceCatalogError::ValidationFailed(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_SVCCAT_VALIDATION_FAILED"),
                message: msg,
                details: vec![],
            },
            ServiceCatalogError::Internal(msg) => ServiceError::Internal {
                code: ErrorCode::new("SYS_SVCCAT_INTERNAL_ERROR"),
                message: msg,
            },
        }
    }
}
