// タスクリポジトリの PostgreSQL 実装。
// outbox テーブルへの書き込みを同一トランザクションで行う Transactional Outbox パターン。
use crate::domain::entity::task::{
    CreateTask, Task, TaskChecklistItem, TaskFilter, TaskPriority, TaskStatus, UpdateTaskStatus,
};
use crate::domain::repository::task_repository::TaskRepository;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

pub struct TaskPostgresRepository {
    pool: PgPool,
}

impl TaskPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// DB の tasks テーブル行を表す中間型。reporter_id と labels カラムを含む。
#[derive(sqlx::FromRow)]
struct TaskRow {
    id: Uuid,
    project_id: Uuid,
    title: String,
    description: Option<String>,
    status: String,
    priority: String,
    assignee_id: Option<String>,
    // 報告者 ID（DB の reporter_id カラムに対応）
    reporter_id: Option<String>,
    due_date: Option<chrono::DateTime<chrono::Utc>>,
    created_by: String,
    updated_by: Option<String>,
    version: i32,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

// TaskRow からドメインエンティティ Task へ変換する。
// reporter_id と labels はエンティティに直接マップする。
impl TryFrom<TaskRow> for Task {
    type Error = anyhow::Error;
    fn try_from(row: TaskRow) -> Result<Self, Self::Error> {
        Ok(Task {
            id: row.id,
            project_id: row.project_id,
            title: row.title,
            description: row.description,
            status: row.status.parse().map_err(|e: String| anyhow::anyhow!(e))?,
            priority: row.priority.parse().map_err(|e: String| anyhow::anyhow!(e))?,
            assignee_id: row.assignee_id,
            reporter_id: row.reporter_id,
            due_date: row.due_date,
            // labels は別テーブルで管理するため、ここでは空リストを返す
            // 完全取得が必要な場合は find_by_id_with_labels を使用する
            labels: vec![],
            created_by: row.created_by,
            updated_by: row.updated_by,
            version: row.version,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[derive(sqlx::FromRow)]
struct ChecklistRow {
    id: Uuid,
    task_id: Uuid,
    title: String,
    is_completed: bool,
    sort_order: i32,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<ChecklistRow> for TaskChecklistItem {
    fn from(row: ChecklistRow) -> Self {
        TaskChecklistItem {
            id: row.id,
            task_id: row.task_id,
            title: row.title,
            is_completed: row.is_completed,
            sort_order: row.sort_order,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[async_trait]
impl TaskRepository for TaskPostgresRepository {
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<Task>> {
        let row = sqlx::query_as::<_, TaskRow>(
            "SELECT id, project_id, title, description, status, priority, assignee_id, due_date, created_by, updated_by, version, created_at, updated_at FROM task.tasks WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(Task::try_from).transpose()
    }

    async fn find_all(&self, filter: &TaskFilter) -> anyhow::Result<Vec<Task>> {
        // 動的フィルター: project_id / assignee_id / status を条件に追加する
        let rows = sqlx::query_as::<_, TaskRow>(
            "SELECT id, project_id, title, description, status, priority, assignee_id, due_date, created_by, updated_by, version, created_at, updated_at FROM task.tasks WHERE ($1::uuid IS NULL OR project_id = $1) AND ($2::text IS NULL OR assignee_id = $2) AND ($3::text IS NULL OR status = $3) ORDER BY created_at DESC LIMIT $4 OFFSET $5",
        )
        .bind(filter.project_id)
        .bind(&filter.assignee_id)
        .bind(filter.status.as_ref().map(|s| s.as_str()))
        .bind(filter.limit.unwrap_or(50))
        .bind(filter.offset.unwrap_or(0))
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(Task::try_from).collect()
    }

    async fn count(&self, filter: &TaskFilter) -> anyhow::Result<i64> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM task.tasks WHERE ($1::uuid IS NULL OR project_id = $1) AND ($2::text IS NULL OR assignee_id = $2) AND ($3::text IS NULL OR status = $3)",
        )
        .bind(filter.project_id)
        .bind(&filter.assignee_id)
        .bind(filter.status.as_ref().map(|s| s.as_str()))
        .fetch_one(&self.pool)
        .await?;
        Ok(count)
    }

    async fn create(&self, input: &CreateTask, created_by: &str) -> anyhow::Result<Task> {
        let mut tx = self.pool.begin().await?;
        let task_id = Uuid::new_v4();

        // タスク本体を INSERT する
        let row = sqlx::query_as::<_, TaskRow>(
            r#"INSERT INTO task.tasks (id, project_id, title, description, status, priority, assignee_id, due_date, created_by, version)
               VALUES ($1, $2, $3, $4, 'open', $5, $6, $7, $8, 1)
               RETURNING id, project_id, title, description, status, priority, assignee_id, due_date, created_by, updated_by, version, created_at, updated_at"#,
        )
        .bind(task_id)
        .bind(input.project_id)
        .bind(&input.title)
        .bind(&input.description)
        .bind(input.priority.as_str())
        .bind(&input.assignee_id)
        .bind(input.due_date)
        .bind(created_by)
        .fetch_one(&mut *tx)
        .await?;

        // チェックリスト項目を INSERT する
        for item in &input.checklist {
            sqlx::query(
                "INSERT INTO task.task_checklist_items (id, task_id, title, sort_order) VALUES ($1, $2, $3, $4)",
            )
            .bind(Uuid::new_v4())
            .bind(task_id)
            .bind(&item.title)
            .bind(item.sort_order)
            .execute(&mut *tx)
            .await?;
        }

        // Outbox にイベントを書き込む
        let payload = serde_json::json!({
            "task_id": task_id,
            "project_id": input.project_id,
            "title": input.title,
            "priority": input.priority.as_str(),
            "assignee_id": input.assignee_id,
        });
        sqlx::query(
            "INSERT INTO task.outbox_events (id, aggregate_id, aggregate_type, event_type, payload) VALUES ($1, $2, 'task', 'TaskCreated', $3)",
        )
        .bind(Uuid::new_v4())
        .bind(task_id)
        .bind(payload)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Task::try_from(row)
    }

    async fn find_checklist(&self, task_id: Uuid) -> anyhow::Result<Vec<TaskChecklistItem>> {
        let rows = sqlx::query_as::<_, ChecklistRow>(
            "SELECT id, task_id, title, is_completed, sort_order, created_at, updated_at FROM task.task_checklist_items WHERE task_id = $1 ORDER BY sort_order",
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn update_status(
        &self,
        id: Uuid,
        input: &UpdateTaskStatus,
        updated_by: &str,
    ) -> anyhow::Result<Task> {
        let mut tx = self.pool.begin().await?;

        let row = sqlx::query_as::<_, TaskRow>(
            r#"UPDATE task.tasks SET status = $2, updated_by = $3, version = version + 1, updated_at = now()
               WHERE id = $1 AND version = $4
               RETURNING id, project_id, title, description, status, priority, assignee_id, due_date, created_by, updated_by, version, created_at, updated_at"#,
        )
        .bind(id)
        .bind(input.status.as_str())
        .bind(updated_by)
        .bind(input.expected_version)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Task '{}' not found or version conflict", id))?;

        // Outbox にイベントを書き込む
        let event_type = if input.status == crate::domain::entity::task::TaskStatus::Cancelled {
            "TaskCancelled"
        } else {
            "TaskUpdated"
        };
        let payload = serde_json::json!({
            "task_id": id,
            "status": input.status.as_str(),
            "updated_by": updated_by,
        });
        sqlx::query(
            "INSERT INTO task.outbox_events (id, aggregate_id, aggregate_type, event_type, payload) VALUES ($1, $2, 'task', $3, $4)",
        )
        .bind(Uuid::new_v4())
        .bind(id)
        .bind(event_type)
        .bind(payload)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Task::try_from(row)
    }
}
