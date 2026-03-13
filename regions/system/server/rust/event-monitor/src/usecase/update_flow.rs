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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::flow_definition::{FlowSlo, FlowStep};
    use crate::domain::repository::flow_definition_repository::MockFlowDefinitionRepository;

    fn make_flow() -> FlowDefinition {
        FlowDefinition::new(
            "order_flow".to_string(),
            "original description".to_string(),
            "service.order".to_string(),
            vec![FlowStep {
                event_type: "OrderCreated".to_string(),
                source: "order-service".to_string(),
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
    async fn success() {
        let flow = make_flow();
        let flow_id = flow.id;
        let mut mock = MockFlowDefinitionRepository::new();
        mock.expect_find_by_id()
            .returning(move |_| Ok(Some(make_flow())));
        mock.expect_update().returning(|_| Ok(()));

        let uc = UpdateFlowUseCase::new(Arc::new(mock));
        let input = UpdateFlowInput {
            id: flow_id,
            description: Some("updated description".to_string()),
            steps: None,
            slo: None,
            enabled: Some(false),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.description, "updated description");
        assert!(!updated.enabled);
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockFlowDefinitionRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = UpdateFlowUseCase::new(Arc::new(mock));
        let input = UpdateFlowInput {
            id: Uuid::new_v4(),
            description: None,
            steps: None,
            slo: None,
            enabled: None,
        };
        let result = uc.execute(&input).await;
        assert!(matches!(result, Err(UpdateFlowError::NotFound(_))));
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockFlowDefinitionRepository::new();
        mock.expect_find_by_id()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = UpdateFlowUseCase::new(Arc::new(mock));
        let input = UpdateFlowInput {
            id: Uuid::new_v4(),
            description: None,
            steps: None,
            slo: None,
            enabled: None,
        };
        let result = uc.execute(&input).await;
        assert!(matches!(result, Err(UpdateFlowError::Internal(_))));
    }
}
