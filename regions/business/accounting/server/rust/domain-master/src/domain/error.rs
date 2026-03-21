//! Accounting Domain Master サービスのドメインエラー型。
//!
//! 文字列マッチングではなく、型安全な分類で HTTP ステータスコードを決定する。

use k1s0_server_common::error::{ErrorCode, ServiceError};

/// AccountingDomainMaster ドメイン固有のエラー型。
/// エラーメッセージはハンドラー層と統一するため日本語で記述する（m-15 対応）。
#[derive(Debug, thiserror::Error)]
pub enum AccountingDomainMasterError {
    /// マスターデータが見つからない
    #[error("会計マスタデータ '{0}' が見つかりません")]
    NotFound(String),

    /// マスターデータが既に存在する
    #[error("会計マスタデータが既に存在します: {0}")]
    AlreadyExists(String),

    /// バージョン競合
    #[error("バージョン競合が発生しました: {0}")]
    VersionConflict(String),

    /// バリデーションエラー
    #[error("バリデーションに失敗しました: {0}")]
    ValidationFailed(String),

    /// 内部エラー
    #[error("内部エラーが発生しました: {0}")]
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
