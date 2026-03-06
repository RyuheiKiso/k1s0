use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::flow_instance::FlowInstance;
use crate::domain::repository::FlowInstanceRepository;

#[derive(Debug, Clone)]
pub struct GetFlowInstancesInput {
    pub flow_id: Uuid,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug)]
pub struct GetFlowInstancesOutput {
    pub instances: Vec<FlowInstance>,
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub has_next: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum GetFlowInstancesError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct GetFlowInstancesUseCase {
    repo: Arc<dyn FlowInstanceRepository>,
}

impl GetFlowInstancesUseCase {
    pub fn new(repo: Arc<dyn FlowInstanceRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        input: &GetFlowInstancesInput,
    ) -> Result<GetFlowInstancesOutput, GetFlowInstancesError> {
        let (instances, total_count) = self
            .repo
            .find_by_flow_id_paginated(&input.flow_id, input.page, input.page_size)
            .await
            .map_err(|e| GetFlowInstancesError::Internal(e.to_string()))?;

        let has_next = (input.page as u64 * input.page_size as u64) < total_count;

        Ok(GetFlowInstancesOutput {
            instances,
            total_count,
            page: input.page,
            page_size: input.page_size,
            has_next,
        })
    }
}
