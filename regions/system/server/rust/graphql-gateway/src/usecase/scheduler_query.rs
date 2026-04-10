use crate::domain::model::{Job, JobExecution};
use crate::infrastructure::grpc::SchedulerGrpcClient;
use std::sync::Arc;
use tracing::instrument;

pub struct SchedulerQueryResolver {
    client: Arc<SchedulerGrpcClient>,
}

impl SchedulerQueryResolver {
    pub fn new(client: Arc<SchedulerGrpcClient>) -> Self {
        Self { client }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_job(&self, job_id: &str) -> anyhow::Result<Option<Job>> {
        self.client.get_job(job_id).await
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_jobs(
        &self,
        status: Option<&str>,
        first: Option<i32>,
        after: Option<i32>,
    ) -> anyhow::Result<Vec<Job>> {
        let page_size = first.unwrap_or(20);
        let page = after.map_or(1, |a| a + 1);
        self.client
            .list_jobs(status, Some(page_size), Some(page))
            .await
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_job_execution(
        &self,
        execution_id: &str,
    ) -> anyhow::Result<Option<JobExecution>> {
        self.client.get_job_execution(execution_id).await
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_executions(
        &self,
        job_id: &str,
        first: Option<i32>,
        after: Option<i32>,
        status: Option<&str>,
    ) -> anyhow::Result<Vec<JobExecution>> {
        let page_size = first.unwrap_or(20);
        let page = after.map_or(1, |a| a + 1);
        self.client
            .list_executions(job_id, Some(page_size), Some(page), status)
            .await
    }
}
