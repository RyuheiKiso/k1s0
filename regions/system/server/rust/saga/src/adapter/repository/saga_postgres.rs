use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::saga_state::{SagaState, SagaStatus};
use crate::domain::entity::saga_step_log::{SagaStepLog, StepAction, StepStatus};
use crate::domain::repository::saga_repository::{SagaListParams, SagaRepository};

/// SagaPostgresRepository はPostgreSQL実装のSagaリポジトリ。
pub struct SagaPostgresRepository {
    pool: PgPool,
}

impl SagaPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SagaRepository for SagaPostgresRepository {
    async fn create(&self, state: &SagaState) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO saga.saga_states
                (id, workflow_name, current_step, status, payload, correlation_id, initiated_by, error_message, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(state.saga_id)
        .bind(&state.workflow_name)
        .bind(state.current_step)
        .bind(state.status.to_string())
        .bind(&state.payload)
        .bind(&state.correlation_id)
        .bind(&state.initiated_by)
        .bind(&state.error_message)
        .bind(state.created_at)
        .bind(state.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update_with_step_log(
        &self,
        state: &SagaState,
        log: &SagaStepLog,
    ) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            r#"
            UPDATE saga.saga_states
            SET current_step = $2, status = $3, payload = $4, error_message = $5, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(state.saga_id)
        .bind(state.current_step)
        .bind(state.status.to_string())
        .bind(&state.payload)
        .bind(&state.error_message)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO saga.saga_step_logs
                (id, saga_id, step_index, step_name, action, status, request_payload, response_payload, error_message, started_at, completed_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(log.id)
        .bind(log.saga_id)
        .bind(log.step_index)
        .bind(&log.step_name)
        .bind(log.action.to_string())
        .bind(log.status.to_string())
        .bind(&log.request_payload)
        .bind(&log.response_payload)
        .bind(&log.error_message)
        .bind(log.started_at)
        .bind(log.completed_at)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    async fn update_status(
        &self,
        saga_id: Uuid,
        status: &SagaStatus,
        error_message: Option<String>,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE saga.saga_states
            SET status = $2, error_message = $3, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(saga_id)
        .bind(status.to_string())
        .bind(error_message)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_by_id(&self, saga_id: Uuid) -> anyhow::Result<Option<SagaState>> {
        let row = sqlx::query_as::<_, SagaStateRow>(
            r#"
            SELECT id, workflow_name, current_step, status, payload, correlation_id, initiated_by, error_message, created_at, updated_at
            FROM saga.saga_states
            WHERE id = $1
            "#,
        )
        .bind(saga_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }

    async fn find_step_logs(&self, saga_id: Uuid) -> anyhow::Result<Vec<SagaStepLog>> {
        let rows = sqlx::query_as::<_, StepLogRow>(
            r#"
            SELECT id, saga_id, step_index, step_name, action, status, request_payload, response_payload, error_message, started_at, completed_at
            FROM saga.saga_step_logs
            WHERE saga_id = $1
            ORDER BY step_index, started_at
            "#,
        )
        .bind(saga_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    async fn list(&self, params: &SagaListParams) -> anyhow::Result<(Vec<SagaState>, i64)> {
        // Dynamic WHERE clause construction (following audit_log_postgres.rs pattern)
        let mut conditions = Vec::new();
        let mut bind_idx = 1u32;

        if params.workflow_name.is_some() {
            conditions.push(format!("workflow_name = ${}", bind_idx));
            bind_idx += 1;
        }
        if params.status.is_some() {
            conditions.push(format!("status = ${}", bind_idx));
            bind_idx += 1;
        }
        if params.correlation_id.is_some() {
            conditions.push(format!("correlation_id = ${}", bind_idx));
            bind_idx += 1;
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        // Count query
        let count_sql = format!(
            "SELECT COUNT(*) as count FROM saga.saga_states {}",
            where_clause
        );

        let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
        if let Some(ref wn) = params.workflow_name {
            count_query = count_query.bind(wn);
        }
        if let Some(ref s) = params.status {
            count_query = count_query.bind(s.to_string());
        }
        if let Some(ref ci) = params.correlation_id {
            count_query = count_query.bind(ci);
        }
        let total = count_query.fetch_one(&self.pool).await?;

        // Data query
        let offset = ((params.page - 1) * params.page_size) as i64;
        let data_sql = format!(
            "SELECT id, workflow_name, current_step, status, payload, correlation_id, initiated_by, error_message, created_at, updated_at FROM saga.saga_states {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            where_clause, bind_idx, bind_idx + 1
        );

        let mut data_query = sqlx::query_as::<_, SagaStateRow>(&data_sql);
        if let Some(ref wn) = params.workflow_name {
            data_query = data_query.bind(wn);
        }
        if let Some(ref s) = params.status {
            data_query = data_query.bind(s.to_string());
        }
        if let Some(ref ci) = params.correlation_id {
            data_query = data_query.bind(ci);
        }
        data_query = data_query.bind(params.page_size as i64).bind(offset);

        let rows = data_query.fetch_all(&self.pool).await?;
        let sagas: anyhow::Result<Vec<SagaState>> =
            rows.into_iter().map(|r| r.try_into()).collect();

        Ok((sagas?, total))
    }

    async fn find_incomplete(&self) -> anyhow::Result<Vec<SagaState>> {
        let rows = sqlx::query_as::<_, SagaStateRow>(
            r#"
            SELECT id, workflow_name, current_step, status, payload, correlation_id, initiated_by, error_message, created_at, updated_at
            FROM saga.saga_states
            WHERE status IN ('STARTED', 'RUNNING', 'COMPENSATING')
            ORDER BY created_at
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }
}

/// SagaStateRow はDB行からのマッピング用。
#[derive(sqlx::FromRow)]
struct SagaStateRow {
    id: Uuid,
    workflow_name: String,
    current_step: i32,
    status: String,
    payload: Option<serde_json::Value>,
    correlation_id: Option<String>,
    initiated_by: Option<String>,
    error_message: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<SagaStateRow> for SagaState {
    type Error = anyhow::Error;

    fn try_from(row: SagaStateRow) -> anyhow::Result<Self> {
        Ok(SagaState {
            saga_id: row.id,
            workflow_name: row.workflow_name,
            current_step: row.current_step,
            status: SagaStatus::from_str_value(&row.status)?,
            payload: row.payload.unwrap_or(serde_json::Value::Null),
            correlation_id: row.correlation_id,
            initiated_by: row.initiated_by,
            error_message: row.error_message,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

/// StepLogRow はDB行からのマッピング用。
#[derive(sqlx::FromRow)]
struct StepLogRow {
    id: Uuid,
    saga_id: Uuid,
    step_index: i32,
    step_name: String,
    action: String,
    status: String,
    request_payload: Option<serde_json::Value>,
    response_payload: Option<serde_json::Value>,
    error_message: Option<String>,
    started_at: chrono::DateTime<chrono::Utc>,
    completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl TryFrom<StepLogRow> for SagaStepLog {
    type Error = anyhow::Error;

    fn try_from(row: StepLogRow) -> anyhow::Result<Self> {
        let action = match row.action.as_str() {
            "EXECUTE" => StepAction::Execute,
            "COMPENSATE" => StepAction::Compensate,
            other => anyhow::bail!("invalid step action: {}", other),
        };
        let status = match row.status.as_str() {
            "SUCCESS" => StepStatus::Success,
            "FAILED" => StepStatus::Failed,
            "TIMEOUT" => StepStatus::Timeout,
            "SKIPPED" => StepStatus::Skipped,
            other => anyhow::bail!("invalid step status: {}", other),
        };

        Ok(SagaStepLog {
            id: row.id,
            saga_id: row.saga_id,
            step_index: row.step_index,
            step_name: row.step_name,
            action,
            status,
            request_payload: row.request_payload,
            response_payload: row.response_payload,
            error_message: row.error_message,
            started_at: row.started_at,
            completed_at: row.completed_at,
        })
    }
}
