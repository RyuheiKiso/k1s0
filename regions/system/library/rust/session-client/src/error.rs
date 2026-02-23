#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("session not found: {0}")]
    NotFound(String),
    #[error("session expired")]
    Expired,
    #[error("session revoked")]
    Revoked,
    #[error("connection error: {0}")]
    Connection(String),
    #[error("internal error: {0}")]
    Internal(String),
}
