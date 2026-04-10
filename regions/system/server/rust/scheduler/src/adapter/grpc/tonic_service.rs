use std::collections::BTreeMap;
use std::sync::Arc;

use tonic::{Request, Response, Status};

use k1s0_auth::Claims;

use crate::proto::k1s0::system::common::v1::{
    PaginationResult as ProtoPaginationResult, Timestamp as ProtoTimestamp,
};
use crate::proto::k1s0::system::scheduler::v1::{
    scheduler_service_server::SchedulerService, CreateJobRequest as ProtoCreateJobRequest,
    CreateJobResponse as ProtoCreateJobResponse, DeleteJobRequest as ProtoDeleteJobRequest,
    DeleteJobResponse as ProtoDeleteJobResponse,
    GetJobExecutionRequest as ProtoGetJobExecutionRequest,
    GetJobExecutionResponse as ProtoGetJobExecutionResponse, GetJobRequest as ProtoGetJobRequest,
    GetJobResponse as ProtoGetJobResponse, Job as ProtoJob, JobExecution as ProtoJobExecution,
    ListExecutionsRequest as ProtoListExecutionsRequest,
    ListExecutionsResponse as ProtoListExecutionsResponse, ListJobsRequest as ProtoListJobsRequest,
    ListJobsResponse as ProtoListJobsResponse, PauseJobRequest as ProtoPauseJobRequest,
    PauseJobResponse as ProtoPauseJobResponse, ResumeJobRequest as ProtoResumeJobRequest,
    ResumeJobResponse as ProtoResumeJobResponse, TriggerJobRequest as ProtoTriggerJobRequest,
    TriggerJobResponse as ProtoTriggerJobResponse, UpdateJobRequest as ProtoUpdateJobRequest,
    UpdateJobResponse as ProtoUpdateJobResponse,
};

use super::scheduler_grpc::{
    CreateJobRequest, DeleteJobRequest, GetJobExecutionRequest, GetJobRequest, GrpcError, JobData,
    JobExecutionData, ListExecutionsRequest, ListJobsRequest, PauseJobRequest, ResumeJobRequest,
    SchedulerGrpcService, TriggerJobRequest, UpdateJobRequest,
};

impl From<GrpcError> for Status {
    fn from(e: GrpcError) -> Self {
        match e {
            GrpcError::NotFound(msg) => Status::not_found(msg),
            GrpcError::AlreadyExists(msg) => Status::already_exists(msg),
            GrpcError::InvalidArgument(msg) => Status::invalid_argument(msg),
            GrpcError::FailedPrecondition(msg) => Status::failed_precondition(msg),
            GrpcError::Aborted(msg) => Status::aborted(msg),
            GrpcError::Internal(msg) => Status::internal(msg),
            GrpcError::Unimplemented(msg) => Status::unimplemented(msg),
        }
    }
}

fn to_proto_timestamp(dt: chrono::DateTime<chrono::Utc>) -> ProtoTimestamp {
    ProtoTimestamp {
        seconds: dt.timestamp(),
        // LOW-008: 安全な型変換（オーバーフロー防止）
        nanos: i32::try_from(dt.timestamp_subsec_nanos()).unwrap_or(i32::MAX),
    }
}

fn from_proto_timestamp(ts: ProtoTimestamp) -> chrono::DateTime<chrono::Utc> {
    // LOW-008: 安全な型変換（オーバーフロー防止）
    chrono::DateTime::<chrono::Utc>::from_timestamp(ts.seconds, u32::try_from(ts.nanos).unwrap_or(0))
        .unwrap_or_else(chrono::Utc::now)
}

fn json_to_prost_value(value: &serde_json::Value) -> prost_types::Value {
    let kind = match value {
        serde_json::Value::Null => prost_types::value::Kind::NullValue(0),
        serde_json::Value::Bool(v) => prost_types::value::Kind::BoolValue(*v),
        serde_json::Value::Number(v) => {
            prost_types::value::Kind::NumberValue(v.as_f64().unwrap_or(0.0))
        }
        serde_json::Value::String(v) => prost_types::value::Kind::StringValue(v.clone()),
        serde_json::Value::Array(values) => {
            let values = values.iter().map(json_to_prost_value).collect();
            prost_types::value::Kind::ListValue(prost_types::ListValue { values })
        }
        serde_json::Value::Object(map) => {
            let fields = map
                .iter()
                .map(|(k, v)| (k.clone(), json_to_prost_value(v)))
                .collect();
            prost_types::value::Kind::StructValue(prost_types::Struct { fields })
        }
    };
    prost_types::Value { kind: Some(kind) }
}

fn json_to_prost_struct(value: &serde_json::Value) -> prost_types::Struct {
    if let serde_json::Value::Object(map) = value {
        let fields: BTreeMap<String, prost_types::Value> = map
            .iter()
            .map(|(k, v)| (k.clone(), json_to_prost_value(v)))
            .collect();
        prost_types::Struct { fields }
    } else {
        let mut fields = BTreeMap::new();
        fields.insert("value".to_string(), json_to_prost_value(value));
        prost_types::Struct { fields }
    }
}

fn prost_value_to_json(value: &prost_types::Value) -> serde_json::Value {
    // NullValue と None は同じ返り値のためアームを統合する
    match &value.kind {
        Some(prost_types::value::Kind::BoolValue(v)) => serde_json::Value::Bool(*v),
        Some(prost_types::value::Kind::NumberValue(v)) => serde_json::json!(*v),
        Some(prost_types::value::Kind::StringValue(v)) => serde_json::Value::String(v.clone()),
        Some(prost_types::value::Kind::ListValue(list)) => {
            serde_json::Value::Array(list.values.iter().map(prost_value_to_json).collect())
        }
        Some(prost_types::value::Kind::StructValue(v)) => prost_struct_to_json(v),
        Some(prost_types::value::Kind::NullValue(_)) | None => serde_json::Value::Null,
    }
}

