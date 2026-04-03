use std::sync::Arc;

use crate::domain::entity::workflow_instance::WorkflowInstance;
use crate::domain::repository::WorkflowInstanceRepository;

// RUST-CRIT-001 対応: テナント分離のため tenant_id フィールドを追加する
#[derive(Debug, Clone)]
pub struct GetInstanceInput {
    pub tenant_id: String,
    pub id: String,
}

#[derive(Debug, thiserror::Error)]
pub enum GetInstanceError {
    #[error("instance not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct GetInstanceUseCase {
    repo: Arc<dyn WorkflowInstanceRepository>,
}

impl GetInstanceUseCase {
    pub fn new(repo: Arc<dyn WorkflowInstanceRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        input: &GetInstanceInput,
    ) -> Result<WorkflowInstance, GetInstanceError> {
        // テナント分離: tenant_id を渡してRLSによるフィルタリングを有効化する
        self.repo
            .find_by_id(&input.tenant_id, &input.id)
            .await
            .map_err(|e| GetInstanceError::Internal(e.to_string()))?
            .ok_or_else(|| GetInstanceError::NotFound(input.id.clone()))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::workflow_instance_repository::MockWorkflowInstanceRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockWorkflowInstanceRepository::new();
        mock.expect_find_by_id().returning(|_, _| {
            Ok(Some(WorkflowInstance::new(
                "inst_001".to_string(),
                "wf_001".to_string(),
                "test".to_string(),
                "title".to_string(),
                "user-001".to_string(),
                Some("step-1".to_string()),
                serde_json::json!({}),
            )))
        });

        let uc = GetInstanceUseCase::new(Arc::new(mock));
        let input = GetInstanceInput {
            tenant_id: "test-tenant".to_string(),
            id: "inst_001".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, "inst_001");
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockWorkflowInstanceRepository::new();
        mock.expect_find_by_id().returning(|_, _| Ok(None));

        let uc = GetInstanceUseCase::new(Arc::new(mock));
        let input = GetInstanceInput {
            tenant_id: "test-tenant".to_string(),
            id: "inst_missing".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(matches!(result.unwrap_err(), GetInstanceError::NotFound(_)));
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockWorkflowInstanceRepository::new();
        mock.expect_find_by_id()
            .returning(|_, _| Err(anyhow::anyhow!("db error")));

        let uc = GetInstanceUseCase::new(Arc::new(mock));
        let input = GetInstanceInput {
            tenant_id: "test-tenant".to_string(),
            id: "inst_001".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(matches!(result.unwrap_err(), GetInstanceError::Internal(_)));
    }
}
