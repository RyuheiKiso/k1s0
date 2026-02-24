use std::sync::Arc;

use crate::usecase::approve_task::{ApproveTaskError, ApproveTaskInput, ApproveTaskUseCase};
use crate::usecase::get_instance::{GetInstanceError, GetInstanceInput, GetInstanceUseCase};
use crate::usecase::reject_task::{RejectTaskError, RejectTaskInput, RejectTaskUseCase};
use crate::usecase::start_instance::{StartInstanceError, StartInstanceInput, StartInstanceUseCase};

// --- gRPC Request/Response Types ---

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
    pub current_step_id: String,
}

#[derive(Debug, Clone)]
pub struct GetInstanceRequest {
    pub instance_id: String,
}

#[derive(Debug, Clone)]
pub struct GetInstanceResponse {
    pub id: String,
    pub workflow_id: String,
    pub workflow_name: String,
    pub title: String,
    pub initiator_id: String,
    pub current_step_id: String,
    pub status: String,
    pub context_json: Vec<u8>,
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

// --- gRPC Error ---

#[derive(Debug, thiserror::Error)]
pub enum GrpcError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("internal: {0}")]
    Internal(String),
}

// --- WorkflowGrpcService ---

pub struct WorkflowGrpcService {
    start_instance_uc: Arc<StartInstanceUseCase>,
    get_instance_uc: Arc<GetInstanceUseCase>,
    approve_task_uc: Arc<ApproveTaskUseCase>,
    reject_task_uc: Arc<RejectTaskUseCase>,
}

impl WorkflowGrpcService {
    pub fn new(
        start_instance_uc: Arc<StartInstanceUseCase>,
        get_instance_uc: Arc<GetInstanceUseCase>,
        approve_task_uc: Arc<ApproveTaskUseCase>,
        reject_task_uc: Arc<RejectTaskUseCase>,
    ) -> Self {
        Self {
            start_instance_uc,
            get_instance_uc,
            approve_task_uc,
            reject_task_uc,
        }
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

        let input = StartInstanceInput {
            workflow_id: req.workflow_id,
            title: req.title,
            initiator_id: req.initiator_id,
            context,
        };

        match self.start_instance_uc.execute(&input).await {
            Ok(output) => Ok(StartInstanceResponse {
                instance_id: output.instance.id.clone(),
                status: output.instance.status.clone(),
                current_step_id: output
                    .instance
                    .current_step_id
                    .unwrap_or_default(),
            }),
            Err(StartInstanceError::WorkflowNotFound(id)) => {
                Err(GrpcError::NotFound(format!("workflow not found: {}", id)))
            }
            Err(StartInstanceError::WorkflowDisabled(id)) => {
                Err(GrpcError::InvalidArgument(format!("workflow disabled: {}", id)))
            }
            Err(StartInstanceError::NoSteps(id)) => {
                Err(GrpcError::InvalidArgument(format!("workflow has no steps: {}", id)))
            }
            Err(StartInstanceError::Internal(e)) => Err(GrpcError::Internal(e)),
        }
    }

    pub async fn get_instance(
        &self,
        req: GetInstanceRequest,
    ) -> Result<GetInstanceResponse, GrpcError> {
        let input = GetInstanceInput {
            id: req.instance_id,
        };

        match self.get_instance_uc.execute(&input).await {
            Ok(instance) => {
                let context_json = serde_json::to_vec(&instance.context)
                    .unwrap_or_default();
                Ok(GetInstanceResponse {
                    id: instance.id,
                    workflow_id: instance.workflow_id,
                    workflow_name: instance.workflow_name,
                    title: instance.title,
                    initiator_id: instance.initiator_id,
                    current_step_id: instance.current_step_id.unwrap_or_default(),
                    status: instance.status,
                    context_json,
                })
            }
            Err(GetInstanceError::NotFound(id)) => {
                Err(GrpcError::NotFound(format!("instance not found: {}", id)))
            }
            Err(GetInstanceError::Internal(e)) => Err(GrpcError::Internal(e)),
        }
    }

