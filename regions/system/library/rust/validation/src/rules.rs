use chrono::{DateTime, Utc};
use regex::Regex;

use crate::error::ValidationError;

fn err(field: &str, code: &str, message: String) -> ValidationError {
    ValidationError::new(field, code, message)
}

pub fn validate_email(field: &str, email: &str) -> Result<(), ValidationError> {
    let re = Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").expect("valid email regex");
    if re.is_match(email) {
        Ok(())
    } else {
        Err(err(
            field,
            "INVALID_EMAIL",
            format!("invalid email: {email}"),
        ))
    }
}

pub fn validate_uuid(field: &str, id: &str) -> Result<(), ValidationError> {
    uuid::Uuid::parse_str(id)
        .map(|_| ())
        .map_err(|_| err(field, "INVALID_UUID", format!("invalid uuid: {id}")))
}

pub fn validate_url(field: &str, input: &str) -> Result<(), ValidationError> {
    let parsed = url::Url::parse(input)
        .map_err(|_| err(field, "INVALID_URL", format!("invalid url: {input}")))?;

    match parsed.scheme() {
        "http" | "https" => Ok(()),
        scheme => Err(err(
            field,
            "INVALID_URL",
            format!("unsupported scheme: {scheme}"),
        )),
    }
}

pub fn validate_tenant_id(field: &str, id: &str) -> Result<(), ValidationError> {
    let re = Regex::new(r"^[a-zA-Z0-9\-]+$").expect("valid tenant regex");
    if id.len() < 3 || id.len() > 63 {
        return Err(err(
            field,
            "INVALID_TENANT_ID",
            format!("length must be 3-63, got {}", id.len()),
        ));
    }
    if !re.is_match(id) {
        return Err(err(
            field,
            "INVALID_TENANT_ID",
            format!("must contain only alphanumeric and hyphens: {id}"),
        ));
    }
    Ok(())
}

pub fn validate_pagination(field: &str, page: u32, per_page: u32) -> Result<(), ValidationError> {
    if page < 1 {
        return Err(err(
            field,
            "INVALID_PAGINATION",
            format!("page must be >= 1, got {page}"),
        ));
    }
    if !(1..=200).contains(&per_page) {
        return Err(err(
            field,
            "INVALID_PAGINATION",
            format!("per_page must be 1-200, got {per_page}"),
        ));
    }
    Ok(())
}

pub fn validate_date_range(
    field: &str,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<(), ValidationError> {
    if start > end {
        return Err(err(
            field,
            "INVALID_DATE_RANGE",
            format!("start ({start}) must be <= end ({end})"),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use crate::{validate, ValidationErrors};

    use super::*;

    #[test]
    fn test_validate_pagination_range_200() {
        assert!(validate_pagination("pagination", 1, 200).is_ok());
        assert!(validate_pagination("pagination", 1, 201).is_err());
    }

    #[test]
    fn test_validate_date_range_datetime_utc() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap();
        assert!(validate_date_range("date_range", start, end).is_ok());
        assert!(validate_date_range("date_range", end, start).is_err());
    }

    #[test]
    fn test_validate_macro_collects_errors() {
        let mut errors = ValidationErrors::new();
        validate!(
            errors,
            validate_email("email", "invalid"),
            validate_uuid("tenant_id", "invalid"),
            validate_pagination("pagination", 0, 300),
        );

        assert!(errors.has_errors());
        assert_eq!(errors.get_errors().len(), 3);
    }
}
