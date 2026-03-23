// タスクリポジトリ trait。
// RLS テナント分離のため、全 DB 操作メソッドに tenant_id パラメータを持つ。
use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::entity::task::{CreateTask, Task, TaskChecklistItem, TaskFilter, UpdateTaskStatus};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait TaskRepository: Send + Sync {
    /// タスクを ID で取得する（RLS テナント分離のため tenant_id を先頭に受け取る）
    async fn find_by_id(&self, tenant_id: &str, id: Uuid) -> anyhow::Result<Option<Task>>;
    /// タスク一覧取得（RLS テナント分離のため tenant_id を先頭に受け取る）
    async fn find_all(&self, tenant_id: &str, filter: &TaskFilter) -> anyhow::Result<Vec<Task>>;
    /// 件数取得（RLS テナント分離のため tenant_id を先頭に受け取る）
    async fn count(&self, tenant_id: &str, filter: &TaskFilter) -> anyhow::Result<i64>;
    /// タスク作成（チェックリスト + outbox を同一トランザクションで保存。RLS テナント分離のため tenant_id を先頭に受け取る）
    async fn create(&self, tenant_id: &str, input: &CreateTask, created_by: &str) -> anyhow::Result<Task>;
    /// チェックリスト取得（RLS テナント分離のため tenant_id を先頭に受け取る）
    async fn find_checklist(&self, tenant_id: &str, task_id: Uuid) -> anyhow::Result<Vec<TaskChecklistItem>>;
    /// ステータス更新（楽観的ロック付き。RLS テナント分離のため tenant_id を先頭に受け取る）
    async fn update_status(
        &self,
        tenant_id: &str,
        id: Uuid,
        input: &UpdateTaskStatus,
        updated_by: &str,
    ) -> anyhow::Result<Task>;
}
