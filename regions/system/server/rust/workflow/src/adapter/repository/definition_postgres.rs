use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::domain::entity::workflow_definition::WorkflowDefinition;
use crate::domain::entity::workflow_step::WorkflowStep;
use crate::domain::repository::WorkflowDefinitionRepository;

pub struct DefinitionPostgresRepository {
    pool: Arc<PgPool>,
}

impl DefinitionPostgresRepository {
    #[must_use]
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct DefinitionRow {
    id: String,
    name: String,
    description: String,
    steps: serde_json::Value,
    enabled: bool,
    version: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<DefinitionRow> for WorkflowDefinition {
    type Error = anyhow::Error;

    fn try_from(r: DefinitionRow) -> anyhow::Result<Self> {
        let steps: Vec<WorkflowStep> = serde_json::from_value(r.steps)
            .map_err(|e| anyhow::anyhow!("failed to deserialize steps: {e}"))?;
        Ok(WorkflowDefinition {
            id: r.id.clone(),
            name: r.name,
            description: r.description,
            // LOW-008: 安全な型変換（オーバーフロー防止）
            version: u32::try_from(r.version).unwrap_or(0),
            enabled: r.enabled,
            steps,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
    }
}

#[async_trait]
impl WorkflowDefinitionRepository for DefinitionPostgresRepository {
    // RUST-CRIT-001 対応: SET LOCAL でテナント分離を有効化してからSELECTを実行する
    async fn find_by_id(
        &self,
        tenant_id: &str,
        id: &str,
    ) -> anyhow::Result<Option<WorkflowDefinition>> {
        let mut tx = self.pool.begin().await?;
        // テナント分離: RLS のために現在のテナントIDをセッション変数に設定する
        // HIGH-006 監査対応: SET LOCAL は $1 パラメータバインドをサポートしないため set_config() を使用する
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        let row: Option<DefinitionRow> = sqlx::query_as(
            "SELECT id, name, description, steps, enabled, version, created_at, updated_at \
             FROM workflow.workflow_definitions WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;

        tx.commit().await?;

        match row {
            Some(r) => Ok(Some(r.try_into()?)),
            None => Ok(None),
        }
    }

    // RUST-CRIT-001 対応: SET LOCAL でテナント分離を有効化してから名前検索を実行する
    async fn find_by_name(
        &self,
        tenant_id: &str,
        name: &str,
    ) -> anyhow::Result<Option<WorkflowDefinition>> {
        let mut tx = self.pool.begin().await?;
        // テナント分離: RLS のために現在のテナントIDをセッション変数に設定する
        // HIGH-006 監査対応: SET LOCAL は $1 パラメータバインドをサポートしないため set_config() を使用する
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        let row: Option<DefinitionRow> = sqlx::query_as(
            "SELECT id, name, description, steps, enabled, version, created_at, updated_at \
             FROM workflow.workflow_definitions WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(&mut *tx)
        .await?;

        tx.commit().await?;

        match row {
            Some(r) => Ok(Some(r.try_into()?)),
            None => Ok(None),
        }
    }

    // RUST-CRIT-001 対応: SET LOCAL でテナント分離を有効化してから一覧取得を実行する
    async fn find_all(
        &self,
        tenant_id: &str,
        enabled_only: bool,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<WorkflowDefinition>, u64)> {
        let offset = i64::from(page.saturating_sub(1) * page_size);
        let limit = i64::from(page_size);

        let mut tx = self.pool.begin().await?;
        // テナント分離: RLS のために現在のテナントIDをセッション変数に設定する
        // HIGH-006 監査対応: SET LOCAL は $1 パラメータバインドをサポートしないため set_config() を使用する
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        let rows: Vec<DefinitionRow> = if enabled_only {
            sqlx::query_as(
                "SELECT id, name, description, steps, enabled, version, created_at, updated_at \
                 FROM workflow.workflow_definitions WHERE enabled = true \
                 ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(&mut *tx)
            .await?
        } else {
            sqlx::query_as(
                "SELECT id, name, description, steps, enabled, version, created_at, updated_at \
                 FROM workflow.workflow_definitions \
                 ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(&mut *tx)
            .await?
        };

        let count: (i64,) = if enabled_only {
            sqlx::query_as(
                "SELECT COUNT(*) FROM workflow.workflow_definitions WHERE enabled = true",
            )
            .fetch_one(&mut *tx)
            .await?
        } else {
            sqlx::query_as("SELECT COUNT(*) FROM workflow.workflow_definitions")
                .fetch_one(&mut *tx)
                .await?
        };

        tx.commit().await?;

        let definitions: Vec<WorkflowDefinition> = rows
            .into_iter()
            .map(TryInto::try_into)
            .collect::<anyhow::Result<Vec<_>>>()?;

        // LOW-008: 安全な型変換（オーバーフロー防止）
        Ok((definitions, u64::try_from(count.0).unwrap_or(0)))
    }

    // RUST-CRIT-001 対応: SET LOCAL でテナント分離を有効化してからINSERTを実行する
    async fn create(&self, tenant_id: &str, definition: &WorkflowDefinition) -> anyhow::Result<()> {
        let steps_json = serde_json::to_value(&definition.steps)?;

        let mut tx = self.pool.begin().await?;
        // テナント分離: RLS のために現在のテナントIDをセッション変数に設定する
        // HIGH-006 監査対応: SET LOCAL は $1 パラメータバインドをサポートしないため set_config() を使用する
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query(
            "INSERT INTO workflow.workflow_definitions \
             (id, name, description, steps, enabled, version, created_at, updated_at, tenant_id) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(&definition.id)
        .bind(&definition.name)
        .bind(&definition.description)
        .bind(&steps_json)
        .bind(definition.enabled)
        // LOW-008: 安全な型変換（オーバーフロー防止）
        .bind(i32::try_from(definition.version).unwrap_or(i32::MAX))
        .bind(definition.created_at)
        .bind(definition.updated_at)
        .bind(tenant_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    // RUST-CRIT-001 対応: SET LOCAL でテナント分離を有効化してからUPDATEを実行する
    async fn update(&self, tenant_id: &str, definition: &WorkflowDefinition) -> anyhow::Result<()> {
        let steps_json = serde_json::to_value(&definition.steps)?;

        let mut tx = self.pool.begin().await?;
        // テナント分離: RLS のために現在のテナントIDをセッション変数に設定する
        // HIGH-006 監査対応: SET LOCAL は $1 パラメータバインドをサポートしないため set_config() を使用する
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query(
            "UPDATE workflow.workflow_definitions \
             SET name = $2, description = $3, steps = $4, enabled = $5, version = $6 \
             WHERE id = $1",
        )
        .bind(&definition.id)
        .bind(&definition.name)
        .bind(&definition.description)
        .bind(&steps_json)
        .bind(definition.enabled)
        // LOW-008: 安全な型変換（オーバーフロー防止）
        .bind(i32::try_from(definition.version).unwrap_or(i32::MAX))
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    // RUST-CRIT-001 対応: SET LOCAL でテナント分離を有効化してからDELETEを実行する
    async fn delete(&self, tenant_id: &str, id: &str) -> anyhow::Result<bool> {
        let mut tx = self.pool.begin().await?;
        // テナント分離: RLS のために現在のテナントIDをセッション変数に設定する
        // HIGH-006 監査対応: SET LOCAL は $1 パラメータバインドをサポートしないため set_config() を使用する
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        let result = sqlx::query("DELETE FROM workflow.workflow_definitions WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(result.rows_affected() > 0)
    }
}
