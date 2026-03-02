use std::sync::Arc;

use crate::domain::entity::policy::Policy;
use crate::domain::repository::PolicyRepository;

#[derive(Debug, Clone)]
pub struct CreatePolicyInput {
    pub name: String,
    pub description: String,
    pub rego_content: String,
    pub package_path: String,
    pub bundle_id: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum CreatePolicyError {
    #[error("policy already exists: {0}")]
    AlreadyExists(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct CreatePolicyUseCase {
    repo: Arc<dyn PolicyRepository>,
}

impl CreatePolicyUseCase {
    pub fn new(repo: Arc<dyn PolicyRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, input: &CreatePolicyInput) -> Result<Policy, CreatePolicyError> {
        let exists = self
            .repo
            .exists_by_name(&input.name)
            .await
            .map_err(|e| CreatePolicyError::Internal(e.to_string()))?;

        if exists {
            return Err(CreatePolicyError::AlreadyExists(input.name.clone()));
        }

        let mut policy = Policy::new(
            input.name.clone(),
            input.description.clone(),
            input.rego_content.clone(),
        );
        policy.package_path = input.package_path.clone();
        policy.bundle_id = input.bundle_id.clone();

        self.repo
            .create(&policy)
            .await
            .map_err(|e| CreatePolicyError::Internal(e.to_string()))?;

        Ok(policy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::policy_repository::MockPolicyRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockPolicyRepository::new();
        mock.expect_exists_by_name()
            .withf(|name| name == "allow-read")
            .returning(|_| Ok(false));
        mock.expect_create().returning(|_| Ok(()));

        let uc = CreatePolicyUseCase::new(Arc::new(mock));
        let input = CreatePolicyInput {
            name: "allow-read".to_string(),
            description: "Allow read access".to_string(),
            rego_content: "package authz\ndefault allow = true".to_string(),
            package_path: "k1s0.system.authz".to_string(),
            bundle_id: Some("bundle-1".to_string()),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let policy = result.unwrap();
        assert_eq!(policy.name, "allow-read");
        assert_eq!(policy.package_path, "k1s0.system.authz");
        assert_eq!(policy.bundle_id.as_deref(), Some("bundle-1"));
        assert_eq!(policy.version, 1);
        assert!(policy.enabled);
    }

    #[tokio::test]
    async fn already_exists() {
        let mut mock = MockPolicyRepository::new();
        mock.expect_exists_by_name()
            .withf(|name| name == "existing-policy")
            .returning(|_| Ok(true));

        let uc = CreatePolicyUseCase::new(Arc::new(mock));
        let input = CreatePolicyInput {
            name: "existing-policy".to_string(),
            description: "Existing".to_string(),
            rego_content: "package authz".to_string(),
            package_path: "k1s0.system.authz".to_string(),
            bundle_id: None,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            CreatePolicyError::AlreadyExists(name) => assert_eq!(name, "existing-policy"),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
