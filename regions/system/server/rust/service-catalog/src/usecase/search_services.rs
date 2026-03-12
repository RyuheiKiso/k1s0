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
