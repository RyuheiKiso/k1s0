use std::sync::Arc;

use crate::domain::entity::service::Service;
use crate::domain::repository::service_repository::ServiceListFilters;
use crate::domain::repository::ServiceRepository;

/// `ListServicesUseCase` はサービス一覧取得ユースケース。
pub struct ListServicesUseCase {
    service_repo: Arc<dyn ServiceRepository>,
}

impl ListServicesUseCase {
    pub fn new(service_repo: Arc<dyn ServiceRepository>) -> Self {
        Self { service_repo }
    }

    // CRIT-004 監査対応: RLS テナント分離のため tenant_id を受け取りリポジトリに渡す。
    pub async fn execute(
        &self,
        tenant_id: &str,
        filters: ServiceListFilters,
    ) -> anyhow::Result<Vec<Service>> {
        self.service_repo.list(tenant_id, filters).await
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::service::{ServiceLifecycle, ServiceTier};
    use crate::domain::repository::service_repository::MockServiceRepository;
    use chrono::Utc;
    use uuid::Uuid;

    /// テスト用 Service ヘルパー
    fn make_service() -> Service {
        Service {
            id: Uuid::new_v4(),
            name: "svc".to_string(),
            description: None,
            team_id: Uuid::new_v4(),
            tier: ServiceTier::Internal,
            lifecycle: ServiceLifecycle::Production,
            repository_url: None,
            api_endpoint: None,
            healthcheck_url: None,
            tags: vec![],
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// 空リストを返す
    #[tokio::test]
    async fn test_list_services_empty() {
        let mut mock = MockServiceRepository::new();
        mock.expect_list().returning(|_, _| Ok(vec![]));

        let uc = ListServicesUseCase::new(Arc::new(mock));
        let result = uc
            .execute("tenant-1", ServiceListFilters::default())
            .await
            .unwrap();
        assert!(result.is_empty());
    }

    /// 複数のサービスを返す
    #[tokio::test]
    async fn test_list_services_returns_all() {
        let services = vec![make_service(), make_service(), make_service()];
        let services_clone = services.clone();
        let mut mock = MockServiceRepository::new();
        mock.expect_list()
            .returning(move |_, _| Ok(services_clone.clone()));

        let uc = ListServicesUseCase::new(Arc::new(mock));
        let result = uc
            .execute("tenant-1", ServiceListFilters::default())
            .await
            .unwrap();
        assert_eq!(result.len(), 3);
    }
}
