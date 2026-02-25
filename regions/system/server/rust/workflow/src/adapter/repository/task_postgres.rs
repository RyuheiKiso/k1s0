use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::domain::entity::workflow_task::WorkflowTask;
use crate::domain::repository::WorkflowTaskRepository;

pub struct TaskPostgresRepository {
    pool: Arc<PgPool>,
}

impl TaskPostgresRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct TaskRow {
    id: uuid::Uuid,
    instance_id: uuid::Uuid,
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
            id: r.id.to_string(),
            instance_id: r.instance_id.to_string(),
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
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<WorkflowTask>> {
        let uuid = uuid::Uuid::parse_str(id)
            .map_err(|e| anyhow::anyhow!("invalid UUID: {}", e))?;

        let row: Option<TaskRow> = sqlx::query_as(
            "SELECT id, instance_id, step_id, step_name, assignee_id, status, \
                    comment, actor_id, due_at, decided_at, created_at, updated_at \
             FROM workflow.workflow_tasks WHERE id = $1",
        )
        .bind(uuid)
        .fetch_optional(self.pool.as_ref())
        .await?;

        Ok(row.map(Into::into))
    }

    async fn find_all(
        &self,
        assignee_id: Option<String>,
        status: Option<String>,
        instance_id: Option<String>,
        overdue_only: bool,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<WorkflowTask>, u64)> {
        let offset = (page.saturating_sub(1) * page_size) as i64;
        let limit = page_size as i64;

        let mut conditions: Vec<String> = Vec::new();
        let mut param_idx = 1u32;

        if assignee_id.is_some() {
            conditions.push(format!("assignee_id = ${}", param_idx));
            param_idx += 1;
        }
        if status.is_some() {
            conditions.push(format!("status = ${}", param_idx));
            param_idx += 1;
        }
        if instance_id.is_some() {
            conditions.push(format!("instance_id = ${}", param_idx));
            param_idx += 1;
        }
        if overdue_only {
            conditions.push("due_at < NOW() AND status IN ('pending', 'assigned')".to_string());
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let query_str = format!(
            "SELECT id, instance_id, step_id, step_name, assignee_id, status, \
                    comment, actor_id, due_at, decided_at, created_at, updated_at \
             FROM workflow.workflow_tasks {} \
             ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            where_clause, param_idx, param_idx + 1
        );

        let count_str = format!(
            "SELECT COUNT(*) FROM workflow.workflow_tasks {}",
            where_clause
        );

        let mut query = sqlx::query_as::<_, TaskRow>(&query_str);
        let mut count_query = sqlx::query_as::<_, (i64,)>(&count_str);

        if let Some(ref a) = assignee_id {
            query = query.bind(a.clone());
            count_query = count_query.bind(a.clone());
        }
        if let Some(ref s) = status {
            query = query.bind(s.clone());
            count_query = count_query.bind(s.clone());
        }
        if let Some(ref i) = instance_id {
            let inst_uuid = uuid::Uuid::parse_str(i)
                .map_err(|e| anyhow::anyhow!("invalid instance UUID: {}", e))?;
            query = query.bind(inst_uuid);
            count_query = count_query.bind(inst_uuid);
        }

        query = query.bind(limit).bind(offset);

        let rows = query.fetch_all(self.pool.as_ref()).await?;
        let count = count_query.fetch_one(self.pool.as_ref()).await?;

        Ok((rows.into_iter().map(Into::into).collect(), count.0 as u64))
    }

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

    async fn create(&self, task: &WorkflowTask) -> anyhow::Result<()> {
        let uuid = uuid::Uuid::parse_str(&task.id)
            .map_err(|e| anyhow::anyhow!("invalid UUID: {}", e))?;
        let inst_uuid = uuid::Uuid::parse_str(&task.instance_id)
            .map_err(|e| anyhow::anyhow!("invalid instance UUID: {}", e))?;
        let assignee = task.assignee_id.as_deref().unwrap_or("").to_string();

        sqlx::query(
            "INSERT INTO workflow.workflow_tasks \
             (id, instance_id, step_id, step_name, assignee_id, status, \
              comment, actor_id, due_at, decided_at, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
        )
        .bind(uuid)
        .bind(inst_uuid)
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
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    async fn update(&self, task: &WorkflowTask) -> anyhow::Result<()> {
        let uuid = uuid::Uuid::parse_str(&task.id)
            .map_err(|e| anyhow::anyhow!("invalid UUID: {}", e))?;
        let assignee = task.assignee_id.as_deref().unwrap_or("").to_string();

        sqlx::query(
            "UPDATE workflow.workflow_tasks \
             SET assignee_id = $2, status = $3, comment = $4, actor_id = $5, \
                 decided_at = $6 \
             WHERE id = $1",
        )
        .bind(uuid)
        .bind(&assignee)
        .bind(&task.status)
        .bind(&task.comment)
        .bind(&task.actor_id)
        .bind(task.decided_at)
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }
}
