use std::sync::Arc;

use uuid::Uuid;

use crate::domain::repository::PolicyRepository;
use crate::infrastructure::opa_client::OpaClient;

#[derive(Debug, Clone)]
pub struct EvaluatePolicyInput {
    pub policy_id: Option<Uuid>,
    pub package_path: String,
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
    opa_client: Option<Arc<OpaClient>>,
}

impl EvaluatePolicyUseCase {
    pub fn new(repo: Arc<dyn PolicyRepository>, opa_client: Option<Arc<OpaClient>>) -> Self {
        Self { repo, opa_client }
    }

    pub async fn execute(
        &self,
        input: &EvaluatePolicyInput,
    ) -> Result<EvaluatePolicyOutput, EvaluatePolicyError> {
        // OPA client available: evaluate via OPA HTTP API
        if let Some(ref opa) = self.opa_client {
            return match opa.evaluate(&input.package_path, &input.input).await {
                Ok(allowed) => {
                    let reason = if allowed {
                        "OPA evaluation: allowed"
                    } else {
                        "OPA evaluation: denied"
                    };
                    Ok(EvaluatePolicyOutput {
                        allowed,
                        reason: Some(reason.to_string()),
                    })
                }
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        package_path = %input.package_path,
                        "OPA evaluation failed, deny by default"
                    );
                    Ok(EvaluatePolicyOutput {
                        allowed: false,
                        reason: Some(format!("OPA evaluation error: {}", e)),
                    })
                }
            };
        }

        // Fallback: use policy.enabled flag from repository
        let policy_id = input.policy_id.ok_or_else(|| {
            EvaluatePolicyError::Internal(
                "no OPA client configured and no policy_id provided".to_string(),
            )
        })?;

        let policy = self
            .repo
            .find_by_id(&policy_id)
            .await
            .map_err(|e| EvaluatePolicyError::Internal(e.to_string()))?
            .ok_or(EvaluatePolicyError::NotFound(policy_id))?;

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

        let uc = EvaluatePolicyUseCase::new(Arc::new(mock), None);
        let input = EvaluatePolicyInput {
            policy_id: Some(id),
            package_path: String::new(),
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

        let uc = EvaluatePolicyUseCase::new(Arc::new(mock), None);
        let input = EvaluatePolicyInput {
            policy_id: Some(id),
            package_path: String::new(),
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

        let uc = EvaluatePolicyUseCase::new(Arc::new(mock), None);
        let input = EvaluatePolicyInput {
            policy_id: Some(id),
            package_path: String::new(),
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
