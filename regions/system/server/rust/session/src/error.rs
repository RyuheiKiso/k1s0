use thiserror::Error;

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("session not found: {0}")]
    NotFound(String),
    #[error("session expired: {0}")]
    #[allow(dead_code)]
    Expired(String),
    #[error("session revoked: {0}")]
    Revoked(String),
    #[error("session already revoked: {0}")]
    AlreadyRevoked(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("too many sessions for user: {0}")]
    #[allow(dead_code)]
    TooManySessions(String),
    #[error("internal error: {0}")]
    Internal(String),
}