    pub async fn approve_task(
        &self,
        req: ApproveTaskRequest,
    ) -> Result<ApproveTaskResponse, GrpcError> {
        let input = ApproveTaskInput {
            task_id: req.task_id,
            actor_id: req.actor_id,
            comment: req.comment,
        };

        match self.approve_task_uc.execute(&input).await {
            Ok(output) => Ok(ApproveTaskResponse {
                task_id: output.task.id.clone(),
                status: output.task.status.clone(),
                next_task_id: output.next_task.map(|t| t.id),
                instance_status: output.instance_status,
            }),
            Err(ApproveTaskError::TaskNotFound(id)) => {
                Err(GrpcError::NotFound(format!("task not found: {}", id)))
            }
            Err(ApproveTaskError::InvalidStatus(s)) => {
                Err(GrpcError::InvalidArgument(format!("invalid task status: {}", s)))
            }
            Err(ApproveTaskError::InstanceNotFound(id)) => {
                Err(GrpcError::NotFound(format!("instance not found: {}", id)))
            }
            Err(ApproveTaskError::DefinitionNotFound(id)) => {
                Err(GrpcError::NotFound(format!("workflow definition not found: {}", id)))
            }
            Err(ApproveTaskError::Internal(e)) => Err(GrpcError::Internal(e)),
        }
    }

