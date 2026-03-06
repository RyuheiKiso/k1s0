use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::flow_definition::{FlowDefinition, FlowSlo, FlowStep};
use crate::domain::repository::FlowDefinitionRepository;

#[derive(Debug, Clone)]
pub struct UpdateFlowInput {
    pub id: Uuid,
    pub description: Option<String>,
    pub steps: Option<Vec<FlowStep>>,
    pub slo: Option<FlowSlo>,
    pub enabled: Option<bool>,
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateFlowError {
    #[error("flow not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct UpdateFlowUseCase {
    repo: Arc<dyn FlowDefinitionRepository>,
}

impl UpdateFlowUseCase {
    pub fn new(repo: Arc<dyn FlowDefinitionRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        input: &UpdateFlowInput,
    ) -> Result<FlowDefinition, UpdateFlowError> {
        let mut flow = self
            .repo
            .find_by_id(&input.id)
            .await
            .map_err(|e| UpdateFlowError::Internal(e.to_string()))?
            .ok_or_else(|| UpdateFlowError::NotFound(input.id.to_string()))?;

        if let Some(ref desc) = input.description {
            flow.description = desc.clone();
        }
        if let Some(ref steps) = input.steps {
            flow.steps = steps.clone();
        }
        if let Some(ref slo) = input.slo {
            flow.slo = slo.clone();
        }
        if let Some(enabled) = input.enabled {
            flow.enabled = enabled;
        }
        flow.updated_at = chrono::Utc::now();

        self.repo
            .update(&flow)
            .await
            .map_err(|e| UpdateFlowError::Internal(e.to_string()))?;

        Ok(flow)
    }
}
