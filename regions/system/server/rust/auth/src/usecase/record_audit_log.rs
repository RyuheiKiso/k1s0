use std::sync::Arc;

use crate::domain::entity::audit_log::{AuditLog, CreateAuditLogRequest, CreateAuditLogResponse};
use crate::domain::repository::AuditLogRepository;
use crate::infrastructure::kafka_producer::AuditEventPublisher;

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
    publisher: Option<Arc<dyn AuditEventPublisher>>,
}

impl RecordAuditLogUseCase {
    pub fn new(audit_repo: Arc<dyn AuditLogRepository>) -> Self {
        Self {
            audit_repo,
            publisher: None,
        }
    }

    pub fn with_publisher(
        audit_repo: Arc<dyn AuditLogRepository>,
        publisher: Arc<dyn AuditEventPublisher>,
    ) -> Self {
        Self {
            audit_repo,
            publisher: Some(publisher),
        }
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
            created_at: log.created_at,
        };

        self.audit_repo
            .create(&log)
            .await
            .map_err(|e| RecordAuditLogError::Internal(e.to_string()))?;

        // Kafka に非同期配信（エラーは無視して記録は成功とする）
        if let Some(ref publisher) = self.publisher {
            let _ = publisher.publish(&log).await;
        }

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::audit_log_repository::MockAuditLogRepository;
    use crate::infrastructure::kafka_producer::MockAuditEventPublisher;

    fn make_valid_request() -> CreateAuditLogRequest {
        CreateAuditLogRequest {
            event_type: "LOGIN_SUCCESS".to_string(),
            user_id: "user-uuid-1234".to_string(),
            ip_address: "192.168.1.100".to_string(),
            user_agent: "Mozilla/5.0".to_string(),
            resource: "/api/v1/auth/token".to_string(),
            resource_id: None,
            action: "POST".to_string(),
            result: "SUCCESS".to_string(),
            detail: Some(serde_json::json!({"client_id": "react-spa"})),
            trace_id: None,
        }
    }

    #[tokio::test]
    async fn test_record_audit_log_success() {
        let mut mock = MockAuditLogRepository::new();
        mock.expect_create().returning(|_| Ok(()));

        let uc = RecordAuditLogUseCase::new(Arc::new(mock));
        let result = uc.execute(make_valid_request()).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.id.is_nil());
    }

    #[tokio::test]
    async fn test_record_audit_log_with_publisher() {
        let mut mock_repo = MockAuditLogRepository::new();
        mock_repo.expect_create().returning(|_| Ok(()));

        let mut mock_pub = MockAuditEventPublisher::new();
        mock_pub.expect_publish().returning(|_| Ok(()));

        let uc = RecordAuditLogUseCase::with_publisher(Arc::new(mock_repo), Arc::new(mock_pub));
        let result = uc.execute(make_valid_request()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_record_audit_log_publisher_error_ignored() {
        let mut mock_repo = MockAuditLogRepository::new();
        mock_repo.expect_create().returning(|_| Ok(()));

        let mut mock_pub = MockAuditEventPublisher::new();
        mock_pub
            .expect_publish()
            .returning(|_| Err(anyhow::anyhow!("kafka error")));

        let uc = RecordAuditLogUseCase::with_publisher(Arc::new(mock_repo), Arc::new(mock_pub));
        // publisher のエラーは無視して成功とする
        let result = uc.execute(make_valid_request()).await;
        assert!(result.is_ok());
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
            e => unreachable!("unexpected error in test: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_record_audit_log_failure_result() {
        let mut mock = MockAuditLogRepository::new();
        mock.expect_create().returning(|_| Ok(()));

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
