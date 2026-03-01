use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use crate::domain::entity::import_job::ImportJob;
use crate::domain::repository::import_job_repository::ImportJobRepository;

pub struct ImportJobPostgresRepository {
    pool: PgPool,
}

impl ImportJobPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ImportJobRepository for ImportJobPostgresRepository {
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<ImportJob>> {
        let row = sqlx::query_as::<_, ImportJobRow>(
            "SELECT * FROM master_maintenance.import_jobs WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    async fn create(&self, job: &ImportJob) -> anyhow::Result<ImportJob> {
        let row = sqlx::query_as::<_, ImportJobRow>(
            r#"INSERT INTO master_maintenance.import_jobs
               (id, table_id, file_name, status, total_rows, processed_rows, error_rows,
                error_details, started_by)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
               RETURNING *"#
        )
        .bind(job.id)
        .bind(job.table_id)
        .bind(&job.file_name)
        .bind(&job.status)
        .bind(job.total_rows)
        .bind(job.processed_rows)
        .bind(job.error_rows)
        .bind(&job.error_details)
        .bind(&job.started_by)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn update(&self, id: Uuid, job: &ImportJob) -> anyhow::Result<ImportJob> {
        let row = sqlx::query_as::<_, ImportJobRow>(
            r#"UPDATE master_maintenance.import_jobs SET
               status = $2,
               processed_rows = $3,
               error_rows = $4,
               error_details = $5,
               completed_at = $6
               WHERE id = $1 RETURNING *"#
        )
        .bind(id)
        .bind(&job.status)
        .bind(job.processed_rows)
        .bind(job.error_rows)
        .bind(&job.error_details)
        .bind(job.completed_at)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }
}

#[derive(sqlx::FromRow)]
struct ImportJobRow {
    id: Uuid,
    table_id: Uuid,
    file_name: String,
    status: String,
    total_rows: i32,
    processed_rows: i32,
    error_rows: i32,
    error_details: Option<serde_json::Value>,
    started_by: String,
    started_at: chrono::DateTime<chrono::Utc>,
    completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<ImportJobRow> for ImportJob {
    fn from(row: ImportJobRow) -> Self {
        Self {
            id: row.id,
            table_id: row.table_id,
            file_name: row.file_name,
            status: row.status,
            total_rows: row.total_rows,
            processed_rows: row.processed_rows,
            error_rows: row.error_rows,
            error_details: row.error_details,
            started_by: row.started_by,
            started_at: row.started_at,
            completed_at: row.completed_at,
        }
    }
}
