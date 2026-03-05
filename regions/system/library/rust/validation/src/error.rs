use std::fmt::{Display, Formatter};

/// ValidationError represents a single field-level validation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub field: String,
    pub code: String,
    pub message: String,
}

impl ValidationError {
    pub fn new(field: impl Into<String>, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            code: code.into(),
            message: message.into(),
        }
    }
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({}): {}", self.field, self.code, self.message)
    }
}

impl std::error::Error for ValidationError {}

/// ValidationErrors is a collection of field-level validation failures.
#[derive(Debug, Default, Clone)]
pub struct ValidationErrors {
    errors: Vec<ValidationError>,
}

impl ValidationErrors {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn has_errors(&self) -> bool {
        !self.is_empty()
    }

    pub fn get_errors(&self) -> &[ValidationError] {
        &self.errors
    }

    pub fn add(&mut self, error: ValidationError) {
        self.errors.push(error);
    }
}

impl Display for ValidationErrors {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.errors.is_empty() {
            return f.write_str("no validation errors");
        }

        let rendered = self
            .errors
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");

        write!(f, "{rendered}")
    }
}

impl std::error::Error for ValidationErrors {}
