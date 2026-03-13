use std::sync::Arc;
use tracing::instrument;
use crate::adapter::graphql_handler::WorkflowStepInput;
use crate::domain::model::{
    ApproveTaskPayload, CancelInstancePayload, CreateWorkflowPayload, DeleteWorkflowPayload,
    ReassignTaskPayload, RejectTaskPayload, StartInstancePayload, UpdateWorkflowPayload,
    UserError,
};
use crate::infrastructure::grpc::WorkflowGrpcClient;

pub struct WorkflowMutationResolver {
    client: Arc<WorkflowGrpcClient>,
}

impl WorkflowMutationResolver {
    pub fn new(client: Arc<WorkflowGrpcClient>) -> Self {
        Self { client }
    }

    #[instrument(skip(self, steps), fields(service = "graphql-gateway"))]
    pub async fn create_workflow(
        &self,
        name: &str,
        description: &str,
        enabled: bool,
        steps: &[WorkflowStepInput],
    ) -> CreateWorkflowPayload {
        match self.client.create_workflow(name, description, enabled, steps).await {
            Ok(workflow) => CreateWorkflowPayload { workflow: Some(workflow), errors: vec![] },
            Err(e) => CreateWorkflowPayload { workflow: None, errors: vec![UserError { field: None, message: e.to_string() }] },
        }
    }

    #[instrument(skip(self, steps), fields(service = "graphql-gateway"))]
    pub async fn update_workflow(
        &self,
        workflow_id: &str,
        name: Option<&str>,
        description: Option<&str>,
        enabled: Option<bool>,
        steps: Option<&[WorkflowStepInput]>,
    ) -> UpdateWorkflowPayload {
        match self.client.update_workflow(workflow_id, name, description, enabled, steps).await {
            Ok(workflow) => UpdateWorkflowPayload { workflow: Some(workflow), errors: vec![] },
            Err(e) => UpdateWorkflowPayload { workflow: None, errors: vec![UserError { field: None, message: e.to_string() }] },
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn delete_workflow(&self, workflow_id: &str) -> DeleteWorkflowPayload {
        match self.client.delete_workflow(workflow_id).await {
            Ok(success) => DeleteWorkflowPayload { success, errors: vec![] },
            Err(e) => DeleteWorkflowPayload { success: false, errors: vec![UserError { field: None, message: e.to_string() }] },
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn start_instance(
        &self,
        workflow_id: &str,
        title: &str,
        initiator_id: &str,
        context_json: Option<&str>,
    ) -> StartInstancePayload {
        match self.client.start_instance(workflow_id, title, initiator_id, context_json).await {
            Ok(instance) => StartInstancePayload { instance: Some(instance), errors: vec![] },
            Err(e) => StartInstancePayload { instance: None, errors: vec![UserError { field: None, message: e.to_string() }] },
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn cancel_instance(
        &self,
        instance_id: &str,
        reason: Option<&str>,
    ) -> CancelInstancePayload {
        match self.client.cancel_instance(instance_id, reason).await {
            Ok(instance) => CancelInstancePayload { instance: Some(instance), errors: vec![] },
            Err(e) => CancelInstancePayload { instance: None, errors: vec![UserError { field: None, message: e.to_string() }] },
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn reassign_task(
        &self,
        task_id: &str,
        new_assignee_id: &str,
        reason: Option<&str>,
        actor_id: &str,
    ) -> ReassignTaskPayload {
        match self.client.reassign_task(task_id, new_assignee_id, reason, actor_id).await {
            Ok((task, previous_assignee_id)) => ReassignTaskPayload {
                task: Some(task),
                previous_assignee_id,
                errors: vec![],
            },
            Err(e) => ReassignTaskPayload {
                task: None,
                previous_assignee_id: None,
                errors: vec![UserError { field: None, message: e.to_string() }],
            },
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn approve_task(
        &self,
        task_id: &str,
        actor_id: &str,
        comment: Option<&str>,
    ) -> ApproveTaskPayload {
        match self.client.approve_task(task_id, actor_id, comment).await {
            Ok((tid, status, next_task_id, instance_status)) => ApproveTaskPayload {
                task_id: Some(tid),
                status: Some(status),
                next_task_id,
                instance_status: Some(instance_status),
                errors: vec![],
            },
            Err(e) => ApproveTaskPayload {
                task_id: None,
                status: None,
                next_task_id: None,
                instance_status: None,
                errors: vec![UserError { field: None, message: e.to_string() }],
            },
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn reject_task(
        &self,
        task_id: &str,
        actor_id: &str,
        comment: Option<&str>,
    ) -> RejectTaskPayload {
        match self.client.reject_task(task_id, actor_id, comment).await {
            Ok((tid, status, next_task_id, instance_status)) => RejectTaskPayload {
                task_id: Some(tid),
                status: Some(status),
                next_task_id,
                instance_status: Some(instance_status),
                errors: vec![],
            },
            Err(e) => RejectTaskPayload {
                task_id: None,
                status: None,
                next_task_id: None,
                instance_status: None,
                errors: vec![UserError { field: None, message: e.to_string() }],
            },
        }
    }
}
