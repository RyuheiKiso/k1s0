use std::sync::Arc;

use chrono::Utc;

use crate::domain::entity::quota::{Period, QuotaPolicy, SubjectType};
use crate::domain::repository::QuotaPolicyRepository;

#[derive(Debug, Clone)]
pub struct UpdateQuotaPolicyInput {
    pub id: String,
    pub name: String,
    pub subject_type: String,
    pub subject_id: String,
    pub limit: u64,
    pub period: String,
    pub enabled: bool,
    pub alert_threshold_percent: Option<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateQuotaPolicyError {
    #[error("quota policy not found: {0}")]
    NotFound(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct UpdateQuotaPolicyUseCase {
    repo: Arc<dyn QuotaPolicyRepository>,
}

impl UpdateQuotaPolicyUseCase {
    pub fn new(repo: Arc<dyn QuotaPolicyRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        input: &UpdateQuotaPolicyInput,
    ) -> Result<QuotaPolicy, UpdateQuotaPolicyError> {
        let subject_type = SubjectType::from_str(&input.subject_type).ok_or_else(|| {
            UpdateQuotaPolicyError::Validation(format!(
                "subject_type must be one of: tenant, user, api_key, got: {}",
                input.subject_type
            ))
        })?;

        let period = Period::from_str(&input.period).ok_or_else(|| {
            UpdateQuotaPolicyError::Validation(format!(
                "period must be one of: daily, monthly, got: {}",
                input.period
            ))
        })?;

        if input.limit == 0 {
            return Err(UpdateQuotaPolicyError::Validation(
                "limit must be greater than 0".to_string(),
            ));
        }

        let mut policy = self
            .repo
            .find_by_id(&input.id)
            .await
            .map_err(|e| UpdateQuotaPolicyError::Internal(e.to_string()))?
            .ok_or_else(|| UpdateQuotaPolicyError::NotFound(input.id.clone()))?;

        policy.name = input.name.clone();
        policy.subject_type = subject_type;
        policy.subject_id = input.subject_id.clone();
        policy.limit = input.limit;
        policy.period = period;
        policy.enabled = input.enabled;
        policy.alert_threshold_percent = input.alert_threshold_percent;
        policy.updated_at = Utc::now();

        self.repo
            .update(&policy)
            .await
            .map_err(|e| UpdateQuotaPolicyError::Internal(e.to_string()))?;

        Ok(policy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::quota::{Period, SubjectType};
    use crate::domain::repository::quota_repository::MockQuotaPolicyRepository;

    fn sample_policy() -> QuotaPolicy {
        QuotaPolicy::new(
            "original".to_string(),
            SubjectType::Tenant,
            "tenant-1".to_string(),
            1000,
            Period::Daily,
            true,
            None,
        )
    }

    #[tokio::test]
    async fn success() {
        let mut mock = MockQuotaPolicyRepository::new();
        let policy = sample_policy();
        let policy_id = policy.id.clone();
        let return_policy = policy.clone();

        mock.expect_find_by_id()
            .withf(move |id| id == policy_id)
            .returning(move |_| Ok(Some(return_policy.clone())));
        mock.expect_update().returning(|_| Ok(()));

        let uc = UpdateQuotaPolicyUseCase::new(Arc::new(mock));
        let input = UpdateQuotaPolicyInput {
            id: policy.id.clone(),
            name: "updated".to_string(),
            subject_type: "user".to_string(),
            subject_id: "user-1".to_string(),
            limit: 5000,
            period: "monthly".to_string(),
            enabled: false,
            alert_threshold_percent: Some(90),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let updated = result.unwrap();
        assert_eq!(updated.name, "updated");
        assert_eq!(updated.subject_type, SubjectType::User);
        assert_eq!(updated.limit, 5000);
        assert_eq!(updated.period, Period::Monthly);
        assert!(!updated.enabled);
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockQuotaPolicyRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = UpdateQuotaPolicyUseCase::new(Arc::new(mock));
        let input = UpdateQuotaPolicyInput {
            id: "nonexistent".to_string(),
            name: "test".to_string(),
            subject_type: "tenant".to_string(),
            subject_id: "id".to_string(),
            limit: 100,
            period: "daily".to_string(),
            enabled: true,
            alert_threshold_percent: None,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            UpdateQuotaPolicyError::NotFound(id) => assert_eq!(id, "nonexistent"),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn validation_error() {
        let mock = MockQuotaPolicyRepository::new();
        let uc = UpdateQuotaPolicyUseCase::new(Arc::new(mock));
        let input = UpdateQuotaPolicyInput {
            id: "some-id".to_string(),
            name: "test".to_string(),
            subject_type: "invalid".to_string(),
            subject_id: "id".to_string(),
            limit: 100,
            period: "daily".to_string(),
            enabled: true,
            alert_threshold_percent: None,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            UpdateQuotaPolicyError::Validation(msg) => assert!(msg.contains("subject_type")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockQuotaPolicyRepository::new();
        mock.expect_find_by_id()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = UpdateQuotaPolicyUseCase::new(Arc::new(mock));
        let input = UpdateQuotaPolicyInput {
            id: "some-id".to_string(),
            name: "test".to_string(),
            subject_type: "tenant".to_string(),
            subject_id: "id".to_string(),
            limit: 100,
            period: "daily".to_string(),
            enabled: true,
            alert_threshold_percent: None,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            UpdateQuotaPolicyError::Internal(msg) => assert!(msg.contains("db error")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
