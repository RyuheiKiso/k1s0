use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::entity::access_log::{AccessAction, SecretAccessLog};
use crate::domain::repository::{AccessLogRepository, SecretStore};
use crate::infrastructure::kafka_producer::{VaultAccessEvent, VaultEventPublisher};

/// MED-011 対応: `tenant_id` をアクセスログに記録するために追加。
/// Vault シークレットストア自体はキーパス規約（{service}/{tenant}/key）でテナント分離するが
/// アクセスログのテナント属性は JWT Claims から取得する必要がある（ADR-0056 フェーズ3）。
#[derive(Debug, Clone)]
pub struct SetSecretInput {
    pub path: String,
    pub data: HashMap<String, String>,
    /// gRPC 層で Claims から抽出したテナント ID。アクセスログに記録する。
    pub tenant_id: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum SetSecretError {
    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Clone)]
pub struct SetSecretOutput {
    pub version: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct SetSecretUseCase {
    store: Arc<dyn SecretStore>,
    audit: Arc<dyn AccessLogRepository>,
    event_publisher: Arc<dyn VaultEventPublisher>,
}

impl SetSecretUseCase {
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

    pub async fn execute(&self, input: &SetSecretInput) -> Result<SetSecretOutput, SetSecretError> {
        let result = self.store.set(&input.path, input.data.clone()).await;

        match result {
            Ok(version) => {
                // MED-011 対応: アクセスログに tenant_id を設定する。
                let mut log = SecretAccessLog::new(input.path.clone(), AccessAction::Write, None, true);
                log.tenant_id = input.tenant_id.clone();
                let _ = self.audit.record(&log).await;
                let _ = self
                    .event_publisher
                    .publish_secret_accessed(&VaultAccessEvent {
                        key_path: input.path.clone(),
                        action: "write".to_string(),
                        actor_id: "system".to_string(),
                        success: true,
                        error_msg: None,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    })
                    .await;

                let secret = self
                    .store
                    .get(&input.path, Some(version))
                    .await
                    .map_err(|e| SetSecretError::Internal(e.to_string()))?;
                let created_at = secret
                    .get_version(Some(version))
                    .map(|sv| sv.created_at)
                    .ok_or_else(|| {
                        SetSecretError::Internal(format!(
                            "failed to resolve created_at for version {version}"
                        ))
                    })?;

                Ok(SetSecretOutput {
                    version,
                    created_at,
                })
            }
            Err(e) => {
                // MED-011 対応: エラー時のアクセスログにも tenant_id を設定する。
                let mut log =
                    SecretAccessLog::new(input.path.clone(), AccessAction::Write, None, false);
                log.tenant_id = input.tenant_id.clone();
                log.error_msg = Some(e.to_string());
                let _ = self.audit.record(&log).await;
                let _ = self
                    .event_publisher
                    .publish_secret_accessed(&VaultAccessEvent {
                        key_path: input.path.clone(),
                        action: "write".to_string(),
                        actor_id: "system".to_string(),
                        success: false,
                        error_msg: Some(e.to_string()),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    })
                    .await;

                Err(SetSecretError::Internal(e.to_string()))
            }
        }
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
    async fn test_set_secret_success() {
        let mut mock_store = MockSecretStore::new();
        let mut mock_audit = MockAccessLogRepository::new();

        mock_store
            .expect_set()
            .withf(|path, data| path == "app/db/password" && data.contains_key("password"))
            .returning(|_, _| Ok(1));
        mock_store.expect_get().returning(|path, version| {
            let mut data = HashMap::new();
            data.insert("password".to_string(), "s3cret".to_string());
            let mut secret = crate::domain::entity::secret::Secret::new(path.to_string(), data);
            if version.unwrap_or(1) > 1 {
                secret = secret.update(HashMap::from([(
                    "password".to_string(),
                    "s3cret-v2".to_string(),
                )]));
            }
            Ok(secret)
        });

        mock_audit.expect_record().returning(|_| Ok(()));

        let uc = SetSecretUseCase::new(
            Arc::new(mock_store),
            Arc::new(mock_audit),
            Arc::new(NoopVaultEventPublisher),
        );
        let input = SetSecretInput {
            path: "app/db/password".to_string(),
            data: HashMap::from([("password".to_string(), "s3cret".to_string())]),
            tenant_id: Some("test-tenant".to_string()),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().version, 1);
    }

    #[tokio::test]
    async fn test_set_secret_store_error() {
        let mut mock_store = MockSecretStore::new();
        let mut mock_audit = MockAccessLogRepository::new();

        mock_store
            .expect_set()
            .returning(|_, _| Err(anyhow::anyhow!("storage backend unavailable")));

        mock_audit.expect_record().returning(|_| Ok(()));

        let uc = SetSecretUseCase::new(
            Arc::new(mock_store),
            Arc::new(mock_audit),
            Arc::new(NoopVaultEventPublisher),
        );
        let input = SetSecretInput {
            path: "app/db/password".to_string(),
            data: HashMap::from([("password".to_string(), "s3cret".to_string())]),
            tenant_id: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            SetSecretError::Internal(msg) => assert!(msg.contains("unavailable")),
        }
    }
}
