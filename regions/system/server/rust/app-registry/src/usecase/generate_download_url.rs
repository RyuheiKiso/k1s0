use std::sync::Arc;

use crate::domain::entity::download_stat::DownloadStat;
use crate::domain::entity::platform::Platform;
use crate::domain::repository::{AppRepository, DownloadStatsRepository, VersionRepository};
use crate::infrastructure::s3_client::S3Client;
use crate::usecase::version_selection::{resolve_version, VersionSelectionError};

const PRESIGNED_URL_EXPIRES_IN_SECS: u64 = 3600;

/// GenerateDownloadUrlError はダウンロード URL 生成に関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum GenerateDownloadUrlError {
    #[error("app not found: {0}")]
    AppNotFound(String),

    #[error("version not found: app={0} version={1}")]
    NotFound(String, String),

    #[error("version selection is ambiguous: app={0} version={1}")]
    AmbiguousVersion(String, String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// DownloadUrlResult はダウンロード URL 生成結果を表す。
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct DownloadUrlResult {
    pub download_url: String,
    pub expires_in: u64,
    pub checksum_sha256: String,
    pub size_bytes: Option<i64>,
}

/// GenerateDownloadUrlUseCase はダウンロード URL 生成ユースケース。
/// バージョン情報を取得し、S3 の署名付き URL を生成する。
pub struct GenerateDownloadUrlUseCase {
    app_repo: Arc<dyn AppRepository>,
    version_repo: Arc<dyn VersionRepository>,
    download_stats_repo: Arc<dyn DownloadStatsRepository>,
    s3_client: Arc<S3Client>,
}

impl GenerateDownloadUrlUseCase {
    pub fn new(
        app_repo: Arc<dyn AppRepository>,
        version_repo: Arc<dyn VersionRepository>,
        download_stats_repo: Arc<dyn DownloadStatsRepository>,
        s3_client: Arc<S3Client>,
    ) -> Self {
        Self {
            app_repo,
            version_repo,
            download_stats_repo,
            s3_client,
        }
    }

    pub async fn execute(
        &self,
        app_id: &str,
        version: &str,
        platform: Option<&Platform>,
        arch: Option<&str>,
        user_id: &str,
    ) -> Result<DownloadUrlResult, GenerateDownloadUrlError> {
        let app = self
            .app_repo
            .find_by_id(app_id)
            .await
            .map_err(|e| GenerateDownloadUrlError::Internal(e.to_string()))?;

        if app.is_none() {
            return Err(GenerateDownloadUrlError::AppNotFound(app_id.to_string()));
        }

        // バージョンを検索
        let versions = self
            .version_repo
            .list_by_app(app_id)
            .await
            .map_err(|e| GenerateDownloadUrlError::Internal(e.to_string()))?;

        let app_version =
            resolve_version(&versions, version, platform, arch).map_err(|error| match error {
                VersionSelectionError::NotFound => {
                    GenerateDownloadUrlError::NotFound(app_id.to_string(), version.to_string())
                }
                VersionSelectionError::Ambiguous => GenerateDownloadUrlError::AmbiguousVersion(
                    app_id.to_string(),
                    version.to_string(),
                ),
            })?;

        // 署名付き URL を生成 (有効期限: 1時間)
        let download_url = self
            .s3_client
            .generate_presigned_url(&app_version.s3_key, PRESIGNED_URL_EXPIRES_IN_SECS)
            .await
            .map_err(|e| GenerateDownloadUrlError::Internal(e.to_string()))?;

        // ダウンロード統計を記録 (非同期、エラーは無視)
        let stat = DownloadStat {
            id: uuid::Uuid::new_v4(),
            app_id: app_id.to_string(),
            version: version.to_string(),
            platform: app_version.platform.to_string(),
            user_id: user_id.to_string(),
            downloaded_at: chrono::Utc::now(),
        };
        if let Err(e) = self.download_stats_repo.record(&stat).await {
            tracing::warn!(error = %e, "failed to record download stat");
        }

        Ok(DownloadUrlResult {
            download_url,
            expires_in: PRESIGNED_URL_EXPIRES_IN_SECS,
            checksum_sha256: app_version.checksum_sha256,
            size_bytes: app_version.size_bytes,
        })
    }
}
