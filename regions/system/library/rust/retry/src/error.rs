use thiserror::Error;

#[derive(Debug, Error)]
pub enum RetryError<E> {
    #[error("All retries exhausted ({attempts} attempts): {last_error}")]
    ExhaustedRetries { attempts: u32, last_error: E },
    #[error("Circuit breaker is open")]
    CircuitBreakerOpen,
    #[error("Operation timed out")]
    Timeout,
}
