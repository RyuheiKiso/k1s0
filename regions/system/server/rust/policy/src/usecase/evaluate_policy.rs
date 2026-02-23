use std::sync::Arc;

use uuid::Uuid;

use crate::domain::repository::PolicyRepository;

#[derive(Debug, Clone)]
pub struct EvaluatePolicyInput {
    pub policy_id: Uuid,
    pub input: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct EvaluatePolicyOutput {
    pub allowed: bool,
    pub reason: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum EvaluatePolicyError {
    #[error("policy not found: {0}")]
    NotFound(Uuid),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct EvaluatePolicyUseCase {
    repo: Arc<dyn PolicyRepository>,
}

impl EvaluatePolicyUseCase {
    pub fn new(repo: Arc<dyn PolicyRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        input: &EvaluatePolicyInput,
    ) -> Result<EvaluatePolicyOutput, EvaluatePolicyError> {
        let policy = self
            .repo
            .find_by_id(&input.policy_id)
            .await
            .map_err(|e| EvaluatePolicyError::Internal(e.to_string()))?
            .ok_or(EvaluatePolicyError::NotFound(input.policy_id))?;

        if policy.enabled {
            Ok(EvaluatePolicyOutput {
                allowed: true,
                reason: Some("policy is enabled".to_string()),
            })
        } else {
            Ok(EvaluatePolicyOutput {
                allowed: false,
                reason: Some("policy is disabled".to_string()),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::policy::Policy;
    use crate::domain::repository::policy_repository::MockPolicyRepository;

    #[tokio::test]
    async fn enabled_policy_allows() {
        let id = Uuid::new_v4();
        let id_clone = id;
        let mut mock = MockPolicyRepository::new();
        mock.expect_find_by_id()
            .withf(move |i| *i == id_clone)
            .returning(move |_| {
                Ok(Some(Policy {
                    id,
                    name: "allow-all".to_string(),
                    description: "Allow all".to_string(),
                    rego_content: "package authz\ndefault allow = true".to_string(),
                    version: 1,
                    enabled: true,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }))
            });

        let uc = EvaluatePolicyUseCase::new(Arc::new(mock));
        let input = EvaluatePolicyInput {
            policy_id: id,
            input: serde_json::json!({"action": "read"}),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.allowed);
        assert!(output.reason.is_some());
    }

    #[tokio::test]
    async fn disabled_policy_denies() {
        let id = Uuid::new_v4();
        let id_clone = id;
        let mut mock = MockPolicyRepository::new();
        mock.expect_find_by_id()
            .withf(move |i| *i == id_clone)
            .returning(move |_| {
                Ok(Some(Policy {
                    id,
                    name: "deny-all".to_string(),
                    description: "Deny all".to_string(),
                    rego_content: "package authz\ndefault allow = false".to_string(),
                    version: 1,
                    enabled: false,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }))
            });

        let uc = EvaluatePolicyUseCase::new(Arc::new(mock));
        let input = EvaluatePolicyInput {
            policy_id: id,
            input: serde_json::json!({"action": "write"}),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(!output.allowed);
    }

    #[tokio::test]
    async fn policy_not_found() {
        let id = Uuid::new_v4();
        let mut mock = MockPolicyRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = EvaluatePolicyUseCase::new(Arc::new(mock));
        let input = EvaluatePolicyInput {
            policy_id: id,
            input: serde_json::json!({}),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            EvaluatePolicyError::NotFound(found_id) => assert_eq!(found_id, id),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
