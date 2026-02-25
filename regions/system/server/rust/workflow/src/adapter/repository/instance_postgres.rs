use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

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
    id: uuid::Uuid,
    definition_id: uuid::Uuid,
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
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<WorkflowInstance>> {
        let uuid = uuid::Uuid::parse_str(id)
            .map_err(|e| anyhow::anyhow!("invalid UUID: {}", e))?;

        let row: Option<InstanceRow> = sqlx::query_as(
            "SELECT id, definition_id, workflow_name, title, initiator_id, current_step_id, \
                    status, context, started_at, completed_at, created_at \
             FROM workflow.workflow_instances WHERE id = $1",
        )
        .bind(uuid)
        .fetch_optional(self.pool.as_ref())
        .await?;

        Ok(row.map(Into::into))
    }

    async fn find_all(
        &self,
        status: Option<String>,
        workflow_id: Option<String>,
        initiator_id: Option<String>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<WorkflowInstance>, u64)> {
        let offset = (page.saturating_sub(1) * page_size) as i64;
        let limit = page_size as i64;

        // Build dynamic query with optional filters
        let mut conditions: Vec<String> = Vec::new();
        let mut param_idx = 1u32;

        if status.is_some() {
            conditions.push(format!("status = ${}", param_idx));
            param_idx += 1;
        }
        if workflow_id.is_some() {
            conditions.push(format!("definition_id = ${}", param_idx));
            param_idx += 1;
        }
        if initiator_id.is_some() {
            conditions.push(format!("initiator_id = ${}", param_idx));
            param_idx += 1;
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let query_str = format!(
            "SELECT id, definition_id, workflow_name, title, initiator_id, current_step_id, \
                    status, context, started_at, completed_at, created_at \
             FROM workflow.workflow_instances {} \
             ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            where_clause, param_idx, param_idx + 1
        );

        let count_str = format!(
            "SELECT COUNT(*) FROM workflow.workflow_instances {}",
            where_clause
        );

        let mut query = sqlx::query_as::<_, InstanceRow>(&query_str);
        let mut count_query = sqlx::query_as::<_, (i64,)>(&count_str);

        if let Some(ref s) = status {
            query = query.bind(s.clone());
            count_query = count_query.bind(s.clone());
        }
        if let Some(ref w) = workflow_id {
            let wf_uuid = uuid::Uuid::parse_str(w)
                .map_err(|e| anyhow::anyhow!("invalid workflow UUID: {}", e))?;
            query = query.bind(wf_uuid);
            count_query = count_query.bind(wf_uuid);
        }
        if let Some(ref i) = initiator_id {
            query = query.bind(i.clone());
            count_query = count_query.bind(i.clone());
        }

        query = query.bind(limit).bind(offset);

        let rows = query.fetch_all(self.pool.as_ref()).await?;
        let count = count_query.fetch_one(self.pool.as_ref()).await?;

        Ok((rows.into_iter().map(Into::into).collect(), count.0 as u64))
    }

    async fn create(&self, instance: &WorkflowInstance) -> anyhow::Result<()> {
        let uuid = uuid::Uuid::parse_str(&instance.id)
            .map_err(|e| anyhow::anyhow!("invalid UUID: {}", e))?;
        let def_uuid = uuid::Uuid::parse_str(&instance.workflow_id)
            .map_err(|e| anyhow::anyhow!("invalid definition UUID: {}", e))?;
        let current_step = instance
            .current_step_id
            .as_deref()
            .unwrap_or("")
            .to_string();

        sqlx::query(
            "INSERT INTO workflow.workflow_instances \
             (id, definition_id, workflow_name, title, initiator_id, current_step_id, \
              status, context, started_at, completed_at, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
        )
        .bind(uuid)
        .bind(def_uuid)
        .bind(&instance.workflow_name)
        .bind(&instance.title)
        .bind(&instance.initiator_id)
        .bind(&current_step)
        .bind(&instance.status)
        .bind(&instance.context)
        .bind(instance.started_at)
        .bind(instance.completed_at)
        .bind(instance.created_at)
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    async fn update(&self, instance: &WorkflowInstance) -> anyhow::Result<()> {
        let uuid = uuid::Uuid::parse_str(&instance.id)
            .map_err(|e| anyhow::anyhow!("invalid UUID: {}", e))?;
        let current_step = instance
            .current_step_id
            .as_deref()
            .unwrap_or("")
            .to_string();

        sqlx::query(
            "UPDATE workflow.workflow_instances \
             SET current_step_id = $2, status = $3, context = $4, completed_at = $5 \
             WHERE id = $1",
        )
        .bind(uuid)
        .bind(&current_step)
        .bind(&instance.status)
        .bind(&instance.context)
        .bind(instance.completed_at)
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }
}
