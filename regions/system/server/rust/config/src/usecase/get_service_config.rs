use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::config_entry::ServiceConfigResult;
use crate::domain::error::ConfigRepositoryError;
use crate::domain::repository::ConfigRepository;

/// GetServiceConfigError はサービス設定取得に関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum GetServiceConfigError {
    #[error("service not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// GetServiceConfigUseCase はサービス向け設定一括取得ユースケース。
pub struct GetServiceConfigUseCase {
    config_repo: Arc<dyn ConfigRepository>,
}

impl GetServiceConfigUseCase {
    pub fn new(config_repo: Arc<dyn ConfigRepository>) -> Self {
        Self { config_repo }
    }

    /// STATIC-CRITICAL-001 監査対応: テナントスコープでサービス名の設定値を一括取得する。
    /// 型安全なパターンマッチングでリポジトリエラーを分類する。
    pub async fn execute(
        &self,
        tenant_id: Uuid,
        service_name: &str,
    ) -> Result<ServiceConfigResult, GetServiceConfigError> {
        let result = self
            .config_repo
            .find_by_service_name(tenant_id, service_name)
            .await
            .map_err(|e| match e {
                ConfigRepositoryError::ServiceNotFound(_) => {
                    GetServiceConfigError::NotFound(service_name.to_string())
                }
                other => GetServiceConfigError::Internal(other.to_string()),
            })?;

        if result.entries.is_empty() {
            return Err(GetServiceConfigError::NotFound(service_name.to_string()));
        }

        Ok(result)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::config_entry::{ServiceConfigEntry, ServiceConfigResult};
    use crate::domain::repository::config_repository::MockConfigRepository;

    /// システムテナントUUID: テスト共通
    fn system_tenant() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
    }

    #[tokio::test]
    async fn test_get_service_config_success() {
        let mut mock = MockConfigRepository::new();
        mock.expect_find_by_service_name()
            .withf(|_tid, name| name == "auth-server")
            .returning(|_, _| {
                Ok(ServiceConfigResult {
                    service_name: "auth-server".to_string(),
                    entries: vec![
                        ServiceConfigEntry {
                            namespace: "system.auth.database".to_string(),
                            key: "max_connections".to_string(),
                            value: serde_json::json!(25),
                            version: 3,
                        },
                        ServiceConfigEntry {
                            namespace: "system.auth.database".to_string(),
                            key: "ssl_mode".to_string(),
                            value: serde_json::json!("require"),
                            version: 1,
                        },
                        ServiceConfigEntry {
                            namespace: "system.auth.jwt".to_string(),
                            key: "issuer".to_string(),
                            value: serde_json::json!(
                                "https://auth.k1s0.internal.example.com/realms/k1s0"
                            ),
                            version: 2,
                        },
                    ],
                })
            });

        let uc = GetServiceConfigUseCase::new(Arc::new(mock));
        let result = uc.execute(system_tenant(), "auth-server").await;
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.service_name, "auth-server");
        assert_eq!(config.entries.len(), 3);
    }

    /// サービスが見つからない場合は ServiceNotFound エラーを返す（型安全なエラー使用）
    #[tokio::test]
    async fn test_get_service_config_not_found() {
        let mut mock = MockConfigRepository::new();
        mock.expect_find_by_service_name().returning(|_, _| {
            Err(ConfigRepositoryError::ServiceNotFound(
                "nonexistent-service".to_string(),
            ))
        });

        let uc = GetServiceConfigUseCase::new(Arc::new(mock));
        let result = uc.execute(system_tenant(), "nonexistent-service").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            GetServiceConfigError::NotFound(name) => assert_eq!(name, "nonexistent-service"),
            e => unreachable!("unexpected error in test: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_service_config_empty_entries() {
        let mut mock = MockConfigRepository::new();
        mock.expect_find_by_service_name().returning(|_, name| {
            Ok(ServiceConfigResult {
                service_name: name.to_string(),
                entries: vec![],
            })
        });

        let uc = GetServiceConfigUseCase::new(Arc::new(mock));
        let result = uc.execute(system_tenant(), "empty-service").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            GetServiceConfigError::NotFound(name) => assert_eq!(name, "empty-service"),
            e => unreachable!("unexpected error in test: {:?}", e),
        }
    }

    /// インフラストラクチャエラーの場合は Internal エラーを返す（型安全なエラー使用）
    #[tokio::test]
    async fn test_get_service_config_internal_error() {
        let mut mock = MockConfigRepository::new();
        mock.expect_find_by_service_name().returning(|_, _| {
            Err(ConfigRepositoryError::Infrastructure(anyhow::anyhow!(
                "connection refused"
            )))
        });

        let uc = GetServiceConfigUseCase::new(Arc::new(mock));
        let result = uc.execute(system_tenant(), "auth-server").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            GetServiceConfigError::Internal(msg) => assert!(msg.contains("connection refused")),
            e => unreachable!("unexpected error in test: {:?}", e),
        }
    }
}
