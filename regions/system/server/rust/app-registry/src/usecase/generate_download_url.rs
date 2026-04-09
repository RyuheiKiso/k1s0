use std::sync::Arc;

use crate::domain::entity::download_stat::DownloadStat;
use crate::domain::entity::platform::Platform;
use crate::domain::repository::{AppRepository, DownloadStatsRepository, VersionRepository};
use crate::infrastructure::file_storage::FileStorage;
use crate::usecase::version_selection::{resolve_version, VersionSelectionError};

/// `GenerateDownloadUrlError` はダウンロード処理に関するエラーを表す。
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

/// `DownloadUrlResult` はファイルの直接配信に必要な情報を表す。
/// S3 presigned URL の代わりに、ファイルのバイト列とメタデータを保持する。
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct DownloadUrlResult {
    /// ファイルのバイト列。HTTP レスポンスのボディとして使用する。
    #[serde(skip)]
    pub file_bytes: Vec<u8>,
    /// ファイルのコンテンツタイプ（拡張子から推定）。
    pub content_type: String,
    /// ファイルの SHA-256 チェックサム。
    pub checksum_sha256: String,
    /// ファイルのバイトサイズ。
    pub size_bytes: Option<i64>,
    /// ファイル名（Content-Disposition ヘッダー用）。
    pub filename: String,
}

/// `GenerateDownloadUrlUseCase` はアプリバイナリのダウンロードユースケース。
/// バージョン情報を取得し、ローカルFS からファイルを読み取って直接配信する。
pub struct GenerateDownloadUrlUseCase {
    app_repo: Arc<dyn AppRepository>,
    version_repo: Arc<dyn VersionRepository>,
    download_stats_repo: Arc<dyn DownloadStatsRepository>,
    file_storage: Arc<FileStorage>,
}

impl GenerateDownloadUrlUseCase {
    pub fn new(
        app_repo: Arc<dyn AppRepository>,
        version_repo: Arc<dyn VersionRepository>,
        download_stats_repo: Arc<dyn DownloadStatsRepository>,
        file_storage: Arc<FileStorage>,
    ) -> Self {
        Self {
            app_repo,
            version_repo,
            download_stats_repo,
            file_storage,
        }
    }

    // CRIT-004 監査対応: RLS テナント分離のため tenant_id を受け取りリポジトリに渡す。
    pub async fn execute(
        &self,
        tenant_id: &str,
        app_id: &str,
        version: &str,
        platform: Option<&Platform>,
        arch: Option<&str>,
        user_id: &str,
    ) -> Result<DownloadUrlResult, GenerateDownloadUrlError> {
        let app = self
            .app_repo
            .find_by_id(tenant_id, app_id)
            .await
            .map_err(|e| GenerateDownloadUrlError::Internal(e.to_string()))?;

        if app.is_none() {
            return Err(GenerateDownloadUrlError::AppNotFound(app_id.to_string()));
        }

        // バージョンを検索してプラットフォーム・アーキテクチャで絞り込む
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

        // ローカルFS からファイルを読み取る
        let file_bytes = self
            .file_storage
            .read_file(&app_version.storage_key)
            .await
            .map_err(|e| GenerateDownloadUrlError::Internal(e.to_string()))?;

        // ファイル名をストレージキーの末尾部分から取得する
        let filename = std::path::Path::new(&app_version.storage_key)
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("download")
            .to_string();

        // 拡張子からコンテンツタイプを推定する
        let content_type = guess_content_type(&filename);

        // ダウンロード統計を記録（非同期、エラーは無視）
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
            file_bytes,
            content_type,
            checksum_sha256: app_version.checksum_sha256,
            size_bytes: app_version.size_bytes,
            filename,
        })
    }
}

/// ファイル名の拡張子からコンテンツタイプを推定する。
fn guess_content_type(filename: &str) -> String {
    let ext = std::path::Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    // "exe" アームと wildcard アームが同一の返り値のため統合する
    match ext {
        "dmg" => "application/x-apple-diskimage",
        "AppImage" | "appimage" => "application/x-executable",
        "deb" => "application/vnd.debian.binary-package",
        "rpm" => "application/x-rpm",
        "zip" => "application/zip",
        "tar" => "application/x-tar",
        "gz" | "tgz" => "application/gzip",
        _ => "application/octet-stream",
    }
    .to_string()
}
