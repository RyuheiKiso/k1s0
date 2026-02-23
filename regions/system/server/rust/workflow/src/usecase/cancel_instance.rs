use std::sync::Arc;

use crate::domain::entity::workflow_instance::WorkflowInstance;
use crate::domain::repository::WorkflowInstanceRepository;

#[derive(Debug, Clone)]
pub struct CancelInstanceInput {
    pub id: String,
    pub reason: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum CancelInstanceError {
    #[error("instance not found: {0}")]
    NotFound(String),

    #[error("invalid status for cancel: instance {0} has status '{1}'")]
    InvalidStatus(String, String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct CancelInstanceUseCase {
    repo: Arc<dyn WorkflowInstanceRepository>,
}

impl CancelInstanceUseCase {
    pub fn new(repo: Arc<dyn WorkflowInstanceRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        input: &CancelInstanceInput,
    ) -> Result<WorkflowInstance, CancelInstanceError> {
        let mut instance = self
            .repo
            .find_by_id(&input.id)
            .await
            .map_err(|e| CancelInstanceError::Internal(e.to_string()))?
            .ok_or_else(|| CancelInstanceError::NotFound(input.id.clone()))?;

        if !instance.is_cancellable() {
            return Err(CancelInstanceError::InvalidStatus(
                input.id.clone(),
                instance.status.clone(),
            ));
        }

        instance.cancel();

        self.repo
            .update(&instance)
            .await
            .map_err(|e| CancelInstanceError::Internal(e.to_string()))?;

        Ok(instance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::workflow_instance_repository::MockWorkflowInstanceRepository;

    fn running_instance() -> WorkflowInstance {
        WorkflowInstance::new(
            "inst_001".to_string(),
            "wf_001".to_string(),
            "test".to_string(),
            "title".to_string(),
            "user-001".to_string(),
            Some("step-1".to_string()),
            serde_json::json!({}),
        )
    }

    #[tokio::test]
    async fn success() {
        let mut mock = MockWorkflowInstanceRepository::new();
        mock.expect_find_by_id()
            .returning(|_| Ok(Some(running_instance())));
        mock.expect_update().returning(|_| Ok(()));

        let uc = CancelInstanceUseCase::new(Arc::new(mock));
        let input = CancelInstanceInput {
            id: "inst_001".to_string(),
            reason: Some("cancelled by user".to_string()),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, "cancelled");
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockWorkflowInstanceRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = CancelInstanceUseCase::new(Arc::new(mock));
        let input = CancelInstanceInput {
            id: "inst_missing".to_string(),
            reason: None,
        };
        let result = uc.execute(&input).await;
        assert!(matches!(
            result.unwrap_err(),
            CancelInstanceError::NotFound(_)
        ));
    }

    #[tokio::test]
    async fn invalid_status_completed() {
        let mut mock = MockWorkflowInstanceRepository::new();
        let mut inst = running_instance();
        inst.complete();
        mock.expect_find_by_id()
            .returning(move |_| Ok(Some(inst.clone())));

        let uc = CancelInstanceUseCase::new(Arc::new(mock));
        let input = CancelInstanceInput {
            id: "inst_001".to_string(),
            reason: None,
        };
        let result = uc.execute(&input).await;
        assert!(matches!(
            result.unwrap_err(),
            CancelInstanceError::InvalidStatus(_, _)
        ));
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockWorkflowInstanceRepository::new();
        mock.expect_find_by_id()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = CancelInstanceUseCase::new(Arc::new(mock));
        let input = CancelInstanceInput {
            id: "inst_001".to_string(),
            reason: None,
        };
        let result = uc.execute(&input).await;
        assert!(matches!(
            result.unwrap_err(),
            CancelInstanceError::Internal(_)
        ));
    }
}
