use crate::error::RetryError;
use crate::policy::RetryConfig;
use std::future::Future;

pub async fn with_retry<F, Fut, T, E>(
    config: &RetryConfig,
    mut operation: F,
) -> Result<T, RetryError<E>>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut last_error = None;
    for attempt in 0..config.max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                tracing::warn!(
                    "リトライ試行 {}/{}: {}",
                    attempt + 1,
                    config.max_attempts,
                    e
                );
                last_error = Some(e);
                if attempt + 1 < config.max_attempts {
                    let delay = config.compute_delay(attempt);
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
    Err(RetryError::ExhaustedRetries {
        attempts: config.max_attempts,
        last_error: last_error.unwrap(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::RetryConfig;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    #[derive(Debug, Clone)]
    struct TestError(String);
    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    #[tokio::test]
    async fn test_success_on_first_attempt() {
        let config = RetryConfig::new(3).with_jitter(false);
        let call_count = Arc::new(AtomicU32::new(0));
        let count = call_count.clone();

        let result: Result<&str, RetryError<TestError>> = with_retry(&config, || {
            let count = count.clone();
            async move {
                count.fetch_add(1, Ordering::SeqCst);
                Ok("success")
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_max_retry_failure() {
        let config = RetryConfig::new(3)
            .with_initial_delay(Duration::from_millis(1))
            .with_jitter(false);
        let call_count = Arc::new(AtomicU32::new(0));
        let count = call_count.clone();

        let result: Result<(), RetryError<TestError>> = with_retry(&config, || {
            let count = count.clone();
            async move {
                count.fetch_add(1, Ordering::SeqCst);
                Err(TestError("always fails".to_string()))
            }
        })
        .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            RetryError::ExhaustedRetries {
                attempts,
                last_error,
            } => {
                assert_eq!(attempts, 3);
                assert_eq!(last_error.0, "always fails");
            }
            other => panic!("unexpected error variant: {:?}", other),
        }
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_exponential_backoff_delay() {
        // initial_delay=50ms, multiplier=2.0, jitter=false の場合:
        // attempt 0 → 50ms, attempt 1 → 100ms
        // 合計待機 >= 150ms 程度
        let config = RetryConfig::new(3)
            .with_initial_delay(Duration::from_millis(50))
            .with_multiplier(2.0)
            .with_jitter(false);

        let start = Instant::now();
        let result: Result<(), RetryError<TestError>> =
            with_retry(&config, || async { Err(TestError("fail".to_string())) }).await;
        let elapsed = start.elapsed();

        assert!(result.is_err());
        // 50ms (attempt 0→1) + 100ms (attempt 1→2) = 150ms 最低
        assert!(
            elapsed >= Duration::from_millis(140),
            "expected at least 140ms of backoff delay, got {:?}",
            elapsed
        );
    }

    #[tokio::test]
    async fn test_custom_config() {
        let config = RetryConfig::new(5)
            .with_initial_delay(Duration::from_millis(10))
            .with_max_delay(Duration::from_millis(50))
            .with_multiplier(3.0)
            .with_jitter(false);

        assert_eq!(config.max_attempts, 5);
        assert_eq!(config.initial_delay, Duration::from_millis(10));
        assert_eq!(config.max_delay, Duration::from_millis(50));
        assert!((config.multiplier - 3.0).abs() < f64::EPSILON);
        assert!(!config.jitter);

        // max_delay=50ms でキャップされることを確認
        // attempt 2: 10 * 3^2 = 90ms → capped to 50ms
        let delay = config.compute_delay(2);
        assert_eq!(delay.as_millis(), 50);
    }

    #[tokio::test]
    async fn test_success_after_retries() {
        let config = RetryConfig::new(5)
            .with_initial_delay(Duration::from_millis(1))
            .with_jitter(false);
        let call_count = Arc::new(AtomicU32::new(0));
        let count = call_count.clone();

        let result: Result<&str, RetryError<TestError>> = with_retry(&config, || {
            let count = count.clone();
            async move {
                let n = count.fetch_add(1, Ordering::SeqCst);
                if n < 2 {
                    Err(TestError("transient".to_string()))
                } else {
                    Ok("recovered")
                }
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "recovered");
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }
}
