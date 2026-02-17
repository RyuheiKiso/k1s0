use async_trait::async_trait;

use crate::domain::entity::audit_log::{AuditLog, AuditLogSearchParams};

/// AuditLogRepository は監査ログの永続化インターフェース。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AuditLogRepository: Send + Sync {
    /// 監査ログエントリを作成する。
    async fn create(&self, log: &AuditLog) -> anyhow::Result<()>;

    /// 監査ログを検索する。(logs, total_count) を返す。
    async fn search(
        &self,
        params: &AuditLogSearchParams,
    ) -> anyhow::Result<(Vec<AuditLog>, i64)>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_mock_audit_log_repository_create() {
        let mut mock = MockAuditLogRepository::new();
        mock.expect_create()
            .returning(|_| Ok(()));

        let log = AuditLog {
            id: Uuid::new_v4(),
            event_type: "LOGIN_SUCCESS".to_string(),
            user_id: "user-1".to_string(),
            ip_address: "127.0.0.1".to_string(),
            user_agent: "test".to_string(),
            resource: "/api/v1/auth/token".to_string(),
            action: "POST".to_string(),
            result: "SUCCESS".to_string(),
            metadata: std::collections::HashMap::new(),
            recorded_at: chrono::Utc::now(),
        };

        let result = mock.create(&log).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_audit_log_repository_search() {
        let mut mock = MockAuditLogRepository::new();
        mock.expect_search()
            .returning(|_| Ok((vec![], 0)));

        let params = AuditLogSearchParams::default();
        let (logs, total) = mock.search(&params).await.unwrap();
        assert!(logs.is_empty());
        assert_eq!(total, 0);
    }
}
