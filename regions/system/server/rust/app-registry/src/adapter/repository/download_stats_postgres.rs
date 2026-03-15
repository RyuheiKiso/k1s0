use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::entity::download_stat::DownloadStat;
use crate::domain::repository::DownloadStatsRepository;

/// DownloadStatsPostgresRepository は PostgreSQL ベースのダウンロード統計リポジトリ。
pub struct DownloadStatsPostgresRepository {
    pool: PgPool,
    metrics: Option<Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl DownloadStatsPostgresRepository {
    #[allow(dead_code)]
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            metrics: None,
        }
    }

    pub fn with_metrics(pool: PgPool, metrics: Arc<k1s0_telemetry::metrics::Metrics>) -> Self {
        Self {
            pool,
            metrics: Some(metrics),
        }
    }
}

#[async_trait]
impl DownloadStatsRepository for DownloadStatsPostgresRepository {
    async fn record(&self, stat: &DownloadStat) -> anyhow::Result<()> {
        let start = std::time::Instant::now();
        sqlx::query(
            r#"
            INSERT INTO app_registry.download_stats
                (app_id, version, platform, user_id, downloaded_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(&stat.app_id)
        .bind(&stat.version)
        .bind(&stat.platform)
        .bind(&stat.user_id)
        .bind(stat.downloaded_at)
        .execute(&self.pool)
        .await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("record", "download_stats", start.elapsed().as_secs_f64());
        }

        Ok(())
    }

    async fn count_by_app(&self, app_id: &str) -> anyhow::Result<i64> {
        let start = std::time::Instant::now();
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM app_registry.download_stats
            WHERE app_id = $1
            "#,
        )
        .bind(app_id)
        .fetch_one(&self.pool)
        .await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration(
                "count_by_app",
                "download_stats",
                start.elapsed().as_secs_f64(),
            );
        }

        Ok(count)
    }

    async fn count_by_version(&self, app_id: &str, version: &str) -> anyhow::Result<i64> {
        let start = std::time::Instant::now();
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM app_registry.download_stats
            WHERE app_id = $1 AND version = $2
            "#,
        )
        .bind(app_id)
        .bind(version)
        .fetch_one(&self.pool)
        .await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration(
                "count_by_version",
                "download_stats",
                start.elapsed().as_secs_f64(),
            );
        }

        Ok(count)
    }
}
