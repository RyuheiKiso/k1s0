use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::domain::entity::workflow_definition::WorkflowDefinition;
use crate::domain::entity::workflow_instance::WorkflowInstance;
use crate::domain::entity::workflow_step::WorkflowStep;
use crate::domain::entity::workflow_task::WorkflowTask;
use crate::usecase::approve_task::{ApproveTaskError, ApproveTaskInput, ApproveTaskUseCase};
use crate::usecase::cancel_instance::{CancelInstanceError, CancelInstanceInput, CancelInstanceUseCase};
use crate::usecase::create_workflow::{CreateWorkflowError, CreateWorkflowInput, CreateWorkflowUseCase};
use crate::usecase::delete_workflow::{DeleteWorkflowError, DeleteWorkflowInput, DeleteWorkflowUseCase};
use crate::usecase::get_instance::{GetInstanceError, GetInstanceInput, GetInstanceUseCase};
use crate::usecase::get_workflow::{GetWorkflowError, GetWorkflowInput, GetWorkflowUseCase};
use crate::usecase::list_instances::{ListInstancesError, ListInstancesInput, ListInstancesUseCase};
use crate::usecase::list_tasks::{ListTasksError, ListTasksInput, ListTasksUseCase};
use crate::usecase::list_workflows::{ListWorkflowsError, ListWorkflowsInput, ListWorkflowsUseCase};
use crate::usecase::reassign_task::{ReassignTaskError, ReassignTaskInput, ReassignTaskUseCase};
use crate::usecase::reject_task::{RejectTaskError, RejectTaskInput, RejectTaskUseCase};
use crate::usecase::start_instance::{StartInstanceError, StartInstanceInput, StartInstanceUseCase};
use crate::usecase::update_workflow::{UpdateWorkflowError, UpdateWorkflowInput, UpdateWorkflowUseCase};

