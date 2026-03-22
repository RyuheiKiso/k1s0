// タスクリポジトリ trait。
use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::entity::task::{CreateTask, Task, TaskChecklistItem, TaskFilter, TaskStatus, UpdateTaskStatus};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait TaskRepository: Send + Sync {
    /// タスクを ID で取得する
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<Task>>;
    /// タスク一覧取得
    async fn find_all(&self, filter: &TaskFilter) -> anyhow::Result<Vec<Task>>;
    /// 件数取得
    async fn count(&self, filter: &TaskFilter) -> anyhow::Result<i64>;
    /// タスク作成（チェックリスト + outbox を同一トランザクションで保存）
    async fn create(&self, input: &CreateTask, created_by: &str) -> anyhow::Result<Task>;
    /// チェックリスト取得
    async fn find_checklist(&self, task_id: Uuid) -> anyhow::Result<Vec<TaskChecklistItem>>;
    /// ステータス更新（楽観的ロック付き）
    async fn update_status(
        &self,
        id: Uuid,
        input: &UpdateTaskStatus,
        updated_by: &str,
    ) -> anyhow::Result<Task>;
}
