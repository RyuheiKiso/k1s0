use std::sync::Arc;

use crate::domain::entity::platform::Platform;
use crate::domain::entity::version::AppVersion;
use crate::domain::repository::{AppRepository, VersionRepository};
use crate::usecase::version_selection::select_latest;

/// GetLatestError は最新バージョン取得に関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum GetLatestError {
    #[error("app not found: {0}")]
    AppNotFound(String),

    #[error("no version found for app={0}")]
    VersionNotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// GetLatestUseCase は最新バージョン取得ユースケース。
pub struct GetLatestUseCase {
    app_repo: Arc<dyn AppRepository>,
    version_repo: Arc<dyn VersionRepository>,
}

impl GetLatestUseCase {
    pub fn new(app_repo: Arc<dyn AppRepository>, version_repo: Arc<dyn VersionRepository>) -> Self {
        Self {
            app_repo,
            version_repo,
        }
    }

    pub async fn execute(
        &self,
        app_id: &str,
        platform: Option<&Platform>,
        arch: Option<&str>,
    ) -> Result<AppVersion, GetLatestError> {
        let app = self
            .app_repo
            .find_by_id(app_id)
            .await
            .map_err(|e| GetLatestError::Internal(e.to_string()))?;

        if app.is_none() {
            return Err(GetLatestError::AppNotFound(app_id.to_string()));
        }

        let versions = self
            .version_repo
            .list_by_app(app_id)
            .await
            .map_err(|e| GetLatestError::Internal(e.to_string()))?;

        select_latest(&versions, platform, arch)
            .ok_or_else(|| GetLatestError::VersionNotFound(app_id.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::app::App;
    use crate::domain::repository::app_repository::MockAppRepository;
    use crate::domain::repository::version_repository::MockVersionRepository;

    #[tokio::test]
    async fn test_get_latest_success() {
        let mut app_repo = MockAppRepository::new();
        app_repo.expect_find_by_id().returning(|_| {
            Ok(Some(App {
                id: "cli".to_string(),
                name: "CLI".to_string(),
                description: None,
                category: "tools".to_string(),
                icon_url: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }))
        });

        let mut version_repo = MockVersionRepository::new();
        version_repo.expect_list_by_app().returning(|_| {
            Ok(vec![AppVersion {
                id: uuid::Uuid::new_v4(),
                app_id: "cli".to_string(),
                version: "2.0.0".to_string(),
                platform: Platform::Linux,
                arch: "amd64".to_string(),
                size_bytes: Some(15_000_000),
                checksum_sha256: "latest_hash".to_string(),
                s3_key: "cli/2.0.0/linux/amd64/k1s0".to_string(),
                release_notes: None,
                mandatory: false,
                published_at: chrono::Utc::now(),
                created_at: chrono::Utc::now(),
            }])
        });

        let uc = GetLatestUseCase::new(Arc::new(app_repo), Arc::new(version_repo));
        let result = uc.execute("cli", Some(&Platform::Linux), Some("amd64")).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().version, "2.0.0");
    }

    #[tokio::test]
    async fn test_get_latest_not_found() {
        let mut app_repo = MockAppRepository::new();
        app_repo.expect_find_by_id().returning(|_| {
            Ok(Some(App {
                id: "cli".to_string(),
                name: "CLI".to_string(),
                description: None,
                category: "tools".to_string(),
                icon_url: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }))
        });

        let mut version_repo = MockVersionRepository::new();
        version_repo.expect_list_by_app().returning(|_| Ok(vec![]));

        let uc = GetLatestUseCase::new(Arc::new(app_repo), Arc::new(version_repo));
        let result = uc.execute("cli", Some(&Platform::Macos), Some("arm64")).await;
        assert!(matches!(result, Err(GetLatestError::VersionNotFound(_))));
    }
}
