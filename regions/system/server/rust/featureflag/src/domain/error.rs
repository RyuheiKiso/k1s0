//! FeatureFlag サービスのドメインエラー型。
//!
//! 文字列マッチングではなく、型安全な分類で HTTP ステータスコードを決定する。

use k1s0_server_common::error::{ErrorCode, ServiceError};

/// FeatureFlag ドメイン固有のエラー型。
#[derive(Debug, thiserror::Error)]
pub enum FeatureFlagError {
    /// フラグが見つからない
    #[error("feature flag '{0}' not found")]
    NotFound(String),

    /// フラグが既に存在する
    #[error("feature flag already exists: {0}")]
    AlreadyExists(String),

    /// フラグの評価に失敗
    #[error("evaluation failed: {0}")]
    EvaluationFailed(String),

    /// バリデーションエラー
    #[error("validation failed: {0}")]
    ValidationFailed(String),

    /// 内部エラー
    #[error("internal error: {0}")]
    Internal(String),
}

/// FeatureFlagError から ServiceError への変換実装
impl From<FeatureFlagError> for ServiceError {
    fn from(err: FeatureFlagError) -> Self {
        match err {
            FeatureFlagError::NotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_FF_NOT_FOUND"),
                message: msg,
            },
            FeatureFlagError::AlreadyExists(msg) => ServiceError::Conflict {
                code: ErrorCode::new("SYS_FF_ALREADY_EXISTS"),
                message: msg,
                details: vec![],
            },
            FeatureFlagError::EvaluationFailed(msg) => ServiceError::Internal {
                code: ErrorCode::new("SYS_FF_EVALUATE_FAILED"),
                message: msg,
            },
            FeatureFlagError::ValidationFailed(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_FF_VALIDATION_FAILED"),
                message: msg,
                details: vec![],
            },
            FeatureFlagError::Internal(msg) => ServiceError::Internal {
                code: ErrorCode::new("SYS_FF_INTERNAL_ERROR"),
                message: msg,
            },
        }
    }
}
