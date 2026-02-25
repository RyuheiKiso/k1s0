//! tonic gRPC サービス実装。
//!
//! proto 生成コード (`src/proto/`) の SchedulerService トレイトを実装する。
//! 各メソッドで proto 型 <-> 手動型の変換を行い、既存の SchedulerGrpcService に委譲する。

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::scheduler::v1::{
    scheduler_service_server::SchedulerService, GetJobExecutionRequest as ProtoGetJobExecutionRequest,
    GetJobExecutionResponse as ProtoGetJobExecutionResponse, JobExecution as ProtoJobExecution,
    TriggerJobRequest as ProtoTriggerJobRequest, TriggerJobResponse as ProtoTriggerJobResponse,
};

use super::scheduler_grpc::{
    GetJobExecutionRequest, GrpcError, SchedulerGrpcService, TriggerJobRequest,
};

// --- GrpcError -> tonic::Status 変換 ---

impl From<GrpcError> for Status {
    fn from(e: GrpcError) -> Self {
        match e {
            GrpcError::NotFound(msg) => Status::not_found(msg),
            GrpcError::InvalidArgument(msg) => Status::invalid_argument(msg),
            GrpcError::Internal(msg) => Status::internal(msg),
            GrpcError::Unimplemented(msg) => Status::unimplemented(msg),
        }
    }
}

// --- SchedulerService tonic ラッパー ---

/// SchedulerServiceTonic は tonic の SchedulerService として SchedulerGrpcService をラップする。
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
    async fn trigger_job(
        &self,
        request: Request<ProtoTriggerJobRequest>,
    ) -> Result<Response<ProtoTriggerJobResponse>, Status> {
        let inner = request.into_inner();
        let req = TriggerJobRequest {
            job_id: inner.job_id,
        };
        let resp = self
            .inner
            .trigger_job(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoTriggerJobResponse {
            execution_id: resp.execution_id,
            job_id: resp.job_id,
            status: resp.status,
            triggered_at: None,
        }))
    }

    async fn get_job_execution(
        &self,
        request: Request<ProtoGetJobExecutionRequest>,
    ) -> Result<Response<ProtoGetJobExecutionResponse>, Status> {
        let inner = request.into_inner();
        let req = GetJobExecutionRequest {
            execution_id: inner.execution_id,
        };
        let resp = self
            .inner
            .get_job_execution(req)
            .await
            .map_err(Into::<Status>::into)?;

        let proto_execution = ProtoJobExecution {
            id: resp.id,
            job_id: resp.job_id,
            status: resp.status,
            triggered_by: resp.triggered_by,
            started_at: None,
            finished_at: None,
            duration_ms: None,
            error_message: None,
        };

        Ok(Response::new(ProtoGetJobExecutionResponse {
            execution: Some(proto_execution),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::scheduler_job::SchedulerJob;
    use crate::domain::repository::scheduler_job_repository::MockSchedulerJobRepository;
    use crate::usecase::trigger_job::TriggerJobUseCase;

    fn make_tonic_service(mock: MockSchedulerJobRepository) -> SchedulerServiceTonic {
        let repo = Arc::new(mock);
        let grpc_svc = Arc::new(SchedulerGrpcService::new(
            Arc::new(TriggerJobUseCase::new(repo)),
        ));
        SchedulerServiceTonic::new(grpc_svc)
    }

    #[test]
    fn test_grpc_error_not_found_to_status() {
        let err = GrpcError::NotFound("job not found".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::NotFound);
        assert!(status.message().contains("job not found"));
    }

    #[test]
    fn test_grpc_error_invalid_argument_to_status() {
        let err = GrpcError::InvalidArgument("invalid job_id".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::InvalidArgument);
        assert!(status.message().contains("invalid job_id"));
    }

    #[test]
    fn test_grpc_error_internal_to_status() {
        let err = GrpcError::Internal("database error".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::Internal);
        assert!(status.message().contains("database error"));
    }

    #[test]
    fn test_grpc_error_unimplemented_to_status() {
        let err = GrpcError::Unimplemented("not yet implemented".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::Unimplemented);
        assert!(status.message().contains("not yet implemented"));
    }

    #[tokio::test]
    async fn test_scheduler_service_tonic_trigger_job() {
        let mut mock = MockSchedulerJobRepository::new();
        let job = SchedulerJob::new(
            "test-job".to_string(),
            "* * * * *".to_string(),
            serde_json::json!({}),
        );
        let job_id = job.id;
        let return_job = job.clone();

        mock.expect_find_by_id()
            .withf(move |id| *id == job_id)
            .returning(move |_| Ok(Some(return_job.clone())));
        mock.expect_update().returning(|_| Ok(()));

        let tonic_svc = make_tonic_service(mock);
        let req = Request::new(ProtoTriggerJobRequest {
            job_id: job_id.to_string(),
        });
        let resp = tonic_svc.trigger_job(req).await.unwrap();
        let inner = resp.into_inner();

        assert_eq!(inner.job_id, job_id.to_string());
        assert_eq!(inner.status, "running");
    }

    #[tokio::test]
    async fn test_scheduler_service_tonic_get_job_execution_unimplemented() {
        let mock = MockSchedulerJobRepository::new();
        let tonic_svc = make_tonic_service(mock);

        let req = Request::new(ProtoGetJobExecutionRequest {
            execution_id: "exec-001".to_string(),
        });
        let result = tonic_svc.get_job_execution(req).await;

        assert!(result.is_err());
        let status = result.unwrap_err();
        assert_eq!(status.code(), tonic::Code::Unimplemented);
    }
}
