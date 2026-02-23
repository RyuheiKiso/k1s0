use async_trait::async_trait;

use crate::domain::entity::workflow_definition::WorkflowDefinition;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait WorkflowDefinitionRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<WorkflowDefinition>>;
    async fn find_by_name(&self, name: &str) -> anyhow::Result<Option<WorkflowDefinition>>;
    async fn find_all(
        &self,
        enabled_only: bool,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<WorkflowDefinition>, u64)>;
    async fn create(&self, definition: &WorkflowDefinition) -> anyhow::Result<()>;
    async fn update(&self, definition: &WorkflowDefinition) -> anyhow::Result<()>;
    async fn delete(&self, id: &str) -> anyhow::Result<bool>;
}
