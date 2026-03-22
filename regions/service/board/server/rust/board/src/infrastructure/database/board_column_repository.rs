// ボードカラムリポジトリの PostgreSQL 実装。
// increment/decrement は楽観的ロックで排他制御する。
use crate::domain::entity::board_column::{
    BoardColumn, BoardColumnFilter, DecrementColumnRequest, IncrementColumnRequest, UpdateWipLimitRequest,
};
use crate::domain::repository::board_column_repository::BoardColumnRepository;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

pub struct BoardColumnPostgresRepository {
    pool: PgPool,
}

impl BoardColumnPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct ColumnRow {
    id: Uuid,
    project_id: Uuid,
    status_code: String,
    wip_limit: i32,
    task_count: i32,
    version: i32,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<ColumnRow> for BoardColumn {
    fn from(row: ColumnRow) -> Self {
        BoardColumn {
            id: row.id,
            project_id: row.project_id,
            status_code: row.status_code,
            wip_limit: row.wip_limit,
            task_count: row.task_count,
            version: row.version,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[async_trait]
impl BoardColumnRepository for BoardColumnPostgresRepository {
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<BoardColumn>> {
        let row = sqlx::query_as::<_, ColumnRow>(
            "SELECT id, project_id, status_code, wip_limit, task_count, version, created_at, updated_at FROM board.board_columns WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(Into::into))
    }

    async fn find_by_project_and_status(
        &self,
        project_id: Uuid,
        status_code: &str,
    ) -> anyhow::Result<Option<BoardColumn>> {
        let row = sqlx::query_as::<_, ColumnRow>(
            "SELECT id, project_id, status_code, wip_limit, task_count, version, created_at, updated_at FROM board.board_columns WHERE project_id = $1 AND status_code = $2",
        )
        .bind(project_id)
        .bind(status_code)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(Into::into))
    }

    async fn find_all(&self, filter: &BoardColumnFilter) -> anyhow::Result<Vec<BoardColumn>> {
        let rows = sqlx::query_as::<_, ColumnRow>(
            "SELECT id, project_id, status_code, wip_limit, task_count, version, created_at, updated_at FROM board.board_columns WHERE ($1::uuid IS NULL OR project_id = $1) AND ($2::text IS NULL OR status_code = $2) ORDER BY status_code LIMIT $3 OFFSET $4",
        )
        .bind(filter.project_id)
        .bind(&filter.status_code)
        .bind(filter.limit.unwrap_or(50))
        .bind(filter.offset.unwrap_or(0))
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn count(&self, filter: &BoardColumnFilter) -> anyhow::Result<i64> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM board.board_columns WHERE ($1::uuid IS NULL OR project_id = $1) AND ($2::text IS NULL OR status_code = $2)",
        )
        .bind(filter.project_id)
        .bind(&filter.status_code)
        .fetch_one(&self.pool)
        .await?;
        Ok(count)
    }

    async fn increment(&self, req: &IncrementColumnRequest) -> anyhow::Result<BoardColumn> {
        // upsert: カラムが存在しなければ作成し、あれば task_count を +1 する
        // WIP 制限チェックは DB 上で行う (task_count < wip_limit OR wip_limit = 0)
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query_as::<_, ColumnRow>(
            r#"INSERT INTO board.board_columns (id, project_id, status_code, wip_limit, task_count, version)
               VALUES ($1, $2, $3, 0, 1, 1)
               ON CONFLICT (project_id, status_code) DO UPDATE SET
                 task_count = board_columns.task_count + 1,
                 version = board_columns.version + 1,
                 updated_at = now()
               WHERE board_columns.wip_limit = 0 OR board_columns.task_count < board_columns.wip_limit
               RETURNING id, project_id, status_code, wip_limit, task_count, version, created_at, updated_at"#,
        )
        .bind(Uuid::new_v4())
        .bind(req.project_id)
        .bind(&req.status_code)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| anyhow::anyhow!("WIP limit exceeded for project '{}' status '{}'", req.project_id, req.status_code))?;

        // Outbox イベント
        sqlx::query(
            "INSERT INTO board.outbox_events (id, aggregate_id, aggregate_type, event_type, payload) VALUES ($1, $2, 'board_column', 'BoardColumnUpdated', $3)",
        )
        .bind(Uuid::new_v4())
        .bind(row.id)
        .bind(serde_json::json!({ "column_id": row.id, "project_id": req.project_id, "status_code": req.status_code, "task_count": row.task_count }))
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(row.into())
    }

    async fn decrement(&self, req: &DecrementColumnRequest) -> anyhow::Result<BoardColumn> {
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query_as::<_, ColumnRow>(
            r#"UPDATE board.board_columns SET
               task_count = GREATEST(0, task_count - 1),
               version = version + 1,
               updated_at = now()
               WHERE project_id = $1 AND status_code = $2
               RETURNING id, project_id, status_code, wip_limit, task_count, version, created_at, updated_at"#,
        )
        .bind(req.project_id)
        .bind(&req.status_code)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| anyhow::anyhow!("BoardColumn not found for project '{}' status '{}'", req.project_id, req.status_code))?;

        sqlx::query(
            "INSERT INTO board.outbox_events (id, aggregate_id, aggregate_type, event_type, payload) VALUES ($1, $2, 'board_column', 'BoardColumnUpdated', $3)",
        )
        .bind(Uuid::new_v4())
        .bind(row.id)
        .bind(serde_json::json!({ "column_id": row.id, "task_count": row.task_count }))
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(row.into())
    }

    async fn update_wip_limit(&self, req: &UpdateWipLimitRequest) -> anyhow::Result<BoardColumn> {
        let row = sqlx::query_as::<_, ColumnRow>(
            r#"UPDATE board.board_columns SET wip_limit = $2, version = version + 1, updated_at = now()
               WHERE id = $1 AND version = $3
               RETURNING id, project_id, status_code, wip_limit, task_count, version, created_at, updated_at"#,
        )
        .bind(req.column_id)
        .bind(req.wip_limit)
        .bind(req.expected_version)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("BoardColumn '{}' not found or version conflict", req.column_id))?;
        Ok(row.into())
    }
}
