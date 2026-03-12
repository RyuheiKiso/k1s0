use std::sync::Arc;

use serde::Serialize;
use utoipa::ToSchema;

use crate::domain::repository::{AppRepository, DownloadStatsRepository, VersionRepository};

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DownloadStatsSummary {
    pub total_downloads: i64,
    pub version_downloads: i64,
    pub latest_version: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum GetDownloadStatsError {
    #[error("app not found: {0}")]
    AppNotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct GetDownloadStatsUseCase {
    app_repo: Arc<dyn AppRepository>,
    version_repo: Arc<dyn VersionRepository>,
    download_stats_repo: Arc<dyn DownloadStatsRepository>,
}

impl GetDownloadStatsUseCase {
    pub fn new(
        app_repo: Arc<dyn AppRepository>,
        version_repo: Arc<dyn VersionRepository>,
        download_stats_repo: Arc<dyn DownloadStatsRepository>,
    ) -> Self {
        Self {
            app_repo,
            version_repo,
            download_stats_repo,
        }
    }

    pub async fn execute(
        &self,
        app_id: &str,
    ) -> Result<DownloadStatsSummary, GetDownloadStatsError> {
        let app = self
            .app_repo
            .find_by_id(app_id)
            .await
            .map_err(|e| GetDownloadStatsError::Internal(e.to_string()))?;

        if app.is_none() {
            return Err(GetDownloadStatsError::AppNotFound(app_id.to_string()));
        }

        let total_downloads = self
            .download_stats_repo
            .count_by_app(app_id)
            .await
            .map_err(|e| GetDownloadStatsError::Internal(e.to_string()))?;

        let versions = self
            .version_repo
            .list_by_app(app_id)
            .await
            .map_err(|e| GetDownloadStatsError::Internal(e.to_string()))?;

        let latest_version = versions
            .iter()
            .max_by(|left, right| left.published_at.cmp(&right.published_at))
            .map(|version| version.version.clone());

        let version_downloads = match latest_version.as_deref() {
            Some(version) => self
                .download_stats_repo
                .count_by_version(app_id, version)
                .await
                .map_err(|e| GetDownloadStatsError::Internal(e.to_string()))?,
            None => 0,
        };

        Ok(DownloadStatsSummary {
            total_downloads,
            version_downloads,
            latest_version,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::app::App;
    use crate::domain::entity::platform::Platform;
    use crate::domain::entity::version::AppVersion;
    use crate::domain::repository::app_repository::MockAppRepository;
    use crate::domain::repository::download_stats_repository::MockDownloadStatsRepository;
    use crate::domain::repository::version_repository::MockVersionRepository;

    #[tokio::test]
    async fn returns_download_counts_for_latest_version() {
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
                version: "1.1.0".to_string(),
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

        let mut stats_repo = MockDownloadStatsRepository::new();
        stats_repo.expect_count_by_app().returning(|_| Ok(42));
        stats_repo
            .expect_count_by_version()
            .withf(|app_id, version| app_id == "cli" && version == "1.1.0")
            .returning(|_, _| Ok(7));

        let uc = GetDownloadStatsUseCase::new(
            Arc::new(app_repo),
            Arc::new(version_repo),
            Arc::new(stats_repo),
        );

        let result = uc.execute("cli").await.unwrap();
        assert_eq!(result.total_downloads, 42);
        assert_eq!(result.version_downloads, 7);
        assert_eq!(result.latest_version.as_deref(), Some("1.1.0"));
    }
}
