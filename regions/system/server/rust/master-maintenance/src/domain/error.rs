//! Master Maintenance サービスのドメインエラー型。
//!
//! 文字列マッチングではなく、型安全な分類で HTTP ステータスコードを決定する。

use k1s0_server_common::error::{ErrorCode, ServiceError};

/// MasterMaintenance ドメイン固有のエラー型。
#[derive(Debug, thiserror::Error)]
pub enum MasterMaintenanceError {
    /// マスターデータが見つからない
    #[error("master data '{0}' not found")]
    NotFound(String),

    /// マスターデータが既に存在する
    #[error("master data already exists: {0}")]
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

/// MasterMaintenanceError から ServiceError への変換実装
impl From<MasterMaintenanceError> for ServiceError {
    fn from(err: MasterMaintenanceError) -> Self {
        match err {
            MasterMaintenanceError::NotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_MSTMNT_NOT_FOUND"),
                message: msg,
            },
            MasterMaintenanceError::AlreadyExists(msg) => ServiceError::Conflict {
                code: ErrorCode::new("SYS_MSTMNT_ALREADY_EXISTS"),
                message: msg,
                details: vec![],
            },
            MasterMaintenanceError::VersionConflict(msg) => ServiceError::Conflict {
                code: ErrorCode::new("SYS_MSTMNT_VERSION_CONFLICT"),
                message: msg,
                details: vec![],
            },
            MasterMaintenanceError::ValidationFailed(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_MSTMNT_VALIDATION_FAILED"),
                message: msg,
                details: vec![],
            },
            MasterMaintenanceError::Internal(msg) => ServiceError::Internal {
                code: ErrorCode::new("SYS_MSTMNT_INTERNAL_ERROR"),
                message: msg,
            },
        }
    }
}
