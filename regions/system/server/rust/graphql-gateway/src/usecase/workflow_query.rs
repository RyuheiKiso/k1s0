use crate::domain::model::{WorkflowDefinition, WorkflowInstance, WorkflowTask};
use crate::infrastructure::grpc::WorkflowGrpcClient;
use std::sync::Arc;
use tracing::instrument;

pub struct WorkflowQueryResolver {
    client: Arc<WorkflowGrpcClient>,
}

impl WorkflowQueryResolver {
    pub fn new(client: Arc<WorkflowGrpcClient>) -> Self {
        Self { client }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_workflow(
        &self,
        workflow_id: &str,
    ) -> anyhow::Result<Option<WorkflowDefinition>> {
        self.client.get_workflow(workflow_id).await
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_workflows(
        &self,
        enabled_only: bool,
        first: Option<i32>,
        after: Option<i32>,
    ) -> anyhow::Result<Vec<WorkflowDefinition>> {
        let page_size = first.unwrap_or(20);
        let page = after.map(|a| a + 1).unwrap_or(1);
        self.client
            .list_workflows(enabled_only, Some(page_size), Some(page))
            .await
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_instance(
        &self,
        instance_id: &str,
    ) -> anyhow::Result<Option<WorkflowInstance>> {
        self.client.get_instance(instance_id).await
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_instances(
        &self,
        status: Option<&str>,
        workflow_id: Option<&str>,
        initiator_id: Option<&str>,
        first: Option<i32>,
        after: Option<i32>,
    ) -> anyhow::Result<Vec<WorkflowInstance>> {
        let page_size = first.unwrap_or(20);
        let page = after.map(|a| a + 1).unwrap_or(1);
        self.client
            .list_instances(
                status,
                workflow_id,
                initiator_id,
                Some(page_size),
                Some(page),
            )
            .await
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_tasks(
        &self,
        assignee_id: Option<&str>,
        status: Option<&str>,
        instance_id: Option<&str>,
        overdue_only: bool,
        first: Option<i32>,
        after: Option<i32>,
    ) -> anyhow::Result<Vec<WorkflowTask>> {
        let page_size = first.unwrap_or(20);
        let page = after.map(|a| a + 1).unwrap_or(1);
        self.client
            .list_tasks(
                assignee_id,
                status,
                instance_id,
                overdue_only,
                Some(page_size),
                Some(page),
            )
            .await
    }
}
