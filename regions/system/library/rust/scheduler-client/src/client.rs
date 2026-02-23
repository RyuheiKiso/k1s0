use async_trait::async_trait;

use crate::error::SchedulerError;
use crate::job::{Job, JobExecution, JobFilter, JobRequest};

#[async_trait]
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait SchedulerClient: Send + Sync {
    async fn create_job(&self, req: JobRequest) -> Result<Job, SchedulerError>;
    async fn cancel_job(&self, job_id: &str) -> Result<(), SchedulerError>;
    async fn pause_job(&self, job_id: &str) -> Result<(), SchedulerError>;
    async fn resume_job(&self, job_id: &str) -> Result<(), SchedulerError>;
    async fn get_job(&self, job_id: &str) -> Result<Job, SchedulerError>;
    async fn list_jobs(&self, filter: JobFilter) -> Result<Vec<Job>, SchedulerError>;
    async fn get_executions(&self, job_id: &str) -> Result<Vec<JobExecution>, SchedulerError>;
}
