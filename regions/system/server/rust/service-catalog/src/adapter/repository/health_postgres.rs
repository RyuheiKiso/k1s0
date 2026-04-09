use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::health::{HealthState, HealthStatus};
use crate::domain::repository::HealthRepository;

#[derive(Debug, Clone, sqlx::FromRow)]
struct HealthRow {
    service_id: Uuid,
    status: String,
    message: Option<String>,
    response_time_ms: Option<i64>,
    checked_at: DateTime<Utc>,
}

impl From<HealthRow> for HealthStatus {
    fn from(row: HealthRow) -> Self {
        let status = match row.status.as_str() {
            "healthy" => HealthState::Healthy,
            "degraded" => HealthState::Degraded,
            "unhealthy" => HealthState::Unhealthy,
            _ => HealthState::Unknown,
        };
        HealthStatus {
            service_id: row.service_id,
            status,
            message: row.message,
            response_time_ms: row.response_time_ms,
            checked_at: row.checked_at,
        }
    }
}

/// `HealthPostgresRepository` は `PostgreSQL` ベースのヘルスステータスリポジトリ。
pub struct HealthPostgresRepository {
    pool: PgPool,
    metrics: Option<Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl HealthPostgresRepository {
    #[allow(dead_code)]
    #[must_use]
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
impl HealthRepository for HealthPostgresRepository {
    async fn get_latest(&self, service_id: Uuid) -> anyhow::Result<Option<HealthStatus>> {
        let start = std::time::Instant::now();

        let row = sqlx::query_as::<_, HealthRow>(
            "SELECT service_id, status, message, response_time_ms, checked_at \
             FROM service_catalog.health_status WHERE service_id = $1",
        )
        .bind(service_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration(
                "get_latest",
                "health_status",
                start.elapsed().as_secs_f64(),
            );
        }

        Ok(row.map(std::convert::Into::into))
    }

    async fn upsert(&self, health: &HealthStatus) -> anyhow::Result<()> {
        let start = std::time::Instant::now();

        sqlx::query(
            "INSERT INTO service_catalog.health_status \
             (service_id, status, message, response_time_ms, checked_at) \
             VALUES ($1, $2, $3, $4, $5) \
             ON CONFLICT (service_id) DO UPDATE SET \
             status = EXCLUDED.status, \
             message = EXCLUDED.message, \
             response_time_ms = EXCLUDED.response_time_ms, \
             checked_at = EXCLUDED.checked_at",
        )
        .bind(health.service_id)
        .bind(health.status.to_string())
        .bind(&health.message)
        .bind(health.response_time_ms)
        .bind(health.checked_at)
        .execute(&self.pool)
        .await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("upsert", "health_status", start.elapsed().as_secs_f64());
        }

        Ok(())
    }

    async fn list_all_latest(&self) -> anyhow::Result<Vec<HealthStatus>> {
        let start = std::time::Instant::now();

        let rows = sqlx::query_as::<_, HealthRow>(
            "SELECT service_id, status, message, response_time_ms, checked_at \
             FROM service_catalog.health_status ORDER BY checked_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration(
                "list_all_latest",
                "health_status",
                start.elapsed().as_secs_f64(),
            );
        }

        Ok(rows.into_iter().map(std::convert::Into::into).collect())
    }
}
