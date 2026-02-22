use std::sync::Arc;

use crate::domain::entity::config_entry::ServiceConfigResult;
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

    /// サービス名で設定値を一括取得する。
    pub async fn execute(
        &self,
        service_name: &str,
    ) -> Result<ServiceConfigResult, GetServiceConfigError> {
        let result = self
            .config_repo
            .find_by_service_name(service_name)
            .await
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("not found") {
                    GetServiceConfigError::NotFound(service_name.to_string())
                } else {
                    GetServiceConfigError::Internal(msg)
                }
            })?;

        if result.entries.is_empty() {
            return Err(GetServiceConfigError::NotFound(service_name.to_string()));
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::config_entry::{ServiceConfigEntry, ServiceConfigResult};
    use crate::domain::repository::config_repository::MockConfigRepository;

    #[tokio::test]
    async fn test_get_service_config_success() {
        let mut mock = MockConfigRepository::new();
        mock.expect_find_by_service_name()
            .withf(|name| name == "auth-server")
            .returning(|_| {
                Ok(ServiceConfigResult {
                    service_name: "auth-server".to_string(),
                    entries: vec![
                        ServiceConfigEntry {
                            namespace: "system.auth.database".to_string(),
                            key: "max_connections".to_string(),
                            value: serde_json::json!(25),
                        },
                        ServiceConfigEntry {
                            namespace: "system.auth.database".to_string(),
                            key: "ssl_mode".to_string(),
                            value: serde_json::json!("require"),
                        },
                        ServiceConfigEntry {
                            namespace: "system.auth.jwt".to_string(),
                            key: "issuer".to_string(),
                            value: serde_json::json!(
                                "https://auth.k1s0.internal.example.com/realms/k1s0"
                            ),
                        },
                    ],
                })
            });

        let uc = GetServiceConfigUseCase::new(Arc::new(mock));
        let result = uc.execute("auth-server").await;
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.service_name, "auth-server");
        assert_eq!(config.entries.len(), 3);
    }

    #[tokio::test]
    async fn test_get_service_config_not_found() {
        let mut mock = MockConfigRepository::new();
        mock.expect_find_by_service_name()
            .returning(|_| Err(anyhow::anyhow!("service not found")));

        let uc = GetServiceConfigUseCase::new(Arc::new(mock));
        let result = uc.execute("nonexistent-service").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            GetServiceConfigError::NotFound(name) => assert_eq!(name, "nonexistent-service"),
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_service_config_empty_entries() {
        let mut mock = MockConfigRepository::new();
        mock.expect_find_by_service_name().returning(|name| {
            Ok(ServiceConfigResult {
                service_name: name.to_string(),
                entries: vec![],
            })
        });

        let uc = GetServiceConfigUseCase::new(Arc::new(mock));
        let result = uc.execute("empty-service").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            GetServiceConfigError::NotFound(name) => assert_eq!(name, "empty-service"),
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_service_config_internal_error() {
        let mut mock = MockConfigRepository::new();
        mock.expect_find_by_service_name()
            .returning(|_| Err(anyhow::anyhow!("connection refused")));

        let uc = GetServiceConfigUseCase::new(Arc::new(mock));
        let result = uc.execute("auth-server").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            GetServiceConfigError::Internal(msg) => assert!(msg.contains("connection refused")),
            e => panic!("unexpected error: {:?}", e),
        }
    }
}
