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
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct DefinitionRow {
    id: uuid::Uuid,
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
            .map_err(|e| anyhow::anyhow!("failed to deserialize steps: {}", e))?;
        Ok(WorkflowDefinition {
            id: r.id.to_string(),
            name: r.name,
            description: r.description,
            version: r.version as u32,
            enabled: r.enabled,
            steps,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
    }
}

#[async_trait]
impl WorkflowDefinitionRepository for DefinitionPostgresRepository {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<WorkflowDefinition>> {
        let uuid = uuid::Uuid::parse_str(id)
            .map_err(|e| anyhow::anyhow!("invalid UUID: {}", e))?;

        let row: Option<DefinitionRow> = sqlx::query_as(
            "SELECT id, name, description, steps, enabled, version, created_at, updated_at \
             FROM workflow.workflow_definitions WHERE id = $1",
        )
        .bind(uuid)
        .fetch_optional(self.pool.as_ref())
        .await?;

        match row {
            Some(r) => Ok(Some(r.try_into()?)),
            None => Ok(None),
        }
    }

    async fn find_by_name(&self, name: &str) -> anyhow::Result<Option<WorkflowDefinition>> {
        let row: Option<DefinitionRow> = sqlx::query_as(
            "SELECT id, name, description, steps, enabled, version, created_at, updated_at \
             FROM workflow.workflow_definitions WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(self.pool.as_ref())
        .await?;

        match row {
            Some(r) => Ok(Some(r.try_into()?)),
            None => Ok(None),
        }
    }

    async fn find_all(
        &self,
        enabled_only: bool,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<WorkflowDefinition>, u64)> {
        let offset = (page.saturating_sub(1) * page_size) as i64;
        let limit = page_size as i64;

        let rows: Vec<DefinitionRow> = if enabled_only {
            sqlx::query_as(
                "SELECT id, name, description, steps, enabled, version, created_at, updated_at \
                 FROM workflow.workflow_definitions WHERE enabled = true \
                 ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool.as_ref())
            .await?
        } else {
            sqlx::query_as(
                "SELECT id, name, description, steps, enabled, version, created_at, updated_at \
                 FROM workflow.workflow_definitions \
                 ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool.as_ref())
            .await?
        };

        let count: (i64,) = if enabled_only {
            sqlx::query_as(
                "SELECT COUNT(*) FROM workflow.workflow_definitions WHERE enabled = true",
            )
            .fetch_one(self.pool.as_ref())
            .await?
        } else {
            sqlx::query_as("SELECT COUNT(*) FROM workflow.workflow_definitions")
                .fetch_one(self.pool.as_ref())
                .await?
        };

        let definitions: Vec<WorkflowDefinition> = rows
            .into_iter()
            .map(TryInto::try_into)
            .collect::<anyhow::Result<Vec<_>>>()?;

        Ok((definitions, count.0 as u64))
    }

    async fn create(&self, definition: &WorkflowDefinition) -> anyhow::Result<()> {
        let uuid = uuid::Uuid::parse_str(&definition.id)
            .map_err(|e| anyhow::anyhow!("invalid UUID: {}", e))?;
        let steps_json = serde_json::to_value(&definition.steps)?;

        sqlx::query(
            "INSERT INTO workflow.workflow_definitions \
             (id, name, description, steps, enabled, version, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(uuid)
        .bind(&definition.name)
        .bind(&definition.description)
        .bind(&steps_json)
        .bind(definition.enabled)
        .bind(definition.version as i32)
        .bind(definition.created_at)
        .bind(definition.updated_at)
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    async fn update(&self, definition: &WorkflowDefinition) -> anyhow::Result<()> {
        let uuid = uuid::Uuid::parse_str(&definition.id)
            .map_err(|e| anyhow::anyhow!("invalid UUID: {}", e))?;
        let steps_json = serde_json::to_value(&definition.steps)?;

        sqlx::query(
            "UPDATE workflow.workflow_definitions \
             SET name = $2, description = $3, steps = $4, enabled = $5, version = $6 \
             WHERE id = $1",
        )
        .bind(uuid)
        .bind(&definition.name)
        .bind(&definition.description)
        .bind(&steps_json)
        .bind(definition.enabled)
        .bind(definition.version as i32)
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    async fn delete(&self, id: &str) -> anyhow::Result<bool> {
        let uuid = uuid::Uuid::parse_str(id)
            .map_err(|e| anyhow::anyhow!("invalid UUID: {}", e))?;

        let result = sqlx::query(
            "DELETE FROM workflow.workflow_definitions WHERE id = $1",
        )
        .bind(uuid)
        .execute(self.pool.as_ref())
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
