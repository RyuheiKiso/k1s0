use std::sync::Arc;

use uuid::Uuid;

use crate::domain::repository::FlowDefinitionRepository;

#[derive(Debug, thiserror::Error)]
pub enum DeleteFlowError {
    #[error("flow not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct DeleteFlowUseCase {
    repo: Arc<dyn FlowDefinitionRepository>,
}

impl DeleteFlowUseCase {
    pub fn new(repo: Arc<dyn FlowDefinitionRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: &Uuid) -> Result<(), DeleteFlowError> {
        let deleted = self
            .repo
            .delete(id)
            .await
            .map_err(|e| DeleteFlowError::Internal(e.to_string()))?;

        if !deleted {
            return Err(DeleteFlowError::NotFound(id.to_string()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::flow_definition_repository::MockFlowDefinitionRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockFlowDefinitionRepository::new();
        mock.expect_delete().returning(|_| Ok(true));

        let uc = DeleteFlowUseCase::new(Arc::new(mock));
        let result = uc.execute(&Uuid::new_v4()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockFlowDefinitionRepository::new();
        mock.expect_delete().returning(|_| Ok(false));

        let uc = DeleteFlowUseCase::new(Arc::new(mock));
        let result = uc.execute(&Uuid::new_v4()).await;
        assert!(matches!(result, Err(DeleteFlowError::NotFound(_))));
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockFlowDefinitionRepository::new();
        mock.expect_delete()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = DeleteFlowUseCase::new(Arc::new(mock));
        let result = uc.execute(&Uuid::new_v4()).await;
        assert!(matches!(result, Err(DeleteFlowError::Internal(_))));
    }
}
