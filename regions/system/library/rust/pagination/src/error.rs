use thiserror::Error;

#[derive(Debug, Error)]
pub enum PaginationError {
    #[error("invalid cursor: {0}")]
    InvalidCursor(String),

    #[error("invalid per_page: {value} (must be between {min} and {max})")]
    InvalidPerPage { value: u32, min: u32, max: u32 },
}
