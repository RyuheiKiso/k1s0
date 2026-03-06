use std::sync::Arc;

use crate::domain::entity::flow_definition::FlowDefinition;
use crate::domain::repository::FlowDefinitionRepository;

#[derive(Debug, Clone)]
pub struct ListFlowsInput {
    pub page: u32,
    pub page_size: u32,
    pub domain: Option<String>,
}

#[derive(Debug)]
pub struct ListFlowsOutput {
    pub flows: Vec<FlowDefinition>,
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub has_next: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum ListFlowsError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct ListFlowsUseCase {
    repo: Arc<dyn FlowDefinitionRepository>,
}

impl ListFlowsUseCase {
    pub fn new(repo: Arc<dyn FlowDefinitionRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        input: &ListFlowsInput,
    ) -> Result<ListFlowsOutput, ListFlowsError> {
        let (flows, total_count) = self
            .repo
            .find_all_paginated(input.page, input.page_size, input.domain.clone())
            .await
            .map_err(|e| ListFlowsError::Internal(e.to_string()))?;

        let has_next = (input.page as u64 * input.page_size as u64) < total_count;

        Ok(ListFlowsOutput {
            flows,
            total_count,
            page: input.page,
            page_size: input.page_size,
            has_next,
        })
    }
}
