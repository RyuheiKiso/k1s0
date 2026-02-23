use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::policy::Policy;
use crate::domain::repository::PolicyRepository;

#[derive(Debug, Clone)]
pub struct UpdatePolicyInput {
    pub id: Uuid,
    pub description: Option<String>,
    pub rego_content: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, thiserror::Error)]
pub enum UpdatePolicyError {
    #[error("policy not found: {0}")]
    NotFound(Uuid),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct UpdatePolicyUseCase {
    repo: Arc<dyn PolicyRepository>,
}

impl UpdatePolicyUseCase {
    pub fn new(repo: Arc<dyn PolicyRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, input: &UpdatePolicyInput) -> Result<Policy, UpdatePolicyError> {
        let policy = self
            .repo
            .find_by_id(&input.id)
            .await
            .map_err(|e| UpdatePolicyError::Internal(e.to_string()))?
            .ok_or(UpdatePolicyError::NotFound(input.id))?;

        let mut updated = policy;
        if let Some(desc) = &input.description {
            updated.description = desc.clone();
        }
        if let Some(rego) = &input.rego_content {
            updated.rego_content = rego.clone();
        }
        if let Some(enabled) = input.enabled {
            updated.enabled = enabled;
        }
        updated.version += 1;
        updated.updated_at = chrono::Utc::now();

        self.repo
            .update(&updated)
            .await
            .map_err(|e| UpdatePolicyError::Internal(e.to_string()))?;

        Ok(updated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::policy_repository::MockPolicyRepository;

    #[tokio::test]
    async fn success() {
        let id = Uuid::new_v4();
        let id_clone = id;
        let mut mock = MockPolicyRepository::new();
        mock.expect_find_by_id()
            .withf(move |i| *i == id_clone)
            .returning(move |_| {
                Ok(Some(Policy {
                    id,
                    name: "test-policy".to_string(),
                    description: "Old description".to_string(),
                    rego_content: "package old".to_string(),
                    version: 1,
                    enabled: true,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }))
            });
        mock.expect_update().returning(|_| Ok(()));

        let uc = UpdatePolicyUseCase::new(Arc::new(mock));
        let input = UpdatePolicyInput {
            id,
            description: Some("New description".to_string()),
            rego_content: None,
            enabled: Some(false),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let policy = result.unwrap();
        assert_eq!(policy.description, "New description");
        assert_eq!(policy.rego_content, "package old");
        assert!(!policy.enabled);
        assert_eq!(policy.version, 2);
    }

    #[tokio::test]
    async fn not_found() {
        let id = Uuid::new_v4();
        let mut mock = MockPolicyRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = UpdatePolicyUseCase::new(Arc::new(mock));
        let input = UpdatePolicyInput {
            id,
            description: None,
            rego_content: None,
            enabled: None,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            UpdatePolicyError::NotFound(found_id) => assert_eq!(found_id, id),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
