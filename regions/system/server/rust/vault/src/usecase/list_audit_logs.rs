use std::sync::Arc;

use crate::domain::entity::access_log::SecretAccessLog;
use crate::domain::repository::AccessLogRepository;

/// ListAuditLogsInput は監査ログ一覧取得の入力。
pub struct ListAuditLogsInput {
    pub offset: u32,
    pub limit: u32,
}

/// ListAuditLogsUseCase は監査ログ一覧取得ユースケース。
pub struct ListAuditLogsUseCase {
    repo: Arc<dyn AccessLogRepository>,
}

impl ListAuditLogsUseCase {
    pub fn new(repo: Arc<dyn AccessLogRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, input: &ListAuditLogsInput) -> anyhow::Result<Vec<SecretAccessLog>> {
        self.repo.list(input.offset, input.limit).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::access_log::AccessAction;
    use crate::domain::repository::access_log_repo::MockAccessLogRepository;

    #[tokio::test]
    async fn test_list_audit_logs_success() {
        let mut mock = MockAccessLogRepository::new();
        mock.expect_list().returning(|_, _| {
            Ok(vec![SecretAccessLog::new(
                "app/db".to_string(),
                AccessAction::Read,
                Some("user-1".to_string()),
                true,
            )])
        });

        let uc = ListAuditLogsUseCase::new(Arc::new(mock));
        let result = uc
            .execute(&ListAuditLogsInput { offset: 0, limit: 20 })
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_list_audit_logs_empty() {
        let mut mock = MockAccessLogRepository::new();
        mock.expect_list().returning(|_, _| Ok(vec![]));

        let uc = ListAuditLogsUseCase::new(Arc::new(mock));
        let result = uc
            .execute(&ListAuditLogsInput { offset: 0, limit: 20 })
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
