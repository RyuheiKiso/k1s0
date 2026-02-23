use std::sync::Arc;

use crate::domain::entity::workflow_definition::WorkflowDefinition;
use crate::domain::repository::WorkflowDefinitionRepository;

#[derive(Debug, Clone)]
pub struct ListWorkflowsInput {
    pub enabled_only: bool,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone)]
pub struct ListWorkflowsOutput {
    pub workflows: Vec<WorkflowDefinition>,
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub has_next: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum ListWorkflowsError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct ListWorkflowsUseCase {
    repo: Arc<dyn WorkflowDefinitionRepository>,
}

impl ListWorkflowsUseCase {
    pub fn new(repo: Arc<dyn WorkflowDefinitionRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        input: &ListWorkflowsInput,
    ) -> Result<ListWorkflowsOutput, ListWorkflowsError> {
        let (workflows, total_count) = self
            .repo
            .find_all(input.enabled_only, input.page, input.page_size)
            .await
            .map_err(|e| ListWorkflowsError::Internal(e.to_string()))?;

        let has_next = (input.page as u64 * input.page_size as u64) < total_count;

        Ok(ListWorkflowsOutput {
            workflows,
            total_count,
            page: input.page,
            page_size: input.page_size,
            has_next,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::workflow_definition_repository::MockWorkflowDefinitionRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockWorkflowDefinitionRepository::new();
        mock.expect_find_all().returning(|_, _, _| {
            Ok((
                vec![WorkflowDefinition::new(
                    "wf_001".to_string(),
                    "test".to_string(),
                    "".to_string(),
                    true,
                    vec![],
                )],
                1,
            ))
        });

        let uc = ListWorkflowsUseCase::new(Arc::new(mock));
        let input = ListWorkflowsInput {
            enabled_only: false,
            page: 1,
            page_size: 20,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.workflows.len(), 1);
        assert_eq!(output.total_count, 1);
        assert!(!output.has_next);
    }

    #[tokio::test]
    async fn has_next_page() {
        let mut mock = MockWorkflowDefinitionRepository::new();
        mock.expect_find_all()
            .returning(|_, _, _| Ok((vec![], 50)));

        let uc = ListWorkflowsUseCase::new(Arc::new(mock));
        let input = ListWorkflowsInput {
            enabled_only: false,
            page: 1,
            page_size: 20,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        assert!(result.unwrap().has_next);
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockWorkflowDefinitionRepository::new();
        mock.expect_find_all()
            .returning(|_, _, _| Err(anyhow::anyhow!("db error")));

        let uc = ListWorkflowsUseCase::new(Arc::new(mock));
        let input = ListWorkflowsInput {
            enabled_only: false,
            page: 1,
            page_size: 20,
        };
        let result = uc.execute(&input).await;
        assert!(matches!(
            result.unwrap_err(),
            ListWorkflowsError::Internal(_)
        ));
    }
}
