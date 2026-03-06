use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::flow_instance::FlowInstance;
use crate::domain::repository::FlowInstanceRepository;

#[derive(Debug, thiserror::Error)]
pub enum GetFlowInstanceError {
    #[error("instance not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct GetFlowInstanceUseCase {
    repo: Arc<dyn FlowInstanceRepository>,
}

impl GetFlowInstanceUseCase {
    pub fn new(repo: Arc<dyn FlowInstanceRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: &Uuid) -> Result<FlowInstance, GetFlowInstanceError> {
        self.repo
            .find_by_id(id)
            .await
            .map_err(|e| GetFlowInstanceError::Internal(e.to_string()))?
            .ok_or_else(|| GetFlowInstanceError::NotFound(id.to_string()))
    }
}
