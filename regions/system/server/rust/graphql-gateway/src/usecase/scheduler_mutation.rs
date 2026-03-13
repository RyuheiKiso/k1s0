use std::sync::Arc;
use tracing::instrument;
use crate::domain::model::{
    CreateJobPayload, DeleteJobPayload, JobExecution, PauseJobPayload,
    ResumeJobPayload, TriggerJobPayload, UpdateJobPayload, UserError,
};
use crate::infrastructure::grpc::SchedulerGrpcClient;

pub struct SchedulerMutationResolver {
    client: Arc<SchedulerGrpcClient>,
}

impl SchedulerMutationResolver {
    pub fn new(client: Arc<SchedulerGrpcClient>) -> Self {
        Self { client }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn create_job(
        &self,
        name: &str,
        description: &str,
        cron_expression: &str,
        timezone: &str,
        target_type: &str,
        target: &str,
    ) -> CreateJobPayload {
        match self.client.create_job(name, description, cron_expression, timezone, target_type, target).await {
            Ok(job) => CreateJobPayload { job: Some(job), errors: vec![] },
            Err(e) => CreateJobPayload { job: None, errors: vec![UserError { field: None, message: e.to_string() }] },
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn update_job(
        &self,
        job_id: &str,
        name: Option<&str>,
        description: Option<&str>,
        cron_expression: Option<&str>,
        timezone: Option<&str>,
        target_type: Option<&str>,
        target: Option<&str>,
    ) -> UpdateJobPayload {
        match self.client.update_job(job_id, name, description, cron_expression, timezone, target_type, target).await {
            Ok(job) => UpdateJobPayload { job: Some(job), errors: vec![] },
            Err(e) => UpdateJobPayload { job: None, errors: vec![UserError { field: None, message: e.to_string() }] },
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn delete_job(&self, job_id: &str) -> DeleteJobPayload {
        match self.client.delete_job(job_id).await {
            Ok(success) => DeleteJobPayload { success, errors: vec![] },
            Err(e) => DeleteJobPayload { success: false, errors: vec![UserError { field: None, message: e.to_string() }] },
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn pause_job(&self, job_id: &str) -> PauseJobPayload {
        match self.client.pause_job(job_id).await {
            Ok(job) => PauseJobPayload { job: Some(job), errors: vec![] },
            Err(e) => PauseJobPayload { job: None, errors: vec![UserError { field: None, message: e.to_string() }] },
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn resume_job(&self, job_id: &str) -> ResumeJobPayload {
        match self.client.resume_job(job_id).await {
            Ok(job) => ResumeJobPayload { job: Some(job), errors: vec![] },
            Err(e) => ResumeJobPayload { job: None, errors: vec![UserError { field: None, message: e.to_string() }] },
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn trigger_job(&self, job_id: &str) -> TriggerJobPayload {
        match self.client.trigger_job(job_id).await {
            Ok((execution_id, jid, status, triggered_at)) => TriggerJobPayload {
                execution: Some(JobExecution {
                    id: execution_id,
                    job_id: jid,
                    status,
                    triggered_by: "manual".to_string(),
                    started_at: triggered_at,
                    finished_at: None,
                    duration_ms: None,
                    error_message: None,
                }),
                errors: vec![],
            },
            Err(e) => TriggerJobPayload {
                execution: None,
                errors: vec![UserError { field: None, message: e.to_string() }],
            },
        }
    }
}
