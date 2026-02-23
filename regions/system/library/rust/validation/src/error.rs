use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum ValidationError {
    #[error("invalid email: {0}")]
    InvalidEmail(String),
    #[error("invalid UUID: {0}")]
    InvalidUuid(String),
    #[error("invalid URL: {0}")]
    InvalidUrl(String),
    #[error("invalid tenant_id: {0}")]
    InvalidTenantId(String),
    #[error("invalid pagination: {0}")]
    InvalidPagination(String),
    #[error("invalid date range: {0}")]
    InvalidDateRange(String),
}
