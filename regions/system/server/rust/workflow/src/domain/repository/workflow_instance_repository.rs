use async_trait::async_trait;

use crate::domain::entity::workflow_instance::WorkflowInstance;

// RUST-CRIT-001 対応: テナント分離のため全メソッドに tenant_id パラメータを追加する
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait WorkflowInstanceRepository: Send + Sync {
    // テナントスコープでIDによるインスタンス検索を行う
    async fn find_by_id(&self, tenant_id: &str, id: &str) -> anyhow::Result<Option<WorkflowInstance>>;
    // テナントスコープでフィルタ付きインスタンス一覧を取得する
    async fn find_all(
        &self,
        tenant_id: &str,
        status: Option<String>,
        workflow_id: Option<String>,
        initiator_id: Option<String>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<WorkflowInstance>, u64)>;
    // テナントスコープでインスタンスを新規作成する
    async fn create(&self, tenant_id: &str, instance: &WorkflowInstance) -> anyhow::Result<()>;
    // テナントスコープでインスタンスを更新する
    async fn update(&self, tenant_id: &str, instance: &WorkflowInstance) -> anyhow::Result<()>;
}
