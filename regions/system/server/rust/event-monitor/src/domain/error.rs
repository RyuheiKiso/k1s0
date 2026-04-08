//! Event Monitor サービスのドメインエラー型。
//!
//! 文字列マッチングではなく、型安全な分類で HTTP ステータスコードを決定する。

use k1s0_server_common::error::{ErrorCode, ServiceError};

/// `EventMonitor` ドメイン固有のエラー型。
#[derive(Debug, thiserror::Error)]
pub enum EventMonitorError {
    /// イベントが見つからない
    #[error("event '{0}' not found")]
    NotFound(String),

    /// アラートルールが見つからない
    #[error("alert rule '{0}' not found")]
    AlertRuleNotFound(String),

    /// バリデーションエラー
    #[error("validation failed: {0}")]
    ValidationFailed(String),

    /// 内部エラー
    #[error("internal error: {0}")]
    Internal(String),
}

/// `EventMonitorError` から `ServiceError` への変換実装
impl From<EventMonitorError> for ServiceError {
    fn from(err: EventMonitorError) -> Self {
        match err {
            EventMonitorError::NotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_EVMON_NOT_FOUND"),
                message: msg,
            },
            EventMonitorError::AlertRuleNotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_EVMON_ALERT_RULE_NOT_FOUND"),
                message: msg,
            },
            EventMonitorError::ValidationFailed(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_EVMON_VALIDATION_FAILED"),
                message: msg,
                details: vec![],
            },
            EventMonitorError::Internal(msg) => ServiceError::Internal {
                code: ErrorCode::new("SYS_EVMON_INTERNAL_ERROR"),
                message: msg,
            },
        }
    }
}
