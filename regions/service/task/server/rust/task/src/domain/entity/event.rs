// タスクドメインイベント定義。
use uuid::Uuid;

/// タスク作成イベント（outbox payload 用）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskCreatedPayload {
    pub task_id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub priority: String,
    pub assignee_id: Option<String>,
}

/// タスク更新イベント（outbox payload 用）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskUpdatedPayload {
    pub task_id: Uuid,
    pub project_id: Uuid,
    pub status: String,
    pub assignee_id: Option<String>,
}

/// タスクキャンセルイベント（outbox payload 用）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskCancelledPayload {
    pub task_id: Uuid,
    pub project_id: Uuid,
    pub reason: String,
}
