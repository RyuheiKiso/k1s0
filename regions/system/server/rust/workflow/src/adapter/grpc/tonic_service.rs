//! tonic gRPC サービス実装。
//!
//! proto 生成コード (`src/proto/`) の WorkflowService トレイトを実装する。
//! 各メソッドで proto 型 <-> 手動型の変換を行い、既存の WorkflowGrpcService に委譲する。

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::workflow::v1::{
    workflow_service_server::WorkflowService,
    ApproveTaskRequest as ProtoApproveTaskRequest,
    ApproveTaskResponse as ProtoApproveTaskResponse,
    GetInstanceRequest as ProtoGetInstanceRequest,
    GetInstanceResponse as ProtoGetInstanceResponse,
    RejectTaskRequest as ProtoRejectTaskRequest,
    RejectTaskResponse as ProtoRejectTaskResponse,
    StartInstanceRequest as ProtoStartInstanceRequest,
    StartInstanceResponse as ProtoStartInstanceResponse,
    WorkflowInstance as ProtoWorkflowInstance,
};

use super::workflow_grpc::{
    ApproveTaskRequest, GetInstanceRequest, GrpcError, RejectTaskRequest, StartInstanceRequest,
    WorkflowGrpcService,
};

// --- GrpcError -> tonic::Status 変換 ---

impl From<GrpcError> for Status {
    fn from(e: GrpcError) -> Self {
        match e {
            GrpcError::NotFound(msg) => Status::not_found(msg),
            GrpcError::InvalidArgument(msg) => Status::invalid_argument(msg),
            GrpcError::Internal(msg) => Status::internal(msg),
        }
    }
}

// --- WorkflowService tonic ラッパー ---

pub struct WorkflowServiceTonic {
    inner: Arc<WorkflowGrpcService>,
}

impl WorkflowServiceTonic {
    pub fn new(inner: Arc<WorkflowGrpcService>) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl WorkflowService for WorkflowServiceTonic {
    async fn start_instance(
        &self,
        request: Request<ProtoStartInstanceRequest>,
    ) -> Result<Response<ProtoStartInstanceResponse>, Status> {
        let inner = request.into_inner();
        let req = StartInstanceRequest {
            workflow_id: inner.workflow_id,
            title: inner.title,
            initiator_id: inner.initiator_id,
            context_json: inner.context_json,
        };
        let resp = self
            .inner
            .start_instance(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoStartInstanceResponse {
            instance_id: resp.instance_id,
            status: resp.status,
            current_step_id: resp.current_step_id,
            started_at: None,
        }))
    }

    async fn get_instance(
        &self,
        request: Request<ProtoGetInstanceRequest>,
    ) -> Result<Response<ProtoGetInstanceResponse>, Status> {
        let req = GetInstanceRequest {
            instance_id: request.into_inner().instance_id,
        };
        let resp = self
            .inner
            .get_instance(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoGetInstanceResponse {
            instance: Some(ProtoWorkflowInstance {
                id: resp.id,
                workflow_id: resp.workflow_id,
                workflow_name: resp.workflow_name,
                title: resp.title,
                initiator_id: resp.initiator_id,
                current_step_id: resp.current_step_id,
                status: resp.status,
                context_json: resp.context_json,
                started_at: None,
                completed_at: None,
            }),
        }))
    }

    async fn approve_task(
        &self,
        request: Request<ProtoApproveTaskRequest>,
    ) -> Result<Response<ProtoApproveTaskResponse>, Status> {
        let inner = request.into_inner();
        let req = ApproveTaskRequest {
            task_id: inner.task_id,
            actor_id: inner.actor_id,
            comment: inner.comment,
        };
        let resp = self
            .inner
            .approve_task(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoApproveTaskResponse {
            task_id: resp.task_id,
            status: resp.status,
            next_task_id: resp.next_task_id,
            instance_status: resp.instance_status,
        }))
    }

    async fn reject_task(
        &self,
        request: Request<ProtoRejectTaskRequest>,
    ) -> Result<Response<ProtoRejectTaskResponse>, Status> {
        let inner = request.into_inner();
        let req = RejectTaskRequest {
            task_id: inner.task_id,
            actor_id: inner.actor_id,
            comment: inner.comment,
        };
        let resp = self
            .inner
            .reject_task(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoRejectTaskResponse {
            task_id: resp.task_id,
            status: resp.status,
            next_task_id: resp.next_task_id,
            instance_status: resp.instance_status,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grpc_error_not_found_to_status() {
        let err = GrpcError::NotFound("workflow not found".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::NotFound);
        assert!(status.message().contains("workflow not found"));
    }

    #[test]
    fn test_grpc_error_invalid_argument_to_status() {
        let err = GrpcError::InvalidArgument("invalid input".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::InvalidArgument);
    }

    #[test]
    fn test_grpc_error_internal_to_status() {
        let err = GrpcError::Internal("internal error".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::Internal);
    }
}
