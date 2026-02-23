use thiserror::Error;

#[derive(Debug, Error)]
pub enum HealthError {
    #[error("health check failed: {0}")]
    CheckFailed(String),
    #[error("timeout: {0}")]
    Timeout(String),
}
