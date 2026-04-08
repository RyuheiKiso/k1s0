//! Tenant サービスのドメインエラー型。
//!
//! 文字列マッチングではなく、型安全な分類で HTTP ステータスコードを決定する。

use k1s0_server_common::error::{ErrorCode, ServiceError};

/// Tenant ドメイン固有のエラー型。
#[derive(Debug, thiserror::Error)]
pub enum TenantError {
    /// テナントが見つからない
    #[error("tenant '{0}' not found")]
    NotFound(String),

    /// テナント名が重複している
    #[error("tenant name conflict: {0}")]
    NameConflict(String),

    /// テナントのステータスが無効
    #[error("invalid tenant status: {0}")]
    InvalidStatus(String),

    /// メンバーが既に存在する
    #[error("member conflict: {0}")]
    MemberConflict(String),

    /// メンバーが見つからない
    #[error("member not found: {0}")]
    MemberNotFound(String),

    /// バリデーションエラー
    #[error("validation failed: {0}")]
    ValidationFailed(String),

    /// 内部エラー
    #[error("internal error: {0}")]
    Internal(String),
}

/// `TenantError` から `ServiceError` への変換実装
impl From<TenantError> for ServiceError {
    fn from(err: TenantError) -> Self {
        match err {
            TenantError::NotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_TENANT_NOT_FOUND"),
                message: msg,
            },
            TenantError::NameConflict(msg) => ServiceError::Conflict {
                code: ErrorCode::new("SYS_TENANT_NAME_CONFLICT"),
                message: msg,
                details: vec![],
            },
            TenantError::InvalidStatus(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_TENANT_INVALID_STATUS"),
                message: msg,
                details: vec![],
            },
            TenantError::MemberConflict(msg) => ServiceError::Conflict {
                code: ErrorCode::new("SYS_TENANT_MEMBER_CONFLICT"),
                message: msg,
                details: vec![],
            },
            TenantError::MemberNotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_TENANT_MEMBER_NOT_FOUND"),
                message: msg,
            },
            TenantError::ValidationFailed(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_TENANT_VALIDATION_ERROR"),
                message: msg,
                details: vec![],
            },
            TenantError::Internal(msg) => ServiceError::Internal {
                code: ErrorCode::new("SYS_TENANT_INTERNAL_ERROR"),
                message: msg,
            },
        }
    }
}
