use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entity::flow_definition::{FlowDefinition, FlowSlo, FlowStep};
use crate::domain::repository::FlowDefinitionRepository;

pub struct FlowDefinitionPostgresRepository {
    pool: Arc<PgPool>,
}

impl FlowDefinitionPostgresRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FlowDefinitionRepository for FlowDefinitionPostgresRepository {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<FlowDefinition>> {
        let row = sqlx::query_as::<_, FlowDefRow>(
            r#"
            SELECT id, name, description, domain, steps, slo_target_completion_secs, slo_target_success_rate, slo_alert_on_violation, enabled, created_at, updated_at
            FROM event_monitor.flow_definitions WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool.as_ref())
        .await?;
        Ok(row.map(Into::into))
    }

    async fn find_all(&self) -> anyhow::Result<Vec<FlowDefinition>> {
        let rows = sqlx::query_as::<_, FlowDefRow>(
            r#"
            SELECT id, name, description, domain, steps, slo_target_completion_secs, slo_target_success_rate, slo_alert_on_violation, enabled, created_at, updated_at
            FROM event_monitor.flow_definitions WHERE enabled = true ORDER BY created_at DESC
            "#,
        )
        .fetch_all(self.pool.as_ref())
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        domain: Option<String>,
    ) -> anyhow::Result<(Vec<FlowDefinition>, u64)> {
        let offset = (page.saturating_sub(1) * page_size) as i64;
        let limit = page_size as i64;

        let rows = sqlx::query_as::<_, FlowDefRow>(
            r#"
            SELECT id, name, description, domain, steps, slo_target_completion_secs, slo_target_success_rate, slo_alert_on_violation, enabled, created_at, updated_at
            FROM event_monitor.flow_definitions
            WHERE ($1::text IS NULL OR domain = $1)
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(&domain)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.as_ref())
        .await?;

        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM event_monitor.flow_definitions WHERE ($1::text IS NULL OR domain = $1)",
        )
        .bind(&domain)
        .fetch_one(self.pool.as_ref())
        .await?;

        Ok((rows.into_iter().map(Into::into).collect(), count.0 as u64))
    }

    async fn find_by_domain_and_event_type(
        &self,
        domain: String,
        _event_type: String,
    ) -> anyhow::Result<Vec<FlowDefinition>> {
        let rows = sqlx::query_as::<_, FlowDefRow>(
            r#"
            SELECT id, name, description, domain, steps, slo_target_completion_secs, slo_target_success_rate, slo_alert_on_violation, enabled, created_at, updated_at
            FROM event_monitor.flow_definitions
            WHERE domain = $1 AND enabled = true
            "#,
        )
        .bind(&domain)
        .fetch_all(self.pool.as_ref())
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn create(&self, flow: &FlowDefinition) -> anyhow::Result<()> {
        let steps_json = serde_json::to_value(&flow.steps)?;
        sqlx::query(
            r#"
            INSERT INTO event_monitor.flow_definitions
                (id, name, description, domain, steps, slo_target_completion_secs, slo_target_success_rate, slo_alert_on_violation, enabled, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(flow.id)
        .bind(&flow.name)
        .bind(&flow.description)
        .bind(&flow.domain)
        .bind(steps_json)
        .bind(flow.slo.target_completion_seconds)
        .bind(flow.slo.target_success_rate)
        .bind(flow.slo.alert_on_violation)
        .bind(flow.enabled)
        .bind(flow.created_at)
        .bind(flow.updated_at)
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }

    async fn update(&self, flow: &FlowDefinition) -> anyhow::Result<()> {
        let steps_json = serde_json::to_value(&flow.steps)?;
        sqlx::query(
            r#"
            UPDATE event_monitor.flow_definitions SET
                description = $2, domain = $3, steps = $4,
                slo_target_completion_secs = $5, slo_target_success_rate = $6, slo_alert_on_violation = $7,
                enabled = $8, updated_at = $9
            WHERE id = $1
            "#,
        )
        .bind(flow.id)
        .bind(&flow.description)
        .bind(&flow.domain)
        .bind(steps_json)
        .bind(flow.slo.target_completion_seconds)
        .bind(flow.slo.target_success_rate)
        .bind(flow.slo.alert_on_violation)
        .bind(flow.enabled)
        .bind(flow.updated_at)
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }

    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool> {
        let result = sqlx::query("DELETE FROM event_monitor.flow_definitions WHERE id = $1")
            .bind(id)
            .execute(self.pool.as_ref())
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn exists_by_name(&self, name: String) -> anyhow::Result<bool> {
        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM event_monitor.flow_definitions WHERE name = $1")
                .bind(&name)
                .fetch_one(self.pool.as_ref())
                .await?;
        Ok(count.0 > 0)
    }
}

#[derive(sqlx::FromRow)]
struct FlowDefRow {
    id: Uuid,
    name: String,
    description: String,
    domain: String,
    steps: serde_json::Value,
    slo_target_completion_secs: i32,
    slo_target_success_rate: f64,
    slo_alert_on_violation: bool,
    enabled: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<FlowDefRow> for FlowDefinition {
    fn from(row: FlowDefRow) -> Self {
        let steps: Vec<FlowStep> = serde_json::from_value(row.steps).unwrap_or_default();
        Self {
            id: row.id,
            name: row.name,
            description: row.description,
            domain: row.domain,
            steps,
            slo: FlowSlo {
                target_completion_seconds: row.slo_target_completion_secs,
                target_success_rate: row.slo_target_success_rate,
                alert_on_violation: row.slo_alert_on_violation,
            },
            enabled: row.enabled,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
