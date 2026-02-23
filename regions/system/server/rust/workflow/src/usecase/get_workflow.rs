use std::sync::Arc;

use crate::domain::entity::workflow_definition::WorkflowDefinition;
use crate::domain::repository::WorkflowDefinitionRepository;

#[derive(Debug, Clone)]
pub struct GetWorkflowInput {
    pub id: String,
}

#[derive(Debug, thiserror::Error)]
pub enum GetWorkflowError {
    #[error("workflow not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct GetWorkflowUseCase {
    repo: Arc<dyn WorkflowDefinitionRepository>,
}

impl GetWorkflowUseCase {
    pub fn new(repo: Arc<dyn WorkflowDefinitionRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        input: &GetWorkflowInput,
    ) -> Result<WorkflowDefinition, GetWorkflowError> {
        self.repo
            .find_by_id(&input.id)
            .await
            .map_err(|e| GetWorkflowError::Internal(e.to_string()))?
            .ok_or_else(|| GetWorkflowError::NotFound(input.id.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::workflow_definition_repository::MockWorkflowDefinitionRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockWorkflowDefinitionRepository::new();
        mock.expect_find_by_id().returning(|_| {
            Ok(Some(WorkflowDefinition::new(
                "wf_001".to_string(),
                "test".to_string(),
                "".to_string(),
                true,
                vec![],
            )))
        });

        let uc = GetWorkflowUseCase::new(Arc::new(mock));
        let input = GetWorkflowInput {
            id: "wf_001".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, "wf_001");
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockWorkflowDefinitionRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = GetWorkflowUseCase::new(Arc::new(mock));
        let input = GetWorkflowInput {
            id: "wf_missing".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(matches!(
            result.unwrap_err(),
            GetWorkflowError::NotFound(_)
        ));
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockWorkflowDefinitionRepository::new();
        mock.expect_find_by_id()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = GetWorkflowUseCase::new(Arc::new(mock));
        let input = GetWorkflowInput {
            id: "wf_001".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(matches!(
            result.unwrap_err(),
            GetWorkflowError::Internal(_)
        ));
    }
}
