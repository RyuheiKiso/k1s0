use thiserror::Error;

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("session not found: {0}")]
    NotFound(String),
    #[error("session expired: {0}")]
    Expired(String),
    #[error("session revoked: {0}")]
    Revoked(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("too many sessions for user: {0}")]
    TooManySessions(String),
    #[error("internal error: {0}")]
    Internal(String),
}
