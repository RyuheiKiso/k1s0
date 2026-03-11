use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::team::Team;
use crate::domain::repository::TeamRepository;

#[derive(Debug, Clone, sqlx::FromRow)]
struct TeamRow {
    id: Uuid,
    name: String,
    description: Option<String>,
    contact_email: Option<String>,
    slack_channel: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<TeamRow> for Team {
    fn from(row: TeamRow) -> Self {
        Team {
            id: row.id,
            name: row.name,
            description: row.description,
            contact_email: row.contact_email,
            slack_channel: row.slack_channel,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// TeamPostgresRepository は PostgreSQL ベースのチームリポジトリ。
pub struct TeamPostgresRepository {
    pool: PgPool,
    metrics: Option<Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl TeamPostgresRepository {
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
impl TeamRepository for TeamPostgresRepository {
    async fn list(&self) -> anyhow::Result<Vec<Team>> {
        let start = std::time::Instant::now();

        let rows = sqlx::query_as::<_, TeamRow>(
            "SELECT id, name, description, contact_email, slack_channel, created_at, updated_at \
             FROM service_catalog.teams ORDER BY name ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("list", "teams", start.elapsed().as_secs_f64());
        }

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<Team>> {
        let start = std::time::Instant::now();

        let row = sqlx::query_as::<_, TeamRow>(
            "SELECT id, name, description, contact_email, slack_channel, created_at, updated_at \
             FROM service_catalog.teams WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("find_by_id", "teams", start.elapsed().as_secs_f64());
        }

        Ok(row.map(|r| r.into()))
    }
}
