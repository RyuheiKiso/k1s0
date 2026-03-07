use std::time::Duration;

use crate::error::ResiliencyError;

pub struct Bulkhead {
    inner: k1s0_bulkhead::Bulkhead,
    max_concurrent: usize,
}

impl Bulkhead {
    pub fn new(max_concurrent: usize, max_wait: Duration) -> Self {
        let config = k1s0_bulkhead::BulkheadConfig {
            max_concurrent_calls: max_concurrent,
            max_wait_duration: max_wait,
        };
        Self {
            inner: k1s0_bulkhead::Bulkhead::new(config),
            max_concurrent,
        }
    }

    pub async fn acquire(&self) -> Result<tokio::sync::OwnedSemaphorePermit, ResiliencyError> {
        self.inner.acquire().await.map_err(|_| {
            ResiliencyError::BulkheadFull {
                max_concurrent: self.max_concurrent,
            }
        })
    }
}
