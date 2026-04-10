//! Policy サービスのドメインエラー型。
//!
//! 文字列マッチングではなく、型安全な分類で HTTP ステータスコードを決定する。

use k1s0_server_common::error::{ErrorCode, ServiceError};

/// Policy ドメイン固有のエラー型。
#[derive(Debug, thiserror::Error)]
pub enum PolicyError {
    /// ポリシーが見つからない
    #[error("policy '{0}' not found")]
    NotFound(String),

    /// ポリシーの評価が失敗
    #[error("policy evaluation failed: {0}")]
    EvaluationFailed(String),

    /// ポリシーが既に存在する
    #[error("policy already exists: {0}")]
    AlreadyExists(String),

    /// バリデーションエラー
    #[error("validation failed: {0}")]
    ValidationFailed(String),

    /// 内部エラー
    #[error("internal error: {0}")]
    Internal(String),
}

/// `PolicyError` から `ServiceError` への変換実装
impl From<PolicyError> for ServiceError {
    fn from(err: PolicyError) -> Self {
        match err {
            PolicyError::NotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_POLICY_NOT_FOUND"),
                message: msg,
            },
            PolicyError::EvaluationFailed(msg) => ServiceError::Internal {
                code: ErrorCode::new("SYS_POLICY_EVALUATION_FAILED"),
                message: msg,
            },
            PolicyError::AlreadyExists(msg) => ServiceError::Conflict {
                code: ErrorCode::new("SYS_POLICY_ALREADY_EXISTS"),
                message: msg,
                details: vec![],
            },
            PolicyError::ValidationFailed(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_POLICY_VALIDATION_FAILED"),
                message: msg,
                details: vec![],
            },
            PolicyError::Internal(msg) => ServiceError::Internal {
                code: ErrorCode::new("SYS_POLICY_INTERNAL_ERROR"),
                message: msg,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// NotFound エラーが ServiceError::NotFound に変換される
    #[test]
    fn not_found_to_service_error() {
        let err = PolicyError::NotFound("rbac".to_string());
        let svc: ServiceError = err.into();
        assert!(matches!(svc, ServiceError::NotFound { .. }));
    }

    /// AlreadyExists エラーが ServiceError::Conflict に変換される
    #[test]
    fn already_exists_to_conflict() {
        let err = PolicyError::AlreadyExists("rbac".to_string());
        let svc: ServiceError = err.into();
        assert!(matches!(svc, ServiceError::Conflict { .. }));
    }

    /// EvaluationFailed エラーが ServiceError::Internal に変換される
    #[test]
    fn evaluation_failed_to_internal() {
        let err = PolicyError::EvaluationFailed("opa timeout".to_string());
        let svc: ServiceError = err.into();
        assert!(matches!(svc, ServiceError::Internal { .. }));
    }

    /// ValidationFailed エラーが ServiceError::BadRequest に変換される
    #[test]
    fn validation_failed_to_bad_request() {
        let err = PolicyError::ValidationFailed("rego syntax error".to_string());
        let svc: ServiceError = err.into();
        assert!(matches!(svc, ServiceError::BadRequest { .. }));
    }
}
