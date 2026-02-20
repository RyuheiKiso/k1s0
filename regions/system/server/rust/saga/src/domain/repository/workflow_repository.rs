use async_trait::async_trait;

use crate::domain::entity::workflow::WorkflowDefinition;

/// WorkflowRepository はワークフロー定義のリポジトリトレイト。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait WorkflowRepository: Send + Sync {
    /// ワークフローを登録する。
    async fn register(&self, workflow: WorkflowDefinition) -> anyhow::Result<()>;

    /// ワークフローを名前で取得する。
    async fn get(&self, name: &str) -> anyhow::Result<Option<WorkflowDefinition>>;

    /// 全ワークフローを一覧取得する。
    async fn list(&self) -> anyhow::Result<Vec<WorkflowDefinition>>;
}
