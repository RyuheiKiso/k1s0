use std::sync::Arc;

use crate::domain::entity::audit_log::{AuditLog, CreateAuditLogRequest, CreateAuditLogResponse};
use crate::domain::repository::AuditLogRepository;

/// RecordAuditLogError は監査ログ記録に関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum RecordAuditLogError {
    #[error("validation error: {0}")]
    Validation(String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// RecordAuditLogUseCase は監査ログ記録ユースケース。
pub struct RecordAuditLogUseCase {
    audit_repo: Arc<dyn AuditLogRepository>,
}

impl RecordAuditLogUseCase {
    pub fn new(audit_repo: Arc<dyn AuditLogRepository>) -> Self {
        Self { audit_repo }
    }

    /// 監査ログエントリを記録する。
    pub async fn execute(
        &self,
        req: CreateAuditLogRequest,
    ) -> Result<CreateAuditLogResponse, RecordAuditLogError> {
        // バリデーション
        if req.event_type.is_empty() {
            return Err(RecordAuditLogError::Validation(
                "event_type is required".to_string(),
            ));
        }
        if req.user_id.is_empty() {
            return Err(RecordAuditLogError::Validation(
                "user_id is required".to_string(),
            ));
        }
        if req.result != "SUCCESS" && req.result != "FAILURE" {
            return Err(RecordAuditLogError::Validation(
                "result must be SUCCESS or FAILURE".to_string(),
            ));
        }

        let log = AuditLog::new(req);
        let response = CreateAuditLogResponse {
            id: log.id,
            recorded_at: log.recorded_at,
        };

        self.audit_repo
            .create(&log)
            .await
            .map_err(|e| RecordAuditLogError::Internal(e.to_string()))?;

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::audit_log_repository::MockAuditLogRepository;
    use std::collections::HashMap;

    fn make_valid_request() -> CreateAuditLogRequest {
        CreateAuditLogRequest {
            event_type: "LOGIN_SUCCESS".to_string(),
            user_id: "user-uuid-1234".to_string(),
            ip_address: "192.168.1.100".to_string(),
            user_agent: "Mozilla/5.0".to_string(),
            resource: "/api/v1/auth/token".to_string(),
            action: "POST".to_string(),
            result: "SUCCESS".to_string(),
            metadata: HashMap::from([
                ("client_id".to_string(), "react-spa".to_string()),
            ]),
        }
    }

    #[tokio::test]
    async fn test_record_audit_log_success() {
        let mut mock = MockAuditLogRepository::new();
        mock.expect_create()
            .returning(|_| Ok(()));

        let uc = RecordAuditLogUseCase::new(Arc::new(mock));
        let result = uc.execute(make_valid_request()).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.id.is_nil());
    }

    #[tokio::test]
    async fn test_record_audit_log_empty_event_type() {
        let mock = MockAuditLogRepository::new();
        let uc = RecordAuditLogUseCase::new(Arc::new(mock));

        let mut req = make_valid_request();
        req.event_type = String::new();

        let result = uc.execute(req).await;
        assert!(matches!(
            result.unwrap_err(),
            RecordAuditLogError::Validation(_)
        ));
    }

    #[tokio::test]
    async fn test_record_audit_log_empty_user_id() {
        let mock = MockAuditLogRepository::new();
        let uc = RecordAuditLogUseCase::new(Arc::new(mock));

        let mut req = make_valid_request();
        req.user_id = String::new();

        let result = uc.execute(req).await;
        assert!(matches!(
            result.unwrap_err(),
            RecordAuditLogError::Validation(_)
        ));
    }

    #[tokio::test]
    async fn test_record_audit_log_invalid_result() {
        let mock = MockAuditLogRepository::new();
        let uc = RecordAuditLogUseCase::new(Arc::new(mock));

        let mut req = make_valid_request();
        req.result = "INVALID".to_string();

        let result = uc.execute(req).await;
        match result.unwrap_err() {
            RecordAuditLogError::Validation(msg) => {
                assert!(msg.contains("SUCCESS or FAILURE"));
            }
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_record_audit_log_failure_result() {
        let mut mock = MockAuditLogRepository::new();
        mock.expect_create()
            .returning(|_| Ok(()));

        let uc = RecordAuditLogUseCase::new(Arc::new(mock));
        let mut req = make_valid_request();
        req.result = "FAILURE".to_string();
        req.event_type = "LOGIN_FAILURE".to_string();

        let result = uc.execute(req).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_record_audit_log_repository_error() {
        let mut mock = MockAuditLogRepository::new();
        mock.expect_create()
            .returning(|_| Err(anyhow::anyhow!("database connection failed")));

        let uc = RecordAuditLogUseCase::new(Arc::new(mock));
        let result = uc.execute(make_valid_request()).await;
        assert!(matches!(
            result.unwrap_err(),
            RecordAuditLogError::Internal(_)
        ));
    }
}
