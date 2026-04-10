use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::domain::entity::service::{Service, ServiceLifecycle, ServiceTier};
use crate::domain::repository::ServiceRepository;

/// `UpdateServiceError` はサービス更新に関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum UpdateServiceError {
    #[error("service not found: {0}")]
    NotFound(Uuid),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// `UpdateServiceInput` はサービス更新の入力データ。
#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateServiceInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub tier: Option<ServiceTier>,
    pub lifecycle: Option<ServiceLifecycle>,
    pub repository_url: Option<String>,
    pub api_endpoint: Option<String>,
    pub healthcheck_url: Option<String>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
}

/// `UpdateServiceUseCase` はサービス更新ユースケース。
pub struct UpdateServiceUseCase {
    service_repo: Arc<dyn ServiceRepository>,
}

impl UpdateServiceUseCase {
    pub fn new(service_repo: Arc<dyn ServiceRepository>) -> Self {
        Self { service_repo }
    }

    // CRIT-004 監査対応: RLS テナント分離のため tenant_id を受け取りリポジトリに渡す。
    pub async fn execute(
        &self,
        tenant_id: &str,
        id: Uuid,
        input: UpdateServiceInput,
    ) -> Result<Service, UpdateServiceError> {
        let mut service = match self.service_repo.find_by_id(tenant_id, id).await {
            Ok(Some(s)) => s,
            Ok(None) => return Err(UpdateServiceError::NotFound(id)),
            Err(e) => return Err(UpdateServiceError::Internal(e.to_string())),
        };

        if let Some(name) = input.name {
            if name.trim().is_empty() {
                return Err(UpdateServiceError::InvalidInput(
                    "service name must not be empty".to_string(),
                ));
            }
            service.name = name;
        }
        if let Some(desc) = input.description {
            service.description = Some(desc);
        }
        if let Some(tier) = input.tier {
            service.tier = tier;
        }
        if let Some(lifecycle) = input.lifecycle {
            service.lifecycle = lifecycle;
        }
        if let Some(url) = input.repository_url {
            service.repository_url = Some(url);
        }
        if let Some(url) = input.api_endpoint {
            service.api_endpoint = Some(url);
        }
        if let Some(url) = input.healthcheck_url {
            service.healthcheck_url = Some(url);
        }
        if let Some(tags) = input.tags {
            service.tags = tags;
        }
        if let Some(metadata) = input.metadata {
            service.metadata = metadata;
        }

        service.updated_at = Utc::now();

        self.service_repo
            .update(tenant_id, &service)
            .await
            .map_err(|e| UpdateServiceError::Internal(e.to_string()))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::service_repository::MockServiceRepository;
    use chrono::Utc;

    /// テスト用 Service ヘルパー
    fn make_service(id: Uuid) -> Service {
        Service {
            id,
            name: "original-name".to_string(),
            description: None,
            team_id: Uuid::new_v4(),
            tier: ServiceTier::Standard,
            lifecycle: ServiceLifecycle::Development,
            repository_url: None,
            api_endpoint: None,
            healthcheck_url: None,
            tags: vec![],
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// 正常に名前と説明を更新できる
    #[tokio::test]
    async fn test_update_service_success() {
        let id = Uuid::new_v4();
        let svc = make_service(id);
        let svc_clone = svc.clone();
        let mut mock = MockServiceRepository::new();
        mock.expect_find_by_id()
            .returning(move |_, _| Ok(Some(svc_clone.clone())));
        mock.expect_update().returning(|_, svc| Ok(svc.clone()));

        let uc = UpdateServiceUseCase::new(Arc::new(mock));
        let result = uc
            .execute(
                "tenant-1",
                id,
                UpdateServiceInput {
                    name: Some("updated-name".to_string()),
                    description: Some("new desc".to_string()),
                    tier: None,
                    lifecycle: None,
                    repository_url: None,
                    api_endpoint: None,
                    healthcheck_url: None,
                    tags: None,
                    metadata: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(result.name, "updated-name");
        assert_eq!(result.description.as_deref(), Some("new desc"));
    }

    /// 空の名前は InvalidInput エラーを返す
    #[tokio::test]
    async fn test_update_service_empty_name() {
        let id = Uuid::new_v4();
        let svc = make_service(id);
        let mut mock = MockServiceRepository::new();
        mock.expect_find_by_id()
            .returning(move |_, _| Ok(Some(svc.clone())));

        let uc = UpdateServiceUseCase::new(Arc::new(mock));
        let result = uc
            .execute(
                "tenant-1",
                id,
                UpdateServiceInput {
                    name: Some("  ".to_string()),
                    description: None,
                    tier: None,
                    lifecycle: None,
                    repository_url: None,
                    api_endpoint: None,
                    healthcheck_url: None,
                    tags: None,
                    metadata: None,
                },
            )
            .await;
        assert!(matches!(result, Err(UpdateServiceError::InvalidInput(_))));
    }

    /// 存在しないサービスは NotFound エラーを返す
    #[tokio::test]
    async fn test_update_service_not_found() {
        let mut mock = MockServiceRepository::new();
        mock.expect_find_by_id().returning(|_, _| Ok(None));

        let uc = UpdateServiceUseCase::new(Arc::new(mock));
        let result = uc
            .execute(
                "tenant-1",
                Uuid::new_v4(),
                UpdateServiceInput {
                    name: Some("new-name".to_string()),
                    description: None,
                    tier: None,
                    lifecycle: None,
                    repository_url: None,
                    api_endpoint: None,
                    healthcheck_url: None,
                    tags: None,
                    metadata: None,
                },
            )
            .await;
        assert!(matches!(result, Err(UpdateServiceError::NotFound(_))));
    }
}
