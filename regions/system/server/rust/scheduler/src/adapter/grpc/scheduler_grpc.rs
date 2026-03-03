use std::sync::Arc;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::entity::scheduler_execution::SchedulerExecution;
use crate::domain::entity::scheduler_job::SchedulerJob;
use crate::domain::repository::SchedulerExecutionRepository;
use crate::usecase::create_job::{CreateJobError, CreateJobInput, CreateJobUseCase};
use crate::usecase::delete_job::{DeleteJobError, DeleteJobUseCase};
use crate::usecase::get_job::{GetJobError, GetJobUseCase};
use crate::usecase::list_executions::{ListExecutionsError, ListExecutionsUseCase};
use crate::usecase::list_jobs::{ListJobsInput, ListJobsUseCase};
use crate::usecase::pause_job::{PauseJobError, PauseJobUseCase};
use crate::usecase::resume_job::{ResumeJobError, ResumeJobUseCase};
use crate::usecase::trigger_job::{TriggerJobError, TriggerJobUseCase};
use crate::usecase::update_job::{UpdateJobError, UpdateJobInput, UpdateJobUseCase};

#[derive(Debug, Clone)]
pub struct JobData {
    pub id: String,
    pub name: String,
    pub description: String,
    pub cron_expression: String,
    pub timezone: String,
    pub target_type: String,
    pub target: String,
    pub payload_json: Vec<u8>,
    pub status: String,
    pub next_run_at: Option<DateTime<Utc>>,
    pub last_run_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct JobExecutionData {
    pub id: String,
    pub job_id: String,
    pub status: String,
    pub triggered_by: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateJobRequest {
    pub name: String,
    pub description: String,
    pub cron_expression: String,
    pub timezone: String,
    pub target_type: String,
    pub target: String,
    pub payload_json: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct CreateJobResponse {
    pub job: JobData,
}

#[derive(Debug, Clone)]
pub struct GetJobRequest {
    pub job_id: String,
}

#[derive(Debug, Clone)]
pub struct GetJobResponse {
    pub job: JobData,
}

#[derive(Debug, Clone)]
pub struct ListJobsRequest {
    pub status: String,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Debug, Clone)]
pub struct ListJobsResponse {
    pub jobs: Vec<JobData>,
    pub total_count: u64,
    pub page: i32,
    pub page_size: i32,
    pub has_next: bool,
}

#[derive(Debug, Clone)]
pub struct UpdateJobRequest {
    pub job_id: String,
    pub name: String,
    pub description: String,
    pub cron_expression: String,
    pub timezone: String,
    pub target_type: String,
    pub target: String,
    pub payload_json: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct UpdateJobResponse {
    pub job: JobData,
}

#[derive(Debug, Clone)]
pub struct DeleteJobRequest {
    pub job_id: String,
}

#[derive(Debug, Clone)]
pub struct DeleteJobResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct PauseJobRequest {
    pub job_id: String,
}

#[derive(Debug, Clone)]
pub struct PauseJobResponse {
    pub job: JobData,
}

#[derive(Debug, Clone)]
pub struct ResumeJobRequest {
    pub job_id: String,
}

#[derive(Debug, Clone)]
pub struct ResumeJobResponse {
    pub job: JobData,
}

#[derive(Debug, Clone)]
pub struct TriggerJobRequest {
    pub job_id: String,
}

#[derive(Debug, Clone)]
pub struct TriggerJobResponse {
    pub execution_id: String,
    pub job_id: String,
    pub status: String,
    pub triggered_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct GetJobExecutionRequest {
    pub execution_id: String,
}

#[derive(Debug, Clone)]
pub struct GetJobExecutionResponse {
    pub execution: JobExecutionData,
}

#[derive(Debug, Clone)]
pub struct ListExecutionsRequest {
    pub job_id: String,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Debug, Clone)]
pub struct ListExecutionsResponse {
    pub executions: Vec<JobExecutionData>,
    pub total_count: u64,
    pub page: i32,
    pub page_size: i32,
    pub has_next: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum GrpcError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("failed precondition: {0}")]
    FailedPrecondition(String),

    #[error("internal: {0}")]
    Internal(String),

    #[error("unimplemented: {0}")]
    Unimplemented(String),
}

pub struct SchedulerGrpcService {
    create_job_uc: Arc<CreateJobUseCase>,
    get_job_uc: Arc<GetJobUseCase>,
    list_jobs_uc: Arc<ListJobsUseCase>,
    update_job_uc: Arc<UpdateJobUseCase>,
    delete_job_uc: Arc<DeleteJobUseCase>,
    pause_job_uc: Arc<PauseJobUseCase>,
    resume_job_uc: Arc<ResumeJobUseCase>,
    trigger_job_uc: Arc<TriggerJobUseCase>,
    list_executions_uc: Arc<ListExecutionsUseCase>,
    execution_repo: Arc<dyn SchedulerExecutionRepository>,
}

impl SchedulerGrpcService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        create_job_uc: Arc<CreateJobUseCase>,
        get_job_uc: Arc<GetJobUseCase>,
        list_jobs_uc: Arc<ListJobsUseCase>,
        update_job_uc: Arc<UpdateJobUseCase>,
        delete_job_uc: Arc<DeleteJobUseCase>,
        pause_job_uc: Arc<PauseJobUseCase>,
        resume_job_uc: Arc<ResumeJobUseCase>,
        trigger_job_uc: Arc<TriggerJobUseCase>,
        list_executions_uc: Arc<ListExecutionsUseCase>,
        execution_repo: Arc<dyn SchedulerExecutionRepository>,
    ) -> Self {
        Self {
            create_job_uc,
            get_job_uc,
            list_jobs_uc,
            update_job_uc,
            delete_job_uc,
            pause_job_uc,
            resume_job_uc,
            trigger_job_uc,
            list_executions_uc,
            execution_repo,
        }
    }

