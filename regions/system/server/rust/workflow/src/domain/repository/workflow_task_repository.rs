use async_trait::async_trait;

use crate::domain::entity::workflow_task::WorkflowTask;

// RUST-CRIT-001 対応: テナント分離のため全メソッドに tenant_id パラメータを追加する
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait WorkflowTaskRepository: Send + Sync {
    // テナントスコープでIDによるタスク検索を行う
    async fn find_by_id(&self, tenant_id: &str, id: &str) -> anyhow::Result<Option<WorkflowTask>>;
    // テナントスコープでフィルタ付きタスク一覧を取得する
    async fn find_all(
        &self,
        tenant_id: &str,
        assignee_id: Option<String>,
        status: Option<String>,
        instance_id: Option<String>,
        overdue_only: bool,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<WorkflowTask>, u64)>;
    // 期限超過タスク一覧を取得する（スケジューラ用、全テナント対象）
    async fn find_overdue(&self) -> anyhow::Result<Vec<WorkflowTask>>;
    // テナントスコープでタスクを新規作成する
    async fn create(&self, tenant_id: &str, task: &WorkflowTask) -> anyhow::Result<()>;
    // テナントスコープでタスクを更新する
    async fn update(&self, tenant_id: &str, task: &WorkflowTask) -> anyhow::Result<()>;
}
