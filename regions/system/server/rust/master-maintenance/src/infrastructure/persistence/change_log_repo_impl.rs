use async_trait::async_trait;
use sqlx::PgPool;
use crate::domain::entity::change_log::ChangeLog;
use crate::domain::repository::change_log_repository::ChangeLogRepository;

pub struct ChangeLogPostgresRepository {
    pool: PgPool,
}

impl ChangeLogPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ChangeLogRepository for ChangeLogPostgresRepository {
    async fn create(&self, log: &ChangeLog) -> anyhow::Result<ChangeLog> {
        let row = sqlx::query_as::<_, ChangeLogRow>(
            r#"INSERT INTO master_maintenance.change_logs
               (id, target_table, target_record_id, operation, before_data, after_data,
                changed_columns, changed_by, change_reason, trace_id)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
               RETURNING *"#
        )
        .bind(log.id)
        .bind(&log.target_table)
        .bind(&log.target_record_id)
        .bind(&log.operation)
        .bind(&log.before_data)
        .bind(&log.after_data)
        .bind(&log.changed_columns)
        .bind(&log.changed_by)
        .bind(&log.change_reason)
        .bind(&log.trace_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn find_by_table(&self, table_name: &str, page: i32, page_size: i32) -> anyhow::Result<(Vec<ChangeLog>, i64)> {
        let offset = (page - 1).max(0) * page_size;

        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM master_maintenance.change_logs WHERE target_table = $1"
        )
        .bind(table_name)
        .fetch_one(&self.pool)
        .await?;

        let rows = sqlx::query_as::<_, ChangeLogRow>(
            "SELECT * FROM master_maintenance.change_logs WHERE target_table = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(table_name)
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok((rows.into_iter().map(|r| r.into()).collect(), total.0))
    }

    async fn find_by_record(&self, table_name: &str, record_id: &str, page: i32, page_size: i32) -> anyhow::Result<(Vec<ChangeLog>, i64)> {
        let offset = (page - 1).max(0) * page_size;

        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM master_maintenance.change_logs WHERE target_table = $1 AND target_record_id = $2"
        )
        .bind(table_name)
        .bind(record_id)
        .fetch_one(&self.pool)
        .await?;

        let rows = sqlx::query_as::<_, ChangeLogRow>(
            "SELECT * FROM master_maintenance.change_logs WHERE target_table = $1 AND target_record_id = $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4"
        )
        .bind(table_name)
        .bind(record_id)
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok((rows.into_iter().map(|r| r.into()).collect(), total.0))
    }
}

#[derive(sqlx::FromRow)]
struct ChangeLogRow {
    id: uuid::Uuid,
    target_table: String,
    target_record_id: String,
    operation: String,
    before_data: Option<serde_json::Value>,
    after_data: Option<serde_json::Value>,
    changed_columns: Option<Vec<String>>,
    changed_by: String,
    change_reason: Option<String>,
    trace_id: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<ChangeLogRow> for ChangeLog {
    fn from(row: ChangeLogRow) -> Self {
        Self {
            id: row.id,
            target_table: row.target_table,
            target_record_id: row.target_record_id,
            operation: row.operation,
            before_data: row.before_data,
            after_data: row.after_data,
            changed_columns: row.changed_columns,
            changed_by: row.changed_by,
            change_reason: row.change_reason,
            trace_id: row.trace_id,
            created_at: row.created_at,
        }
    }
}
