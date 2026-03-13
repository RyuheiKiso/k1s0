use std::time::Duration;

use chrono::{DateTime, Utc};
use tonic::transport::Channel;
use tracing::instrument;

use crate::adapter::graphql_handler::WorkflowStepInput;
use crate::domain::model::{WorkflowDefinition, WorkflowInstance, WorkflowStep, WorkflowTask};
use crate::infrastructure::config::BackendConfig;

#[allow(dead_code)]
pub mod proto {
    pub mod k1s0 {
        pub mod system {
            pub mod common {
                pub mod v1 {
                    tonic::include_proto!("k1s0.system.common.v1");
                }
            }
            pub mod workflow {
                pub mod v1 {
                    tonic::include_proto!("k1s0.system.workflow.v1");
                }
            }
        }
    }
}

use proto::k1s0::system::workflow::v1::workflow_service_client::WorkflowServiceClient;

pub struct WorkflowGrpcClient {
    client: WorkflowServiceClient<Channel>,
}

impl WorkflowGrpcClient {
    pub async fn connect(cfg: &BackendConfig) -> anyhow::Result<Self> {
        let channel = Channel::from_shared(cfg.address.clone())?
            .timeout(Duration::from_millis(cfg.timeout_ms))
            .connect()
            .await?;
        Ok(Self {
            client: WorkflowServiceClient::new(channel),
        })
    }

    fn step_from_proto(
        s: proto::k1s0::system::workflow::v1::WorkflowStep,
    ) -> WorkflowStep {
        WorkflowStep {
            step_id: s.step_id,
            name: s.name,
            step_type: s.step_type,
            assignee_role: s.assignee_role.filter(|v| !v.is_empty()),
            timeout_hours: s.timeout_hours.map(|v| v as i32),
            on_approve: s.on_approve.filter(|v| !v.is_empty()),
            on_reject: s.on_reject.filter(|v| !v.is_empty()),
        }
    }

    fn definition_from_proto(
        d: proto::k1s0::system::workflow::v1::WorkflowDefinition,
    ) -> WorkflowDefinition {
        WorkflowDefinition {
            id: d.id,
            name: d.name,
            description: d.description,
            version: d.version as i32,
            enabled: d.enabled,
            steps: d.steps.into_iter().map(Self::step_from_proto).collect(),
            created_at: timestamp_to_rfc3339(d.created_at),
            updated_at: timestamp_to_rfc3339(d.updated_at),
        }
    }

    fn instance_from_proto(
        i: proto::k1s0::system::workflow::v1::WorkflowInstance,
    ) -> WorkflowInstance {
        let context_json = if i.context_json.is_empty() {
            None
        } else {
            Some(String::from_utf8_lossy(&i.context_json).to_string())
        };

        WorkflowInstance {
            id: i.id,
            workflow_id: i.workflow_id,
            workflow_name: i.workflow_name,
            title: i.title,
            initiator_id: i.initiator_id,
            current_step_id: i.current_step_id.filter(|v| !v.is_empty()),
            status: i.status,
            context_json,
            started_at: timestamp_to_rfc3339(i.started_at),
            completed_at: optional_timestamp_to_rfc3339(i.completed_at),
            created_at: optional_timestamp_to_rfc3339(i.created_at),
        }
    }

    fn task_from_proto(
        t: proto::k1s0::system::workflow::v1::WorkflowTask,
    ) -> WorkflowTask {
        WorkflowTask {
            id: t.id,
            instance_id: t.instance_id,
            step_id: t.step_id,
            step_name: t.step_name,
            assignee_id: t.assignee_id.filter(|v| !v.is_empty()),
            status: t.status,
            due_at: optional_timestamp_to_rfc3339(t.due_at),
            comment: t.comment.filter(|v| !v.is_empty()),
            actor_id: t.actor_id.filter(|v| !v.is_empty()),
            decided_at: optional_timestamp_to_rfc3339(t.decided_at),
            created_at: timestamp_to_rfc3339(t.created_at),
            updated_at: timestamp_to_rfc3339(t.updated_at),
        }
    }

