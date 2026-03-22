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
    version: i32,
    definition: serde_json::Value,
    enabled: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<WorkflowDefinitionRow> for WorkflowDefinition {
    type Error = anyhow::Error;

    fn try_from(row: WorkflowDefinitionRow) -> anyhow::Result<Self> {
        // definition JSON から total_timeout_secs を取得（なければデフォルト300秒）
        let total_timeout_secs = row
            .definition
            .get("total_timeout_secs")
            .and_then(|v| v.as_u64())
            .unwrap_or(300);
        let steps: Vec<WorkflowStep> = if row.definition.is_array() {
            // Backward compatibility: previous schema stored the step array directly.
            serde_json::from_value(row.definition)?
        } else {
            let steps_value = row
                .definition
                .get("steps")
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("workflow definition must contain 'steps'"))?;
            serde_json::from_value(steps_value)?
        };
        Ok(WorkflowDefinition {
            name: row.name,
            version: row.version,
            enabled: row.enabled,
            total_timeout_secs,
            steps,
        })
    }
}

#[async_trait]
impl WorkflowRepository for WorkflowPostgresRepository {
    async fn register(&self, workflow: WorkflowDefinition) -> anyhow::Result<()> {
        let definition_json = serde_json::json!({
            "steps": workflow.steps,
        });

        sqlx::query(
            r#"
            INSERT INTO saga.workflow_definitions (name, version, definition, enabled, created_at, updated_at)
            VALUES ($1, $2, $3, $4, NOW(), NOW())
            ON CONFLICT (name) DO UPDATE
            SET version = EXCLUDED.version,
                definition = EXCLUDED.definition,
                enabled = EXCLUDED.enabled,
                updated_at = NOW()
            "#,
        )
        .bind(&workflow.name)
        .bind(workflow.version)
        .bind(&definition_json)
        .bind(workflow.enabled)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get(&self, name: &str) -> anyhow::Result<Option<WorkflowDefinition>> {
        let row = sqlx::query_as::<_, WorkflowDefinitionRow>(
            r#"
            SELECT name, version, definition, enabled, created_at, updated_at
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
            SELECT name, version, definition, enabled, created_at, updated_at
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
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::workflow::WorkflowDefinition;

    #[test]
    fn test_workflow_definition_row_conversion() {
        let definition_json = serde_json::json!({
            "steps": [{
                "name": "step1",
                "service": "svc",
                "method": "Svc.Do",
                "timeout_secs": 30
            }]
        });

        let row = WorkflowDefinitionRow {
            name: "test-workflow".to_string(),
            version: 1,
            definition: definition_json,
            enabled: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let def: WorkflowDefinition = row.try_into().unwrap();
        assert_eq!(def.name, "test-workflow");
        assert_eq!(def.version, 1);
        assert!(def.enabled);
        assert_eq!(def.steps.len(), 1);
        assert_eq!(def.steps[0].name, "step1");
        assert_eq!(def.steps[0].service, "svc");
        assert_eq!(def.steps[0].method, "Svc.Do");
        assert_eq!(def.steps[0].timeout_secs, 30);
    }

    #[test]
    fn test_workflow_definition_row_conversion_with_retry() {
        let definition_json = serde_json::json!({
            "steps": [{
                "name": "create-task",
                "service": "task-server",
                "method": "TaskService.CreateTask",
                "compensate": "TaskService.CancelTask",
                "timeout_secs": 30,
                "retry": {
                    "max_attempts": 3,
                    "backoff": "exponential",
                    "initial_interval_ms": 1000
                }
            }]
        });

        let row = WorkflowDefinitionRow {
            name: "task-assignment".to_string(),
            version: 2,
            definition: definition_json,
            enabled: false,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let def: WorkflowDefinition = row.try_into().unwrap();
        assert_eq!(def.name, "task-assignment");
        assert_eq!(def.version, 2);
        assert!(!def.enabled);
        assert_eq!(def.steps.len(), 1);
        assert_eq!(def.steps[0].name, "create-task");
        assert_eq!(
            def.steps[0].compensate.as_deref(),
            Some("TaskService.CancelTask")
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
            version: 1,
            definition: serde_json::json!({"invalid": true}),
            enabled: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let result: Result<WorkflowDefinition, _> = row.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn test_workflow_definition_row_backward_compat_array_definition() {
        let row = WorkflowDefinitionRow {
            name: "legacy-workflow".to_string(),
            version: 1,
            definition: serde_json::json!([
                {
                    "name": "step1",
                    "service": "svc",
                    "method": "Svc.Do",
                    "timeout_secs": 30
                }
            ]),
            enabled: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let def: WorkflowDefinition = row.try_into().unwrap();
        assert_eq!(def.name, "legacy-workflow");
        assert_eq!(def.steps.len(), 1);
    }
}
