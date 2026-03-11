use std::sync::Arc;

use crate::domain::entity::app::App;
use crate::domain::entity::version::AppVersion;
use crate::domain::repository::{AppRepository, VersionRepository};

#[derive(Debug, thiserror::Error)]
pub enum ListVersionsError {
    #[error("app not found: {0}")]
    AppNotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// ListVersionsUseCase はアプリバージョン一覧取得ユースケース。
pub struct ListVersionsUseCase {
    app_repo: Arc<dyn AppRepository>,
    version_repo: Arc<dyn VersionRepository>,
}

impl ListVersionsUseCase {
    pub fn new(app_repo: Arc<dyn AppRepository>, version_repo: Arc<dyn VersionRepository>) -> Self {
        Self {
            app_repo,
            version_repo,
        }
    }

    pub async fn execute(&self, app_id: &str) -> Result<Vec<AppVersion>, ListVersionsError> {
        let app: Option<App> = self
            .app_repo
            .find_by_id(app_id)
            .await
            .map_err(|e| ListVersionsError::Internal(e.to_string()))?;

        if app.is_none() {
            return Err(ListVersionsError::AppNotFound(app_id.to_string()));
        }

        self.version_repo
            .list_by_app(app_id)
            .await
            .map_err(|e| ListVersionsError::Internal(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::platform::Platform;
    use crate::domain::repository::app_repository::MockAppRepository;
    use crate::domain::repository::version_repository::MockVersionRepository;

    #[tokio::test]
    async fn test_list_versions_success() {
        let mut app_repo = MockAppRepository::new();
        app_repo.expect_find_by_id().returning(|_| {
            Ok(Some(crate::domain::entity::app::App {
                id: "cli".to_string(),
                name: "CLI".to_string(),
                description: None,
                category: "tools".to_string(),
                icon_url: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }))
        });

        let mut mock = MockVersionRepository::new();
        mock.expect_list_by_app()
            .withf(|id| id == "cli")
            .returning(|_| {
                Ok(vec![AppVersion {
                    id: uuid::Uuid::new_v4(),
                    app_id: "cli".to_string(),
                    version: "1.0.0".to_string(),
                    platform: Platform::Linux,
                    arch: "amd64".to_string(),
                    size_bytes: Some(10_000_000),
                    checksum_sha256: "abc123".to_string(),
                    s3_key: "cli/1.0.0/linux/amd64/k1s0".to_string(),
                    release_notes: None,
                    mandatory: false,
                    published_at: chrono::Utc::now(),
                    created_at: chrono::Utc::now(),
                }])
            });

        let uc = ListVersionsUseCase::new(Arc::new(app_repo), Arc::new(mock));
        let result = uc.execute("cli").await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].version, "1.0.0");
    }

    #[tokio::test]
    async fn test_list_versions_app_not_found() {
        let mut app_repo = MockAppRepository::new();
        app_repo.expect_find_by_id().returning(|_| Ok(None));

        let mock = MockVersionRepository::new();
        let uc = ListVersionsUseCase::new(Arc::new(app_repo), Arc::new(mock));

        let result = uc.execute("missing").await;
        assert!(matches!(result, Err(ListVersionsError::AppNotFound(_))));
    }
}
