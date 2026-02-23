use std::sync::Arc;

use chrono::{Datelike, TimeZone, Utc};

use crate::domain::entity::quota::{Period, QuotaUsage};
use crate::domain::repository::{QuotaPolicyRepository, QuotaUsageRepository};

#[derive(Debug, thiserror::Error)]
pub enum GetQuotaUsageError {
    #[error("quota policy not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct GetQuotaUsageUseCase {
    policy_repo: Arc<dyn QuotaPolicyRepository>,
    usage_repo: Arc<dyn QuotaUsageRepository>,
}

impl GetQuotaUsageUseCase {
    pub fn new(
        policy_repo: Arc<dyn QuotaPolicyRepository>,
        usage_repo: Arc<dyn QuotaUsageRepository>,
    ) -> Self {
        Self {
            policy_repo,
            usage_repo,
        }
    }

    pub async fn execute(&self, quota_id: &str) -> Result<QuotaUsage, GetQuotaUsageError> {
        let policy = self
            .policy_repo
            .find_by_id(quota_id)
            .await
            .map_err(|e| GetQuotaUsageError::Internal(e.to_string()))?
            .ok_or_else(|| GetQuotaUsageError::NotFound(quota_id.to_string()))?;

        let used = self
            .usage_repo
            .get_usage(quota_id)
            .await
            .map_err(|e| GetQuotaUsageError::Internal(e.to_string()))?
            .unwrap_or(0);

        let now = Utc::now();
        let (period_start, period_end, reset_at) = match policy.period {
            Period::Daily => {
                let start = Utc
                    .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
                    .unwrap();
                let end = start + chrono::Duration::days(1) - chrono::Duration::milliseconds(1);
                let reset = start + chrono::Duration::days(1);
                (start, end, reset)
            }
            Period::Monthly => {
                let start = Utc
                    .with_ymd_and_hms(now.year(), now.month(), 1, 0, 0, 0)
                    .unwrap();
                let next_month = if now.month() == 12 {
                    Utc.with_ymd_and_hms(now.year() + 1, 1, 1, 0, 0, 0)
                        .unwrap()
                } else {
                    Utc.with_ymd_and_hms(now.year(), now.month() + 1, 1, 0, 0, 0)
                        .unwrap()
                };
                let end = next_month - chrono::Duration::milliseconds(1);
                (start, end, next_month)
            }
        };

        Ok(QuotaUsage::new(&policy, used, period_start, period_end, reset_at))
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
            Some(80),
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

        usage_mock
            .expect_get_usage()
            .returning(|_| Ok(Some(7500)));

        let uc = GetQuotaUsageUseCase::new(Arc::new(policy_mock), Arc::new(usage_mock));
        let result = uc.execute(&policy.id).await;
        assert!(result.is_ok());

        let usage = result.unwrap();
        assert_eq!(usage.used, 7500);
        assert_eq!(usage.remaining, 2500);
        assert!(!usage.exceeded);
    }

    #[tokio::test]
    async fn not_found() {
        let mut policy_mock = MockQuotaPolicyRepository::new();
        let usage_mock = MockQuotaUsageRepository::new();

        policy_mock
            .expect_find_by_id()
            .returning(|_| Ok(None));

        let uc = GetQuotaUsageUseCase::new(Arc::new(policy_mock), Arc::new(usage_mock));
        let result = uc.execute("nonexistent").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            GetQuotaUsageError::NotFound(id) => assert_eq!(id, "nonexistent"),
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

        let uc = GetQuotaUsageUseCase::new(Arc::new(policy_mock), Arc::new(usage_mock));
        let result = uc.execute("some-id").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            GetQuotaUsageError::Internal(msg) => assert!(msg.contains("db error")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn zero_usage() {
        let mut policy_mock = MockQuotaPolicyRepository::new();
        let mut usage_mock = MockQuotaUsageRepository::new();

        let policy = sample_policy();
        let policy_id = policy.id.clone();
        let return_policy = policy.clone();

        policy_mock
            .expect_find_by_id()
            .withf(move |id| id == policy_id)
            .returning(move |_| Ok(Some(return_policy.clone())));

        usage_mock
            .expect_get_usage()
            .returning(|_| Ok(None));

        let uc = GetQuotaUsageUseCase::new(Arc::new(policy_mock), Arc::new(usage_mock));
        let result = uc.execute(&policy.id).await;
        assert!(result.is_ok());

        let usage = result.unwrap();
        assert_eq!(usage.used, 0);
        assert_eq!(usage.remaining, 10000);
        assert!(!usage.exceeded);
    }
}