fn prost_struct_to_json(value: &prost_types::Struct) -> serde_json::Value {
    let map: serde_json::Map<String, serde_json::Value> = value
        .fields
        .iter()
        .map(|(k, v)| (k.clone(), prost_value_to_json(v)))
        .collect();
    serde_json::Value::Object(map)
}

fn json_bytes_to_prost_struct(bytes: &[u8]) -> Option<prost_types::Struct> {
    if bytes.is_empty() {
        return None;
    }
    let value: serde_json::Value = serde_json::from_slice(bytes).ok()?;
    Some(json_to_prost_struct(&value))
}

#[allow(clippy::result_large_err)]
fn prost_struct_to_json_bytes(payload: Option<prost_types::Struct>) -> Result<Vec<u8>, Status> {
    let payload = payload
        .as_ref()
        .map_or_else(|| serde_json::json!({}), prost_struct_to_json);
    serde_json::to_vec(&payload)
        .map_err(|e| Status::invalid_argument(format!("invalid payload: {e}")))
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
        payload: json_bytes_to_prost_struct(&job.payload),
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

/// CRIT-005 対応: gRPC リクエストの Extensions から JWT Claims を取得してテナント ID を抽出する。
/// Claims が存在しない場合（認証なし環境）はデフォルト値 "system" を返す。
fn tenant_id_from_request<T>(request: &Request<T>) -> String {
    request
        .extensions()
        .get::<Claims>()
        .map_or_else(|| "system".to_string(), |c| c.tenant_id().to_string())
}

pub struct SchedulerServiceTonic {
    inner: Arc<SchedulerGrpcService>,
}

impl SchedulerServiceTonic {
    #[must_use]
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
        // CRIT-005 対応: JWT Claims からテナント ID を取得する
        let tenant_id = tenant_id_from_request(&request);
        let inner = request.into_inner();
        let payload = prost_struct_to_json_bytes(inner.payload)?;
        let resp = self
            .inner
            .create_job(CreateJobRequest {
                name: inner.name,
                description: inner.description,
                cron_expression: inner.cron_expression,
                timezone: inner.timezone,
                target_type: inner.target_type,
                target: inner.target,
                payload,
                tenant_id,
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
        // CRIT-005 対応: JWT Claims からテナント ID を取得する
        let tenant_id = tenant_id_from_request(&request);
        let inner = request.into_inner();
        let resp = self
            .inner
            .get_job(GetJobRequest {
                job_id: inner.job_id,
                tenant_id,
            })
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
        // CRIT-005 対応: JWT Claims からテナント ID を取得する
        let tenant_id = tenant_id_from_request(&request);
        let inner = request.into_inner();
        let (page, page_size) = inner.pagination.map_or((1, 20), |p| (p.page, p.page_size));
        let resp = self
            .inner
            .list_jobs(ListJobsRequest {
                status: inner.status,
                page,
                page_size,
                tenant_id,
            })
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoListJobsResponse {
            jobs: resp.jobs.into_iter().map(to_proto_job).collect(),
            pagination: Some(ProtoPaginationResult {
                // LOW-008: 安全な型変換（オーバーフロー防止）
                total_count: i64::try_from(resp.total_count).unwrap_or(i64::MAX),
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
        // CRIT-005 対応: JWT Claims からテナント ID を取得する
        let tenant_id = tenant_id_from_request(&request);
        let inner = request.into_inner();
        let payload = prost_struct_to_json_bytes(inner.payload)?;
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
                payload,
                tenant_id,
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
        // CRIT-005 対応: JWT Claims からテナント ID を取得する
        let tenant_id = tenant_id_from_request(&request);
        let inner = request.into_inner();
        let resp = self
            .inner
            .delete_job(DeleteJobRequest {
                job_id: inner.job_id,
                tenant_id,
            })
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
        // CRIT-005 対応: JWT Claims からテナント ID を取得する
        let tenant_id = tenant_id_from_request(&request);
        let inner = request.into_inner();
        let resp = self
            .inner
            .pause_job(PauseJobRequest {
                job_id: inner.job_id,
                tenant_id,
            })
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
        // CRIT-005 対応: JWT Claims からテナント ID を取得する
        let tenant_id = tenant_id_from_request(&request);
        let inner = request.into_inner();
        let resp = self
            .inner
            .resume_job(ResumeJobRequest {
                job_id: inner.job_id,
                tenant_id,
            })
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
        // CRIT-005 対応: JWT Claims からテナント ID を取得する
        let tenant_id = tenant_id_from_request(&request);
        let inner = request.into_inner();
        let resp = self
            .inner
            .trigger_job(TriggerJobRequest {
                job_id: inner.job_id,
                tenant_id,
            })
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
        let (page, page_size) = inner.pagination.map_or((1, 20), |p| (p.page, p.page_size));
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
            executions: resp
                .executions
                .into_iter()
                .map(to_proto_execution)
                .collect(),
            pagination: Some(ProtoPaginationResult {
                // LOW-008: 安全な型変換（オーバーフロー防止）
                total_count: i64::try_from(resp.total_count).unwrap_or(i64::MAX),
                page: resp.page,
                page_size: resp.page_size,
                has_next: resp.has_next,
            }),
        }))
    }
}
