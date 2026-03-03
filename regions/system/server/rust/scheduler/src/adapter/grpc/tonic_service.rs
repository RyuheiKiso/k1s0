use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::common::v1::{
    PaginationResult as ProtoPaginationResult, Timestamp as ProtoTimestamp,
};
use crate::proto::k1s0::system::scheduler::v1::{
    scheduler_service_server::SchedulerService, CreateJobRequest as ProtoCreateJobRequest,
    CreateJobResponse as ProtoCreateJobResponse, DeleteJobRequest as ProtoDeleteJobRequest,
    DeleteJobResponse as ProtoDeleteJobResponse, GetJobExecutionRequest as ProtoGetJobExecutionRequest,
    GetJobExecutionResponse as ProtoGetJobExecutionResponse, GetJobRequest as ProtoGetJobRequest,
    GetJobResponse as ProtoGetJobResponse, Job as ProtoJob, JobExecution as ProtoJobExecution,
    ListExecutionsRequest as ProtoListExecutionsRequest, ListExecutionsResponse as ProtoListExecutionsResponse,
    ListJobsRequest as ProtoListJobsRequest, ListJobsResponse as ProtoListJobsResponse,
    PauseJobRequest as ProtoPauseJobRequest, PauseJobResponse as ProtoPauseJobResponse,
    ResumeJobRequest as ProtoResumeJobRequest, ResumeJobResponse as ProtoResumeJobResponse,
    TriggerJobRequest as ProtoTriggerJobRequest, TriggerJobResponse as ProtoTriggerJobResponse,
    UpdateJobRequest as ProtoUpdateJobRequest, UpdateJobResponse as ProtoUpdateJobResponse,
};

use super::scheduler_grpc::{
    CreateJobRequest, DeleteJobRequest, GetJobExecutionRequest, GetJobRequest, GrpcError,
    JobData, JobExecutionData, ListExecutionsRequest, ListJobsRequest, PauseJobRequest,
    ResumeJobRequest, SchedulerGrpcService, TriggerJobRequest, UpdateJobRequest,
};

impl From<GrpcError> for Status {
    fn from(e: GrpcError) -> Self {
        match e {
            GrpcError::NotFound(msg) => Status::not_found(msg),
            GrpcError::AlreadyExists(msg) => Status::already_exists(msg),
            GrpcError::InvalidArgument(msg) => Status::invalid_argument(msg),
            GrpcError::FailedPrecondition(msg) => Status::failed_precondition(msg),
            GrpcError::Internal(msg) => Status::internal(msg),
            GrpcError::Unimplemented(msg) => Status::unimplemented(msg),
        }
    }
}

fn to_proto_timestamp(dt: chrono::DateTime<chrono::Utc>) -> ProtoTimestamp {
    ProtoTimestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    }
}

fn from_proto_timestamp(ts: ProtoTimestamp) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::<chrono::Utc>::from_timestamp(ts.seconds, ts.nanos as u32)
        .unwrap_or_else(chrono::Utc::now)
}

fn to_proto_job(job: JobData) -> ProtoJob {
    ProtoJob {
        id: job.id,
        name: job.name,
        description: job.description,
        cron_expression: job.cron_expression,
        timezone: job.timezone,
        target_type: job.target_type,
        target: job.target,
        payload: job.payload,
        status: job.status,
        next_run_at: job.next_run_at.map(to_proto_timestamp),
        last_run_at: job.last_run_at.map(to_proto_timestamp),
        created_at: Some(to_proto_timestamp(job.created_at)),
        updated_at: Some(to_proto_timestamp(job.updated_at)),
    }
}

fn to_proto_execution(execution: JobExecutionData) -> ProtoJobExecution {
    ProtoJobExecution {
        id: execution.id,
        job_id: execution.job_id,
        status: execution.status,
        triggered_by: execution.triggered_by,
        started_at: Some(to_proto_timestamp(execution.started_at)),
        finished_at: execution.finished_at.map(to_proto_timestamp),
        duration_ms: execution.duration_ms,
        error_message: execution.error_message,
    }
}

pub struct SchedulerServiceTonic {
    inner: Arc<SchedulerGrpcService>,
}

