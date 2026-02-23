use std::sync::Arc;

use crate::domain::entity::quota::{Period, QuotaPolicy, SubjectType};
use crate::domain::repository::QuotaPolicyRepository;

#[derive(Debug, Clone)]
pub struct CreateQuotaPolicyInput {
    pub name: String,
    pub subject_type: String,
    pub subject_id: String,
    pub limit: u64,
    pub period: String,
    pub enabled: bool,
    pub alert_threshold_percent: Option<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateQuotaPolicyError {
    #[error("validation error: {0}")]
    Validation(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct CreateQuotaPolicyUseCase {
    repo: Arc<dyn QuotaPolicyRepository>,
}

impl CreateQuotaPolicyUseCase {
    pub fn new(repo: Arc<dyn QuotaPolicyRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        input: &CreateQuotaPolicyInput,
    ) -> Result<QuotaPolicy, CreateQuotaPolicyError> {
        let subject_type = SubjectType::from_str(&input.subject_type)
            .ok_or_else(|| {
                CreateQuotaPolicyError::Validation(format!(
                    "subject_type must be one of: tenant, user, api_key, got: {}",
                    input.subject_type
                ))
            })?;

        let period = Period::from_str(&input.period)
            .ok_or_else(|| {
                CreateQuotaPolicyError::Validation(format!(
                    "period must be one of: daily, monthly, got: {}",
                    input.period
                ))
            })?;

        if input.limit == 0 {
            return Err(CreateQuotaPolicyError::Validation(
                "limit must be greater than 0".to_string(),
            ));
        }

        if let Some(threshold) = input.alert_threshold_percent {
            if threshold > 100 {
                return Err(CreateQuotaPolicyError::Validation(
                    "alert_threshold_percent must be between 0 and 100".to_string(),
                ));
            }
        }

        let policy = QuotaPolicy::new(
            input.name.clone(),
            subject_type,
            input.subject_id.clone(),
            input.limit,
            period,
            input.enabled,
            input.alert_threshold_percent,
        );

        self.repo
            .create(&policy)
            .await
            .map_err(|e| CreateQuotaPolicyError::Internal(e.to_string()))?;

        Ok(policy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::quota_repository::MockQuotaPolicyRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockQuotaPolicyRepository::new();
        mock.expect_create().returning(|_| Ok(()));

        let uc = CreateQuotaPolicyUseCase::new(Arc::new(mock));
        let input = CreateQuotaPolicyInput {
            name: "Standard Plan".to_string(),
            subject_type: "tenant".to_string(),
            subject_id: "tenant-abc".to_string(),
            limit: 10000,
            period: "daily".to_string(),
            enabled: true,
            alert_threshold_percent: Some(80),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let policy = result.unwrap();
        assert_eq!(policy.name, "Standard Plan");
        assert_eq!(policy.subject_type, SubjectType::Tenant);
        assert_eq!(policy.limit, 10000);
        assert_eq!(policy.period, Period::Daily);
        assert!(policy.enabled);
        assert_eq!(policy.alert_threshold_percent, Some(80));
    }

    #[tokio::test]
    async fn invalid_subject_type() {
        let mock = MockQuotaPolicyRepository::new();
        let uc = CreateQuotaPolicyUseCase::new(Arc::new(mock));
        let input = CreateQuotaPolicyInput {
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
            CreateQuotaPolicyError::Validation(msg) => {
                assert!(msg.contains("subject_type"));
            }
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn invalid_period() {
        let mock = MockQuotaPolicyRepository::new();
        let uc = CreateQuotaPolicyUseCase::new(Arc::new(mock));
        let input = CreateQuotaPolicyInput {
            name: "test".to_string(),
            subject_type: "tenant".to_string(),
            subject_id: "id".to_string(),
            limit: 100,
            period: "weekly".to_string(),
            enabled: true,
            alert_threshold_percent: None,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CreateQuotaPolicyError::Validation(msg) => {
                assert!(msg.contains("period"));
            }
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn zero_limit() {
        let mock = MockQuotaPolicyRepository::new();
        let uc = CreateQuotaPolicyUseCase::new(Arc::new(mock));
        let input = CreateQuotaPolicyInput {
            name: "test".to_string(),
            subject_type: "tenant".to_string(),
            subject_id: "id".to_string(),
            limit: 0,
            period: "daily".to_string(),
            enabled: true,
            alert_threshold_percent: None,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CreateQuotaPolicyError::Validation(msg) => {
                assert!(msg.contains("limit"));
            }
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockQuotaPolicyRepository::new();
        mock.expect_create()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = CreateQuotaPolicyUseCase::new(Arc::new(mock));
        let input = CreateQuotaPolicyInput {
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
            CreateQuotaPolicyError::Internal(msg) => {
                assert!(msg.contains("db error"));
            }
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
