use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::scheduler_execution::SchedulerExecution;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SchedulerExecutionRepository: Send + Sync {
    async fn create(&self, execution: &SchedulerExecution) -> anyhow::Result<()>;
    async fn find_by_job_id(&self, job_id: &Uuid) -> anyhow::Result<Vec<SchedulerExecution>>;
    async fn update_status(
        &self,
        id: &Uuid,
        status: String,
        error_message: Option<String>,
    ) -> anyhow::Result<()>;
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<SchedulerExecution>>;
}
