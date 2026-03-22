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

    pub async fn execute(&self, input: &ListFlowsInput) -> Result<ListFlowsOutput, ListFlowsError> {
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::flow_definition::{FlowSlo, FlowStep};
    use crate::domain::repository::flow_definition_repository::MockFlowDefinitionRepository;

    fn make_flow(name: &str) -> FlowDefinition {
        FlowDefinition::new(
            name.to_string(),
            "test".to_string(),
            "service.task".to_string(),
            vec![FlowStep {
                event_type: "TaskCreated".to_string(),
                source: "task-server".to_string(),
                source_filter: None,
                timeout_seconds: 30,
                description: String::new(),
            }],
            FlowSlo {
                target_completion_seconds: 120,
                target_success_rate: 0.99,
                alert_on_violation: true,
            },
        )
    }

    #[tokio::test]
    async fn paginated() {
        let mut mock = MockFlowDefinitionRepository::new();
        mock.expect_find_all_paginated()
            .returning(|_, _, _| Ok((vec![make_flow("flow_a"), make_flow("flow_b")], 5)));

        let uc = ListFlowsUseCase::new(Arc::new(mock));
        let input = ListFlowsInput {
            page: 1,
            page_size: 2,
            domain: None,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.flows.len(), 2);
        assert_eq!(output.total_count, 5);
        assert!(output.has_next);
        assert_eq!(output.page, 1);
        assert_eq!(output.page_size, 2);
    }

    #[tokio::test]
    async fn empty() {
        let mut mock = MockFlowDefinitionRepository::new();
        mock.expect_find_all_paginated()
            .returning(|_, _, _| Ok((vec![], 0)));

        let uc = ListFlowsUseCase::new(Arc::new(mock));
        let input = ListFlowsInput {
            page: 1,
            page_size: 10,
            domain: None,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.flows.is_empty());
        assert!(!output.has_next);
    }
}