    pub async fn reject_task(
        &self,
        req: RejectTaskRequest,
    ) -> Result<RejectTaskResponse, GrpcError> {
        let input = RejectTaskInput {
            task_id: req.task_id,
            actor_id: req.actor_id,
            comment: req.comment,
        };

        match self.reject_task_uc.execute(&input).await {
            Ok(output) => Ok(RejectTaskResponse {
                task_id: output.task.id.clone(),
                status: output.task.status.clone(),
                next_task_id: output.next_task.map(|t| t.id),
                instance_status: output.instance_status,
            }),
            Err(RejectTaskError::TaskNotFound(id)) => {
                Err(GrpcError::NotFound(format!("task not found: {}", id)))
            }
            Err(RejectTaskError::InvalidStatus(s)) => {
                Err(GrpcError::InvalidArgument(format!("invalid task status: {}", s)))
            }
            Err(RejectTaskError::InstanceNotFound(id)) => {
                Err(GrpcError::NotFound(format!("instance not found: {}", id)))
            }
            Err(RejectTaskError::DefinitionNotFound(id)) => {
                Err(GrpcError::NotFound(format!("workflow definition not found: {}", id)))
            }
            Err(RejectTaskError::Internal(e)) => Err(GrpcError::Internal(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::workflow_definition::WorkflowDefinition;
    use crate::domain::entity::workflow_instance::WorkflowInstance;
    use crate::domain::entity::workflow_step::WorkflowStep;
    use crate::domain::entity::workflow_task::WorkflowTask;
    use crate::domain::repository::workflow_definition_repository::MockWorkflowDefinitionRepository;
    use crate::domain::repository::workflow_instance_repository::MockWorkflowInstanceRepository;
    use crate::domain::repository::workflow_task_repository::MockWorkflowTaskRepository;

    fn sample_definition() -> WorkflowDefinition {
        WorkflowDefinition::new(
            "wf_001".to_string(),
            "purchase-approval".to_string(),
            "".to_string(),
            true,
            vec![WorkflowStep::new(
                "step-1".to_string(),
                "Approval".to_string(),
                "human_task".to_string(),
                Some("manager".to_string()),
                Some(48),
                Some("end".to_string()),
                Some("end".to_string()),
            )],
        )
    }

    fn running_instance() -> WorkflowInstance {
        WorkflowInstance::new(
            "inst_001".to_string(),
            "wf_001".to_string(),
            "purchase-approval".to_string(),
            "PC Purchase".to_string(),
            "user-001".to_string(),
            Some("step-1".to_string()),
            serde_json::json!({}),
        )
    }

    fn pending_task() -> WorkflowTask {
        WorkflowTask::new(
            "task_001".to_string(),
            "inst_001".to_string(),
            "step-1".to_string(),
            "Approval".to_string(),
            Some("user-002".to_string()),
            None,
        )
    }

    fn make_service(
        def_mock: MockWorkflowDefinitionRepository,
        inst_mock: MockWorkflowInstanceRepository,
        task_mock: MockWorkflowTaskRepository,
    ) -> WorkflowGrpcService {
        let def_repo = Arc::new(def_mock);
        let inst_repo = Arc::new(inst_mock);
        let task_repo = Arc::new(task_mock);
        WorkflowGrpcService::new(
            Arc::new(StartInstanceUseCase::new(
                def_repo.clone(),
                inst_repo.clone(),
                task_repo.clone(),
            )),
            Arc::new(GetInstanceUseCase::new(inst_repo.clone())),
            Arc::new(ApproveTaskUseCase::new(
                task_repo.clone(),
                inst_repo.clone(),
                def_repo.clone(),
            )),
            Arc::new(RejectTaskUseCase::new(
                task_repo,
                inst_repo,
                def_repo,
            )),
        )
    }

    #[tokio::test]
    async fn test_start_instance_success() {
        let mut def_mock = MockWorkflowDefinitionRepository::new();
        let mut inst_mock = MockWorkflowInstanceRepository::new();
        let mut task_mock = MockWorkflowTaskRepository::new();

        def_mock
            .expect_find_by_id()
            .returning(|_| Ok(Some(sample_definition())));
        inst_mock.expect_create().returning(|_| Ok(()));
        task_mock.expect_create().returning(|_| Ok(()));

        let svc = make_service(def_mock, inst_mock, task_mock);
        let req = StartInstanceRequest {
            workflow_id: "wf_001".to_string(),
            title: "PC Purchase".to_string(),
            initiator_id: "user-001".to_string(),
            context_json: b"{}".to_vec(),
        };
        let resp = svc.start_instance(req).await.unwrap();

        assert_eq!(resp.status, "running");
        assert!(!resp.instance_id.is_empty());
    }

    #[tokio::test]
    async fn test_start_instance_workflow_not_found() {
        let mut def_mock = MockWorkflowDefinitionRepository::new();
        let inst_mock = MockWorkflowInstanceRepository::new();
        let task_mock = MockWorkflowTaskRepository::new();

        def_mock.expect_find_by_id().returning(|_| Ok(None));

        let svc = make_service(def_mock, inst_mock, task_mock);
        let req = StartInstanceRequest {
            workflow_id: "wf_missing".to_string(),
            title: "test".to_string(),
            initiator_id: "user-001".to_string(),
            context_json: vec![],
        };
        let result = svc.start_instance(req).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            GrpcError::NotFound(msg) => assert!(msg.contains("not found")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_instance_success() {
        let def_mock = MockWorkflowDefinitionRepository::new();
        let mut inst_mock = MockWorkflowInstanceRepository::new();
        let task_mock = MockWorkflowTaskRepository::new();

        inst_mock
            .expect_find_by_id()
            .returning(|_| Ok(Some(running_instance())));

        let svc = make_service(def_mock, inst_mock, task_mock);
        let req = GetInstanceRequest {
            instance_id: "inst_001".to_string(),
        };
        let resp = svc.get_instance(req).await.unwrap();

        assert_eq!(resp.id, "inst_001");
        assert_eq!(resp.status, "running");
    }

    #[tokio::test]
    async fn test_approve_task_success() {
        let mut def_mock = MockWorkflowDefinitionRepository::new();
        let mut inst_mock = MockWorkflowInstanceRepository::new();
        let mut task_mock = MockWorkflowTaskRepository::new();

        task_mock
            .expect_find_by_id()
            .returning(|_| Ok(Some(pending_task())));
        task_mock.expect_update().returning(|_| Ok(()));
        inst_mock
            .expect_find_by_id()
            .returning(|_| Ok(Some(running_instance())));
        inst_mock.expect_update().returning(|_| Ok(()));
        def_mock
            .expect_find_by_id()
            .returning(|_| Ok(Some(sample_definition())));

        let svc = make_service(def_mock, inst_mock, task_mock);
        let req = ApproveTaskRequest {
            task_id: "task_001".to_string(),
            actor_id: "user-002".to_string(),
            comment: Some("Approved".to_string()),
        };
        let resp = svc.approve_task(req).await.unwrap();

        assert_eq!(resp.task_id, "task_001");
        assert_eq!(resp.status, "approved");
    }

    #[tokio::test]
    async fn test_reject_task_success() {
        let mut def_mock = MockWorkflowDefinitionRepository::new();
        let mut inst_mock = MockWorkflowInstanceRepository::new();
        let mut task_mock = MockWorkflowTaskRepository::new();

        task_mock
            .expect_find_by_id()
            .returning(|_| Ok(Some(pending_task())));
        task_mock.expect_update().returning(|_| Ok(()));
        inst_mock
            .expect_find_by_id()
            .returning(|_| Ok(Some(running_instance())));
        inst_mock.expect_update().returning(|_| Ok(()));
        def_mock
            .expect_find_by_id()
            .returning(|_| Ok(Some(sample_definition())));

        let svc = make_service(def_mock, inst_mock, task_mock);
        let req = RejectTaskRequest {
            task_id: "task_001".to_string(),
            actor_id: "user-002".to_string(),
            comment: Some("Rejected".to_string()),
        };
        let resp = svc.reject_task(req).await.unwrap();

        assert_eq!(resp.task_id, "task_001");
        assert_eq!(resp.status, "rejected");
    }
}
