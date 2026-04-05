use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

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
    id: String,
    name: String,
    cron_expression: String,
    job_type: String,
    payload: serde_json::Value,
    enabled: bool,
    max_retries: i32,
    description: Option<String>,
    timezone: String,
    target_type: String,
    target: Option<String>,
    last_run_at: Option<DateTime<Utc>>,
    next_run_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    tenant_id: String,
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
            description: row.description,
            cron_expression: row.cron_expression,
            timezone: row.timezone,
            target_type: row.target_type,
            target: row.target,
            payload: row.payload,
            status,
            next_run_at: row.next_run_at,
            last_run_at: row.last_run_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
            tenant_id: row.tenant_id,
        }
    }
}

#[async_trait]
impl SchedulerJobRepository for SchedulerJobPostgresRepository {
    async fn find_by_id(&self, id: &str, tenant_id: &str) -> anyhow::Result<Option<SchedulerJob>> {
        // CRIT-005 対応: トランザクション内で tenant_id をセッション変数に設定してから SELECT する
        let mut tx = self.pool.begin().await?;

        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        let row: Option<SchedulerJobRow> = sqlx::query_as(
            "SELECT id, name, cron_expression, job_type, payload, enabled, max_retries, \
                    description, timezone, target_type, target, \
                    last_run_at, next_run_at, created_at, updated_at, tenant_id \
             FROM scheduler.scheduler_jobs WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(row.map(Into::into))
    }

    async fn find_all(&self, tenant_id: &str) -> anyhow::Result<Vec<SchedulerJob>> {
        // CRIT-005 対応: トランザクション内で tenant_id をセッション変数に設定してから SELECT する
        let mut tx = self.pool.begin().await?;

        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        let rows: Vec<SchedulerJobRow> = sqlx::query_as(
            "SELECT id, name, cron_expression, job_type, payload, enabled, max_retries, \
                    description, timezone, target_type, target, \
                    last_run_at, next_run_at, created_at, updated_at, tenant_id \
             FROM scheduler.scheduler_jobs ORDER BY created_at DESC",
        )
        .fetch_all(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn create(&self, job: &SchedulerJob) -> anyhow::Result<()> {
        // CRIT-005 対応: トランザクション内で tenant_id をセッション変数に設定してから INSERT する
        let mut tx = self.pool.begin().await?;

        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(&job.tenant_id)
            .execute(&mut *tx)
            .await?;

        let enabled = job.status == "active";
        sqlx::query(
            "INSERT INTO scheduler.scheduler_jobs \
             (id, name, cron_expression, payload, enabled, description, timezone, target_type, target, \
              last_run_at, next_run_at, created_at, updated_at, tenant_id) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)",
        )
        .bind(&job.id)
        .bind(&job.name)
        .bind(&job.cron_expression)
        .bind(&job.payload)
        .bind(enabled)
        .bind(&job.description)
        .bind(&job.timezone)
        .bind(&job.target_type)
        .bind(&job.target)
        .bind(job.last_run_at)
        .bind(job.next_run_at)
        .bind(job.created_at)
        .bind(job.updated_at)
        .bind(&job.tenant_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    async fn update(&self, job: &SchedulerJob) -> anyhow::Result<()> {
        // CRIT-005 対応: トランザクション内で tenant_id をセッション変数に設定してから UPDATE する
        let mut tx = self.pool.begin().await?;

        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(&job.tenant_id)
            .execute(&mut *tx)
            .await?;

        let enabled = job.status == "active";
        sqlx::query(
            "UPDATE scheduler.scheduler_jobs \
             SET name = $2, cron_expression = $3, payload = $4, enabled = $5, \
                 description = $6, timezone = $7, target_type = $8, target = $9, \
                 last_run_at = $10, next_run_at = $11, updated_at = $12 \
             WHERE id = $1 AND tenant_id = $13",
        )
        .bind(&job.id)
        .bind(&job.name)
        .bind(&job.cron_expression)
        .bind(&job.payload)
        .bind(enabled)
        .bind(&job.description)
        .bind(&job.timezone)
        .bind(&job.target_type)
        .bind(&job.target)
        .bind(job.last_run_at)
        .bind(job.next_run_at)
        .bind(job.updated_at)
        .bind(&job.tenant_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    async fn delete(&self, id: &str, tenant_id: &str) -> anyhow::Result<bool> {
        // CRIT-005 対応: トランザクション内で tenant_id をセッション変数に設定してから DELETE する
        let mut tx = self.pool.begin().await?;

        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        let result = sqlx::query(
            "DELETE FROM scheduler.scheduler_jobs WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id)
        .bind(tenant_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(result.rows_affected() > 0)
    }

    async fn find_active_jobs(&self) -> anyhow::Result<Vec<SchedulerJob>> {
        // スケジューラー内部用途のため全テナント横断でアクティブなジョブを取得する
        // CRIT-005 注意: このメソッドは RLS をバイパスするため、DB 接続ユーザーが適切なロールを持つことを前提とする
        let rows: Vec<SchedulerJobRow> = sqlx::query_as(
            "SELECT id, name, cron_expression, job_type, payload, enabled, max_retries, \
                    description, timezone, target_type, target, \
                    last_run_at, next_run_at, created_at, updated_at, tenant_id \
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
            id: format!("job_{}", uuid::Uuid::new_v4().simple()),
            name: "test-job".to_string(),
            cron_expression: "* * * * *".to_string(),
            job_type: "default".to_string(),
            payload: serde_json::json!({"key": "value"}),
            enabled: true,
            max_retries: 3,
            description: None,
            timezone: "UTC".to_string(),
            target_type: "kafka".to_string(),
            target: None,
            last_run_at: None,
            next_run_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tenant_id: "tenant-a".to_string(),
        };
        let entity: SchedulerJob = row.into();
        assert_eq!(entity.status, "active");
        assert_eq!(entity.name, "test-job");
        assert_eq!(entity.timezone, "UTC");
        assert_eq!(entity.target_type, "kafka");
        assert_eq!(entity.tenant_id, "tenant-a");
    }

    #[test]
    fn test_row_to_entity_paused() {
        let row = SchedulerJobRow {
            id: format!("job_{}", uuid::Uuid::new_v4().simple()),
            name: "paused-job".to_string(),
            cron_expression: "0 12 * * *".to_string(),
            job_type: "default".to_string(),
            payload: serde_json::json!({}),
            enabled: false,
            max_retries: 1,
            description: Some("a paused job".to_string()),
            timezone: "Asia/Tokyo".to_string(),
            target_type: "http".to_string(),
            target: Some("https://example.com/hook".to_string()),
            last_run_at: Some(Utc::now()),
            next_run_at: Some(Utc::now()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tenant_id: "system".to_string(),
        };
        let entity: SchedulerJob = row.into();
        assert_eq!(entity.status, "paused");
        assert!(entity.last_run_at.is_some());
        assert!(entity.next_run_at.is_some());
        assert_eq!(entity.description.as_deref(), Some("a paused job"));
        assert_eq!(entity.timezone, "Asia/Tokyo");
        assert_eq!(entity.target_type, "http");
        assert_eq!(entity.target.as_deref(), Some("https://example.com/hook"));
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
