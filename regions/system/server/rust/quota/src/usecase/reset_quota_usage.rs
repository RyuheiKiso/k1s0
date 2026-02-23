use std::sync::Arc;

use chrono::Utc;

use crate::domain::repository::{QuotaPolicyRepository, QuotaUsageRepository};

#[derive(Debug, Clone)]
pub struct ResetQuotaUsageInput {
    pub quota_id: String,
    pub reason: String,
    pub reset_by: String,
}

#[derive(Debug, Clone)]
pub struct ResetQuotaUsageOutput {
    pub quota_id: String,
    pub used: u64,
    pub reset_at: String,
    pub reset_by: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ResetQuotaUsageError {
    #[error("quota policy not found: {0}")]
    NotFound(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct ResetQuotaUsageUseCase {
    policy_repo: Arc<dyn QuotaPolicyRepository>,
    usage_repo: Arc<dyn QuotaUsageRepository>,
}

impl ResetQuotaUsageUseCase {
    pub fn new(
        policy_repo: Arc<dyn QuotaPolicyRepository>,
        usage_repo: Arc<dyn QuotaUsageRepository>,
    ) -> Self {
        Self {
            policy_repo,
            usage_repo,
        }
    }

    pub async fn execute(
        &self,
        input: &ResetQuotaUsageInput,
    ) -> Result<ResetQuotaUsageOutput, ResetQuotaUsageError> {
        if input.reason.is_empty() {
            return Err(ResetQuotaUsageError::Validation(
                "reason is required".to_string(),
            ));
        }

        let _policy = self
            .policy_repo
            .find_by_id(&input.quota_id)
            .await
            .map_err(|e| ResetQuotaUsageError::Internal(e.to_string()))?
            .ok_or_else(|| ResetQuotaUsageError::NotFound(input.quota_id.clone()))?;

        self.usage_repo
            .reset(&input.quota_id)
            .await
            .map_err(|e| ResetQuotaUsageError::Internal(e.to_string()))?;

        Ok(ResetQuotaUsageOutput {
            quota_id: input.quota_id.clone(),
            used: 0,
            reset_at: Utc::now().to_rfc3339(),
            reset_by: input.reset_by.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::quota::{Period, SubjectType};
    use crate::domain::repository::quota_repository::{
        MockQuotaPolicyRepository, MockQuotaUsageRepository,
    };

    fn sample_policy() -> crate::domain::entity::quota::QuotaPolicy {
        crate::domain::entity::quota::QuotaPolicy::new(
            "test".to_string(),
            SubjectType::Tenant,
            "tenant-1".to_string(),
            10000,
            Period::Daily,
            true,
            None,
        )
    }

    #[tokio::test]
    async fn success() {
        let mut policy_mock = MockQuotaPolicyRepository::new();
        let mut usage_mock = MockQuotaUsageRepository::new();

        let policy = sample_policy();
        let policy_id = policy.id.clone();
        let return_policy = policy.clone();

        policy_mock
            .expect_find_by_id()
            .withf(move |id| id == policy_id)
            .returning(move |_| Ok(Some(return_policy.clone())));

        usage_mock.expect_reset().returning(|_| Ok(()));

        let uc = ResetQuotaUsageUseCase::new(Arc::new(policy_mock), Arc::new(usage_mock));
        let input = ResetQuotaUsageInput {
            quota_id: policy.id.clone(),
            reason: "plan change".to_string(),
            reset_by: "admin@example.com".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.used, 0);
        assert_eq!(output.reset_by, "admin@example.com");
    }

    #[tokio::test]
    async fn not_found() {
        let mut policy_mock = MockQuotaPolicyRepository::new();
        let usage_mock = MockQuotaUsageRepository::new();

        policy_mock
            .expect_find_by_id()
            .returning(|_| Ok(None));

        let uc = ResetQuotaUsageUseCase::new(Arc::new(policy_mock), Arc::new(usage_mock));
        let input = ResetQuotaUsageInput {
            quota_id: "nonexistent".to_string(),
            reason: "test".to_string(),
            reset_by: "admin".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ResetQuotaUsageError::NotFound(id) => assert_eq!(id, "nonexistent"),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn validation_error_empty_reason() {
        let policy_mock = MockQuotaPolicyRepository::new();
        let usage_mock = MockQuotaUsageRepository::new();

        let uc = ResetQuotaUsageUseCase::new(Arc::new(policy_mock), Arc::new(usage_mock));
        let input = ResetQuotaUsageInput {
            quota_id: "some-id".to_string(),
            reason: "".to_string(),
            reset_by: "admin".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ResetQuotaUsageError::Validation(msg) => assert!(msg.contains("reason")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn internal_error() {
        let mut policy_mock = MockQuotaPolicyRepository::new();
        let usage_mock = MockQuotaUsageRepository::new();

        policy_mock
            .expect_find_by_id()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = ResetQuotaUsageUseCase::new(Arc::new(policy_mock), Arc::new(usage_mock));
        let input = ResetQuotaUsageInput {
            quota_id: "some-id".to_string(),
            reason: "test".to_string(),
            reset_by: "admin".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ResetQuotaUsageError::Internal(msg) => assert!(msg.contains("db error")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
