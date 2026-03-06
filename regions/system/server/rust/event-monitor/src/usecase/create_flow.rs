use std::sync::Arc;

use crate::domain::entity::flow_definition::{FlowDefinition, FlowSlo, FlowStep};
use crate::domain::repository::FlowDefinitionRepository;

#[derive(Debug, Clone)]
pub struct CreateFlowInput {
    pub name: String,
    pub description: String,
    pub domain: String,
    pub steps: Vec<FlowStep>,
    pub slo: FlowSlo,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateFlowError {
    #[error("flow already exists: {0}")]
    AlreadyExists(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct CreateFlowUseCase {
    repo: Arc<dyn FlowDefinitionRepository>,
}

impl CreateFlowUseCase {
    pub fn new(repo: Arc<dyn FlowDefinitionRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        input: &CreateFlowInput,
    ) -> Result<FlowDefinition, CreateFlowError> {
        if input.name.is_empty() {
            return Err(CreateFlowError::Validation("name is required".to_string()));
        }
        if input.steps.is_empty() {
            return Err(CreateFlowError::Validation(
                "at least one step is required".to_string(),
            ));
        }

        let exists = self
            .repo
            .exists_by_name(input.name.clone())
            .await
            .map_err(|e| CreateFlowError::Internal(e.to_string()))?;

        if exists {
            return Err(CreateFlowError::AlreadyExists(input.name.clone()));
        }

        let flow = FlowDefinition::new(
            input.name.clone(),
            input.description.clone(),
            input.domain.clone(),
            input.steps.clone(),
            input.slo.clone(),
        );

        self.repo
            .create(&flow)
            .await
            .map_err(|e| CreateFlowError::Internal(e.to_string()))?;

        Ok(flow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::flow_definition_repository::MockFlowDefinitionRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockFlowDefinitionRepository::new();
        mock.expect_exists_by_name()
            .withf(|name| name == "order_flow")
            .returning(|_| Ok(false));
        mock.expect_create().returning(|_| Ok(()));

        let uc = CreateFlowUseCase::new(Arc::new(mock));
        let input = CreateFlowInput {
            name: "order_flow".to_string(),
            description: "test".to_string(),
            domain: "service.order".to_string(),
            steps: vec![FlowStep {
                event_type: "OrderCreated".to_string(),
                source: "order-service".to_string(),
                timeout_seconds: 0,
                description: String::new(),
            }],
            slo: FlowSlo {
                target_completion_seconds: 120,
                target_success_rate: 0.99,
                alert_on_violation: true,
            },
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "order_flow");
    }

    #[tokio::test]
    async fn already_exists() {
        let mut mock = MockFlowDefinitionRepository::new();
        mock.expect_exists_by_name().returning(|_| Ok(true));

        let uc = CreateFlowUseCase::new(Arc::new(mock));
        let input = CreateFlowInput {
            name: "existing".to_string(),
            description: "test".to_string(),
            domain: "service.order".to_string(),
            steps: vec![FlowStep {
                event_type: "OrderCreated".to_string(),
                source: "order-service".to_string(),
                timeout_seconds: 0,
                description: String::new(),
            }],
            slo: FlowSlo {
                target_completion_seconds: 120,
                target_success_rate: 0.99,
                alert_on_violation: true,
            },
        };
        let result = uc.execute(&input).await;
        assert!(matches!(result, Err(CreateFlowError::AlreadyExists(_))));
    }
}
