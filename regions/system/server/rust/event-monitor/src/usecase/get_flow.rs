use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::flow_definition::FlowDefinition;
use crate::domain::repository::FlowDefinitionRepository;

#[derive(Debug, thiserror::Error)]
pub enum GetFlowError {
    #[error("flow not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct GetFlowUseCase {
    repo: Arc<dyn FlowDefinitionRepository>,
}

impl GetFlowUseCase {
    pub fn new(repo: Arc<dyn FlowDefinitionRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: &Uuid) -> Result<FlowDefinition, GetFlowError> {
        self.repo
            .find_by_id(id)
            .await
            .map_err(|e| GetFlowError::Internal(e.to_string()))?
            .ok_or_else(|| GetFlowError::NotFound(id.to_string()))
    }
}
