//! Master Maintenance サービスのドメインエラー型。
//!
//! 文字列マッチングではなく、型安全な分類で HTTP ステータスコードを決定する（C-04対応）。

use crate::usecase::crud_records::RecordValidationError;
use k1s0_server_common::error::{ErrorCode, ServiceError};

/// master-maintenance ドメインのエラー型。
/// 型安全なエラー分類によりエラーメッセージの文字列マッチングを不要にする（C-04対応）。
#[derive(Debug, thiserror::Error)]
pub enum MasterMaintenanceError {
    /// テーブル定義が見つからない
    #[error("table '{0}' not found")]
    TableNotFound(String),

    /// レコードが見つからない
    #[error("record '{0}' not found")]
    RecordNotFound(String),

    /// 整合性ルールが見つからない
    #[error("rule not found: {0}")]
    RuleNotFound(String),

    /// 表示設定が見つからない
    #[error("display config not found: {0}")]
    DisplayConfigNotFound(String),

    /// インポートジョブが見つからない
    #[error("import job not found: {0}")]
    ImportJobNotFound(String),

    /// テーブルリレーションシップが見つからない
    #[error("relationship not found: {0}")]
    RelationshipNotFound(String),

    /// カラム定義が見つからない
    #[error("column '{0}' not found")]
    ColumnNotFound(String),

    /// テーブルの操作権限がない（Create/Update/Delete not allowed）
    #[error("{operation} not allowed for table '{table_name}'")]
    OperationNotAllowed { table_name: String, operation: String },

    /// テーブル名が重複している
    #[error("table already exists: {0}")]
    DuplicateTable(String),

    /// カラム名が重複している
    #[error("column already exists: {0}")]
    DuplicateColumn(String),

    /// 整合性ルールが無効
    #[error("invalid rule: {0}")]
    InvalidRule(String),

    /// インポート処理が失敗した
    #[error("import failed: {0}")]
    ImportFailed(String),

    /// SQL構築エラー（動的クエリのビルドに失敗）
    #[error("sql build error: {0}")]
    SqlBuildError(String),

    /// バリデーションエラー
    #[error("validation failed: {0}")]
    ValidationFailed(String),

    /// レコードバリデーションエラー（ルール評価結果付き）
    // RecordValidationError の機能を統合。errors/warnings フィールドは既存の型を使用する。
    #[error("record validation failed")]
    RecordValidation(Box<RecordValidationError>),

    /// バージョン競合（楽観ロック）
    #[error("version conflict: {0}")]
    VersionConflict(String),

    /// 内部エラー
    #[error("internal error: {0}")]
    Internal(String),
}

/// anyhow::Error を MasterMaintenanceError に変換する実装。
/// 型変換が未対応の usecase から上がってきたエラーを Internal にフォールバックする（暫定対応）。
impl From<anyhow::Error> for MasterMaintenanceError {
    fn from(err: anyhow::Error) -> Self {
        // RecordValidationError を内包する anyhow::Error を型安全に変換する
        if let Some(validation) =
            err.downcast_ref::<crate::usecase::crud_records::RecordValidationError>()
        {
            // RecordValidationError は Clone を実装していないため、to_string() 経由で保存する
            return Self::ValidationFailed(validation.to_string());
        }
        Self::Internal(err.to_string())
    }
}

/// MasterMaintenanceError から ServiceError への変換実装。
/// 既存の SYS_MM_* エラーコードとの後方互換性を維持する。
impl From<MasterMaintenanceError> for ServiceError {
    fn from(err: MasterMaintenanceError) -> Self {
        match err {
            MasterMaintenanceError::TableNotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_MM_TABLE_NOT_FOUND"),
                message: msg,
            },
            MasterMaintenanceError::RecordNotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_MM_RECORD_NOT_FOUND"),
                message: msg,
            },
            MasterMaintenanceError::RuleNotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_MM_RULE_NOT_FOUND"),
                message: msg,
            },
            MasterMaintenanceError::DisplayConfigNotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_MM_DISPLAY_CONFIG_NOT_FOUND"),
                message: msg,
            },
            MasterMaintenanceError::ImportJobNotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_MM_IMPORT_JOB_NOT_FOUND"),
                message: msg,
            },
            MasterMaintenanceError::RelationshipNotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_MM_RELATIONSHIP_NOT_FOUND"),
                message: msg,
            },
            MasterMaintenanceError::ColumnNotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_MM_COLUMN_NOT_FOUND"),
                message: msg,
            },
            MasterMaintenanceError::OperationNotAllowed { table_name, operation } => {
                ServiceError::Forbidden {
                    code: ErrorCode::new("SYS_MM_OPERATION_NOT_ALLOWED"),
                    message: format!("{} not allowed for table '{}'", operation, table_name),
                }
            }
            MasterMaintenanceError::DuplicateTable(msg) => ServiceError::Conflict {
                code: ErrorCode::new("SYS_MM_DUPLICATE_TABLE"),
                message: msg,
                details: vec![],
            },
            MasterMaintenanceError::DuplicateColumn(msg) => ServiceError::Conflict {
                code: ErrorCode::new("SYS_MM_DUPLICATE_COLUMN"),
                message: msg,
                details: vec![],
            },
            MasterMaintenanceError::InvalidRule(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_MM_INVALID_RULE"),
                message: msg,
                details: vec![],
            },
            MasterMaintenanceError::ImportFailed(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_MM_IMPORT_FAILED"),
                message: msg,
                details: vec![],
            },
            MasterMaintenanceError::SqlBuildError(msg) => ServiceError::Internal {
                code: ErrorCode::new("SYS_MM_INTERNAL_ERROR"),
                message: msg,
            },
            MasterMaintenanceError::ValidationFailed(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_MM_VALIDATION_ERROR"),
                message: msg,
                details: vec![],
            },
            MasterMaintenanceError::RecordValidation(err) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_MM_VALIDATION_ERROR"),
                message: err.to_string(),
                details: vec![],
            },
            MasterMaintenanceError::VersionConflict(msg) => ServiceError::Conflict {
                code: ErrorCode::new("SYS_MM_VERSION_CONFLICT"),
                message: msg,
                details: vec![],
            },
            MasterMaintenanceError::Internal(msg) => ServiceError::Internal {
                code: ErrorCode::new("SYS_MM_INTERNAL_ERROR"),
                message: msg,
            },
        }
    }
}
