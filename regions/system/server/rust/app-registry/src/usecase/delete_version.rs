use std::sync::Arc;

use crate::domain::entity::platform::Platform;
use crate::domain::repository::{AppRepository, VersionRepository};
use crate::usecase::version_selection::{resolve_version, VersionSelectionError};

#[derive(Debug, thiserror::Error)]
pub enum DeleteVersionError {
    #[error("app not found: {0}")]
    AppNotFound(String),

    #[error("version not found: app={0} version={1}")]
    VersionNotFound(String, String),

    #[error("version selection is ambiguous: app={0} version={1}")]
    AmbiguousVersion(String, String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// DeleteVersionUseCase はアプリバージョン削除ユースケース。
pub struct DeleteVersionUseCase {
    app_repo: Arc<dyn AppRepository>,
    version_repo: Arc<dyn VersionRepository>,
}

impl DeleteVersionUseCase {
    pub fn new(app_repo: Arc<dyn AppRepository>, version_repo: Arc<dyn VersionRepository>) -> Self {
        Self {
            app_repo,
            version_repo,
        }
    }

    pub async fn execute(
        &self,
        app_id: &str,
        version: &str,
        platform: Option<&Platform>,
        arch: Option<&str>,
    ) -> Result<(), DeleteVersionError> {
        let app = self
            .app_repo
            .find_by_id(app_id)
            .await
            .map_err(|e| DeleteVersionError::Internal(e.to_string()))?;

        if app.is_none() {
            return Err(DeleteVersionError::AppNotFound(app_id.to_string()));
        }

        let versions = self
            .version_repo
            .list_by_app(app_id)
            .await
            .map_err(|e| DeleteVersionError::Internal(e.to_string()))?;

        let selected = resolve_version(&versions, version, platform, arch).map_err(|error| match error {
            VersionSelectionError::NotFound => {
                DeleteVersionError::VersionNotFound(app_id.to_string(), version.to_string())
            }
            VersionSelectionError::Ambiguous => {
                DeleteVersionError::AmbiguousVersion(app_id.to_string(), version.to_string())
            }
        })?;

        self.version_repo
            .delete(app_id, version, &selected.platform, &selected.arch)
            .await
            .map_err(|e| DeleteVersionError::Internal(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::app::App;
    use crate::domain::entity::version::AppVersion;
    use crate::domain::repository::app_repository::MockAppRepository;
    use crate::domain::repository::version_repository::MockVersionRepository;

    #[tokio::test]
    async fn test_delete_version_success() {
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

        let mut mock = MockVersionRepository::new();
        mock.expect_list_by_app().returning(|_| {
            Ok(vec![AppVersion {
                id: uuid::Uuid::new_v4(),
                app_id: "cli".to_string(),
                version: "1.0.0".to_string(),
                platform: Platform::Linux,
                arch: "amd64".to_string(),
                size_bytes: Some(1),
                checksum_sha256: "checksum".to_string(),
                s3_key: "key".to_string(),
                release_notes: None,
                mandatory: false,
                published_at: chrono::Utc::now(),
                created_at: chrono::Utc::now(),
            }])
        });
        mock.expect_delete().returning(|_, _, _, _| Ok(()));

        let uc = DeleteVersionUseCase::new(Arc::new(app_repo), Arc::new(mock));
        let result = uc
            .execute("cli", "1.0.0", Some(&Platform::Linux), Some("amd64"))
            .await;
        assert!(result.is_ok());
    }
}
