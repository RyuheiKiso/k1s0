use std::sync::Arc;

use crate::domain::entity::download_stat::DownloadStat;
use crate::domain::entity::platform::Platform;
use crate::domain::repository::{DownloadStatsRepository, VersionRepository};
use crate::infrastructure::s3_client::S3Client;

/// GenerateDownloadUrlError はダウンロード URL 生成に関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum GenerateDownloadUrlError {
    #[error("version not found: app={0} version={1} platform={2} arch={3}")]
    NotFound(String, String, String, String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// DownloadUrlResult はダウンロード URL 生成結果を表す。
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct DownloadUrlResult {
    pub download_url: String,
    pub checksum_sha256: String,
    pub size_bytes: Option<i64>,
}

/// GenerateDownloadUrlUseCase はダウンロード URL 生成ユースケース。
/// バージョン情報を取得し、S3 の署名付き URL を生成する。
pub struct GenerateDownloadUrlUseCase {
    version_repo: Arc<dyn VersionRepository>,
    download_stats_repo: Arc<dyn DownloadStatsRepository>,
    s3_client: Arc<S3Client>,
}

impl GenerateDownloadUrlUseCase {
    pub fn new(
        version_repo: Arc<dyn VersionRepository>,
        download_stats_repo: Arc<dyn DownloadStatsRepository>,
        s3_client: Arc<S3Client>,
    ) -> Self {
        Self {
            version_repo,
            download_stats_repo,
            s3_client,
        }
    }

    pub async fn execute(
        &self,
        app_id: &str,
        version: &str,
        platform: &Platform,
        arch: &str,
        user_id: &str,
    ) -> Result<DownloadUrlResult, GenerateDownloadUrlError> {
        // バージョンを検索
        let versions = self
            .version_repo
            .list_by_app(app_id)
            .await
            .map_err(|e| GenerateDownloadUrlError::Internal(e.to_string()))?;

        let app_version = versions
            .into_iter()
            .find(|v| v.version == version && v.platform == *platform && v.arch == arch)
            .ok_or_else(|| {
                GenerateDownloadUrlError::NotFound(
                    app_id.to_string(),
                    version.to_string(),
                    platform.to_string(),
                    arch.to_string(),
                )
            })?;

        // 署名付き URL を生成 (有効期限: 10分)
        let download_url = self
            .s3_client
            .generate_presigned_url(&app_version.s3_key, 600)
            .await
            .map_err(|e| GenerateDownloadUrlError::Internal(e.to_string()))?;

        // ダウンロード統計を記録 (非同期、エラーは無視)
        let stat = DownloadStat {
            id: uuid::Uuid::new_v4(),
            app_id: app_id.to_string(),
            version: version.to_string(),
            platform: platform.to_string(),
            user_id: user_id.to_string(),
            downloaded_at: chrono::Utc::now(),
        };
        if let Err(e) = self.download_stats_repo.record(&stat).await {
            tracing::warn!(error = %e, "failed to record download stat");
        }

        Ok(DownloadUrlResult {
            download_url,
            checksum_sha256: app_version.checksum_sha256,
            size_bytes: app_version.size_bytes,
        })
    }
}
