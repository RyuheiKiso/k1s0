use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
// L-001 監査対応: QueryBuilder を使ってパラメータインデックスを自動管理する
use sqlx::QueryBuilder;

use crate::domain::entity::workflow_task::WorkflowTask;
use crate::domain::repository::WorkflowTaskRepository;

pub struct TaskPostgresRepository {
    pool: Arc<PgPool>,
}

impl TaskPostgresRepository {
    #[must_use] 
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct TaskRow {
    id: String,
    instance_id: String,
    step_id: String,
    step_name: String,
    assignee_id: String,
    status: String,
    comment: Option<String>,
    actor_id: Option<String>,
    due_at: Option<DateTime<Utc>>,
    decided_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<TaskRow> for WorkflowTask {
    fn from(r: TaskRow) -> Self {
        let assignee = if r.assignee_id.is_empty() {
            None
        } else {
            Some(r.assignee_id)
        };
        WorkflowTask {
            id: r.id.clone(),
            instance_id: r.instance_id.clone(),
            step_id: r.step_id,
            step_name: r.step_name,
            assignee_id: assignee,
            status: r.status,
            due_at: r.due_at,
            comment: r.comment,
            actor_id: r.actor_id,
            decided_at: r.decided_at,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[async_trait]
impl WorkflowTaskRepository for TaskPostgresRepository {
    // RUST-CRIT-001 対応: SET LOCAL でテナント分離を有効化してからSELECTを実行する
    async fn find_by_id(&self, tenant_id: &str, id: &str) -> anyhow::Result<Option<WorkflowTask>> {
        let mut tx = self.pool.begin().await?;
        // テナント分離: RLS のために現在のテナントIDをセッション変数に設定する
        // HIGH-006 監査対応: SET LOCAL は $1 パラメータバインドをサポートしないため set_config() を使用する
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        let row: Option<TaskRow> = sqlx::query_as(
            "SELECT id, instance_id, step_id, step_name, assignee_id, status, \
                    comment, actor_id, due_at, decided_at, created_at, updated_at \
             FROM workflow.workflow_tasks WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(row.map(Into::into))
    }

    // RUST-CRIT-001 対応: SET LOCAL でテナント分離を有効化してから一覧取得を実行する
    // L-001 監査対応: QueryBuilder を使ってパラメータインデックスの手動カウントを廃止する
    // push_bind() が自動的に $1, $2, ... を割り当てるため、条件追加時のバグリスクが解消される
    async fn find_all(
        &self,
        tenant_id: &str,
        assignee_id: Option<String>,
        status: Option<String>,
        instance_id: Option<String>,
        overdue_only: bool,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<WorkflowTask>, u64)> {
        let offset = i64::from(page.saturating_sub(1) * page_size);
        let limit = i64::from(page_size);

        // データ取得クエリを QueryBuilder で構築する
        // SELECT 句は固定、WHERE 句は動的に push_bind() で条件を追加する
        let mut query_builder: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
            "SELECT id, instance_id, step_id, step_name, assignee_id, status, \
                    comment, actor_id, due_at, decided_at, created_at, updated_at \
             FROM workflow.workflow_tasks",
        );

        // カウントクエリも同様に QueryBuilder で構築する
        let mut count_builder: QueryBuilder<sqlx::Postgres> =
            QueryBuilder::new("SELECT COUNT(*) FROM workflow.workflow_tasks");

        // 動的フィルタ条件を QueryBuilder に追加する
        let mut first_condition = true;

        // assignee_id フィルタ: 担当者IDが指定された場合のみ追加する
        if let Some(ref a) = assignee_id {
            query_builder.push(if first_condition { " WHERE " } else { " AND " });
            query_builder.push("assignee_id = ");
            query_builder.push_bind(a.clone());
            count_builder.push(if first_condition { " WHERE " } else { " AND " });
            count_builder.push("assignee_id = ");
            count_builder.push_bind(a.clone());
            first_condition = false;
        }

        // status フィルタ: ステータスが指定された場合のみ追加する
        if let Some(ref s) = status {
            query_builder.push(if first_condition { " WHERE " } else { " AND " });
            query_builder.push("status = ");
            query_builder.push_bind(s.clone());
            count_builder.push(if first_condition { " WHERE " } else { " AND " });
            count_builder.push("status = ");
            count_builder.push_bind(s.clone());
            first_condition = false;
        }

        // instance_id フィルタ: インスタンスIDが指定された場合のみ追加する
        if let Some(ref i) = instance_id {
            query_builder.push(if first_condition { " WHERE " } else { " AND " });
            query_builder.push("instance_id = ");
            query_builder.push_bind(i.clone());
            count_builder.push(if first_condition { " WHERE " } else { " AND " });
            count_builder.push("instance_id = ");
            count_builder.push_bind(i.clone());
            first_condition = false;
        }

        // overdue_only フィルタ: 期限超過タスクのみを対象とする場合
        // この条件はパラメータバインドではなくリテラルSQL で追加する（NOW() は定数ではないため）
        if overdue_only {
            query_builder.push(if first_condition { " WHERE " } else { " AND " });
            query_builder.push("due_at < NOW() AND status IN ('pending', 'assigned')");
            count_builder.push(if first_condition { " WHERE " } else { " AND " });
            count_builder.push("due_at < NOW() AND status IN ('pending', 'assigned')");
        }

        // ORDER BY / LIMIT / OFFSET を追加する（インデックスは QueryBuilder が自動管理）
        query_builder.push(" ORDER BY created_at DESC LIMIT ");
        query_builder.push_bind(limit);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);

        let mut tx = self.pool.begin().await?;
        // テナント分離: RLS のために現在のテナントIDをセッション変数に設定する
        // HIGH-006 監査対応: SET LOCAL は $1 パラメータバインドをサポートしないため set_config() を使用する
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        // QueryBuilder からクエリを生成して実行する
        let rows = query_builder
            .build_query_as::<TaskRow>()
            .fetch_all(&mut *tx)
            .await?;

        let count: (i64,) = count_builder
            .build_query_as::<(i64,)>()
            .fetch_one(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok((rows.into_iter().map(Into::into).collect(), count.0 as u64))
    }

    // 期限超過タスク一覧を全テナント対象で取得する（スケジューラ用）
    async fn find_overdue(&self) -> anyhow::Result<Vec<WorkflowTask>> {
        let rows: Vec<TaskRow> = sqlx::query_as(
            "SELECT id, instance_id, step_id, step_name, assignee_id, status, \
                    comment, actor_id, due_at, decided_at, created_at, updated_at \
             FROM workflow.workflow_tasks \
             WHERE due_at < NOW() AND status IN ('pending', 'assigned') \
             ORDER BY due_at ASC",
        )
        .fetch_all(self.pool.as_ref())
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    // RUST-CRIT-001 対応: SET LOCAL でテナント分離を有効化してからINSERTを実行する
    async fn create(&self, tenant_id: &str, task: &WorkflowTask) -> anyhow::Result<()> {
        let assignee = task.assignee_id.as_deref().unwrap_or("").to_string();

        let mut tx = self.pool.begin().await?;
        // テナント分離: RLS のために現在のテナントIDをセッション変数に設定する
        // HIGH-006 監査対応: SET LOCAL は $1 パラメータバインドをサポートしないため set_config() を使用する
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query(
            "INSERT INTO workflow.workflow_tasks \
             (id, instance_id, step_id, step_name, assignee_id, status, \
              comment, actor_id, due_at, decided_at, created_at, updated_at, tenant_id) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
        )
        .bind(&task.id)
        .bind(&task.instance_id)
        .bind(&task.step_id)
        .bind(&task.step_name)
        .bind(&assignee)
        .bind(&task.status)
        .bind(&task.comment)
        .bind(&task.actor_id)
        .bind(task.due_at)
        .bind(task.decided_at)
        .bind(task.created_at)
        .bind(task.updated_at)
        .bind(tenant_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    // RUST-CRIT-001 対応: SET LOCAL でテナント分離を有効化してからUPDATEを実行する
    async fn update(&self, tenant_id: &str, task: &WorkflowTask) -> anyhow::Result<()> {
        let assignee = task.assignee_id.as_deref().unwrap_or("").to_string();

        let mut tx = self.pool.begin().await?;
        // テナント分離: RLS のために現在のテナントIDをセッション変数に設定する
        // HIGH-006 監査対応: SET LOCAL は $1 パラメータバインドをサポートしないため set_config() を使用する
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query(
            "UPDATE workflow.workflow_tasks \
             SET assignee_id = $2, status = $3, comment = $4, actor_id = $5, \
                 decided_at = $6 \
             WHERE id = $1",
        )
        .bind(&task.id)
        .bind(&assignee)
        .bind(&task.status)
        .bind(&task.comment)
        .bind(&task.actor_id)
        .bind(task.decided_at)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }
}
