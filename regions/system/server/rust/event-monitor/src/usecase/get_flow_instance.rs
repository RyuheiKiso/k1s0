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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::flow_instance_repository::MockFlowInstanceRepository;

    #[tokio::test]
    async fn success() {
        let instance =
            FlowInstance::new("system".to_string(), Uuid::new_v4(), "corr-123".to_string());
        let instance_id = instance.id;
        let mut mock = MockFlowInstanceRepository::new();
        mock.expect_find_by_id().returning(move |_| {
            Ok(Some(FlowInstance::new(
                "system".to_string(),
                Uuid::new_v4(),
                "corr-123".to_string(),
            )))
        });

        let uc = GetFlowInstanceUseCase::new(Arc::new(mock));
        let result = uc.execute(&instance_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().correlation_id, "corr-123");
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockFlowInstanceRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = GetFlowInstanceUseCase::new(Arc::new(mock));
        let result = uc.execute(&Uuid::new_v4()).await;
        assert!(matches!(result, Err(GetFlowInstanceError::NotFound(_))));
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockFlowInstanceRepository::new();
        mock.expect_find_by_id()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = GetFlowInstanceUseCase::new(Arc::new(mock));
        let result = uc.execute(&Uuid::new_v4()).await;
        assert!(matches!(result, Err(GetFlowInstanceError::Internal(_))));
    }
}
