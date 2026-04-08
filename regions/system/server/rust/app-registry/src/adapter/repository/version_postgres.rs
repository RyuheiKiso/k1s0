use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::platform::Platform;
use crate::domain::entity::version::AppVersion;
use crate::domain::repository::VersionRepository;

/// `VersionRow` は `app_registry.app_versions` テーブルの行を表す中間構造体。
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct VersionRow {
    pub id: Uuid,
    pub app_id: String,
    pub version: String,
    pub platform: String,
    pub arch: String,
    pub size_bytes: Option<i64>,
    pub checksum_sha256: String,
    pub storage_key: String,
    pub release_notes: Option<String>,
    pub mandatory: bool,
    /// STATIC-CRITICAL-002: Cosign 署名（base64）。NULL は未検証または開発環境。
    pub cosign_signature: Option<String>,
    pub published_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<VersionRow> for AppVersion {
    type Error = anyhow::Error;

    fn try_from(row: VersionRow) -> anyhow::Result<Self> {
        let platform: Platform = row.platform.parse()?;
        Ok(AppVersion {
            id: row.id,
            app_id: row.app_id,
            version: row.version,
            platform,
            arch: row.arch,
            size_bytes: row.size_bytes,
            checksum_sha256: row.checksum_sha256,
            storage_key: row.storage_key,
            release_notes: row.release_notes,
            mandatory: row.mandatory,
            cosign_signature: row.cosign_signature,
            published_at: row.published_at,
            created_at: row.created_at,
        })
    }
}

/// `VersionPostgresRepository` は `PostgreSQL` ベースのバージョンリポジトリ。
pub struct VersionPostgresRepository {
    pool: PgPool,
    metrics: Option<Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl VersionPostgresRepository {
    // テスト環境でのみ使用するコンストラクタ。本番では with_metrics を使用する（M-01対応）
    #[cfg(test)]
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            metrics: None,
        }
    }

    #[must_use] 
    pub fn with_metrics(pool: PgPool, metrics: Arc<k1s0_telemetry::metrics::Metrics>) -> Self {
        Self {
            pool,
            metrics: Some(metrics),
        }
    }
}

#[async_trait]
impl VersionRepository for VersionPostgresRepository {
    async fn list_by_app(&self, app_id: &str) -> anyhow::Result<Vec<AppVersion>> {
        let start = std::time::Instant::now();
        let rows = sqlx::query_as::<_, VersionRow>(
            r"
            SELECT id, app_id, version, platform, arch, size_bytes, checksum_sha256,
                   storage_key, release_notes, mandatory, cosign_signature, published_at, created_at
            FROM app_registry.app_versions
            WHERE app_id = $1
            ORDER BY published_at DESC
            ",
        )
        .bind(app_id)
        .fetch_all(&self.pool)
        .await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration(
                "list_by_app",
                "app_versions",
                start.elapsed().as_secs_f64(),
            );
        }

        rows.into_iter().map(std::convert::TryInto::try_into).collect()
    }

    async fn create(&self, version: &AppVersion) -> anyhow::Result<AppVersion> {
        let start = std::time::Instant::now();
        // STATIC-CRITICAL-002: cosign_signature を INSERT に含め、署名を永続化する
        let row = sqlx::query_as::<_, VersionRow>(
            r"
            INSERT INTO app_registry.app_versions
                (app_id, version, platform, arch, size_bytes, checksum_sha256,
                 storage_key, release_notes, mandatory, cosign_signature, published_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, app_id, version, platform, arch, size_bytes, checksum_sha256,
                      storage_key, release_notes, mandatory, cosign_signature, published_at, created_at
            ",
        )
        .bind(&version.app_id)
        .bind(&version.version)
        .bind(version.platform.to_string())
        .bind(&version.arch)
        .bind(version.size_bytes)
        .bind(&version.checksum_sha256)
        .bind(&version.storage_key)
        .bind(&version.release_notes)
        .bind(version.mandatory)
        .bind(&version.cosign_signature)
        .bind(version.published_at)
        .fetch_one(&self.pool)
        .await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("create", "app_versions", start.elapsed().as_secs_f64());
        }

        row.try_into()
    }

    async fn delete(
        &self,
        app_id: &str,
        version: &str,
        platform: &Platform,
        arch: &str,
    ) -> anyhow::Result<()> {
        let start = std::time::Instant::now();
        let result = sqlx::query(
            r"
            DELETE FROM app_registry.app_versions
            WHERE app_id = $1 AND version = $2 AND platform = $3 AND arch = $4
            ",
        )
        .bind(app_id)
        .bind(version)
        .bind(platform.to_string())
        .bind(arch)
        .execute(&self.pool)
        .await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("delete", "app_versions", start.elapsed().as_secs_f64());
        }

        if result.rows_affected() == 0 {
            anyhow::bail!(
                "version not found: app={app_id} version={version} platform={platform} arch={arch}"
            );
        }

        Ok(())
    }
}
