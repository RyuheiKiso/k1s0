use async_trait::async_trait;

use crate::domain::entity::workflow_instance::WorkflowInstance;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait WorkflowInstanceRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<WorkflowInstance>>;
    async fn find_all(
        &self,
        status: Option<String>,
        workflow_id: Option<String>,
        initiator_id: Option<String>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<WorkflowInstance>, u64)>;
    async fn create(&self, instance: &WorkflowInstance) -> anyhow::Result<()>;
    async fn update(&self, instance: &WorkflowInstance) -> anyhow::Result<()>;
}
