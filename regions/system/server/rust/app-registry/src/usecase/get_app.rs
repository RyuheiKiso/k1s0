use std::sync::Arc;

use crate::domain::entity::app::App;
use crate::domain::repository::AppRepository;

/// `GetAppError` はアプリ取得に関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum GetAppError {
    #[error("app not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// `GetAppUseCase` はアプリ情報取得ユースケース。
pub struct GetAppUseCase {
    app_repo: Arc<dyn AppRepository>,
}

impl GetAppUseCase {
    pub fn new(app_repo: Arc<dyn AppRepository>) -> Self {
        Self { app_repo }
    }

    // CRIT-004 監査対応: RLS テナント分離のため tenant_id を受け取りリポジトリに渡す。
    pub async fn execute(&self, tenant_id: &str, id: &str) -> Result<App, GetAppError> {
        match self.app_repo.find_by_id(tenant_id, id).await {
            Ok(Some(app)) => Ok(app),
            Ok(None) => Err(GetAppError::NotFound(id.to_string())),
            Err(e) => Err(GetAppError::Internal(e.to_string())),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::app_repository::MockAppRepository;

    #[tokio::test]
    async fn test_get_app_success() {
        let mut mock = MockAppRepository::new();
        mock.expect_find_by_id()
            .withf(|_tenant_id, id| id == "cli")
            .returning(|_, _| {
                Ok(Some(App {
                    id: "cli".to_string(),
                    name: "k1s0 CLI".to_string(),
                    description: None,
                    category: "tools".to_string(),
                    icon_url: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }))
            });

        let uc = GetAppUseCase::new(Arc::new(mock));
        let result = uc.execute("tenant-1", "cli").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, "cli");
    }

    #[tokio::test]
    async fn test_get_app_not_found() {
        let mut mock = MockAppRepository::new();
        mock.expect_find_by_id().returning(|_, _| Ok(None));

        let uc = GetAppUseCase::new(Arc::new(mock));
        let result = uc.execute("tenant-1", "nonexistent").await;
        assert!(matches!(result, Err(GetAppError::NotFound(_))));
    }
}
