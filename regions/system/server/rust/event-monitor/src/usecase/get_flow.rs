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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::flow_definition::{FlowSlo, FlowStep};
    use crate::domain::repository::flow_definition_repository::MockFlowDefinitionRepository;

    fn make_flow() -> FlowDefinition {
        FlowDefinition::new(
            "order_flow".to_string(),
            "test".to_string(),
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

        let uc = GetFlowUseCase::new(Arc::new(mock));
        let result = uc.execute(&flow_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "order_flow");
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockFlowDefinitionRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = GetFlowUseCase::new(Arc::new(mock));
        let result = uc.execute(&Uuid::new_v4()).await;
        assert!(matches!(result, Err(GetFlowError::NotFound(_))));
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockFlowDefinitionRepository::new();
        mock.expect_find_by_id()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = GetFlowUseCase::new(Arc::new(mock));
        let result = uc.execute(&Uuid::new_v4()).await;
        assert!(matches!(result, Err(GetFlowError::Internal(_))));
    }
}
