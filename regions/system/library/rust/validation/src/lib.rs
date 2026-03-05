pub mod error;
pub mod macros;
pub mod rules;

pub use error::{ValidationError, ValidationErrors};
pub use rules::{
    validate_date_range, validate_email, validate_pagination, validate_tenant_id, validate_url,
    validate_uuid,
};

/// Validator is an extension point for domain/request specific validation.
pub trait Validator {
    fn validate(&self) -> Result<(), ValidationErrors>;
}
