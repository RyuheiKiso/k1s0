use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::common::v1::{
    PaginationResult as ProtoPaginationResult, Timestamp as ProtoTimestamp,
};
use crate::proto::k1s0::system::workflow::v1::{
    workflow_service_server::WorkflowService, ApproveTaskRequest as ProtoApproveTaskRequest,
    ApproveTaskResponse as ProtoApproveTaskResponse,
    CancelInstanceRequest as ProtoCancelInstanceRequest,
    CancelInstanceResponse as ProtoCancelInstanceResponse,
    CreateWorkflowRequest as ProtoCreateWorkflowRequest,
    CreateWorkflowResponse as ProtoCreateWorkflowResponse,
    DeleteWorkflowRequest as ProtoDeleteWorkflowRequest,
    DeleteWorkflowResponse as ProtoDeleteWorkflowResponse,
    GetInstanceRequest as ProtoGetInstanceRequest, GetInstanceResponse as ProtoGetInstanceResponse,
    GetWorkflowRequest as ProtoGetWorkflowRequest, GetWorkflowResponse as ProtoGetWorkflowResponse,
    ListInstancesRequest as ProtoListInstancesRequest,
    ListInstancesResponse as ProtoListInstancesResponse, ListTasksRequest as ProtoListTasksRequest,
    ListTasksResponse as ProtoListTasksResponse, ListWorkflowsRequest as ProtoListWorkflowsRequest,
    ListWorkflowsResponse as ProtoListWorkflowsResponse,
    ReassignTaskRequest as ProtoReassignTaskRequest,
    ReassignTaskResponse as ProtoReassignTaskResponse, RejectTaskRequest as ProtoRejectTaskRequest,
    RejectTaskResponse as ProtoRejectTaskResponse,
    StartInstanceRequest as ProtoStartInstanceRequest,
    StartInstanceResponse as ProtoStartInstanceResponse,
    UpdateWorkflowRequest as ProtoUpdateWorkflowRequest,
    UpdateWorkflowResponse as ProtoUpdateWorkflowResponse,
    WorkflowDefinition as ProtoWorkflowDefinition, WorkflowInstance as ProtoWorkflowInstance,
    WorkflowStep as ProtoWorkflowStep, WorkflowStepType, WorkflowSteps as ProtoWorkflowSteps,
    WorkflowTask as ProtoWorkflowTask,
};

use super::workflow_grpc::{
    ApproveTaskRequest, CancelInstanceRequest, CreateWorkflowRequest, DeleteWorkflowRequest,
    GetInstanceRequest, GetWorkflowRequest, GrpcError, ListInstancesRequest, ListTasksRequest,
    ListWorkflowsRequest, ReassignTaskRequest, RejectTaskRequest, StartInstanceRequest,
    UpdateWorkflowRequest, WorkflowDefinitionData, WorkflowGrpcService, WorkflowInstanceData,
    WorkflowStepData, WorkflowTaskData,
};

impl From<GrpcError> for Status {
    fn from(e: GrpcError) -> Self {
        match e {
            GrpcError::NotFound(msg) => Status::not_found(msg),
            GrpcError::InvalidArgument(msg) => Status::invalid_argument(msg),
            GrpcError::AlreadyExists(msg) => Status::already_exists(msg),
            GrpcError::FailedPrecondition(msg) => Status::failed_precondition(msg),
            GrpcError::Internal(msg) => Status::internal(msg),
        }
    }
}

fn to_proto_timestamp(dt: chrono::DateTime<chrono::Utc>) -> ProtoTimestamp {
    ProtoTimestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    }
}

fn to_proto_step(step: WorkflowStepData) -> ProtoWorkflowStep {
    // dual-write: 旧文字列フィールドと新 enum フィールドを同時設定して後方互換性維持
    let step_type_enum = match step.step_type.as_str() {
        "approval" => WorkflowStepType::Approval as i32,
        "automated" => WorkflowStepType::Automated as i32,
        "notification" => WorkflowStepType::Notification as i32,
        _ => WorkflowStepType::Unspecified as i32,
    };
    ProtoWorkflowStep {
        step_id: step.step_id,
        name: step.name,
        step_type: step.step_type,
        assignee_role: step.assignee_role,
        timeout_hours: step.timeout_hours,
        on_approve: step.on_approve,
        on_reject: step.on_reject,
        step_type_enum,
    }
}

fn from_proto_step(step: ProtoWorkflowStep) -> WorkflowStepData {
    WorkflowStepData {
        step_id: step.step_id,
        name: step.name,
        step_type: step.step_type,
        assignee_role: step.assignee_role,
        timeout_hours: step.timeout_hours,
        on_approve: step.on_approve,
        on_reject: step.on_reject,
    }
}