    pub async fn create_job(&self, req: CreateJobRequest) -> Result<CreateJobResponse, GrpcError> {
        let payload = parse_payload(&req.payload_json)?;
        let created = self
            .create_job_uc
            .execute(&CreateJobInput {
                name: req.name,
                description: if req.description.is_empty() {
                    None
                } else {
                    Some(req.description)
                },
                cron_expression: req.cron_expression,
                timezone: if req.timezone.is_empty() {
                    "UTC".to_string()
                } else {
                    req.timezone
                },
                target_type: if req.target_type.is_empty() {
                    "kafka".to_string()
                } else {
                    req.target_type
                },
                target: if req.target.is_empty() {
                    None
                } else {
                    Some(req.target)
                },
                payload,
            })
            .await
            .map_err(|e| match e {
                CreateJobError::InvalidCron(expr) => GrpcError::InvalidArgument(format!(
                    "invalid cron expression: {}",
                    expr
                )),
                CreateJobError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok(CreateJobResponse {
            job: to_job_data(created),
        })
    }

    pub async fn get_job(&self, req: GetJobRequest) -> Result<GetJobResponse, GrpcError> {
        let id = parse_uuid(&req.job_id, "job_id")?;
        let job = self.get_job_uc.execute(&id).await.map_err(|e| match e {
            GetJobError::NotFound(id) => GrpcError::NotFound(format!("job not found: {}", id)),
            GetJobError::Internal(msg) => GrpcError::Internal(msg),
        })?;

        Ok(GetJobResponse {
            job: to_job_data(job),
        })
    }

    pub async fn list_jobs(&self, req: ListJobsRequest) -> Result<ListJobsResponse, GrpcError> {
        let page = if req.page <= 0 { 1 } else { req.page as u32 };
        let page_size = if req.page_size <= 0 {
            20
        } else {
            req.page_size as u32
        };
        let output = self
            .list_jobs_uc
            .execute(&ListJobsInput {
                status: if req.status.is_empty() {
                    None
                } else {
                    Some(req.status)
                },
                page,
                page_size,
            })
            .await
            .map_err(|e| GrpcError::Internal(e.to_string()))?;

        Ok(ListJobsResponse {
            jobs: output.jobs.into_iter().map(to_job_data).collect(),
            total_count: output.total_count,
            page: output.page as i32,
            page_size: output.page_size as i32,
            has_next: output.has_next,
        })
    }

    pub async fn update_job(&self, req: UpdateJobRequest) -> Result<UpdateJobResponse, GrpcError> {
        let id = parse_uuid(&req.job_id, "job_id")?;
        let payload = parse_payload(&req.payload_json)?;

        let updated = self
            .update_job_uc
            .execute(&UpdateJobInput {
                id,
                name: req.name,
                description: if req.description.is_empty() {
                    None
                } else {
                    Some(req.description)
                },
                cron_expression: req.cron_expression,
                timezone: if req.timezone.is_empty() {
                    "UTC".to_string()
                } else {
                    req.timezone
                },
                target_type: if req.target_type.is_empty() {
                    "kafka".to_string()
                } else {
                    req.target_type
                },
                target: if req.target.is_empty() {
                    None
                } else {
                    Some(req.target)
                },
                payload,
            })
            .await
            .map_err(|e| match e {
                UpdateJobError::NotFound(id) => GrpcError::NotFound(format!("job not found: {}", id)),
                UpdateJobError::InvalidCron(expr) => {
                    GrpcError::InvalidArgument(format!("invalid cron expression: {}", expr))
                }
                UpdateJobError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok(UpdateJobResponse {
            job: to_job_data(updated),
        })
    }

    pub async fn delete_job(&self, req: DeleteJobRequest) -> Result<DeleteJobResponse, GrpcError> {
        let id = parse_uuid(&req.job_id, "job_id")?;
        self.delete_job_uc.execute(&id).await.map_err(|e| match e {
            DeleteJobError::NotFound(id) => GrpcError::NotFound(format!("job not found: {}", id)),
            DeleteJobError::JobRunning(id) => {
                GrpcError::FailedPrecondition(format!("job is currently running: {}", id))
            }
            DeleteJobError::Internal(msg) => GrpcError::Internal(msg),
        })?;

        Ok(DeleteJobResponse {
            success: true,
            message: format!("job {} deleted", req.job_id),
        })
    }

    pub async fn pause_job(&self, req: PauseJobRequest) -> Result<PauseJobResponse, GrpcError> {
        let id = parse_uuid(&req.job_id, "job_id")?;
        let job = self.pause_job_uc.execute(&id).await.map_err(|e| match e {
            PauseJobError::NotFound(id) => GrpcError::NotFound(format!("job not found: {}", id)),
            PauseJobError::Internal(msg) => GrpcError::Internal(msg),
        })?;
        Ok(PauseJobResponse {
            job: to_job_data(job),
        })
    }

    pub async fn resume_job(&self, req: ResumeJobRequest) -> Result<ResumeJobResponse, GrpcError> {
        let id = parse_uuid(&req.job_id, "job_id")?;
        let job = self.resume_job_uc.execute(&id).await.map_err(|e| match e {
            ResumeJobError::NotFound(id) => GrpcError::NotFound(format!("job not found: {}", id)),
            ResumeJobError::Internal(msg) => GrpcError::Internal(msg),
        })?;
        Ok(ResumeJobResponse {
            job: to_job_data(job),
        })
    }

    pub async fn trigger_job(
        &self,
        req: TriggerJobRequest,
    ) -> Result<TriggerJobResponse, GrpcError> {
        let job_id = parse_uuid(&req.job_id, "job_id")?;

        match self.trigger_job_uc.execute(&job_id).await {
            Ok(execution) => Ok(TriggerJobResponse {
                execution_id: execution.id.to_string(),
                job_id: execution.job_id.to_string(),
                status: execution.status,
                triggered_at: execution.started_at,
            }),
            Err(TriggerJobError::NotFound(id)) => {
                Err(GrpcError::NotFound(format!("job not found: {}", id)))
            }
            Err(TriggerJobError::NotActive(id)) => {
                Err(GrpcError::FailedPrecondition(format!("job not active: {}", id)))
            }
            Err(TriggerJobError::Internal(e)) => Err(GrpcError::Internal(e)),
        }
    }

    pub async fn get_job_execution(
        &self,
        req: GetJobExecutionRequest,
    ) -> Result<GetJobExecutionResponse, GrpcError> {
        let execution_id = parse_uuid(&req.execution_id, "execution_id")?;
        let execution = self
            .execution_repo
            .find_by_id(&execution_id)
            .await
            .map_err(|e| GrpcError::Internal(e.to_string()))?
            .ok_or_else(|| GrpcError::NotFound(format!("execution not found: {}", execution_id)))?;

        Ok(GetJobExecutionResponse {
            execution: to_execution_data(execution),
        })
    }

    pub async fn list_executions(
        &self,
        req: ListExecutionsRequest,
    ) -> Result<ListExecutionsResponse, GrpcError> {
        let job_id = parse_uuid(&req.job_id, "job_id")?;
        let executions = self
            .list_executions_uc
            .execute(&job_id)
            .await
            .map_err(|e| match e {
                ListExecutionsError::NotFound(id) => {
                    GrpcError::NotFound(format!("job not found: {}", id))
                }
                ListExecutionsError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        let page = if req.page <= 0 { 1 } else { req.page as usize };
        let page_size = if req.page_size <= 0 {
            20
        } else {
            req.page_size as usize
        };
        let total_count = executions.len() as u64;
        let start = (page - 1) * page_size;
        let page_items: Vec<SchedulerExecution> = executions
            .into_iter()
            .skip(start)
            .take(page_size)
            .collect();
        let has_next = start + page_items.len() < total_count as usize;

        Ok(ListExecutionsResponse {
            executions: page_items.into_iter().map(to_execution_data).collect(),
            total_count,
            page: page as i32,
            page_size: page_size as i32,
            has_next,
        })
    }
}

fn parse_uuid(value: &str, field: &str) -> Result<Uuid, GrpcError> {
    Uuid::parse_str(value)
        .map_err(|_| GrpcError::InvalidArgument(format!("invalid {}: {}", field, value)))
}

fn parse_payload(payload_json: &[u8]) -> Result<serde_json::Value, GrpcError> {
    if payload_json.is_empty() {
        return Ok(serde_json::json!({}));
    }
    serde_json::from_slice(payload_json)
        .map_err(|e| GrpcError::InvalidArgument(format!("invalid payload_json: {}", e)))
}

fn to_job_data(job: SchedulerJob) -> JobData {
    JobData {
        id: job.id.to_string(),
        name: job.name,
        description: job.description.unwrap_or_default(),
        cron_expression: job.cron_expression,
        timezone: job.timezone,
        target_type: job.target_type,
        target: job.target.unwrap_or_default(),
        payload_json: serde_json::to_vec(&job.payload).unwrap_or_default(),
        status: job.status,
        next_run_at: job.next_run_at,
        last_run_at: job.last_run_at,
        created_at: job.created_at,
        updated_at: job.updated_at,
    }
}

fn to_execution_data(execution: SchedulerExecution) -> JobExecutionData {
    let duration_ms = execution.completed_at.and_then(|finished_at| {
        let duration = finished_at - execution.started_at;
        if duration.num_milliseconds() >= 0 {
            Some(duration.num_milliseconds() as u64)
        } else {
            None
        }
    });

    JobExecutionData {
        id: execution.id.to_string(),
        job_id: execution.job_id.to_string(),
        status: normalize_status(&execution.status),
        triggered_by: "unknown".to_string(),
        started_at: execution.started_at,
        finished_at: execution.completed_at,
        duration_ms,
        error_message: execution.error_message,
    }
}

fn normalize_status(status: &str) -> String {
    match status {
        "completed" => "succeeded".to_string(),
        other => other.to_string(),
    }
}
