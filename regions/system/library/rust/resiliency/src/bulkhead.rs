use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

use crate::error::ResiliencyError;

#[derive(Debug)]
pub struct Bulkhead {
    semaphore: Arc<Semaphore>,
    max_concurrent: usize,
    max_wait: Duration,
}

impl Bulkhead {
    pub fn new(max_concurrent: usize, max_wait: Duration) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            max_concurrent,
            max_wait,
        }
    }

    pub async fn acquire(&self) -> Result<tokio::sync::OwnedSemaphorePermit, ResiliencyError> {
        match tokio::time::timeout(
            self.max_wait,
            self.semaphore.clone().acquire_owned(),
        )
        .await
        {
            Ok(Ok(permit)) => Ok(permit),
            Ok(Err(_)) => Err(ResiliencyError::BulkheadFull {
                max_concurrent: self.max_concurrent,
            }),
            Err(_) => Err(ResiliencyError::BulkheadFull {
                max_concurrent: self.max_concurrent,
            }),
        }
    }
}
