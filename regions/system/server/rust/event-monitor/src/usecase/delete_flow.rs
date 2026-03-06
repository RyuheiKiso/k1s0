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
