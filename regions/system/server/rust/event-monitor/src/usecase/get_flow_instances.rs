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

        let has_next = (u64::from(input.page) * u64::from(input.page_size)) < total_count;

        Ok(GetFlowInstancesOutput {
            instances,
            total_count,
            page: input.page,
            page_size: input.page_size,
            has_next,
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::flow_instance_repository::MockFlowInstanceRepository;

    #[tokio::test]
    async fn paginated() {
        let flow_id = Uuid::new_v4();
        let mut mock = MockFlowInstanceRepository::new();
        mock.expect_find_by_flow_id_paginated()
            .returning(move |_, _, _| {
                Ok((
                    vec![
                        FlowInstance::new("system".to_string(), flow_id, "corr-1".to_string()),
                        FlowInstance::new("system".to_string(), flow_id, "corr-2".to_string()),
                    ],
                    5,
                ))
            });

        let uc = GetFlowInstancesUseCase::new(Arc::new(mock));
        let input = GetFlowInstancesInput {
            flow_id,
            page: 1,
            page_size: 2,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.instances.len(), 2);
        assert_eq!(output.total_count, 5);
        assert!(output.has_next);
    }

    #[tokio::test]
    async fn empty() {
        let mut mock = MockFlowInstanceRepository::new();
        mock.expect_find_by_flow_id_paginated()
            .returning(|_, _, _| Ok((vec![], 0)));

        let uc = GetFlowInstancesUseCase::new(Arc::new(mock));
        let input = GetFlowInstancesInput {
            flow_id: Uuid::new_v4(),
            page: 1,
            page_size: 10,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.instances.is_empty());
        assert!(!output.has_next);
    }
}
