//! Rule Engine サービスのドメインエラー型。
//!
//! 文字列マッチングではなく、型安全な分類で HTTP ステータスコードを決定する。

use k1s0_server_common::error::{ErrorCode, ServiceError};

/// RuleEngine ドメイン固有のエラー型。
#[derive(Debug, thiserror::Error)]
pub enum RuleEngineError {
    /// ルールが見つからない
    #[error("rule '{0}' not found")]
    NotFound(String),

    /// ルールの評価に失敗
    #[error("rule evaluation failed: {0}")]
    EvaluationFailed(String),

    /// ルールが既に存在する
    #[error("rule already exists: {0}")]
    AlreadyExists(String),

    /// バリデーションエラー
    #[error("validation failed: {0}")]
    ValidationFailed(String),

    /// 内部エラー
    #[error("internal error: {0}")]
    Internal(String),
}

/// RuleEngineError から ServiceError への変換実装
impl From<RuleEngineError> for ServiceError {
    fn from(err: RuleEngineError) -> Self {
        match err {
            RuleEngineError::NotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_RULE_NOT_FOUND"),
                message: msg,
            },
            RuleEngineError::EvaluationFailed(msg) => ServiceError::Internal {
                code: ErrorCode::new("SYS_RULE_EVALUATION_FAILED"),
                message: msg,
            },
            RuleEngineError::AlreadyExists(msg) => ServiceError::Conflict {
                code: ErrorCode::new("SYS_RULE_ALREADY_EXISTS"),
                message: msg,
                details: vec![],
            },
            RuleEngineError::ValidationFailed(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_RULE_VALIDATION_FAILED"),
                message: msg,
                details: vec![],
            },
            RuleEngineError::Internal(msg) => ServiceError::Internal {
                code: ErrorCode::new("SYS_RULE_INTERNAL_ERROR"),
                message: msg,
            },
        }
    }
}
