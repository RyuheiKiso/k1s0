use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entity::flow_instance::{FlowInstance, FlowInstanceStatus};
use crate::domain::repository::FlowInstanceRepository;

pub struct FlowInstancePostgresRepository {
    pool: Arc<PgPool>,
}

impl FlowInstancePostgresRepository {
    #[must_use]
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FlowInstanceRepository for FlowInstancePostgresRepository {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<FlowInstance>> {
        // "system" テナントコンテキストでフローインスタンスを取得する
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', 'system', true)")
            .execute(&mut *tx)
            .await?;
        let row = sqlx::query_as::<_, FlowInstanceRow>(
            r"
            SELECT id, tenant_id, flow_id, correlation_id, status, current_step_index, started_at, completed_at, duration_ms
            FROM event_monitor.flow_instances WHERE id = $1
            ",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(row.map(Into::into))
    }

    async fn find_by_correlation_id(
        &self,
        correlation_id: String,
    ) -> anyhow::Result<Option<FlowInstance>> {
        // "system" テナントコンテキストで correlation_id からフローインスタンスを取得する
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', 'system', true)")
            .execute(&mut *tx)
            .await?;
        let row = sqlx::query_as::<_, FlowInstanceRow>(
            r"
            SELECT id, tenant_id, flow_id, correlation_id, status, current_step_index, started_at, completed_at, duration_ms
            FROM event_monitor.flow_instances WHERE correlation_id = $1
            ",
        )
        .bind(correlation_id)
        .fetch_optional(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(row.map(Into::into))
    }

    async fn find_by_flow_id_paginated(
        &self,
        flow_id: &Uuid,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<FlowInstance>, u64)> {
        let offset = i64::from(page.saturating_sub(1) * page_size);
        let limit = i64::from(page_size);

        // "system" テナントコンテキストでフロー別インスタンスを取得する
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', 'system', true)")
            .execute(&mut *tx)
            .await?;

        let rows = sqlx::query_as::<_, FlowInstanceRow>(
            r"
            SELECT id, tenant_id, flow_id, correlation_id, status, current_step_index, started_at, completed_at, duration_ms
            FROM event_monitor.flow_instances
            WHERE flow_id = $1
            ORDER BY started_at DESC
            LIMIT $2 OFFSET $3
            ",
        )
        .bind(flow_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&mut *tx)
        .await?;

        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM event_monitor.flow_instances WHERE flow_id = $1")
                .bind(flow_id)
                .fetch_one(&mut *tx)
                .await?;

        tx.commit().await?;
        // LOW-008: 安全な型変換（オーバーフロー防止）
        Ok((rows.into_iter().map(Into::into).collect(), u64::try_from(count.0).unwrap_or(0)))
    }

    async fn find_in_progress(&self) -> anyhow::Result<Vec<FlowInstance>> {
        // "system" テナントコンテキストで進行中インスタンスを取得する（タイムアウト検出用）
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', 'system', true)")
            .execute(&mut *tx)
            .await?;
        let rows = sqlx::query_as::<_, FlowInstanceRow>(
            r"
            SELECT id, tenant_id, flow_id, correlation_id, status, current_step_index, started_at, completed_at, duration_ms
            FROM event_monitor.flow_instances WHERE status = 'in_progress'
            ",
        )
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn create(&self, instance: &FlowInstance) -> anyhow::Result<()> {
        // INSERT 前に RLS セッション変数を設定する
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(&instance.tenant_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            r"
            INSERT INTO event_monitor.flow_instances
                (id, tenant_id, flow_id, correlation_id, status, current_step_index, started_at, completed_at, duration_ms)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ",
        )
        .bind(instance.id)
        .bind(&instance.tenant_id)
        .bind(instance.flow_id)
        .bind(&instance.correlation_id)
        .bind(instance.status.as_str())
        .bind(instance.current_step_index)
        .bind(instance.started_at)
        .bind(instance.completed_at)
        .bind(instance.duration_ms)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }

    async fn update(&self, instance: &FlowInstance) -> anyhow::Result<()> {
        // UPDATE 前に RLS セッション変数を設定する
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(&instance.tenant_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            r"
            UPDATE event_monitor.flow_instances SET
                status = $2, current_step_index = $3, completed_at = $4, duration_ms = $5
            WHERE id = $1
            ",
        )
        .bind(instance.id)
        .bind(instance.status.as_str())
        .bind(instance.current_step_index)
        .bind(instance.completed_at)
        .bind(instance.duration_ms)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }
}

/// DB `から取得したフローインスタンスの中間表現。tenant_id` を含む。
#[derive(sqlx::FromRow)]
struct FlowInstanceRow {
    id: Uuid,
    tenant_id: String,
    flow_id: Uuid,
    correlation_id: String,
    status: String,
    current_step_index: i32,
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
    duration_ms: Option<i64>,
}

/// `FlowInstanceRow` `からドメインエンティティへ変換する。tenant_id` を含める。
impl From<FlowInstanceRow> for FlowInstance {
    fn from(row: FlowInstanceRow) -> Self {
        Self {
            id: row.id,
            tenant_id: row.tenant_id,
            flow_id: row.flow_id,
            correlation_id: row.correlation_id,
            status: FlowInstanceStatus::from_str(&row.status),
            current_step_index: row.current_step_index,
            started_at: row.started_at,
            completed_at: row.completed_at,
            duration_ms: row.duration_ms,
        }
    }
}