impl SchedulerServiceTonic {
    pub fn new(inner: Arc<SchedulerGrpcService>) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl SchedulerService for SchedulerServiceTonic {
    async fn create_job(
        &self,
        request: Request<ProtoCreateJobRequest>,
    ) -> Result<Response<ProtoCreateJobResponse>, Status> {
        let inner = request.into_inner();
        let resp = self
            .inner
            .create_job(CreateJobRequest {
                name: inner.name,
                description: inner.description,
                cron_expression: inner.cron_expression,
                timezone: inner.timezone,
                target_type: inner.target_type,
                target: inner.target,
                payload: inner.payload,
            })
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoCreateJobResponse {
            job: Some(to_proto_job(resp.job)),
        }))
    }

    async fn get_job(
        &self,
        request: Request<ProtoGetJobRequest>,
    ) -> Result<Response<ProtoGetJobResponse>, Status> {
        let inner = request.into_inner();
        let resp = self
            .inner
            .get_job(GetJobRequest { job_id: inner.job_id })
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoGetJobResponse {
            job: Some(to_proto_job(resp.job)),
        }))
    }

    async fn list_jobs(
        &self,
        request: Request<ProtoListJobsRequest>,
    ) -> Result<Response<ProtoListJobsResponse>, Status> {
        let inner = request.into_inner();
        let (page, page_size) = inner.pagination.map(|p| (p.page, p.page_size)).unwrap_or((1, 20));
        let resp = self
            .inner
            .list_jobs(ListJobsRequest {
                status: inner.status,
                page,
                page_size,
            })
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoListJobsResponse {
            jobs: resp.jobs.into_iter().map(to_proto_job).collect(),
            pagination: Some(ProtoPaginationResult {
                total_count: resp.total_count.min(i32::MAX as u64) as i32,
                page: resp.page,
                page_size: resp.page_size,
                has_next: resp.has_next,
            }),
        }))
    }

    async fn update_job(
        &self,
        request: Request<ProtoUpdateJobRequest>,
    ) -> Result<Response<ProtoUpdateJobResponse>, Status> {
        let inner = request.into_inner();
        let resp = self
            .inner
            .update_job(UpdateJobRequest {
                job_id: inner.job_id,
                name: inner.name,
                description: inner.description,
                cron_expression: inner.cron_expression,
                timezone: inner.timezone,
                target_type: inner.target_type,
                target: inner.target,
                payload: inner.payload,
            })
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoUpdateJobResponse {
            job: Some(to_proto_job(resp.job)),
        }))
    }

    async fn delete_job(
        &self,
        request: Request<ProtoDeleteJobRequest>,
    ) -> Result<Response<ProtoDeleteJobResponse>, Status> {
        let inner = request.into_inner();
        let resp = self
            .inner
            .delete_job(DeleteJobRequest { job_id: inner.job_id })
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoDeleteJobResponse {
            success: resp.success,
            message: resp.message,
        }))
    }

    async fn pause_job(
        &self,
        request: Request<ProtoPauseJobRequest>,
    ) -> Result<Response<ProtoPauseJobResponse>, Status> {
        let inner = request.into_inner();
        let resp = self
            .inner
            .pause_job(PauseJobRequest { job_id: inner.job_id })
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoPauseJobResponse {
            job: Some(to_proto_job(resp.job)),
        }))
    }

    async fn resume_job(
        &self,
        request: Request<ProtoResumeJobRequest>,
    ) -> Result<Response<ProtoResumeJobResponse>, Status> {
        let inner = request.into_inner();
        let resp = self
            .inner
            .resume_job(ResumeJobRequest { job_id: inner.job_id })
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoResumeJobResponse {
            job: Some(to_proto_job(resp.job)),
        }))
    }

    async fn trigger_job(
        &self,
        request: Request<ProtoTriggerJobRequest>,
    ) -> Result<Response<ProtoTriggerJobResponse>, Status> {
        let inner = request.into_inner();
        let resp = self
            .inner
            .trigger_job(TriggerJobRequest { job_id: inner.job_id })
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoTriggerJobResponse {
            execution_id: resp.execution_id,
            job_id: resp.job_id,
            status: resp.status,
            triggered_at: Some(to_proto_timestamp(resp.triggered_at)),
        }))
    }

    async fn get_job_execution(
        &self,
        request: Request<ProtoGetJobExecutionRequest>,
    ) -> Result<Response<ProtoGetJobExecutionResponse>, Status> {
        let inner = request.into_inner();
        let resp = self
            .inner
            .get_job_execution(GetJobExecutionRequest {
                execution_id: inner.execution_id,
            })
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoGetJobExecutionResponse {
            execution: Some(to_proto_execution(resp.execution)),
        }))
    }

    async fn list_executions(
        &self,
        request: Request<ProtoListExecutionsRequest>,
    ) -> Result<Response<ProtoListExecutionsResponse>, Status> {
        let inner = request.into_inner();
        let (page, page_size) = inner.pagination.map(|p| (p.page, p.page_size)).unwrap_or((1, 20));
        let resp = self
            .inner
            .list_executions(ListExecutionsRequest {
                job_id: inner.job_id,
                page,
                page_size,
                status: inner.status,
                from: inner.from.map(from_proto_timestamp),
                to: inner.to.map(from_proto_timestamp),
            })
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoListExecutionsResponse {
            executions: resp.executions.into_iter().map(to_proto_execution).collect(),
            pagination: Some(ProtoPaginationResult {
                total_count: resp.total_count.min(i32::MAX as u64) as i32,
                page: resp.page,
                page_size: resp.page_size,
                has_next: resp.has_next,
            }),
        }))
    }
}
