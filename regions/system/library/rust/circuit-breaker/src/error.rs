use std::fmt;

#[derive(Debug)]
pub enum CircuitBreakerError<E> {
    Open,
    Inner(E),
}

impl<E: fmt::Display> fmt::Display for CircuitBreakerError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CircuitBreakerError::Open => write!(f, "circuit breaker is open"),
            CircuitBreakerError::Inner(e) => write!(f, "inner error: {}", e),
        }
    }
}

impl<E: fmt::Debug + fmt::Display> std::error::Error for CircuitBreakerError<E> {}
