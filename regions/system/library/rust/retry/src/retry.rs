use crate::error::RetryError;
use crate::policy::RetryConfig;
use std::future::Future;

pub async fn with_retry<F, Fut, T, E>(config: &RetryConfig, mut operation: F) -> Result<T, RetryError<E>>
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
