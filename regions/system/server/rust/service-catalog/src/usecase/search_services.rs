use std::sync::Arc;

use crate::domain::entity::service::{Service, ServiceTier};
use crate::domain::repository::ServiceRepository;

/// SearchServicesError は検索に関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum SearchServicesError {
    #[error("internal error: {0}")]
    Internal(String),
}

/// SearchServicesUseCase はサービス検索ユースケース。
pub struct SearchServicesUseCase {
    service_repo: Arc<dyn ServiceRepository>,
}

impl SearchServicesUseCase {
    pub fn new(service_repo: Arc<dyn ServiceRepository>) -> Self {
        Self { service_repo }
    }

    pub async fn execute(
        &self,
        query: Option<String>,
        tags: Option<Vec<String>>,
        tier: Option<ServiceTier>,
    ) -> Result<Vec<Service>, SearchServicesError> {
        self.service_repo
            .search(query, tags, tier)
            .await
            .map_err(|e| SearchServicesError::Internal(e.to_string()))
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

    fn sample_service(name: &str) -> Service {
        Service {
            id: Uuid::new_v4(),
            name: name.to_string(),
            description: None,
            team_id: Uuid::new_v4(),
            tier: ServiceTier::Standard,
            lifecycle: ServiceLifecycle::Production,
            repository_url: None,
            api_endpoint: None,
            healthcheck_url: None,
            tags: vec!["api".to_string()],
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// クエリで検索すると結果が返る
    #[tokio::test]
    async fn search_with_query() {
        let mut mock = MockServiceRepository::new();
        mock.expect_search()
            .returning(|_, _, _| Ok(vec![sample_service("auth-service")]));

        let uc = SearchServicesUseCase::new(Arc::new(mock));
        let result = uc
            .execute(Some("auth".to_string()), None, None)
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "auth-service");
    }

    /// リポジトリエラー時は Internal エラーを返す
    #[tokio::test]
    async fn search_internal_error() {
        let mut mock = MockServiceRepository::new();
        mock.expect_search()
            .returning(|_, _, _| Err(anyhow::anyhow!("db error")));

        let uc = SearchServicesUseCase::new(Arc::new(mock));
        let result = uc.execute(None, None, None).await;
        assert!(matches!(result, Err(SearchServicesError::Internal(_))));
    }
}
