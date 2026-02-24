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

impl ValidationError {
    /// Returns the error code string for this validation error.
    pub fn code(&self) -> &'static str {
        match self {
            ValidationError::InvalidEmail(_) => "INVALID_EMAIL",
            ValidationError::InvalidUuid(_) => "INVALID_UUID",
            ValidationError::InvalidUrl(_) => "INVALID_URL",
            ValidationError::InvalidTenantId(_) => "INVALID_TENANT_ID",
            ValidationError::InvalidPagination(_) => "INVALID_PAGINATION",
            ValidationError::InvalidDateRange(_) => "INVALID_DATE_RANGE",
        }
    }
}

/// A collection of `ValidationError` instances.
#[derive(Debug, Default)]
pub struct ValidationErrors {
    errors: Vec<ValidationError>,
}

impl ValidationErrors {
    /// Creates a new empty `ValidationErrors`.
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    /// Returns `true` if there are any errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Returns a slice of all collected errors.
    pub fn get_errors(&self) -> &[ValidationError] {
        &self.errors
    }

    /// Adds a validation error to the collection.
    pub fn add(&mut self, error: ValidationError) {
        self.errors.push(error);
    }
}