fn to_proto_definition(def: WorkflowDefinitionData) -> ProtoWorkflowDefinition {
    ProtoWorkflowDefinition {
        id: def.id,
        name: def.name,
        description: def.description,
        version: def.version,
        enabled: def.enabled,
        steps: def.steps.into_iter().map(to_proto_step).collect(),
        created_at: Some(to_proto_timestamp(def.created_at)),
        updated_at: Some(to_proto_timestamp(def.updated_at)),
    }
}

fn to_proto_instance(instance: WorkflowInstanceData) -> ProtoWorkflowInstance {
    ProtoWorkflowInstance {
        id: instance.id,
        workflow_id: instance.workflow_id,
        workflow_name: instance.workflow_name,
        title: instance.title,
        initiator_id: instance.initiator_id,
        current_step_id: instance.current_step_id,
        status: instance.status,
        context_json: instance.context_json,
        started_at: Some(to_proto_timestamp(instance.started_at)),
        completed_at: instance.completed_at.map(to_proto_timestamp),
        created_at: Some(to_proto_timestamp(instance.created_at)),
    }
}

fn to_proto_task(task: WorkflowTaskData) -> ProtoWorkflowTask {
    ProtoWorkflowTask {
        id: task.id,
        instance_id: task.instance_id,
        step_id: task.step_id,
        step_name: task.step_name,
        assignee_id: task.assignee_id,
        status: task.status,
        due_at: task.due_at.map(to_proto_timestamp),
        comment: task.comment,
        actor_id: task.actor_id,
        decided_at: task.decided_at.map(to_proto_timestamp),
        created_at: Some(to_proto_timestamp(task.created_at)),
        updated_at: Some(to_proto_timestamp(task.updated_at)),
    }
}

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
    async fn list_workflows(
        &self,
        request: Request<ProtoListWorkflowsRequest>,
    ) -> Result<Response<ProtoListWorkflowsResponse>, Status> {
        let inner = request.into_inner();
        let (page, page_size) = inner
            .pagination
            .map(|p| (p.page, p.page_size))
            .unwrap_or((1, 20));
        let resp = self
            .inner
            .list_workflows(ListWorkflowsRequest {
                enabled_only: inner.enabled_only,
                page,
                page_size,
            })
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoListWorkflowsResponse {
            workflows: resp
                .workflows
                .into_iter()
                .map(to_proto_definition)
                .collect(),
            pagination: Some(ProtoPaginationResult {
                total_count: resp.total_count as i64,
                page: resp.page,
                page_size: resp.page_size,
                has_next: resp.has_next,
            }),
        }))
    }

    async fn create_workflow(
        &self,
        request: Request<ProtoCreateWorkflowRequest>,
    ) -> Result<Response<ProtoCreateWorkflowResponse>, Status> {
        let inner = request.into_inner();
        let resp = self
            .inner
            .create_workflow(CreateWorkflowRequest {
                name: inner.name,
                description: inner.description,
                enabled: inner.enabled,
                steps: inner.steps.into_iter().map(from_proto_step).collect(),
            })
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoCreateWorkflowResponse {
            workflow: Some(to_proto_definition(resp.workflow)),
        }))
    }

    async fn get_workflow(
        &self,
        request: Request<ProtoGetWorkflowRequest>,
    ) -> Result<Response<ProtoGetWorkflowResponse>, Status> {
        let inner = request.into_inner();
        let resp = self
            .inner
            .get_workflow(GetWorkflowRequest {
                workflow_id: inner.workflow_id,
            })
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoGetWorkflowResponse {
            workflow: Some(to_proto_definition(resp.workflow)),
        }))
    }

    async fn update_workflow(
        &self,
        request: Request<ProtoUpdateWorkflowRequest>,
    ) -> Result<Response<ProtoUpdateWorkflowResponse>, Status> {
        let inner = request.into_inner();
        let resp = self
            .inner
            .update_workflow(UpdateWorkflowRequest {
                workflow_id: inner.workflow_id,
                name: inner.name,
                description: inner.description,
                enabled: inner.enabled,
                steps: inner.steps.map(|s: ProtoWorkflowSteps| {
                    s.items.into_iter().map(from_proto_step).collect()
                }),
            })
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoUpdateWorkflowResponse {
            workflow: Some(to_proto_definition(resp.workflow)),
        }))
    }

    async fn delete_workflow(
        &self,
        request: Request<ProtoDeleteWorkflowRequest>,
    ) -> Result<Response<ProtoDeleteWorkflowResponse>, Status> {
        let inner = request.into_inner();
        let resp = self
            .inner
            .delete_workflow(DeleteWorkflowRequest {
                workflow_id: inner.workflow_id,
            })
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoDeleteWorkflowResponse {
            success: resp.success,
            message: resp.message,
        }))
    }

    async fn start_instance(
        &self,
        request: Request<ProtoStartInstanceRequest>,
    ) -> Result<Response<ProtoStartInstanceResponse>, Status> {
        let inner = request.into_inner();
        let resp = self
            .inner
            .start_instance(StartInstanceRequest {
                workflow_id: inner.workflow_id,
                title: inner.title,
                initiator_id: inner.initiator_id,
                context_json: inner.context_json,
            })
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoStartInstanceResponse {
            instance_id: resp.instance_id,
            status: resp.status,
            current_step_id: resp.current_step_id,
            started_at: Some(to_proto_timestamp(resp.started_at)),
            workflow_id: resp.workflow_id,
            workflow_name: resp.workflow_name,
            title: resp.title,
            initiator_id: resp.initiator_id,
            context_json: resp.context_json,
        }))
    }

    async fn get_instance(
        &self,
        request: Request<ProtoGetInstanceRequest>,
    ) -> Result<Response<ProtoGetInstanceResponse>, Status> {
        let inner = request.into_inner();
        let resp = self
            .inner
            .get_instance(GetInstanceRequest {
                instance_id: inner.instance_id,
            })
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoGetInstanceResponse {
            instance: Some(to_proto_instance(resp.instance)),
        }))
    }

    async fn list_instances(
        &self,
        request: Request<ProtoListInstancesRequest>,
    ) -> Result<Response<ProtoListInstancesResponse>, Status> {
        let inner = request.into_inner();
        let (page, page_size) = inner
            .pagination
            .map(|p| (p.page, p.page_size))
            .unwrap_or((1, 20));
        let resp = self
            .inner
            .list_instances(ListInstancesRequest {
                status: inner.status,
                workflow_id: inner.workflow_id,
                initiator_id: inner.initiator_id,
                page,
                page_size,
            })
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoListInstancesResponse {
            instances: resp.instances.into_iter().map(to_proto_instance).collect(),
            pagination: Some(ProtoPaginationResult {
                total_count: resp.total_count as i64,
                page: resp.page,
                page_size: resp.page_size,
                has_next: resp.has_next,
            }),
        }))
    }

    async fn cancel_instance(
        &self,
        request: Request<ProtoCancelInstanceRequest>,
    ) -> Result<Response<ProtoCancelInstanceResponse>, Status> {
        let inner = request.into_inner();
        let resp = self
            .inner
            .cancel_instance(CancelInstanceRequest {
                instance_id: inner.instance_id,
                reason: inner.reason,
            })
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoCancelInstanceResponse {
            instance: Some(to_proto_instance(resp.instance)),
        }))
    }

    async fn list_tasks(
        &self,
        request: Request<ProtoListTasksRequest>,
    ) -> Result<Response<ProtoListTasksResponse>, Status> {
        let inner = request.into_inner();
        let (page, page_size) = inner
            .pagination
            .map(|p| (p.page, p.page_size))
            .unwrap_or((1, 20));
        let resp = self
            .inner
            .list_tasks(ListTasksRequest {
                assignee_id: inner.assignee_id,
                status: inner.status,
                instance_id: inner.instance_id,
                overdue_only: inner.overdue_only,
                page,
                page_size,
            })
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoListTasksResponse {
            tasks: resp.tasks.into_iter().map(to_proto_task).collect(),
            pagination: Some(ProtoPaginationResult {
                total_count: resp.total_count as i64,
                page: resp.page,
                page_size: resp.page_size,
                has_next: resp.has_next,
            }),
        }))
    }

    async fn reassign_task(
        &self,
        request: Request<ProtoReassignTaskRequest>,
    ) -> Result<Response<ProtoReassignTaskResponse>, Status> {
        let inner = request.into_inner();
        let resp = self
            .inner
            .reassign_task(ReassignTaskRequest {
                task_id: inner.task_id,
                new_assignee_id: inner.new_assignee_id,
                reason: inner.reason,
                actor_id: inner.actor_id,
            })
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoReassignTaskResponse {
            task: Some(to_proto_task(resp.task)),
            previous_assignee_id: resp.previous_assignee_id,
        }))
    }

    async fn approve_task(
        &self,
        request: Request<ProtoApproveTaskRequest>,
    ) -> Result<Response<ProtoApproveTaskResponse>, Status> {
        let inner = request.into_inner();
        let resp = self
            .inner
            .approve_task(ApproveTaskRequest {
                task_id: inner.task_id,
                actor_id: inner.actor_id,
                comment: inner.comment,
            })
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
        let resp = self
            .inner
            .reject_task(RejectTaskRequest {
                task_id: inner.task_id,
                actor_id: inner.actor_id,
                comment: inner.comment,
            })
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
