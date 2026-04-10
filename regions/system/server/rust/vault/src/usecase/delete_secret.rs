use std::sync::Arc;

use crate::domain::entity::access_log::{AccessAction, SecretAccessLog};
use crate::domain::repository::{AccessLogRepository, SecretStore};
use crate::infrastructure::kafka_producer::{VaultAccessEvent, VaultEventPublisher};

/// MED-011 対応: `tenant_id` をアクセスログに記録するために追加。
#[derive(Debug, Clone)]
pub struct DeleteSecretInput {
    pub path: String,
    pub versions: Vec<i64>,
    /// gRPC 層で Claims から抽出したテナント ID。アクセスログに記録する。
    pub tenant_id: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteSecretError {
    #[error("secret not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct DeleteSecretUseCase {
    store: Arc<dyn SecretStore>,
    audit: Arc<dyn AccessLogRepository>,
    event_publisher: Arc<dyn VaultEventPublisher>,
}

impl DeleteSecretUseCase {
    pub fn new(
        store: Arc<dyn SecretStore>,
        audit: Arc<dyn AccessLogRepository>,
        event_publisher: Arc<dyn VaultEventPublisher>,
    ) -> Self {
        Self {
            store,
            audit,
            event_publisher,
        }
    }

    pub async fn execute(&self, input: &DeleteSecretInput) -> Result<(), DeleteSecretError> {
        let result = self.store.delete(&input.path, input.versions.clone()).await;

        match &result {
            Ok(()) => {
                // MED-011 対応: アクセスログに tenant_id を設定する。
                let mut log =
                    SecretAccessLog::new(input.path.clone(), AccessAction::Delete, None, true);
                log.tenant_id = input.tenant_id.clone();
                let _ = self.audit.record(&log).await;
                let _ = self
                    .event_publisher
                    .publish_secret_accessed(&VaultAccessEvent {
                        key_path: input.path.clone(),
                        action: "delete".to_string(),
                        actor_id: "system".to_string(),
                        success: true,
                        error_msg: None,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    })
                    .await;
            }
            Err(e) => {
                // MED-011 対応: エラー時のアクセスログにも tenant_id を設定する。
                let mut log =
                    SecretAccessLog::new(input.path.clone(), AccessAction::Delete, None, false);
                log.tenant_id = input.tenant_id.clone();
                log.error_msg = Some(e.to_string());
                let _ = self.audit.record(&log).await;
                let _ = self
                    .event_publisher
                    .publish_secret_accessed(&VaultAccessEvent {
                        key_path: input.path.clone(),
                        action: "delete".to_string(),
                        actor_id: "system".to_string(),
                        success: false,
                        error_msg: Some(e.to_string()),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    })
                    .await;
            }
        }

        result.map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                DeleteSecretError::NotFound(input.path.clone())
            } else {
                DeleteSecretError::Internal(msg)
            }
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::access_log_repo::MockAccessLogRepository;
    use crate::domain::repository::secret_store::MockSecretStore;
    use crate::infrastructure::kafka_producer::NoopVaultEventPublisher;

    #[tokio::test]
    async fn test_delete_secret_success() {
        let mut mock_store = MockSecretStore::new();
        let mut mock_audit = MockAccessLogRepository::new();

        mock_store
            .expect_delete()
            .withf(|path, versions| path == "app/db/password" && versions == &[1])
            .returning(|_, _| Ok(()));

        mock_audit.expect_record().returning(|_| Ok(()));

        let uc = DeleteSecretUseCase::new(
            Arc::new(mock_store),
            Arc::new(mock_audit),
            Arc::new(NoopVaultEventPublisher),
        );
        let input = DeleteSecretInput {
            path: "app/db/password".to_string(),
            versions: vec![1],
            tenant_id: Some("test-tenant".to_string()),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_secret_not_found() {
        let mut mock_store = MockSecretStore::new();
        let mut mock_audit = MockAccessLogRepository::new();

        mock_store
            .expect_delete()
            .returning(|_, _| Err(anyhow::anyhow!("secret not found: nonexistent")));

        mock_audit.expect_record().returning(|_| Ok(()));

        let uc = DeleteSecretUseCase::new(
            Arc::new(mock_store),
            Arc::new(mock_audit),
            Arc::new(NoopVaultEventPublisher),
        );
        let input = DeleteSecretInput {
            path: "nonexistent".to_string(),
            versions: vec![],
            tenant_id: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            DeleteSecretError::NotFound(path) => assert_eq!(path, "nonexistent"),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
