use std::sync::Arc;

use crate::domain::repository::WorkflowDefinitionRepository;

// RUST-CRIT-001 対応: テナント分離のため tenant_id フィールドを追加する
#[derive(Debug, Clone)]
pub struct DeleteWorkflowInput {
    pub tenant_id: String,
    pub id: String,
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteWorkflowError {
    #[error("workflow not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct DeleteWorkflowUseCase {
    repo: Arc<dyn WorkflowDefinitionRepository>,
}

impl DeleteWorkflowUseCase {
    pub fn new(repo: Arc<dyn WorkflowDefinitionRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, input: &DeleteWorkflowInput) -> Result<(), DeleteWorkflowError> {
        // テナント分離: tenant_id を渡してRLSによるフィルタリングを有効化する
        let deleted = self
            .repo
            .delete(&input.tenant_id, &input.id)
            .await
            .map_err(|e| DeleteWorkflowError::Internal(e.to_string()))?;

        if !deleted {
            return Err(DeleteWorkflowError::NotFound(input.id.clone()));
        }

        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::workflow_definition_repository::MockWorkflowDefinitionRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockWorkflowDefinitionRepository::new();
        mock.expect_delete().returning(|_, _| Ok(true));

        let uc = DeleteWorkflowUseCase::new(Arc::new(mock));
        let input = DeleteWorkflowInput {
            tenant_id: "test-tenant".to_string(),
            id: "wf_001".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockWorkflowDefinitionRepository::new();
        mock.expect_delete().returning(|_, _| Ok(false));

        let uc = DeleteWorkflowUseCase::new(Arc::new(mock));
        let input = DeleteWorkflowInput {
            tenant_id: "test-tenant".to_string(),
            id: "wf_missing".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(matches!(
            result.unwrap_err(),
            DeleteWorkflowError::NotFound(_)
        ));
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockWorkflowDefinitionRepository::new();
        mock.expect_delete()
            .returning(|_, _| Err(anyhow::anyhow!("db error")));

        let uc = DeleteWorkflowUseCase::new(Arc::new(mock));
        let input = DeleteWorkflowInput {
            tenant_id: "test-tenant".to_string(),
            id: "wf_001".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(matches!(
            result.unwrap_err(),
            DeleteWorkflowError::Internal(_)
        ));
    }
}
