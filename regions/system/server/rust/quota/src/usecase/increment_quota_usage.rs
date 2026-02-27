use std::sync::Arc;

use crate::domain::entity::quota::IncrementResult;
use crate::domain::repository::{CheckAndIncrementResult, QuotaPolicyRepository, QuotaUsageRepository};
use crate::infrastructure::kafka_producer::{
    QuotaEventPublisher, QuotaExceededEvent, QuotaThresholdReachedEvent,
};

#[derive(Debug, Clone)]
pub struct IncrementQuotaUsageInput {
    pub quota_id: String,
    pub amount: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum IncrementQuotaUsageError {
    #[error("quota policy not found: {0}")]
    NotFound(String),

    #[error("quota exceeded for {subject_id}: {used}/{limit} ({period})")]
    Exceeded {
        quota_id: String,
        subject_id: String,
        used: u64,
        limit: u64,
        period: String,
    },

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct IncrementQuotaUsageUseCase {
    policy_repo: Arc<dyn QuotaPolicyRepository>,
    usage_repo: Arc<dyn QuotaUsageRepository>,
    event_publisher: Arc<dyn QuotaEventPublisher>,
}

impl IncrementQuotaUsageUseCase {
    pub fn new(
        policy_repo: Arc<dyn QuotaPolicyRepository>,
        usage_repo: Arc<dyn QuotaUsageRepository>,
        event_publisher: Arc<dyn QuotaEventPublisher>,
    ) -> Self {
        Self {
            policy_repo,
            usage_repo,
            event_publisher,
        }
    }

    pub fn new_without_publisher(
        policy_repo: Arc<dyn QuotaPolicyRepository>,
        usage_repo: Arc<dyn QuotaUsageRepository>,
    ) -> Self {
        use crate::infrastructure::kafka_producer::NoopQuotaEventPublisher;
        Self {
            policy_repo,
            usage_repo,
            event_publisher: Arc::new(NoopQuotaEventPublisher),
        }
    }

    pub async fn execute(
        &self,
        input: &IncrementQuotaUsageInput,
    ) -> Result<IncrementResult, IncrementQuotaUsageError> {
        // 1. ポリシーを取得してリミットを確認
        let policy = self
            .policy_repo
            .find_by_id(&input.quota_id)
            .await
            .map_err(|e| IncrementQuotaUsageError::Internal(e.to_string()))?
            .ok_or_else(|| IncrementQuotaUsageError::NotFound(input.quota_id.clone()))?;

        // 2. アトミックに check-and-increment を実行
        let CheckAndIncrementResult { used, allowed } = self
            .usage_repo
            .check_and_increment(&input.quota_id, input.amount, policy.limit)
            .await
            .map_err(|e| IncrementQuotaUsageError::Internal(e.to_string()))?;

        // 3. 許可されなかった場合は超過エラー
        if !allowed {
            let event = QuotaExceededEvent {
                event_type: "QUOTA_EXCEEDED".to_string(),
                quota_id: policy.id.clone(),
                subject_type: policy.subject_type.as_str().to_string(),
                subject_id: policy.subject_id.clone(),
                period: policy.period.as_str().to_string(),
                limit: policy.limit,
                used,
                exceeded_at: chrono::Utc::now().to_rfc3339(),
                reset_at: "".to_string(),
            };
            let _ = self.event_publisher.publish_quota_exceeded(&event).await;

            return Err(IncrementQuotaUsageError::Exceeded {
                quota_id: policy.id,
                subject_id: policy.subject_id,
                used,
                limit: policy.limit,
                period: policy.period.as_str().to_string(),
            });
        }

        // 4. 使用率を計算
        let remaining = if used >= policy.limit {
            0
        } else {
            policy.limit - used
        };
        let usage_percent = if policy.limit == 0 {
            100.0
        } else {
            (used as f64 / policy.limit as f64) * 100.0
        };

        // 5. 閾値到達チェック（増分前の使用量で判定）
        if let Some(threshold) = policy.alert_threshold_percent {
            let prev_percent = if policy.limit == 0 {
                100.0
            } else {
                ((used.saturating_sub(input.amount)) as f64 / policy.limit as f64) * 100.0
            };
            if usage_percent >= threshold as f64 && prev_percent < threshold as f64 {
                let event = QuotaThresholdReachedEvent {
                    event_type: "QUOTA_THRESHOLD_REACHED".to_string(),
                    quota_id: policy.id.clone(),
                    subject_type: policy.subject_type.as_str().to_string(),
                    subject_id: policy.subject_id.clone(),
                    period: policy.period.as_str().to_string(),
                    limit: policy.limit,
                    used,
                    usage_percent,
                    alert_threshold_percent: threshold,
                    reached_at: chrono::Utc::now().to_rfc3339(),
                };
                let _ = self.event_publisher.publish_threshold_reached(&event).await;
            }
        }

        Ok(IncrementResult {
            quota_id: policy.id,
            used,
            remaining,
            usage_percent,
            exceeded: false,
            allowed: true,
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
            "tenant-abc".to_string(),
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
            .expect_check_and_increment()
            .returning(|_, _, _| {
                Ok(CheckAndIncrementResult {
                    used: 7524,
                    allowed: true,
                })
            });

        let uc = IncrementQuotaUsageUseCase::new_without_publisher(
            Arc::new(policy_mock),
            Arc::new(usage_mock),
        );
        let input = IncrementQuotaUsageInput {
            quota_id: policy.id.clone(),
            amount: 1,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let inc = result.unwrap();
        assert_eq!(inc.used, 7524);
        assert_eq!(inc.remaining, 2476);
        assert!(!inc.exceeded);
        assert!(inc.allowed);
    }

    #[tokio::test]
    async fn exceeded() {
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
            .expect_check_and_increment()
            .returning(|_, _, _| {
                Ok(CheckAndIncrementResult {
                    used: 10000,
                    allowed: false,
                })
            });

        let uc = IncrementQuotaUsageUseCase::new_without_publisher(
            Arc::new(policy_mock),
            Arc::new(usage_mock),
        );
        let input = IncrementQuotaUsageInput {
            quota_id: policy.id.clone(),
            amount: 1,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            IncrementQuotaUsageError::Exceeded {
                used, limit, ..
            } => {
                assert_eq!(used, 10000);
                assert_eq!(limit, 10000);
            }
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn not_found() {
        let mut policy_mock = MockQuotaPolicyRepository::new();
        let usage_mock = MockQuotaUsageRepository::new();

        policy_mock
            .expect_find_by_id()
            .returning(|_| Ok(None));

        let uc = IncrementQuotaUsageUseCase::new_without_publisher(
            Arc::new(policy_mock),
            Arc::new(usage_mock),
        );
        let input = IncrementQuotaUsageInput {
            quota_id: "nonexistent".to_string(),
            amount: 1,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            IncrementQuotaUsageError::NotFound(id) => assert_eq!(id, "nonexistent"),
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

        let uc = IncrementQuotaUsageUseCase::new_without_publisher(
            Arc::new(policy_mock),
            Arc::new(usage_mock),
        );
        let input = IncrementQuotaUsageInput {
            quota_id: "some-id".to_string(),
            amount: 1,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            IncrementQuotaUsageError::Internal(msg) => assert!(msg.contains("db error")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
