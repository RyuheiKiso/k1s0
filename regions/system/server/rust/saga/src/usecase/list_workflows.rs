use std::sync::Arc;

use crate::domain::entity::workflow::WorkflowDefinition;
use crate::domain::repository::WorkflowRepository;

/// ListWorkflowsUseCase はワークフロー一覧取得を担う。
pub struct ListWorkflowsUseCase {
    workflow_repo: Arc<dyn WorkflowRepository>,
}

impl ListWorkflowsUseCase {
    pub fn new(workflow_repo: Arc<dyn WorkflowRepository>) -> Self {
        Self { workflow_repo }
    }

    /// 全ワークフローを取得する。
    pub async fn execute(&self) -> anyhow::Result<Vec<WorkflowDefinition>> {
        self.workflow_repo.list().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::workflow_repository::MockWorkflowRepository;

    #[tokio::test]
    async fn test_list_workflows_empty() {
        let mut mock = MockWorkflowRepository::new();
        mock.expect_list().returning(|| Ok(vec![]));

        let uc = ListWorkflowsUseCase::new(Arc::new(mock));
        let result = uc.execute().await.unwrap();
        assert!(result.is_empty());
    }
}
