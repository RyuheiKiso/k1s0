//! Vault サービスのドメインエラー型。
//!
//! 文字列マッチングではなく、型安全な分類で HTTP ステータスコードを決定する。

use k1s0_server_common::error::{ErrorCode, ServiceError};

/// Vault ドメイン固有のエラー型。
#[derive(Debug, thiserror::Error)]
pub enum VaultError {
    /// シークレットが見つからない
    #[error("secret '{0}' not found")]
    NotFound(String),

    /// シークレットが既に存在する
    #[error("secret already exists: {0}")]
    AlreadyExists(String),

    /// アクセス権限がない
    #[error("access denied: {0}")]
    AccessDenied(String),

    /// バリデーションエラー
    #[error("validation failed: {0}")]
    ValidationFailed(String),

    /// 暗号化・復号化エラー
    #[error("encryption error: {0}")]
    EncryptionError(String),

    /// 内部エラー
    #[error("internal error: {0}")]
    Internal(String),
}

/// VaultError から ServiceError への変換実装
impl From<VaultError> for ServiceError {
    fn from(err: VaultError) -> Self {
        match err {
            VaultError::NotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_VAULT_NOT_FOUND"),
                message: msg,
            },
            VaultError::AlreadyExists(msg) => ServiceError::Conflict {
                code: ErrorCode::new("SYS_VAULT_ALREADY_EXISTS"),
                message: msg,
                details: vec![],
            },
            VaultError::AccessDenied(msg) => ServiceError::Forbidden {
                code: ErrorCode::new("SYS_VAULT_ACCESS_DENIED"),
                message: msg,
            },
            VaultError::ValidationFailed(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_VAULT_VALIDATION_FAILED"),
                message: msg,
                details: vec![],
            },
            VaultError::EncryptionError(msg) => ServiceError::Internal {
                code: ErrorCode::new("SYS_VAULT_ENCRYPTION_ERROR"),
                message: msg,
            },
            VaultError::Internal(msg) => ServiceError::Internal {
                code: ErrorCode::new("SYS_VAULT_INTERNAL_ERROR"),
                message: msg,
            },
        }
    }
}
