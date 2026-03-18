//! Accounting Domain Master サービスのドメインエラー型。
//!
//! 文字列マッチングではなく、型安全な分類で HTTP ステータスコードを決定する。

use k1s0_server_common::error::{ErrorCode, ServiceError};

/// AccountingDomainMaster ドメイン固有のエラー型。
#[derive(Debug, thiserror::Error)]
pub enum AccountingDomainMasterError {
    /// マスターデータが見つからない
    #[error("accounting master data '{0}' not found")]
    NotFound(String),

    /// マスターデータが既に存在する
    #[error("accounting master data already exists: {0}")]
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

/// AccountingDomainMasterError から ServiceError への変換実装
impl From<AccountingDomainMasterError> for ServiceError {
    fn from(err: AccountingDomainMasterError) -> Self {
        match err {
            AccountingDomainMasterError::NotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("BIZ_ACCTMST_NOT_FOUND"),
                message: msg,
            },
            AccountingDomainMasterError::AlreadyExists(msg) => ServiceError::Conflict {
                code: ErrorCode::new("BIZ_ACCTMST_ALREADY_EXISTS"),
                message: msg,
                details: vec![],
            },
            AccountingDomainMasterError::VersionConflict(msg) => ServiceError::Conflict {
                code: ErrorCode::new("BIZ_ACCTMST_VERSION_CONFLICT"),
                message: msg,
                details: vec![],
            },
            AccountingDomainMasterError::ValidationFailed(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("BIZ_ACCTMST_VALIDATION_FAILED"),
                message: msg,
                details: vec![],
            },
            AccountingDomainMasterError::Internal(msg) => ServiceError::Internal {
                code: ErrorCode::new("BIZ_ACCTMST_INTERNAL_ERROR"),
                message: msg,
            },
        }
    }
}