#[derive(Debug, Clone)]
pub struct WorkflowStepData {
    pub step_id: String,
    pub name: String,
    pub step_type: String,
    pub assignee_role: Option<String>,
    pub timeout_hours: Option<u32>,
    pub on_approve: Option<String>,
    pub on_reject: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WorkflowDefinitionData {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: u32,
    pub enabled: bool,
    pub steps: Vec<WorkflowStepData>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct WorkflowInstanceData {
    pub id: String,
    pub workflow_id: String,
    pub workflow_name: String,
    pub title: String,
    pub initiator_id: String,
    pub current_step_id: Option<String>,
    pub status: String,
    pub context_json: Vec<u8>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct WorkflowTaskData {
    pub id: String,
    pub instance_id: String,
    pub step_id: String,
    pub step_name: String,
    pub assignee_id: Option<String>,
    pub status: String,
    pub due_at: Option<DateTime<Utc>>,
    pub comment: Option<String>,
    pub actor_id: Option<String>,
    pub decided_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ListWorkflowsRequest {
    pub enabled_only: bool,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Debug, Clone)]
pub struct ListWorkflowsResponse {
    pub workflows: Vec<WorkflowDefinitionData>,
    pub total_count: u64,
    pub page: i32,
    pub page_size: i32,
    pub has_next: bool,
}

#[derive(Debug, Clone)]
pub struct CreateWorkflowRequest {
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub steps: Vec<WorkflowStepData>,
}

#[derive(Debug, Clone)]
pub struct CreateWorkflowResponse {
    pub workflow: WorkflowDefinitionData,
}

#[derive(Debug, Clone)]
pub struct GetWorkflowRequest {
    pub workflow_id: String,
}

#[derive(Debug, Clone)]
pub struct GetWorkflowResponse {
    pub workflow: WorkflowDefinitionData,
}

#[derive(Debug, Clone)]
pub struct UpdateWorkflowRequest {
    pub workflow_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub enabled: Option<bool>,
    pub steps: Option<Vec<WorkflowStepData>>,
}

#[derive(Debug, Clone)]
pub struct UpdateWorkflowResponse {
    pub workflow: WorkflowDefinitionData,
}

#[derive(Debug, Clone)]
pub struct DeleteWorkflowRequest {
    pub workflow_id: String,
}

#[derive(Debug, Clone)]
pub struct DeleteWorkflowResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct StartInstanceRequest {
    pub workflow_id: String,
    pub title: String,
    pub initiator_id: String,
    pub context_json: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct StartInstanceResponse {
    pub instance_id: String,
    pub status: String,
    pub current_step_id: Option<String>,
    pub started_at: DateTime<Utc>,
    pub workflow_id: String,
    pub workflow_name: String,
    pub title: String,
    pub initiator_id: String,
    pub context_json: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct GetInstanceRequest {
    pub instance_id: String,
}

#[derive(Debug, Clone)]
pub struct GetInstanceResponse {
    pub instance: WorkflowInstanceData,
}

#[derive(Debug, Clone)]
pub struct ListInstancesRequest {
    pub status: String,
    pub workflow_id: String,
    pub initiator_id: String,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Debug, Clone)]
pub struct ListInstancesResponse {
    pub instances: Vec<WorkflowInstanceData>,
    pub total_count: u64,
    pub page: i32,
    pub page_size: i32,
    pub has_next: bool,
}

#[derive(Debug, Clone)]
pub struct CancelInstanceRequest {
    pub instance_id: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CancelInstanceResponse {
    pub instance: WorkflowInstanceData,
}

#[derive(Debug, Clone)]
pub struct ListTasksRequest {
    pub assignee_id: String,
    pub status: String,
    pub instance_id: String,
    pub overdue_only: bool,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Debug, Clone)]
pub struct ListTasksResponse {
    pub tasks: Vec<WorkflowTaskData>,
    pub total_count: u64,
    pub page: i32,
    pub page_size: i32,
    pub has_next: bool,
}

#[derive(Debug, Clone)]
pub struct ReassignTaskRequest {
    pub task_id: String,
    pub new_assignee_id: String,
    pub reason: Option<String>,
    pub actor_id: String,
}

#[derive(Debug, Clone)]
pub struct ReassignTaskResponse {
    pub task: WorkflowTaskData,
    pub previous_assignee_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ApproveTaskRequest {
    pub task_id: String,
    pub actor_id: String,
    pub comment: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ApproveTaskResponse {
    pub task_id: String,
    pub status: String,
    pub next_task_id: Option<String>,
    pub instance_status: String,
}

#[derive(Debug, Clone)]
pub struct RejectTaskRequest {
    pub task_id: String,
    pub actor_id: String,
    pub comment: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RejectTaskResponse {
    pub task_id: String,
    pub status: String,
    pub next_task_id: Option<String>,
    pub instance_status: String,
}

#[derive(Debug, thiserror::Error)]
pub enum GrpcError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("already exists: {0}")]
    AlreadyExists(String),

    #[error("failed precondition: {0}")]
    FailedPrecondition(String),

    #[error("internal: {0}")]
    Internal(String),
}

pub struct WorkflowGrpcService {
    list_workflows_uc: Arc<ListWorkflowsUseCase>,
    create_workflow_uc: Arc<CreateWorkflowUseCase>,
    get_workflow_uc: Arc<GetWorkflowUseCase>,
    update_workflow_uc: Arc<UpdateWorkflowUseCase>,
    delete_workflow_uc: Arc<DeleteWorkflowUseCase>,
    start_instance_uc: Arc<StartInstanceUseCase>,
    get_instance_uc: Arc<GetInstanceUseCase>,
    list_instances_uc: Arc<ListInstancesUseCase>,
    cancel_instance_uc: Arc<CancelInstanceUseCase>,
    list_tasks_uc: Arc<ListTasksUseCase>,
    reassign_task_uc: Arc<ReassignTaskUseCase>,
    approve_task_uc: Arc<ApproveTaskUseCase>,
    reject_task_uc: Arc<RejectTaskUseCase>,
}

impl WorkflowGrpcService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        list_workflows_uc: Arc<ListWorkflowsUseCase>,
        create_workflow_uc: Arc<CreateWorkflowUseCase>,
        get_workflow_uc: Arc<GetWorkflowUseCase>,
        update_workflow_uc: Arc<UpdateWorkflowUseCase>,
        delete_workflow_uc: Arc<DeleteWorkflowUseCase>,
        start_instance_uc: Arc<StartInstanceUseCase>,
        get_instance_uc: Arc<GetInstanceUseCase>,
        list_instances_uc: Arc<ListInstancesUseCase>,
        cancel_instance_uc: Arc<CancelInstanceUseCase>,
        list_tasks_uc: Arc<ListTasksUseCase>,
        reassign_task_uc: Arc<ReassignTaskUseCase>,
        approve_task_uc: Arc<ApproveTaskUseCase>,
        reject_task_uc: Arc<RejectTaskUseCase>,
    ) -> Self {
        Self {
            list_workflows_uc,
            create_workflow_uc,
            get_workflow_uc,
            update_workflow_uc,
            delete_workflow_uc,
            start_instance_uc,
            get_instance_uc,
            list_instances_uc,
            cancel_instance_uc,
            list_tasks_uc,
            reassign_task_uc,
            approve_task_uc,
            reject_task_uc,
        }
    }

    pub async fn list_workflows(
        &self,
        req: ListWorkflowsRequest,
    ) -> Result<ListWorkflowsResponse, GrpcError> {
        let page = if req.page <= 0 { 1 } else { req.page as u32 };
        let page_size = if req.page_size <= 0 {
            20
        } else {
            req.page_size as u32
        };
        let out = self
            .list_workflows_uc
            .execute(&ListWorkflowsInput {
                enabled_only: req.enabled_only,
                page,
                page_size,
            })
            .await
            .map_err(|e| match e {
                ListWorkflowsError::Internal(msg) => GrpcError::Internal(msg),
            })?;
        Ok(ListWorkflowsResponse {
            workflows: out.workflows.into_iter().map(to_workflow_definition_data).collect(),
            total_count: out.total_count,
            page: out.page as i32,
            page_size: out.page_size as i32,
            has_next: out.has_next,
        })
    }

    pub async fn create_workflow(
        &self,
        req: CreateWorkflowRequest,
    ) -> Result<CreateWorkflowResponse, GrpcError> {
        let steps = req.steps.into_iter().map(to_domain_step).collect();
        let created = self
            .create_workflow_uc
            .execute(&CreateWorkflowInput {
                name: req.name,
                description: req.description,
                enabled: req.enabled,
                steps,
            })
            .await
            .map_err(|e| match e {
                CreateWorkflowError::AlreadyExists(name) => {
                    GrpcError::AlreadyExists(format!("workflow already exists: {}", name))
                }
                CreateWorkflowError::Validation(msg) => GrpcError::InvalidArgument(msg),
                CreateWorkflowError::Internal(msg) => GrpcError::Internal(msg),
            })?;
        Ok(CreateWorkflowResponse {
            workflow: to_workflow_definition_data(created),
        })
    }

    pub async fn get_workflow(
        &self,
        req: GetWorkflowRequest,
    ) -> Result<GetWorkflowResponse, GrpcError> {
        let workflow = self
            .get_workflow_uc
            .execute(&GetWorkflowInput {
                id: req.workflow_id.clone(),
            })
            .await
            .map_err(|e| match e {
                GetWorkflowError::NotFound(id) => GrpcError::NotFound(format!("workflow not found: {}", id)),
                GetWorkflowError::Internal(msg) => GrpcError::Internal(msg),
            })?;
        Ok(GetWorkflowResponse {
            workflow: to_workflow_definition_data(workflow),
        })
    }

    pub async fn update_workflow(
        &self,
        req: UpdateWorkflowRequest,
    ) -> Result<UpdateWorkflowResponse, GrpcError> {
        let updated = self
            .update_workflow_uc
            .execute(&UpdateWorkflowInput {
                id: req.workflow_id.clone(),
                name: req.name,
                description: req.description,
                enabled: req.enabled,
                steps: req
                    .steps
                    .map(|items| items.into_iter().map(to_domain_step).collect()),
            })
            .await
            .map_err(|e| match e {
                UpdateWorkflowError::NotFound(id) => GrpcError::NotFound(format!("workflow not found: {}", id)),
                UpdateWorkflowError::Internal(msg) => GrpcError::Internal(msg),
            })?;
        Ok(UpdateWorkflowResponse {
            workflow: to_workflow_definition_data(updated),
        })
    }

    pub async fn delete_workflow(
        &self,
        req: DeleteWorkflowRequest,
    ) -> Result<DeleteWorkflowResponse, GrpcError> {
        self.delete_workflow_uc
            .execute(&DeleteWorkflowInput {
                id: req.workflow_id.clone(),
            })
            .await
            .map_err(|e| match e {
                DeleteWorkflowError::NotFound(id) => GrpcError::NotFound(format!("workflow not found: {}", id)),
                DeleteWorkflowError::Internal(msg) => GrpcError::Internal(msg),
            })?;
        Ok(DeleteWorkflowResponse {
            success: true,
            message: format!("workflow {} deleted", req.workflow_id),
        })
    }

    pub async fn start_instance(
        &self,
        req: StartInstanceRequest,
    ) -> Result<StartInstanceResponse, GrpcError> {
        let context: serde_json::Value = if req.context_json.is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_slice(&req.context_json)
                .map_err(|e| GrpcError::InvalidArgument(format!("invalid context_json: {}", e)))?
        };

        let output = self
            .start_instance_uc
            .execute(&StartInstanceInput {
                workflow_id: req.workflow_id,
                title: req.title,
                initiator_id: req.initiator_id,
                context,
            })
            .await
            .map_err(|e| match e {
                StartInstanceError::WorkflowNotFound(id) => {
                    GrpcError::NotFound(format!("workflow not found: {}", id))
                }
                StartInstanceError::WorkflowDisabled(id) => {
                    GrpcError::FailedPrecondition(format!("workflow disabled: {}", id))
                }
                StartInstanceError::NoSteps(id) => {
                    GrpcError::FailedPrecondition(format!("workflow has no steps: {}", id))
                }
                StartInstanceError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok(StartInstanceResponse {
            instance_id: output.instance.id,
            status: output.instance.status,
            current_step_id: output.instance.current_step_id,
            started_at: output.instance.started_at,
            workflow_id: output.instance.workflow_id,
            workflow_name: output.instance.workflow_name,
            title: output.instance.title,
            initiator_id: output.instance.initiator_id,
            context_json: serde_json::to_vec(&output.instance.context).unwrap_or_default(),
        })
    }

    pub async fn get_instance(
        &self,
        req: GetInstanceRequest,
    ) -> Result<GetInstanceResponse, GrpcError> {
        let instance = self
            .get_instance_uc
            .execute(&GetInstanceInput { id: req.instance_id })
            .await
            .map_err(|e| match e {
                GetInstanceError::NotFound(id) => GrpcError::NotFound(format!("instance not found: {}", id)),
                GetInstanceError::Internal(msg) => GrpcError::Internal(msg),
            })?;
        Ok(GetInstanceResponse {
            instance: to_workflow_instance_data(instance),
        })
    }

    pub async fn list_instances(
        &self,
        req: ListInstancesRequest,
    ) -> Result<ListInstancesResponse, GrpcError> {
        let page = if req.page <= 0 { 1 } else { req.page as u32 };
        let page_size = if req.page_size <= 0 {
            20
        } else {
            req.page_size as u32
        };
        let out = self
            .list_instances_uc
            .execute(&ListInstancesInput {
                status: if req.status.is_empty() {
                    None
                } else {
                    Some(req.status)
                },
                workflow_id: if req.workflow_id.is_empty() {
                    None
                } else {
                    Some(req.workflow_id)
                },
                initiator_id: if req.initiator_id.is_empty() {
                    None
                } else {
                    Some(req.initiator_id)
                },
                page,
                page_size,
            })
            .await
            .map_err(|e| match e {
                ListInstancesError::Internal(msg) => GrpcError::Internal(msg),
            })?;
        Ok(ListInstancesResponse {
            instances: out
                .instances
                .into_iter()
                .map(to_workflow_instance_data)
                .collect(),
            total_count: out.total_count,
            page: out.page as i32,
            page_size: out.page_size as i32,
            has_next: out.has_next,
        })
    }

    pub async fn cancel_instance(
        &self,
        req: CancelInstanceRequest,
    ) -> Result<CancelInstanceResponse, GrpcError> {
        let instance = self
            .cancel_instance_uc
            .execute(&CancelInstanceInput {
                id: req.instance_id,
                reason: req.reason,
            })
            .await
            .map_err(|e| match e {
                CancelInstanceError::NotFound(id) => {
                    GrpcError::NotFound(format!("instance not found: {}", id))
                }
                CancelInstanceError::InvalidStatus(id, status) => GrpcError::FailedPrecondition(
                    format!("instance {} cannot be cancelled in status '{}'", id, status),
                ),
                CancelInstanceError::Internal(msg) => GrpcError::Internal(msg),
            })?;
        Ok(CancelInstanceResponse {
            instance: to_workflow_instance_data(instance),
        })
    }

    pub async fn list_tasks(&self, req: ListTasksRequest) -> Result<ListTasksResponse, GrpcError> {
        let page = if req.page <= 0 { 1 } else { req.page as u32 };
        let page_size = if req.page_size <= 0 {
            20
        } else {
            req.page_size as u32
        };
        let out = self
            .list_tasks_uc
            .execute(&ListTasksInput {
                assignee_id: if req.assignee_id.is_empty() {
                    None
                } else {
                    Some(req.assignee_id)
                },
                status: if req.status.is_empty() {
                    None
                } else {
                    Some(req.status)
                },
                instance_id: if req.instance_id.is_empty() {
                    None
                } else {
                    Some(req.instance_id)
                },
                overdue_only: req.overdue_only,
                page,
                page_size,
            })
            .await
            .map_err(|e| match e {
                ListTasksError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok(ListTasksResponse {
            tasks: out.tasks.into_iter().map(to_workflow_task_data).collect(),
            total_count: out.total_count,
            page: out.page as i32,
            page_size: out.page_size as i32,
            has_next: out.has_next,
        })
    }

    pub async fn reassign_task(
        &self,
        req: ReassignTaskRequest,
    ) -> Result<ReassignTaskResponse, GrpcError> {
        let out = self
            .reassign_task_uc
            .execute(&ReassignTaskInput {
                task_id: req.task_id,
                new_assignee_id: req.new_assignee_id,
                reason: req.reason,
                actor_id: req.actor_id,
            })
            .await
            .map_err(|e| match e {
                ReassignTaskError::TaskNotFound(id) => {
                    GrpcError::NotFound(format!("task not found: {}", id))
                }
                ReassignTaskError::InvalidStatus(status) => {
                    GrpcError::FailedPrecondition(format!("invalid task status: {}", status))
                }
                ReassignTaskError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok(ReassignTaskResponse {
            task: to_workflow_task_data(out.task),
            previous_assignee_id: out.previous_assignee_id,
        })
    }

    pub async fn approve_task(
        &self,
        req: ApproveTaskRequest,
    ) -> Result<ApproveTaskResponse, GrpcError> {
        let output = self
            .approve_task_uc
            .execute(&ApproveTaskInput {
                task_id: req.task_id,
                actor_id: req.actor_id,
                comment: req.comment,
            })
            .await
            .map_err(|e| match e {
                ApproveTaskError::TaskNotFound(id) => {
                    GrpcError::NotFound(format!("task not found: {}", id))
                }
                ApproveTaskError::InvalidStatus(status) => {
                    GrpcError::FailedPrecondition(format!("invalid task status: {}", status))
                }
                ApproveTaskError::InstanceNotFound(id) => {
                    GrpcError::NotFound(format!("instance not found: {}", id))
                }
                ApproveTaskError::DefinitionNotFound(id) => {
                    GrpcError::NotFound(format!("workflow definition not found: {}", id))
                }
                ApproveTaskError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok(ApproveTaskResponse {
            task_id: output.task.id,
            status: output.task.status,
            next_task_id: output.next_task.map(|t| t.id),
            instance_status: output.instance_status,
        })
    }

    pub async fn reject_task(
        &self,
        req: RejectTaskRequest,
    ) -> Result<RejectTaskResponse, GrpcError> {
        let output = self
            .reject_task_uc
            .execute(&RejectTaskInput {
                task_id: req.task_id,
                actor_id: req.actor_id,
                comment: req.comment,
            })
            .await
            .map_err(|e| match e {
                RejectTaskError::TaskNotFound(id) => {
                    GrpcError::NotFound(format!("task not found: {}", id))
                }
                RejectTaskError::InvalidStatus(status) => {
                    GrpcError::FailedPrecondition(format!("invalid task status: {}", status))
                }
                RejectTaskError::InstanceNotFound(id) => {
                    GrpcError::NotFound(format!("instance not found: {}", id))
                }
                RejectTaskError::DefinitionNotFound(id) => {
                    GrpcError::NotFound(format!("workflow definition not found: {}", id))
                }
                RejectTaskError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok(RejectTaskResponse {
            task_id: output.task.id,
            status: output.task.status,
            next_task_id: output.next_task.map(|t| t.id),
            instance_status: output.instance_status,
        })
    }
}

fn to_domain_step(step: WorkflowStepData) -> WorkflowStep {
    WorkflowStep::new(
        step.step_id,
        step.name,
        step.step_type,
        step.assignee_role,
        step.timeout_hours,
        step.on_approve,
        step.on_reject,
    )
}

fn to_step_data(step: WorkflowStep) -> WorkflowStepData {
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

fn to_workflow_definition_data(def: WorkflowDefinition) -> WorkflowDefinitionData {
    WorkflowDefinitionData {
        id: def.id,
        name: def.name,
        description: def.description,
        version: def.version,
        enabled: def.enabled,
        steps: def.steps.into_iter().map(to_step_data).collect(),
        created_at: def.created_at,
        updated_at: def.updated_at,
    }
}

fn to_workflow_instance_data(instance: WorkflowInstance) -> WorkflowInstanceData {
    WorkflowInstanceData {
        id: instance.id,
        workflow_id: instance.workflow_id,
        workflow_name: instance.workflow_name,
        title: instance.title,
        initiator_id: instance.initiator_id,
        current_step_id: instance.current_step_id,
        status: instance.status,
        context_json: serde_json::to_vec(&instance.context).unwrap_or_default(),
        started_at: instance.started_at,
        completed_at: instance.completed_at,
        created_at: instance.created_at,
    }
}

fn to_workflow_task_data(task: WorkflowTask) -> WorkflowTaskData {
    WorkflowTaskData {
        id: task.id,
        instance_id: task.instance_id,
        step_id: task.step_id,
        step_name: task.step_name,
        assignee_id: task.assignee_id,
        status: task.status,
        due_at: task.due_at,
        comment: task.comment,
        actor_id: task.actor_id,
        decided_at: task.decided_at,
        created_at: task.created_at,
        updated_at: task.updated_at,
    }
}
