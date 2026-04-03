//! HTTPレスポンス用のアプリケーションエラー型。
//!
//! MasterMaintenanceError から型安全に HTTP ステータスコードへ変換する（C-04対応）。
//! 文字列マッチングによるエラー分類を廃止し、enum の match で変換する。

use crate::adapter::presenter::response::{ErrorDetail, ErrorResponse};
use crate::domain::error::MasterMaintenanceError;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

/// HTTP レスポンスとして返却するアプリケーションエラー型。
#[derive(Debug)]
pub struct AppError {
    pub status: StatusCode,
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl AppError {
    /// 404 Not Found レスポンスを生成する。
    pub fn not_found(code: &str, message: &str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: code.to_string(),
            message: message.to_string(),
            details: None,
        }
    }

    /// 400 Bad Request レスポンスを生成する。
    pub fn bad_request(code: &str, message: &str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: code.to_string(),
            message: message.to_string(),
            details: None,
        }
    }

    /// 409 Conflict レスポンスを生成する。
    pub fn conflict(code: &str, message: &str) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            code: code.to_string(),
            message: message.to_string(),
            details: None,
        }
    }

    /// 401 Unauthorized レスポンスを生成する。
    pub fn unauthorized(code: &str, message: &str) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: code.to_string(),
            message: message.to_string(),
            details: None,
        }
    }

    /// 403 Forbidden レスポンスを生成する。
    pub fn forbidden(code: &str, message: &str) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            code: code.to_string(),
            message: message.to_string(),
            details: None,
        }
    }

    /// 500 Internal Server Error レスポンスを生成する。
    pub fn internal(code: &str, message: &str) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: code.to_string(),
            message: message.to_string(),
            details: None,
        }
    }

    /// エラー詳細情報を付与する。バリデーションエラーの errors/warnings に使用する。
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

/// AppError を Axum の HTTP レスポンスに変換する実装。
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = ErrorResponse {
            error: ErrorDetail {
                code: self.code,
                message: self.message,
                request_id: None,
                details: self.details,
            },
        };
        (self.status, Json(body)).into_response()
    }
}

/// anyhow::Error を AppError に変換する実装。
/// MasterMaintenanceError に変換できない anyhow::Error を Internal としてフォールバックする。
/// 文字列マッチングは行わず、型安全な変換を介して処理する。
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        // anyhow::Error を MasterMaintenanceError に変換してから AppError に変換する
        Self::from(MasterMaintenanceError::from(err))
    }
}

/// MasterMaintenanceError を AppError に型安全に変換する実装（C-04対応）。
/// 文字列マッチングを廃止し、enum の各バリアントに対して正確な HTTP ステータスコードとエラーコードを割り当てる。
/// RUST-LOW-004 対応: Internal/SqlBuildError の詳細はクライアントに漏洩しない（固定文字列を返す）
impl From<MasterMaintenanceError> for AppError {
    fn from(err: MasterMaintenanceError) -> Self {
        match &err {
            MasterMaintenanceError::TableNotFound(_) => {
                Self::not_found("SYS_MM_TABLE_NOT_FOUND", &err.to_string())
            }
            MasterMaintenanceError::RecordNotFound(_) => {
                Self::not_found("SYS_MM_RECORD_NOT_FOUND", &err.to_string())
            }
            MasterMaintenanceError::RuleNotFound(_) => {
                Self::not_found("SYS_MM_RULE_NOT_FOUND", &err.to_string())
            }
            MasterMaintenanceError::DisplayConfigNotFound(_) => {
                Self::not_found("SYS_MM_DISPLAY_CONFIG_NOT_FOUND", &err.to_string())
            }
            MasterMaintenanceError::ImportJobNotFound(_) => {
                Self::not_found("SYS_MM_IMPORT_JOB_NOT_FOUND", &err.to_string())
            }
            MasterMaintenanceError::RelationshipNotFound(_) => {
                Self::not_found("SYS_MM_RELATIONSHIP_NOT_FOUND", &err.to_string())
            }
            MasterMaintenanceError::ColumnNotFound(_) => {
                Self::not_found("SYS_MM_COLUMN_NOT_FOUND", &err.to_string())
            }
            MasterMaintenanceError::OperationNotAllowed { .. } => {
                Self::forbidden("SYS_MM_OPERATION_NOT_ALLOWED", &err.to_string())
            }
            MasterMaintenanceError::DuplicateTable(_) => {
                Self::conflict("SYS_MM_DUPLICATE_TABLE", &err.to_string())
            }
            MasterMaintenanceError::DuplicateColumn(_) => {
                Self::conflict("SYS_MM_DUPLICATE_COLUMN", &err.to_string())
            }
            MasterMaintenanceError::InvalidRule(_) => {
                Self::bad_request("SYS_MM_INVALID_RULE", &err.to_string())
            }
            MasterMaintenanceError::ImportFailed(_) => {
                Self::bad_request("SYS_MM_IMPORT_FAILED", &err.to_string())
            }
            // RUST-LOW-004 対応: SQL 構築エラーの詳細（クエリ内容等）はクライアントに漏洩しない
            // 詳細はサーバーログにのみ記録する
            MasterMaintenanceError::SqlBuildError(e) => {
                tracing::error!("SQL構築エラーが発生しました: {:?}", e);
                Self::internal("SYS_MM_INTERNAL_ERROR", "内部エラーが発生しました")
            }
            MasterMaintenanceError::ValidationFailed(_) => {
                Self::bad_request("SYS_MM_VALIDATION_ERROR", &err.to_string())
            }
            // RecordValidation は errors/warnings の詳細情報を JSON で付与する
            MasterMaintenanceError::RecordValidation(validation) => {
                Self::bad_request("SYS_MM_VALIDATION_ERROR", &err.to_string()).with_details(
                    serde_json::json!({
                        "errors": validation.errors,
                        "warnings": validation.warnings,
                    }),
                )
            }
            MasterMaintenanceError::VersionConflict(_) => {
                Self::conflict("SYS_MM_VERSION_CONFLICT", &err.to_string())
            }
            // RUST-LOW-004 対応: Internal エラーの詳細はクライアントに漏洩しない
            // 詳細はサーバーログにのみ記録する
            MasterMaintenanceError::Internal(e) => {
                tracing::error!("内部エラーが発生しました: {:?}", e);
                Self::internal("SYS_MM_INTERNAL_ERROR", "内部エラーが発生しました")
            }
        }
    }
}
