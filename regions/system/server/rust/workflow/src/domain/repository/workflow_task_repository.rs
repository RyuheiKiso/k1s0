use async_trait::async_trait;

use crate::domain::entity::workflow_task::WorkflowTask;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait WorkflowTaskRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<WorkflowTask>>;
    async fn find_all(
        &self,
        assignee_id: Option<String>,
        status: Option<String>,
        instance_id: Option<String>,
        overdue_only: bool,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<WorkflowTask>, u64)>;
    async fn find_overdue(&self) -> anyhow::Result<Vec<WorkflowTask>>;
    async fn create(&self, task: &WorkflowTask) -> anyhow::Result<()>;
    async fn update(&self, task: &WorkflowTask) -> anyhow::Result<()>;
}
