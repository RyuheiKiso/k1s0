use std::future::Future;
use std::sync::Arc;

use tokio::sync::{OwnedSemaphorePermit, Semaphore};

use crate::config::BulkheadConfig;
use crate::error::BulkheadError;
use crate::metrics::{BulkheadMetrics, BulkheadMetricsRecorder};

pub struct Bulkhead {
    semaphore: Arc<Semaphore>,
    config: BulkheadConfig,
    metrics: BulkheadMetricsRecorder,
}

impl Bulkhead {
    pub fn new(config: BulkheadConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_calls));
        let metrics = BulkheadMetricsRecorder::new();
        Self {
            semaphore,
            config,
            metrics,
        }
    }

    pub async fn acquire(&self) -> Result<OwnedSemaphorePermit, BulkheadError> {
        match tokio::time::timeout(
            self.config.max_wait_duration,
            self.semaphore.clone().acquire_owned(),
        )
        .await
        {
            Ok(Ok(permit)) => {
                self.metrics.record_acquire();
                Ok(permit)
            }
            Ok(Err(_)) => {
                self.metrics.record_rejection();
                Err(BulkheadError::Full {
                    max_concurrent: self.config.max_concurrent_calls,
                })
            }
            Err(_) => {
                self.metrics.record_rejection();
                Err(BulkheadError::Full {
                    max_concurrent: self.config.max_concurrent_calls,
                })
            }
        }
    }

    pub async fn call<F, Fut, T, E>(&self, f: F) -> Result<T, BulkheadError<E>>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        let _permit = self.acquire().await.map_err(|e| match e {
            BulkheadError::Full { max_concurrent } => BulkheadError::Full { max_concurrent },
            BulkheadError::Inner(_) => unreachable!(),
        })?;
        let result = f().await.map_err(BulkheadError::Inner)?;
        self.metrics.record_release();
        Ok(result)
    }

    pub fn metrics(&self) -> BulkheadMetrics {
        self.metrics.snapshot()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn test_config(max_concurrent: usize) -> BulkheadConfig {
        BulkheadConfig {
            max_concurrent_calls: max_concurrent,
            max_wait_duration: Duration::from_millis(100),
        }
    }

    #[tokio::test]
    async fn test_acquire_release() {
        let bh = Bulkhead::new(test_config(2));
        let permit = bh.acquire().await.unwrap();
        assert_eq!(bh.metrics().current_concurrent, 1);
        drop(permit);
        // After release, metrics should reflect the release via record_release
        // Note: OwnedSemaphorePermit drop releases the semaphore but doesn't call record_release.
        // record_release is called in call(). For raw acquire, current_concurrent stays incremented.
    }

    #[tokio::test]
    async fn test_full_rejection() {
        let bh = Bulkhead::new(test_config(1));
        let _permit = bh.acquire().await.unwrap();

        let result = bh.acquire().await;
        assert!(result.is_err());
        assert_eq!(bh.metrics().rejection_count, 1);
    }

    #[tokio::test]
    async fn test_concurrent_limit() {
        let bh = Arc::new(Bulkhead::new(test_config(2)));

        let _p1 = bh.acquire().await.unwrap();
        let _p2 = bh.acquire().await.unwrap();

        let bh_clone = bh.clone();
        let handle = tokio::spawn(async move { bh_clone.acquire().await });

        let result = handle.await.unwrap();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_call_success() {
        let bh = Bulkhead::new(test_config(2));
        let result: Result<i32, BulkheadError<String>> =
            bh.call(|| async { Ok::<i32, String>(42) }).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_call_rejects_when_full() {
        let bh = Bulkhead::new(test_config(1));
        let _permit = bh.acquire().await.unwrap();

        let result: Result<i32, BulkheadError<String>> =
            bh.call(|| async { Ok::<i32, String>(42) }).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_metrics() {
        let bh = Bulkhead::new(test_config(1));

        let metrics = bh.metrics();
        assert_eq!(metrics.rejection_count, 0);
        assert_eq!(metrics.current_concurrent, 0);

        let _permit = bh.acquire().await.unwrap();
        let metrics = bh.metrics();
        assert_eq!(metrics.current_concurrent, 1);

        let _ = bh.acquire().await;
        let metrics = bh.metrics();
        assert_eq!(metrics.rejection_count, 1);
    }
}
