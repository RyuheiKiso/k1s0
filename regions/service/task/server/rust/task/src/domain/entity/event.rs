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

#[cfg(test)]
mod tests {
    use super::*;

    // イベントペイロードの JSON シリアライズ/デシリアライズが正常に動作することを検証する
    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_task_event_serde_roundtrip() {
        let task_id = Uuid::new_v4();
        let project_id = Uuid::new_v4();

        // TaskCreatedPayload のラウンドトリップ検証
        let created = TaskCreatedPayload {
            task_id,
            project_id,
            title: "新しいタスク".to_string(),
            priority: "high".to_string(),
            assignee_id: Some("user-001".to_string()),
        };
        let json = serde_json::to_string(&created).unwrap();
        let decoded: TaskCreatedPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.task_id, task_id);
        assert_eq!(decoded.project_id, project_id);
        assert_eq!(decoded.title, "新しいタスク");
        assert_eq!(decoded.priority, "high");
        assert_eq!(decoded.assignee_id, Some("user-001".to_string()));

        // TaskUpdatedPayload のラウンドトリップ検証
        let updated = TaskUpdatedPayload {
            task_id,
            project_id,
            status: "in_progress".to_string(),
            assignee_id: None,
        };
        let json = serde_json::to_string(&updated).unwrap();
        let decoded: TaskUpdatedPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.task_id, task_id);
        assert_eq!(decoded.status, "in_progress");
        assert!(decoded.assignee_id.is_none());

        // TaskCancelledPayload のラウンドトリップ検証
        let cancelled = TaskCancelledPayload {
            task_id,
            project_id,
            reason: "重複タスクのため".to_string(),
        };
        let json = serde_json::to_string(&cancelled).unwrap();
        let decoded: TaskCancelledPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.task_id, task_id);
        assert_eq!(decoded.reason, "重複タスクのため");
    }
}
