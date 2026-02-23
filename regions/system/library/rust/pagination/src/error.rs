use thiserror::Error;

#[derive(Debug, Error)]
pub enum PaginationError {
    #[error("invalid cursor: {0}")]
    InvalidCursor(String),
}
