use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::policy_bundle::PolicyBundle;
use crate::domain::repository::PolicyBundleRepository;

#[derive(Debug, Clone)]
pub struct CreateBundleInput {
    pub name: String,
    pub policy_ids: Vec<Uuid>,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateBundleError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct CreateBundleUseCase {
    repo: Arc<dyn PolicyBundleRepository>,
}

impl CreateBundleUseCase {
    pub fn new(repo: Arc<dyn PolicyBundleRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        input: &CreateBundleInput,
    ) -> Result<PolicyBundle, CreateBundleError> {
        let bundle = PolicyBundle::new(input.name.clone(), input.policy_ids.clone());

        self.repo
            .create(&bundle)
            .await
            .map_err(|e| CreateBundleError::Internal(e.to_string()))?;

        Ok(bundle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::bundle_repository::MockPolicyBundleRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockPolicyBundleRepository::new();
        mock.expect_create().returning(|_| Ok(()));

        let uc = CreateBundleUseCase::new(Arc::new(mock));
        let policy_ids = vec![Uuid::new_v4(), Uuid::new_v4()];
        let input = CreateBundleInput {
            name: "security-bundle".to_string(),
            policy_ids: policy_ids.clone(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let bundle = result.unwrap();
        assert_eq!(bundle.name, "security-bundle");
        assert_eq!(bundle.policy_ids.len(), 2);
    }

    #[tokio::test]
    async fn repo_error() {
        let mut mock = MockPolicyBundleRepository::new();
        mock.expect_create()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = CreateBundleUseCase::new(Arc::new(mock));
        let input = CreateBundleInput {
            name: "fail-bundle".to_string(),
            policy_ids: vec![],
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
    }
}
