use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::entity::workflow::{WorkflowDefinition, WorkflowStep};
use crate::domain::repository::WorkflowRepository;

/// WorkflowPostgresRepository はPostgreSQL実装のワークフローリポジトリ。
pub struct WorkflowPostgresRepository {
    pool: PgPool,
}

impl WorkflowPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
struct WorkflowDefinitionRow {
    name: String,
    steps: serde_json::Value,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<WorkflowDefinitionRow> for WorkflowDefinition {
    type Error = anyhow::Error;

    fn try_from(row: WorkflowDefinitionRow) -> anyhow::Result<Self> {
        let steps: Vec<WorkflowStep> = serde_json::from_value(row.steps)?;
        Ok(WorkflowDefinition {
            name: row.name,
            steps,
        })
    }
}

#[async_trait]
impl WorkflowRepository for WorkflowPostgresRepository {
    async fn register(&self, workflow: WorkflowDefinition) -> anyhow::Result<()> {
        let steps_json = serde_json::to_value(&workflow.steps)?;

        sqlx::query(
            r#"
            INSERT INTO saga.workflow_definitions (name, steps, created_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT (name) DO UPDATE
            SET steps = EXCLUDED.steps
            "#,
        )
        .bind(&workflow.name)
        .bind(&steps_json)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get(&self, name: &str) -> anyhow::Result<Option<WorkflowDefinition>> {
        let row = sqlx::query_as::<_, WorkflowDefinitionRow>(
            r#"
            SELECT name, steps, created_at
            FROM saga.workflow_definitions
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }

    async fn list(&self) -> anyhow::Result<Vec<WorkflowDefinition>> {
        let rows = sqlx::query_as::<_, WorkflowDefinitionRow>(
            r#"
            SELECT name, steps, created_at
            FROM saga.workflow_definitions
            ORDER BY name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::workflow::WorkflowDefinition;

    #[test]
    fn test_workflow_definition_row_conversion() {
        let steps_json = serde_json::json!([
            {
                "name": "step1",
                "service": "svc",
                "method": "Svc.Do",
                "timeout_secs": 30
            }
        ]);

        let row = WorkflowDefinitionRow {
            name: "test-workflow".to_string(),
            steps: steps_json,
            created_at: chrono::Utc::now(),
        };

        let def: WorkflowDefinition = row.try_into().unwrap();
        assert_eq!(def.name, "test-workflow");
        assert_eq!(def.steps.len(), 1);
        assert_eq!(def.steps[0].name, "step1");
        assert_eq!(def.steps[0].service, "svc");
        assert_eq!(def.steps[0].method, "Svc.Do");
        assert_eq!(def.steps[0].timeout_secs, 30);
    }

    #[test]
    fn test_workflow_definition_row_conversion_with_retry() {
        let steps_json = serde_json::json!([
            {
                "name": "reserve-inventory",
                "service": "inventory-service",
                "method": "InventoryService.Reserve",
                "compensate": "InventoryService.Release",
                "timeout_secs": 30,
                "retry": {
                    "max_attempts": 3,
                    "backoff": "exponential",
                    "initial_interval_ms": 1000
                }
            }
        ]);

        let row = WorkflowDefinitionRow {
            name: "order-fulfillment".to_string(),
            steps: steps_json,
            created_at: chrono::Utc::now(),
        };

        let def: WorkflowDefinition = row.try_into().unwrap();
        assert_eq!(def.name, "order-fulfillment");
        assert_eq!(def.steps.len(), 1);
        assert_eq!(def.steps[0].name, "reserve-inventory");
        assert_eq!(
            def.steps[0].compensate.as_deref(),
            Some("InventoryService.Release")
        );
        let retry = def.steps[0].retry.as_ref().unwrap();
        assert_eq!(retry.max_attempts, 3);
        assert_eq!(retry.backoff, "exponential");
        assert_eq!(retry.initial_interval_ms, 1000);
    }

    #[test]
    fn test_workflow_definition_row_invalid_steps_json() {
        let row = WorkflowDefinitionRow {
            name: "bad-workflow".to_string(),
            steps: serde_json::json!("not-an-array"),
            created_at: chrono::Utc::now(),
        };

        let result: Result<WorkflowDefinition, _> = row.try_into();
        assert!(result.is_err());
    }
}
