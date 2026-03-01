use std::sync::Arc;

use crate::domain::service::navigation_filter::FilteredNavigation;
use crate::usecase::GetNavigationUseCase;

#[derive(Debug, thiserror::Error)]
pub enum GrpcError {
    #[error("config load failed: {0}")]
    ConfigLoad(String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// ナビゲーション gRPC サービスのビジネスロジック委譲。
pub struct NavigationGrpcService {
    get_navigation_uc: Arc<GetNavigationUseCase>,
}

impl NavigationGrpcService {
    pub fn new(get_navigation_uc: Arc<GetNavigationUseCase>) -> Self {
        Self { get_navigation_uc }
    }

    pub async fn get_navigation(
        &self,
        bearer_token: &str,
    ) -> Result<FilteredNavigation, GrpcError> {
        self.get_navigation_uc
            .execute(bearer_token)
            .await
            .map_err(|e| match e {
                crate::usecase::get_navigation::NavigationError::ConfigLoad(msg) => {
                    GrpcError::ConfigLoad(msg)
                }
                crate::usecase::get_navigation::NavigationError::TokenVerification(msg) => {
                    GrpcError::Internal(msg)
                }
            })
    }
}
