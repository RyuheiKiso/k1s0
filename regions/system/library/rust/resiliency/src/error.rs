use std::fmt;

#[derive(Debug)]
pub enum ResiliencyError {
    MaxRetriesExceeded {
        attempts: u32,
        last_error: Box<dyn std::error::Error + Send + Sync>,
    },
    CircuitBreakerOpen {
        remaining_duration: std::time::Duration,
    },
    BulkheadFull {
        max_concurrent: usize,
    },
    Timeout {
        after: std::time::Duration,
    },
}

impl fmt::Display for ResiliencyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MaxRetriesExceeded { attempts, last_error } => {
                write!(f, "max retries exceeded after {} attempts: {}", attempts, last_error)
            }
            Self::CircuitBreakerOpen { remaining_duration } => {
                write!(f, "circuit breaker is open, remaining: {:?}", remaining_duration)
            }
            Self::BulkheadFull { max_concurrent } => {
                write!(f, "bulkhead full, max concurrent calls: {}", max_concurrent)
            }
            Self::Timeout { after } => {
                write!(f, "operation timed out after {:?}", after)
            }
        }
    }
}

impl std::error::Error for ResiliencyError {}