    fn steps_to_proto(
        steps: &[WorkflowStepInput],
    ) -> Vec<proto::k1s0::system::workflow::v1::WorkflowStep> {
        steps
            .iter()
            .map(|s| proto::k1s0::system::workflow::v1::WorkflowStep {
                step_id: s.step_id.clone(),
                name: s.name.clone(),
                step_type: s.step_type.clone(),
                assignee_role: s.assignee_role.clone(),
                timeout_hours: s.timeout_hours.map(|v| v as u32),
                on_approve: s.on_approve.clone(),
                on_reject: s.on_reject.clone(),
            })
            .collect()
    }

    // ── Workflow Definition ──

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_workflow(
        &self,
        workflow_id: &str,
    ) -> anyhow::Result<Option<WorkflowDefinition>> {
        let request = tonic::Request::new(
            proto::k1s0::system::workflow::v1::GetWorkflowRequest {
                workflow_id: workflow_id.to_owned(),
            },
        );

        match self.client.clone().get_workflow(request).await {
            Ok(resp) => {
                let d = match resp.into_inner().workflow {
                    Some(d) => d,
                    None => return Ok(None),
                };
                Ok(Some(Self::definition_from_proto(d)))
            }
            Err(status) if status.code() == tonic::Code::NotFound => Ok(None),
            Err(e) => Err(anyhow::anyhow!(
                "WorkflowService.GetWorkflow failed: {}",
                e
            )),
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_workflows(
        &self,
        enabled_only: bool,
        page_size: Option<i32>,
        page: Option<i32>,
    ) -> anyhow::Result<Vec<WorkflowDefinition>> {
        let request = tonic::Request::new(
            proto::k1s0::system::workflow::v1::ListWorkflowsRequest {
                enabled_only,
                pagination: Some(proto::k1s0::system::common::v1::Pagination {
                    page: page.unwrap_or(1),
                    page_size: page_size.unwrap_or(20),
                }),
            },
        );

        let resp = self
            .client
            .clone()
            .list_workflows(request)
            .await
            .map_err(|e| anyhow::anyhow!("WorkflowService.ListWorkflows failed: {}", e))?
            .into_inner();

        Ok(resp
            .workflows
            .into_iter()
            .map(Self::definition_from_proto)
            .collect())
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn create_workflow(
        &self,
        name: &str,
        description: &str,
        enabled: bool,
        steps: &[WorkflowStepInput],
    ) -> anyhow::Result<WorkflowDefinition> {
        let request = tonic::Request::new(
            proto::k1s0::system::workflow::v1::CreateWorkflowRequest {
                name: name.to_owned(),
                description: description.to_owned(),
                enabled,
                steps: Self::steps_to_proto(steps),
            },
        );

        let d = self
            .client
            .clone()
            .create_workflow(request)
            .await
            .map_err(|e| anyhow::anyhow!("WorkflowService.CreateWorkflow failed: {}", e))?
            .into_inner()
            .workflow
            .ok_or_else(|| anyhow::anyhow!("empty workflow in create response"))?;

        Ok(Self::definition_from_proto(d))
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn update_workflow(
        &self,
        workflow_id: &str,
        name: Option<&str>,
        description: Option<&str>,
        enabled: Option<bool>,
        steps: Option<&[WorkflowStepInput]>,
    ) -> anyhow::Result<WorkflowDefinition> {
        let request = tonic::Request::new(
            proto::k1s0::system::workflow::v1::UpdateWorkflowRequest {
                workflow_id: workflow_id.to_owned(),
                name: name.map(|s| s.to_owned()),
                description: description.map(|s| s.to_owned()),
                enabled,
                steps: steps.map(|s| {
                    proto::k1s0::system::workflow::v1::WorkflowSteps {
                        items: Self::steps_to_proto(s),
                    }
                }),
            },
        );

        let d = self
            .client
            .clone()
            .update_workflow(request)
            .await
            .map_err(|e| anyhow::anyhow!("WorkflowService.UpdateWorkflow failed: {}", e))?
            .into_inner()
            .workflow
            .ok_or_else(|| anyhow::anyhow!("empty workflow in update response"))?;

        Ok(Self::definition_from_proto(d))
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn delete_workflow(&self, workflow_id: &str) -> anyhow::Result<bool> {
        let request = tonic::Request::new(
            proto::k1s0::system::workflow::v1::DeleteWorkflowRequest {
                workflow_id: workflow_id.to_owned(),
            },
        );

        let resp = self
            .client
            .clone()
            .delete_workflow(request)
            .await
            .map_err(|e| anyhow::anyhow!("WorkflowService.DeleteWorkflow failed: {}", e))?
            .into_inner();

        Ok(resp.success)
    }

    // ── Workflow Instance ──

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn start_instance(
        &self,
        workflow_id: &str,
        title: &str,
        initiator_id: &str,
        context_json: Option<&str>,
    ) -> anyhow::Result<WorkflowInstance> {
        let request = tonic::Request::new(
            proto::k1s0::system::workflow::v1::StartInstanceRequest {
                workflow_id: workflow_id.to_owned(),
                title: title.to_owned(),
                initiator_id: initiator_id.to_owned(),
                context_json: context_json
                    .map(|s| s.as_bytes().to_vec())
                    .unwrap_or_default(),
            },
        );

        let resp = self
            .client
            .clone()
            .start_instance(request)
            .await
            .map_err(|e| anyhow::anyhow!("WorkflowService.StartInstance failed: {}", e))?
            .into_inner();

        let context_json = if resp.context_json.is_empty() {
            None
        } else {
            Some(String::from_utf8_lossy(&resp.context_json).to_string())
        };

        Ok(WorkflowInstance {
            id: resp.instance_id,
            workflow_id: resp.workflow_id,
            workflow_name: resp.workflow_name,
            title: resp.title,
            initiator_id: resp.initiator_id,
            current_step_id: resp.current_step_id.filter(|s| !s.is_empty()),
            status: resp.status,
            context_json,
            started_at: timestamp_to_rfc3339(resp.started_at),
            completed_at: None,
            created_at: None,
        })
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_instance(
        &self,
        instance_id: &str,
    ) -> anyhow::Result<Option<WorkflowInstance>> {
        let request = tonic::Request::new(
            proto::k1s0::system::workflow::v1::GetInstanceRequest {
                instance_id: instance_id.to_owned(),
            },
        );

        match self.client.clone().get_instance(request).await {
            Ok(resp) => {
                let i = match resp.into_inner().instance {
                    Some(i) => i,
                    None => return Ok(None),
                };
                Ok(Some(Self::instance_from_proto(i)))
            }
            Err(status) if status.code() == tonic::Code::NotFound => Ok(None),
            Err(e) => Err(anyhow::anyhow!(
                "WorkflowService.GetInstance failed: {}",
                e
            )),
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_instances(
        &self,
        status: Option<&str>,
        workflow_id: Option<&str>,
        initiator_id: Option<&str>,
        page_size: Option<i32>,
        page: Option<i32>,
    ) -> anyhow::Result<Vec<WorkflowInstance>> {
        let request = tonic::Request::new(
            proto::k1s0::system::workflow::v1::ListInstancesRequest {
                status: status.unwrap_or_default().to_owned(),
                workflow_id: workflow_id.unwrap_or_default().to_owned(),
                initiator_id: initiator_id.unwrap_or_default().to_owned(),
                pagination: Some(proto::k1s0::system::common::v1::Pagination {
                    page: page.unwrap_or(1),
                    page_size: page_size.unwrap_or(20),
                }),
            },
        );

        let resp = self
            .client
            .clone()
            .list_instances(request)
            .await
            .map_err(|e| anyhow::anyhow!("WorkflowService.ListInstances failed: {}", e))?
            .into_inner();

        Ok(resp
            .instances
            .into_iter()
            .map(Self::instance_from_proto)
            .collect())
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn cancel_instance(
        &self,
        instance_id: &str,
        reason: Option<&str>,
    ) -> anyhow::Result<WorkflowInstance> {
        let request = tonic::Request::new(
            proto::k1s0::system::workflow::v1::CancelInstanceRequest {
                instance_id: instance_id.to_owned(),
                reason: reason.map(|s| s.to_owned()),
            },
        );

        let i = self
            .client
            .clone()
            .cancel_instance(request)
            .await
            .map_err(|e| anyhow::anyhow!("WorkflowService.CancelInstance failed: {}", e))?
            .into_inner()
            .instance
            .ok_or_else(|| anyhow::anyhow!("empty instance in cancel response"))?;

        Ok(Self::instance_from_proto(i))
    }

    // ── Workflow Task ──

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_tasks(
        &self,
        assignee_id: Option<&str>,
        status: Option<&str>,
        instance_id: Option<&str>,
        overdue_only: bool,
        page_size: Option<i32>,
        page: Option<i32>,
    ) -> anyhow::Result<Vec<WorkflowTask>> {
        let request = tonic::Request::new(
            proto::k1s0::system::workflow::v1::ListTasksRequest {
                assignee_id: assignee_id.unwrap_or_default().to_owned(),
                status: status.unwrap_or_default().to_owned(),
                instance_id: instance_id.unwrap_or_default().to_owned(),
                overdue_only,
                pagination: Some(proto::k1s0::system::common::v1::Pagination {
                    page: page.unwrap_or(1),
                    page_size: page_size.unwrap_or(20),
                }),
            },
        );

        let resp = self
            .client
            .clone()
            .list_tasks(request)
            .await
            .map_err(|e| anyhow::anyhow!("WorkflowService.ListTasks failed: {}", e))?
            .into_inner();

        Ok(resp.tasks.into_iter().map(Self::task_from_proto).collect())
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn reassign_task(
        &self,
        task_id: &str,
        new_assignee_id: &str,
        reason: Option<&str>,
        actor_id: &str,
    ) -> anyhow::Result<(WorkflowTask, Option<String>)> {
        let request = tonic::Request::new(
            proto::k1s0::system::workflow::v1::ReassignTaskRequest {
                task_id: task_id.to_owned(),
                new_assignee_id: new_assignee_id.to_owned(),
                reason: reason.map(|s| s.to_owned()),
                actor_id: actor_id.to_owned(),
            },
        );

        let resp = self
            .client
            .clone()
            .reassign_task(request)
            .await
            .map_err(|e| anyhow::anyhow!("WorkflowService.ReassignTask failed: {}", e))?
            .into_inner();

        let task = resp
            .task
            .ok_or_else(|| anyhow::anyhow!("empty task in reassign response"))?;
        let previous_assignee_id = resp.previous_assignee_id.filter(|s| !s.is_empty());

        Ok((Self::task_from_proto(task), previous_assignee_id))
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn approve_task(
        &self,
        task_id: &str,
        actor_id: &str,
        comment: Option<&str>,
    ) -> anyhow::Result<(String, String, Option<String>, String)> {
        let request = tonic::Request::new(
            proto::k1s0::system::workflow::v1::ApproveTaskRequest {
                task_id: task_id.to_owned(),
                actor_id: actor_id.to_owned(),
                comment: comment.map(|s| s.to_owned()),
            },
        );

        let resp = self
            .client
            .clone()
            .approve_task(request)
            .await
            .map_err(|e| anyhow::anyhow!("WorkflowService.ApproveTask failed: {}", e))?
            .into_inner();

        Ok((
            resp.task_id,
            resp.status,
            resp.next_task_id.filter(|s| !s.is_empty()),
            resp.instance_status,
        ))
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn reject_task(
        &self,
        task_id: &str,
        actor_id: &str,
        comment: Option<&str>,
    ) -> anyhow::Result<(String, String, Option<String>, String)> {
        let request = tonic::Request::new(
            proto::k1s0::system::workflow::v1::RejectTaskRequest {
                task_id: task_id.to_owned(),
                actor_id: actor_id.to_owned(),
                comment: comment.map(|s| s.to_owned()),
            },
        );

        let resp = self
            .client
            .clone()
            .reject_task(request)
            .await
            .map_err(|e| anyhow::anyhow!("WorkflowService.RejectTask failed: {}", e))?
            .into_inner();

        Ok((
            resp.task_id,
            resp.status,
            resp.next_task_id.filter(|s| !s.is_empty()),
            resp.instance_status,
        ))
    }

    // ── Health Check ──

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn health_check(&self) -> anyhow::Result<()> {
        self.list_workflows(false, Some(1), Some(1)).await?;
        Ok(())
    }
}

fn timestamp_to_rfc3339(ts: Option<proto::k1s0::system::common::v1::Timestamp>) -> String {
    ts.and_then(|ts| DateTime::<Utc>::from_timestamp(ts.seconds, ts.nanos as u32))
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_default()
}

fn optional_timestamp_to_rfc3339(
    ts: Option<proto::k1s0::system::common::v1::Timestamp>,
) -> Option<String> {
    ts.and_then(|ts| DateTime::<Utc>::from_timestamp(ts.seconds, ts.nanos as u32))
        .map(|dt| dt.to_rfc3339())
}

