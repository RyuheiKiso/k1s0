use async_trait::async_trait;

use crate::domain::entity::workflow_definition::WorkflowDefinition;

// RUST-CRIT-001 対応: テナント分離のため全メソッドに tenant_id パラメータを追加する
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait WorkflowDefinitionRepository: Send + Sync {
    // テナントスコープでIDによるワークフロー定義検索を行う
    async fn find_by_id(&self, tenant_id: &str, id: &str) -> anyhow::Result<Option<WorkflowDefinition>>;
    // テナントスコープで名前によるワークフロー定義検索を行う
    async fn find_by_name(&self, tenant_id: &str, name: &str) -> anyhow::Result<Option<WorkflowDefinition>>;
    // テナントスコープでフィルタ付きワークフロー定義一覧を取得する
    async fn find_all(
        &self,
        tenant_id: &str,
        enabled_only: bool,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<WorkflowDefinition>, u64)>;
    // テナントスコープでワークフロー定義を新規作成する
    async fn create(&self, tenant_id: &str, definition: &WorkflowDefinition) -> anyhow::Result<()>;
    // テナントスコープでワークフロー定義を更新する
    async fn update(&self, tenant_id: &str, definition: &WorkflowDefinition) -> anyhow::Result<()>;
    // テナントスコープでワークフロー定義を削除する
    async fn delete(&self, tenant_id: &str, id: &str) -> anyhow::Result<bool>;
}
