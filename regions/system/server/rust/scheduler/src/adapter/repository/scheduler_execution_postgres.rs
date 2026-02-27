use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::scheduler_execution::SchedulerExecution;
use crate::domain::repository::SchedulerExecutionRepository;

/// PostgreSQL によるスケジューラ実行履歴リポジトリの実装。
pub struct SchedulerExecutionPostgresRepository {
    pool: Arc<PgPool>,
}

impl SchedulerExecutionPostgresRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

/// DB 行を表す中間構造体。
#[derive(sqlx::FromRow)]
#[allow(dead_code)]
struct SchedulerExecutionRow {
    id: Uuid,
    job_id: Uuid,
    status: String,
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
    error_message: Option<String>,
}

impl From<SchedulerExecutionRow> for SchedulerExecution {
    fn from(row: SchedulerExecutionRow) -> Self {
        SchedulerExecution {
            id: row.id,
            job_id: row.job_id,
            status: row.status,
            started_at: row.started_at,
            completed_at: row.completed_at,
            error_message: row.error_message,
        }
    }
}

#[async_trait]
impl SchedulerExecutionRepository for SchedulerExecutionPostgresRepository {
    async fn create(&self, execution: &SchedulerExecution) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO scheduler.job_executions \
             (id, job_id, status, started_at, completed_at, error_message) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(execution.id)
        .bind(execution.job_id)
        .bind(&execution.status)
        .bind(execution.started_at)
        .bind(execution.completed_at)
        .bind(&execution.error_message)
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }

    async fn find_by_job_id(&self, job_id: &Uuid) -> anyhow::Result<Vec<SchedulerExecution>> {
        let rows: Vec<SchedulerExecutionRow> = sqlx::query_as(
            "SELECT id, job_id, status, started_at, completed_at, error_message \
             FROM scheduler.job_executions \
             WHERE job_id = $1 \
             ORDER BY started_at DESC",
        )
        .bind(job_id)
        .fetch_all(self.pool.as_ref())
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn update_status(
        &self,
        id: &Uuid,
        status: String,
        error_message: Option<String>,
    ) -> anyhow::Result<()> {
        sqlx::query(
            "UPDATE scheduler.job_executions \
             SET status = $2, completed_at = $3, error_message = $4 \
             WHERE id = $1",
        )
        .bind(id)
        .bind(&status)
        .bind(Utc::now())
        .bind(&error_message)
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }

    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<SchedulerExecution>> {
        let row: Option<SchedulerExecutionRow> = sqlx::query_as(
            "SELECT id, job_id, status, started_at, completed_at, error_message \
             FROM scheduler.job_executions WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(self.pool.as_ref())
        .await?;
        Ok(row.map(Into::into))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_row_to_entity() {
        let now = Utc::now();
        let row = SchedulerExecutionRow {
            id: Uuid::new_v4(),
            job_id: Uuid::new_v4(),
            status: "running".to_string(),
            started_at: now,
            completed_at: None,
            error_message: None,
        };
        let entity: SchedulerExecution = row.into();
        assert_eq!(entity.status, "running");
        assert!(entity.completed_at.is_none());
        assert!(entity.error_message.is_none());
    }

    #[test]
    fn test_row_to_entity_completed() {
        let now = Utc::now();
        let row = SchedulerExecutionRow {
            id: Uuid::new_v4(),
            job_id: Uuid::new_v4(),
            status: "failed".to_string(),
            started_at: now,
            completed_at: Some(now),
            error_message: Some("timeout".to_string()),
        };
        let entity: SchedulerExecution = row.into();
        assert_eq!(entity.status, "failed");
        assert!(entity.completed_at.is_some());
        assert_eq!(entity.error_message.as_deref(), Some("timeout"));
    }
}
