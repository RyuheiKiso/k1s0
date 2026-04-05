// ボードカラムリポジトリの PostgreSQL 実装。
// increment/decrement は楽観的ロックで排他制御する。
// テナント分離のため全メソッドでトランザクション開始後に set_config('app.current_tenant_id', $1, true) を設定する。
// 戻り値型は BoardError（クリーンアーキテクチャ準拠。anyhow::Error は BoardError::Infrastructure に変換する）。
use crate::domain::entity::board_column::{
    BoardColumn, BoardColumnFilter, DecrementColumnRequest, IncrementColumnRequest, UpdateWipLimitRequest,
};
use crate::domain::error::BoardError;
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
    async fn find_by_id(&self, tenant_id: &str, id: Uuid) -> Result<Option<BoardColumn>, BoardError> {
        // テナント分離のため SET LOCAL でセッション変数を設定してから SELECT を実行する
        let mut tx = self.pool.begin().await.map_err(|e| BoardError::Infrastructure(e.into()))?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| BoardError::Infrastructure(e.into()))?;
        let row = sqlx::query_as::<_, ColumnRow>(
            "SELECT id, project_id, status_code, wip_limit, task_count, version, created_at, updated_at FROM board_service.board_columns WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| BoardError::Infrastructure(e.into()))?;
        tx.commit().await.map_err(|e| BoardError::Infrastructure(e.into()))?;
        Ok(row.map(Into::into))
    }

    async fn find_by_project_and_status(
        &self,
        tenant_id: &str,
        project_id: Uuid,
        status_code: &str,
    ) -> Result<Option<BoardColumn>, BoardError> {
        // テナント分離のため SET LOCAL でセッション変数を設定してから SELECT を実行する
        let mut tx = self.pool.begin().await.map_err(|e| BoardError::Infrastructure(e.into()))?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| BoardError::Infrastructure(e.into()))?;
        let row = sqlx::query_as::<_, ColumnRow>(
            "SELECT id, project_id, status_code, wip_limit, task_count, version, created_at, updated_at FROM board_service.board_columns WHERE project_id = $1 AND status_code = $2",
        )
        .bind(project_id)
        .bind(status_code)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| BoardError::Infrastructure(e.into()))?;
        tx.commit().await.map_err(|e| BoardError::Infrastructure(e.into()))?;
        Ok(row.map(Into::into))
    }

    async fn find_all(&self, tenant_id: &str, filter: &BoardColumnFilter) -> Result<Vec<BoardColumn>, BoardError> {
        // テナント分離のため SET LOCAL でセッション変数を設定してから SELECT を実行する
        let mut tx = self.pool.begin().await.map_err(|e| BoardError::Infrastructure(e.into()))?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| BoardError::Infrastructure(e.into()))?;
        let rows = sqlx::query_as::<_, ColumnRow>(
            // project_id カラムは DB 上 text 型のため $1::text でキャストして比較する
            "SELECT id, project_id, status_code, wip_limit, task_count, version, created_at, updated_at FROM board_service.board_columns WHERE ($1::text IS NULL OR project_id = $1::text) AND ($2::text IS NULL OR status_code = $2) ORDER BY status_code LIMIT $3 OFFSET $4",
        )
        .bind(filter.project_id.map(|id| id.to_string()))
        .bind(&filter.status_code)
        .bind(filter.limit.unwrap_or(50))
        .bind(filter.offset.unwrap_or(0))
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| BoardError::Infrastructure(e.into()))?;
        tx.commit().await.map_err(|e| BoardError::Infrastructure(e.into()))?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn count(&self, tenant_id: &str, filter: &BoardColumnFilter) -> Result<i64, BoardError> {
        // テナント分離のため SET LOCAL でセッション変数を設定してから COUNT を実行する
        let mut tx = self.pool.begin().await.map_err(|e| BoardError::Infrastructure(e.into()))?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| BoardError::Infrastructure(e.into()))?;
        let count: i64 = sqlx::query_scalar(
            // project_id カラムは DB 上 text 型のため $1::text でキャストして比較する
            "SELECT COUNT(*) FROM board_service.board_columns WHERE ($1::text IS NULL OR project_id = $1::text) AND ($2::text IS NULL OR status_code = $2)",
        )
        .bind(filter.project_id.map(|id| id.to_string()))
        .bind(&filter.status_code)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| BoardError::Infrastructure(e.into()))?;
        tx.commit().await.map_err(|e| BoardError::Infrastructure(e.into()))?;
        Ok(count)
    }

    async fn increment(&self, tenant_id: &str, req: &IncrementColumnRequest) -> Result<BoardColumn, BoardError> {
        // upsert: カラムが存在しなければ作成し、あれば task_count を +1 する
        // WIP 制限チェックは DB 上で行う (task_count < wip_limit OR wip_limit = 0)
        // テナント分離のため SET LOCAL でセッション変数をトランザクション開始直後に設定する
        let mut tx = self.pool.begin().await.map_err(|e| BoardError::Infrastructure(e.into()))?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| BoardError::Infrastructure(e.into()))?;
        let row = sqlx::query_as::<_, ColumnRow>(
            r#"INSERT INTO board_service.board_columns (id, project_id, status_code, wip_limit, task_count, version)
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
        .await
        .map_err(|e| BoardError::Infrastructure(e.into()))?
        .ok_or_else(|| BoardError::WipLimitExceeded {
            column_id: format!("project={} status={}", req.project_id, req.status_code),
            current: 0,
            limit: 0,
        })?;

        // HIGH-005 監査対応: Outbox イベントに tenant_id を含めてテナント分離を保証する
        sqlx::query(
            "INSERT INTO board_service.outbox_events (id, aggregate_id, aggregate_type, event_type, payload, tenant_id) VALUES ($1, $2, 'board_column', 'BoardColumnUpdated', $3, $4)",
        )
        .bind(Uuid::new_v4())
        .bind(row.id)
        .bind(serde_json::json!({ "column_id": row.id, "project_id": req.project_id, "status_code": req.status_code, "task_count": row.task_count }))
        .bind(tenant_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| BoardError::Infrastructure(e.into()))?;

        tx.commit().await.map_err(|e| BoardError::Infrastructure(e.into()))?;
        Ok(row.into())
    }

    async fn decrement(&self, tenant_id: &str, req: &DecrementColumnRequest) -> Result<BoardColumn, BoardError> {
        // テナント分離のため SET LOCAL でセッション変数をトランザクション開始直後に設定する
        let mut tx = self.pool.begin().await.map_err(|e| BoardError::Infrastructure(e.into()))?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| BoardError::Infrastructure(e.into()))?;
        let row = sqlx::query_as::<_, ColumnRow>(
            r#"UPDATE board_service.board_columns SET
               task_count = GREATEST(0, task_count - 1),
               version = version + 1,
               updated_at = now()
               WHERE project_id = $1 AND status_code = $2
               RETURNING id, project_id, status_code, wip_limit, task_count, version, created_at, updated_at"#,
        )
        .bind(req.project_id)
        .bind(&req.status_code)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| BoardError::Infrastructure(e.into()))?
        .ok_or_else(|| BoardError::NotFound(format!("project={} status={}", req.project_id, req.status_code)))?;

        sqlx::query(
            "INSERT INTO board_service.outbox_events (id, aggregate_id, aggregate_type, event_type, payload) VALUES ($1, $2, 'board_column', 'BoardColumnUpdated', $3)",
        )
        .bind(Uuid::new_v4())
        .bind(row.id)
        .bind(serde_json::json!({ "column_id": row.id, "task_count": row.task_count }))
        .execute(&mut *tx)
        .await
        .map_err(|e| BoardError::Infrastructure(e.into()))?;

        tx.commit().await.map_err(|e| BoardError::Infrastructure(e.into()))?;
        Ok(row.into())
    }

    async fn update_wip_limit(&self, tenant_id: &str, req: &UpdateWipLimitRequest) -> Result<BoardColumn, BoardError> {
        // テナント分離のため SET LOCAL でセッション変数を設定してから UPDATE を実行する
        let mut tx = self.pool.begin().await.map_err(|e| BoardError::Infrastructure(e.into()))?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| BoardError::Infrastructure(e.into()))?;
        let row = sqlx::query_as::<_, ColumnRow>(
            r#"UPDATE board_service.board_columns SET wip_limit = $2, version = version + 1, updated_at = now()
               WHERE id = $1 AND version = $3
               RETURNING id, project_id, status_code, wip_limit, task_count, version, created_at, updated_at"#,
        )
        .bind(req.column_id)
        .bind(req.wip_limit)
        .bind(req.expected_version)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| BoardError::Infrastructure(e.into()))?
        .ok_or_else(|| BoardError::NotFound(format!("BoardColumn '{}'", req.column_id)))?;
        tx.commit().await.map_err(|e| BoardError::Infrastructure(e.into()))?;
        Ok(row.into())
    }
}
