use async_trait::async_trait;

use crate::error::RateLimitError;
use crate::types::{RateLimitPolicy, RateLimitResult, RateLimitStatus};

#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait RateLimitClient: Send + Sync {
    async fn check(&self, key: &str, cost: u32) -> Result<RateLimitStatus, RateLimitError>;
    async fn consume(&self, key: &str, cost: u32) -> Result<RateLimitResult, RateLimitError>;
    async fn get_limit(&self, key: &str) -> Result<RateLimitPolicy, RateLimitError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_mock_check_allowed() {
        let mut mock = MockRateLimitClient::new();
        mock.expect_check().once().returning(|_, _| {
            Box::pin(async {
                Ok(RateLimitStatus {
                    allowed: true,
                    remaining: 99,
                    reset_at: Utc::now() + chrono::Duration::seconds(60),
                    retry_after_secs: None,
                })
            })
        });

        let result = mock.check("test-key", 1).await.unwrap();
        assert!(result.allowed);
        assert_eq!(result.remaining, 99);
        assert!(result.retry_after_secs.is_none());
    }

    #[tokio::test]
    async fn test_mock_check_denied() {
        let mut mock = MockRateLimitClient::new();
        mock.expect_check().once().returning(|_, _| {
            Box::pin(async {
                Ok(RateLimitStatus {
                    allowed: false,
                    remaining: 0,
                    reset_at: Utc::now() + chrono::Duration::seconds(30),
                    retry_after_secs: Some(30),
                })
            })
        });

        let result = mock.check("test-key", 1).await.unwrap();
        assert!(!result.allowed);
        assert_eq!(result.remaining, 0);
        assert_eq!(result.retry_after_secs, Some(30));
    }

    #[tokio::test]
    async fn test_mock_consume() {
        let mut mock = MockRateLimitClient::new();
        mock.expect_consume().once().returning(|_, _| {
            Box::pin(async {
                Ok(RateLimitResult {
                    remaining: 98,
                    reset_at: Utc::now() + chrono::Duration::seconds(60),
                })
            })
        });

        let result = mock.consume("test-key", 1).await.unwrap();
        assert_eq!(result.remaining, 98);
    }

    #[tokio::test]
    async fn test_mock_get_limit() {
        let mut mock = MockRateLimitClient::new();
        mock.expect_get_limit().once().returning(|key| {
            let key = key.to_string();
            Box::pin(async move {
                Ok(RateLimitPolicy {
                    key,
                    limit: 100,
                    window_secs: 3600,
                    algorithm: "token_bucket".to_string(),
                })
            })
        });

        let policy = mock.get_limit("tenant:TENANT-001").await.unwrap();
        assert_eq!(policy.key, "tenant:TENANT-001");
        assert_eq!(policy.limit, 100);
        assert_eq!(policy.window_secs, 3600);
        assert_eq!(policy.algorithm, "token_bucket");
    }

    #[tokio::test]
    async fn test_mock_check_limit_exceeded_error() {
        let mut mock = MockRateLimitClient::new();
        mock.expect_check().once().returning(|_, _| {
            Box::pin(async {
                Err(RateLimitError::LimitExceeded {
                    retry_after_secs: 42,
                })
            })
        });

        let result = mock.check("test-key", 1).await;
        assert!(matches!(
            result,
            Err(RateLimitError::LimitExceeded {
                retry_after_secs: 42
            })
        ));
    }

    #[tokio::test]
    async fn test_mock_get_limit_key_not_found() {
        let mut mock = MockRateLimitClient::new();
        mock.expect_get_limit().once().returning(|key| {
            let key = key.to_string();
            Box::pin(async move { Err(RateLimitError::KeyNotFound { key }) })
        });

        let result = mock.get_limit("unknown-key").await;
        assert!(matches!(result, Err(RateLimitError::KeyNotFound { .. })));
    }
}
