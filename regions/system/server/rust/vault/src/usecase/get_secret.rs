use std::sync::Arc;

use crate::domain::entity::access_log::{AccessAction, SecretAccessLog};
use crate::domain::entity::secret::Secret;
use crate::domain::repository::{AccessLogRepository, SecretStore};
use crate::infrastructure::kafka_producer::{VaultAccessEvent, VaultEventPublisher};

#[derive(Debug, Clone)]
pub struct GetSecretInput {
    pub path: String,
    pub version: Option<i64>,
    /// MED-011 監査対応: JWT クレームから伝播したテナント ID
    /// None `の場合はシステム全体スコープとして扱う（key_path` による論理分離: ADR-0056 参照）
    pub tenant_id: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum GetSecretError {
    #[error("secret not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct GetSecretUseCase {
    store: Arc<dyn SecretStore>,
    audit: Arc<dyn AccessLogRepository>,
    event_publisher: Arc<dyn VaultEventPublisher>,
}

impl GetSecretUseCase {
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

    pub async fn execute(&self, input: &GetSecretInput) -> Result<Secret, GetSecretError> {
        let result = self.store.get(&input.path, input.version).await;

        match &result {
            Ok(_) => {
                // MED-011 監査対応: tenant_id を access_log に設定して監査ログのテナント分離を実現する
                let mut log = SecretAccessLog::new(input.path.clone(), AccessAction::Read, None, true);
                log.tenant_id = input.tenant_id.clone();
                let _ = self.audit.record(&log).await;
                let _ = self
                    .event_publisher
                    .publish_secret_accessed(&VaultAccessEvent {
                        key_path: input.path.clone(),
                        action: "read".to_string(),
                        actor_id: "system".to_string(),
                        success: true,
                        error_msg: None,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    })
                    .await;
            }
            Err(e) => {
                // MED-011 監査対応: エラー時の access_log にも tenant_id を設定する
                let mut log =
                    SecretAccessLog::new(input.path.clone(), AccessAction::Read, None, false);
                log.tenant_id = input.tenant_id.clone();
                log.error_msg = Some(e.to_string());
                let _ = self.audit.record(&log).await;
                let _ = self
                    .event_publisher
                    .publish_secret_accessed(&VaultAccessEvent {
                        key_path: input.path.clone(),
                        action: "read".to_string(),
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
                GetSecretError::NotFound(input.path.clone())
            } else {
                GetSecretError::Internal(msg)
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
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_get_secret_success() {
        let mut mock_store = MockSecretStore::new();
        let mut mock_audit = MockAccessLogRepository::new();

        let data = HashMap::from([("password".to_string(), "s3cret".to_string())]);
        let secret = Secret::new("app/db/password".to_string(), data);
        let expected_path = secret.path.clone();

        mock_store
            .expect_get()
            .withf(|path, version| path == "app/db/password" && version.is_none())
            .returning(move |_, _| Ok(secret.clone()));

        mock_audit.expect_record().returning(|_| Ok(()));

        let uc = GetSecretUseCase::new(
            Arc::new(mock_store),
            Arc::new(mock_audit),
            Arc::new(NoopVaultEventPublisher),
        );
        let input = GetSecretInput {
            path: "app/db/password".to_string(),
            version: None,
            tenant_id: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let secret = result.unwrap();
        assert_eq!(secret.path, expected_path);
        assert_eq!(secret.versions[0].value.data["password"], "s3cret");
    }

    #[tokio::test]
    async fn test_get_secret_not_found() {
        let mut mock_store = MockSecretStore::new();
        let mut mock_audit = MockAccessLogRepository::new();

        mock_store
            .expect_get()
            .returning(|_, _| Err(anyhow::anyhow!("secret not found: nonexistent")));

        mock_audit.expect_record().returning(|_| Ok(()));

        let uc = GetSecretUseCase::new(
            Arc::new(mock_store),
            Arc::new(mock_audit),
            Arc::new(NoopVaultEventPublisher),
        );
        let input = GetSecretInput {
            path: "nonexistent".to_string(),
            version: None,
            tenant_id: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            GetSecretError::NotFound(path) => assert_eq!(path, "nonexistent"),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
