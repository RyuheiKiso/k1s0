use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::scheduler_job::SchedulerJob;
use crate::domain::repository::SchedulerJobRepository;

/// PostgreSQL によるスケジューラジョブリポジトリの実装。
pub struct SchedulerJobPostgresRepository {
    pool: Arc<PgPool>,
}

impl SchedulerJobPostgresRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

/// DB 行を表す中間構造体。
/// ドメインエンティティの `status` (String) は DB の `enabled` (bool) にマッピングする。
#[derive(sqlx::FromRow)]
#[allow(dead_code)]
struct SchedulerJobRow {
    id: Uuid,
    name: String,
    cron_expression: String,
    job_type: String,
    payload: serde_json::Value,
    enabled: bool,
    max_retries: i32,
    last_run_at: Option<DateTime<Utc>>,
    next_run_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<SchedulerJobRow> for SchedulerJob {
    fn from(row: SchedulerJobRow) -> Self {
        let status = if row.enabled {
            "active".to_string()
        } else {
            "paused".to_string()
        };
        SchedulerJob {
            id: row.id,
            name: row.name,
            cron_expression: row.cron_expression,
            payload: row.payload,
            status,
            next_run_at: row.next_run_at,
            last_run_at: row.last_run_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[async_trait]
impl SchedulerJobRepository for SchedulerJobPostgresRepository {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<SchedulerJob>> {
        let row: Option<SchedulerJobRow> = sqlx::query_as(
            "SELECT id, name, cron_expression, job_type, payload, enabled, max_retries, \
                    last_run_at, next_run_at, created_at, updated_at \
             FROM scheduler.scheduler_jobs WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(self.pool.as_ref())
        .await?;
        Ok(row.map(Into::into))
    }

    async fn find_all(&self) -> anyhow::Result<Vec<SchedulerJob>> {
        let rows: Vec<SchedulerJobRow> = sqlx::query_as(
            "SELECT id, name, cron_expression, job_type, payload, enabled, max_retries, \
                    last_run_at, next_run_at, created_at, updated_at \
             FROM scheduler.scheduler_jobs ORDER BY created_at DESC",
        )
        .fetch_all(self.pool.as_ref())
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn create(&self, job: &SchedulerJob) -> anyhow::Result<()> {
        let enabled = job.status == "active";
        sqlx::query(
            "INSERT INTO scheduler.scheduler_jobs \
             (id, name, cron_expression, payload, enabled, last_run_at, next_run_at, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(job.id)
        .bind(&job.name)
        .bind(&job.cron_expression)
        .bind(&job.payload)
        .bind(enabled)
        .bind(job.last_run_at)
        .bind(job.next_run_at)
        .bind(job.created_at)
        .bind(job.updated_at)
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }

    async fn update(&self, job: &SchedulerJob) -> anyhow::Result<()> {
        let enabled = job.status == "active";
        sqlx::query(
            "UPDATE scheduler.scheduler_jobs \
             SET name = $2, cron_expression = $3, payload = $4, enabled = $5, \
                 last_run_at = $6, next_run_at = $7, updated_at = $8 \
             WHERE id = $1",
        )
        .bind(job.id)
        .bind(&job.name)
        .bind(&job.cron_expression)
        .bind(&job.payload)
        .bind(enabled)
        .bind(job.last_run_at)
        .bind(job.next_run_at)
        .bind(job.updated_at)
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }

    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool> {
        let result = sqlx::query("DELETE FROM scheduler.scheduler_jobs WHERE id = $1")
            .bind(id)
            .execute(self.pool.as_ref())
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn find_active_jobs(&self) -> anyhow::Result<Vec<SchedulerJob>> {
        let rows: Vec<SchedulerJobRow> = sqlx::query_as(
            "SELECT id, name, cron_expression, job_type, payload, enabled, max_retries, \
                    last_run_at, next_run_at, created_at, updated_at \
             FROM scheduler.scheduler_jobs \
             WHERE enabled = true \
             ORDER BY next_run_at ASC NULLS LAST",
        )
        .fetch_all(self.pool.as_ref())
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_row_to_entity_active() {
        let row = SchedulerJobRow {
            id: Uuid::new_v4(),
            name: "test-job".to_string(),
            cron_expression: "* * * * *".to_string(),
            job_type: "default".to_string(),
            payload: serde_json::json!({"key": "value"}),
            enabled: true,
            max_retries: 3,
            last_run_at: None,
            next_run_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let entity: SchedulerJob = row.into();
        assert_eq!(entity.status, "active");
        assert_eq!(entity.name, "test-job");
    }

    #[test]
    fn test_row_to_entity_paused() {
        let row = SchedulerJobRow {
            id: Uuid::new_v4(),
            name: "paused-job".to_string(),
            cron_expression: "0 12 * * *".to_string(),
            job_type: "default".to_string(),
            payload: serde_json::json!({}),
            enabled: false,
            max_retries: 1,
            last_run_at: Some(Utc::now()),
            next_run_at: Some(Utc::now()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let entity: SchedulerJob = row.into();
        assert_eq!(entity.status, "paused");
        assert!(entity.last_run_at.is_some());
        assert!(entity.next_run_at.is_some());
    }

    #[test]
    fn test_enabled_mapping_from_status() {
        // "active" -> enabled = true
        let job = SchedulerJob::new(
            "active-job".to_string(),
            "* * * * *".to_string(),
            serde_json::json!({}),
        );
        assert_eq!(job.status, "active");
        assert!(job.status == "active"); // would map to enabled = true

        // "paused" -> enabled = false
        let mut paused_job = job.clone();
        paused_job.status = "paused".to_string();
        assert!(paused_job.status != "active"); // would map to enabled = false
    }
}
