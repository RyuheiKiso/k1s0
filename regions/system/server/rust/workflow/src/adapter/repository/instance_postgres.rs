use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
// L-001 監査対応: QueryBuilder を使ってパラメータインデックスを自動管理する
use sqlx::QueryBuilder;

use crate::domain::entity::workflow_instance::WorkflowInstance;
use crate::domain::repository::WorkflowInstanceRepository;

pub struct InstancePostgresRepository {
    pool: Arc<PgPool>,
}

impl InstancePostgresRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct InstanceRow {
    id: String,
    definition_id: String,
    workflow_name: String,
    title: String,
    initiator_id: String,
    current_step_id: String,
    status: String,
    context: serde_json::Value,
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl From<InstanceRow> for WorkflowInstance {
    fn from(r: InstanceRow) -> Self {
        let current_step = if r.current_step_id.is_empty() {
            None
        } else {
            Some(r.current_step_id)
        };
        WorkflowInstance {
            id: r.id.to_string(),
            workflow_id: r.definition_id.to_string(),
            workflow_name: r.workflow_name,
            title: r.title,
            initiator_id: r.initiator_id,
            current_step_id: current_step,
            status: r.status,
            context: r.context,
            started_at: r.started_at.unwrap_or(r.created_at),
            completed_at: r.completed_at,
            created_at: r.created_at,
        }
    }
}

#[async_trait]
impl WorkflowInstanceRepository for InstancePostgresRepository {
    // RUST-CRIT-001 対応: SET LOCAL でテナント分離を有効化してからSELECTを実行する
    async fn find_by_id(&self, tenant_id: &str, id: &str) -> anyhow::Result<Option<WorkflowInstance>> {
        let mut tx = self.pool.begin().await?;
        // テナント分離: RLS のために現在のテナントIDをセッション変数に設定する
        // HIGH-006 監査対応: SET LOCAL は $1 パラメータバインドをサポートしないため set_config() を使用する
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        let row: Option<InstanceRow> = sqlx::query_as(
            "SELECT id, definition_id, workflow_name, title, initiator_id, current_step_id, \
                    status, context, started_at, completed_at, created_at \
             FROM workflow.workflow_instances WHERE id = $1",
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
        status: Option<String>,
        workflow_id: Option<String>,
        initiator_id: Option<String>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<WorkflowInstance>, u64)> {
        let offset = (page.saturating_sub(1) * page_size) as i64;
        let limit = page_size as i64;

        // データ取得クエリを QueryBuilder で構築する
        // SELECT 句は固定、WHERE 句は動的に push_bind() で条件を追加する
        let mut query_builder: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
            "SELECT id, definition_id, workflow_name, title, initiator_id, current_step_id, \
                    status, context, started_at, completed_at, created_at \
             FROM workflow.workflow_instances",
        );

        // カウントクエリも同様に QueryBuilder で構築する
        let mut count_builder: QueryBuilder<sqlx::Postgres> =
            QueryBuilder::new("SELECT COUNT(*) FROM workflow.workflow_instances");

        // 動的フィルタ条件を QueryBuilder に追加する
        // push_where_or_and() の代わりに手動で WHERE/AND を管理する
        let mut first_condition = true;

        // status フィルタ: 指定された場合のみ WHERE 句に追加する
        if let Some(ref s) = status {
            query_builder.push(if first_condition { " WHERE " } else { " AND " });
            query_builder.push("status = ");
            query_builder.push_bind(s.clone());
            count_builder.push(if first_condition { " WHERE " } else { " AND " });
            count_builder.push("status = ");
            count_builder.push_bind(s.clone());
            first_condition = false;
        }

        // workflow_id フィルタ: 指定された場合のみ条件に追加する
        if let Some(ref w) = workflow_id {
            query_builder.push(if first_condition { " WHERE " } else { " AND " });
            query_builder.push("definition_id = ");
            query_builder.push_bind(w.clone());
            count_builder.push(if first_condition { " WHERE " } else { " AND " });
            count_builder.push("definition_id = ");
            count_builder.push_bind(w.clone());
            first_condition = false;
        }

        // initiator_id フィルタ: 指定された場合のみ条件に追加する
        if let Some(ref i) = initiator_id {
            query_builder.push(if first_condition { " WHERE " } else { " AND " });
            query_builder.push("initiator_id = ");
            query_builder.push_bind(i.clone());
            count_builder.push(if first_condition { " WHERE " } else { " AND " });
            count_builder.push("initiator_id = ");
            count_builder.push_bind(i.clone());
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
            .build_query_as::<InstanceRow>()
            .fetch_all(&mut *tx)
            .await?;

        let count: (i64,) = count_builder
            .build_query_as::<(i64,)>()
            .fetch_one(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok((rows.into_iter().map(Into::into).collect(), count.0 as u64))
    }

    // RUST-CRIT-001 対応: SET LOCAL でテナント分離を有効化してからINSERTを実行する
    async fn create(&self, tenant_id: &str, instance: &WorkflowInstance) -> anyhow::Result<()> {
        let current_step = instance
            .current_step_id
            .as_deref()
            .unwrap_or("")
            .to_string();

        let mut tx = self.pool.begin().await?;
        // テナント分離: RLS のために現在のテナントIDをセッション変数に設定する
        // HIGH-006 監査対応: SET LOCAL は $1 パラメータバインドをサポートしないため set_config() を使用する
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query(
            "INSERT INTO workflow.workflow_instances \
             (id, definition_id, workflow_name, title, initiator_id, current_step_id, \
              status, context, started_at, completed_at, created_at, tenant_id) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
        )
        .bind(&instance.id)
        .bind(&instance.workflow_id)
        .bind(&instance.workflow_name)
        .bind(&instance.title)
        .bind(&instance.initiator_id)
        .bind(&current_step)
        .bind(&instance.status)
        .bind(&instance.context)
        .bind(instance.started_at)
        .bind(instance.completed_at)
        .bind(instance.created_at)
        .bind(tenant_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    // RUST-CRIT-001 対応: SET LOCAL でテナント分離を有効化してからUPDATEを実行する
    async fn update(&self, tenant_id: &str, instance: &WorkflowInstance) -> anyhow::Result<()> {
        let current_step = instance
            .current_step_id
            .as_deref()
            .unwrap_or("")
            .to_string();

        let mut tx = self.pool.begin().await?;
        // テナント分離: RLS のために現在のテナントIDをセッション変数に設定する
        // HIGH-006 監査対応: SET LOCAL は $1 パラメータバインドをサポートしないため set_config() を使用する
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query(
            "UPDATE workflow.workflow_instances \
             SET current_step_id = $2, status = $3, context = $4, completed_at = $5 \
             WHERE id = $1",
        )
        .bind(&instance.id)
        .bind(&current_step)
        .bind(&instance.status)
        .bind(&instance.context)
        .bind(instance.completed_at)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }
}
