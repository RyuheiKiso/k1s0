// タスクリポジトリ trait。
// クリーンアーキテクチャに従い、戻り値型に anyhow::Result ではなく TaskError を使用する。
// RLS テナント分離のため、全 DB 操作メソッドに tenant_id パラメータを持つ。
use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::entity::task::{AddChecklistItem, CreateTask, Task, TaskChecklistItem, TaskFilter, UpdateChecklistItem, UpdateTaskStatus};
use crate::domain::error::TaskError;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait TaskRepository: Send + Sync {
    /// タスクを ID で取得する（RLS テナント分離のため tenant_id を先頭に受け取る）
    async fn find_by_id(&self, tenant_id: &str, id: Uuid) -> Result<Option<Task>, TaskError>;
    /// タスク一覧取得（RLS テナント分離のため tenant_id を先頭に受け取る）
    async fn find_all(&self, tenant_id: &str, filter: &TaskFilter) -> Result<Vec<Task>, TaskError>;
    /// 件数取得（RLS テナント分離のため tenant_id を先頭に受け取る）
    async fn count(&self, tenant_id: &str, filter: &TaskFilter) -> Result<i64, TaskError>;
    /// タスク作成（チェックリスト + outbox を同一トランザクションで保存。RLS テナント分離のため tenant_id を先頭に受け取る）
    async fn create(&self, tenant_id: &str, input: &CreateTask, created_by: &str) -> Result<Task, TaskError>;
    /// チェックリスト取得（RLS テナント分離のため tenant_id を先頭に受け取る）
    async fn find_checklist(&self, tenant_id: &str, task_id: Uuid) -> Result<Vec<TaskChecklistItem>, TaskError>;
    /// チェックリスト項目追加（RLS テナント分離のため tenant_id を先頭に受け取る）
    async fn add_checklist_item(&self, tenant_id: &str, task_id: Uuid, input: &AddChecklistItem) -> Result<TaskChecklistItem, TaskError>;
    /// チェックリスト項目更新（RLS テナント分離のため tenant_id を先頭に受け取る）
    async fn update_checklist_item(&self, tenant_id: &str, task_id: Uuid, item_id: Uuid, input: &UpdateChecklistItem) -> Result<TaskChecklistItem, TaskError>;
    /// チェックリスト項目削除（RLS テナント分離のため tenant_id を先頭に受け取る）
    async fn delete_checklist_item(&self, tenant_id: &str, task_id: Uuid, item_id: Uuid) -> Result<(), TaskError>;
    /// ステータス更新（楽観的ロック付き。RLS テナント分離のため tenant_id を先頭に受け取る）
    async fn update_status(
        &self,
        tenant_id: &str,
        id: Uuid,
        input: &UpdateTaskStatus,
        updated_by: &str,
    ) -> Result<Task, TaskError>;
}
