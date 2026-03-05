use async_trait::async_trait;

use crate::domain::entity::scheduler_job::SchedulerJob;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SchedulerJobRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<SchedulerJob>>;
    async fn find_all(&self) -> anyhow::Result<Vec<SchedulerJob>>;
    async fn create(&self, job: &SchedulerJob) -> anyhow::Result<()>;
    async fn update(&self, job: &SchedulerJob) -> anyhow::Result<()>;
    async fn delete(&self, id: &str) -> anyhow::Result<bool>;
    async fn find_active_jobs(&self) -> anyhow::Result<Vec<SchedulerJob>>;
}
