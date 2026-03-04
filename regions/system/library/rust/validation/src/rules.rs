use chrono::NaiveDateTime;
use regex::Regex;

use crate::error::ValidationError;

pub fn validate_email(field: &str, email: &str) -> Result<(), ValidationError> {
    let re = Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
    if re.is_match(email) {
        Ok(())
    } else {
        Err(ValidationError::InvalidEmail(format!(
            "{}: {}",
            field, email
        )))
    }
}

pub fn validate_uuid(field: &str, id: &str) -> Result<(), ValidationError> {
    uuid::Uuid::parse_str(id)
        .map(|_| ())
        .map_err(|_| ValidationError::InvalidUuid(format!("{}: {}", field, id)))
}

pub fn validate_url(field: &str, input: &str) -> Result<(), ValidationError> {
    let parsed =
        url::Url::parse(input).map_err(|_| ValidationError::InvalidUrl(format!("{}: {}", field, input)))?;
    match parsed.scheme() {
        "http" | "https" => Ok(()),
        _ => Err(ValidationError::InvalidUrl(format!(
            "{}: unsupported scheme: {}",
            field,
            parsed.scheme()
        ))),
    }
}

pub fn validate_tenant_id(field: &str, id: &str) -> Result<(), ValidationError> {
    let re = Regex::new(r"^[a-zA-Z0-9\-]+$").unwrap();
    if id.len() < 3 || id.len() > 63 {
        return Err(ValidationError::InvalidTenantId(format!(
            "{}: length must be 3-63, got {}",
            field,
            id.len()
        )));
    }
    if !re.is_match(id) {
        return Err(ValidationError::InvalidTenantId(format!(
            "{}: must contain only alphanumeric and hyphens: {}",
            field,
            id
        )));
    }
    Ok(())
}

pub fn validate_pagination(page: u32, per_page: u32) -> Result<(), ValidationError> {
    if page < 1 {
        return Err(ValidationError::InvalidPagination(format!(
            "page must be >= 1, got {}",
            page
        )));
    }
    if per_page < 1 || per_page > 100 {
        return Err(ValidationError::InvalidPagination(format!(
            "per_page must be 1-100, got {}",
            per_page
        )));
    }
    Ok(())
}

pub fn validate_date_range(
    start: NaiveDateTime,
    end: NaiveDateTime,
) -> Result<(), ValidationError> {
    if start > end {
        return Err(ValidationError::InvalidDateRange(format!(
            "start ({}) must be <= end ({})",
            start, end
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_validate_email_success() {
        assert!(validate_email("email", "user@example.com").is_ok());
        assert!(validate_email("email", "a@b.c").is_ok());
    }

    #[test]
    fn test_validate_email_failure() {
        assert!(validate_email("email", "invalid").is_err());
        assert!(validate_email("email", "@example.com").is_err());
        assert!(validate_email("email", "user@").is_err());
        assert!(validate_email("email", "user@example").is_err());
    }

    #[test]
    fn test_validate_uuid_success() {
        assert!(validate_uuid("tenant_id", "550e8400-e29b-41d4-a716-446655440000").is_ok());
    }

    #[test]
    fn test_validate_uuid_failure() {
        assert!(validate_uuid("tenant_id", "not-a-uuid").is_err());
        assert!(validate_uuid("tenant_id", "").is_err());
    }

    #[test]
    fn test_validate_url_success() {
        assert!(validate_url("endpoint", "https://example.com").is_ok());
        assert!(validate_url("endpoint", "http://example.com/path?q=1").is_ok());
    }

    #[test]
    fn test_validate_url_failure() {
        assert!(validate_url("endpoint", "ftp://example.com").is_err());
        assert!(validate_url("endpoint", "not a url").is_err());
    }

    #[test]
    fn test_validate_tenant_id_success() {
        assert!(validate_tenant_id("tenant_id", "abc").is_ok());
        assert!(validate_tenant_id("tenant_id", "my-tenant-123").is_ok());
    }

    #[test]
    fn test_validate_tenant_id_failure() {
        assert!(validate_tenant_id("tenant_id", "ab").is_err()); // too short
        assert!(validate_tenant_id("tenant_id", &"a".repeat(64)).is_err()); // too long
        assert!(validate_tenant_id("tenant_id", "invalid_underscore").is_err());
    }

    #[test]
    fn test_validate_pagination_success() {
        assert!(validate_pagination(1, 10).is_ok());
        assert!(validate_pagination(5, 100).is_ok());
        assert!(validate_pagination(1, 1).is_ok());
    }

    #[test]
    fn test_validate_pagination_failure() {
        assert!(validate_pagination(0, 10).is_err()); // page < 1
        assert!(validate_pagination(1, 0).is_err()); // per_page < 1
        assert!(validate_pagination(1, 101).is_err()); // per_page > 100
    }

    #[test]
    fn test_validation_error_code() {
        let err = ValidationError::InvalidEmail("bad".to_string());
        assert_eq!(err.code(), "INVALID_EMAIL");

        let err = ValidationError::InvalidPagination("bad".to_string());
        assert_eq!(err.code(), "INVALID_PAGINATION");

        let err = ValidationError::InvalidDateRange("bad".to_string());
        assert_eq!(err.code(), "INVALID_DATE_RANGE");
    }

    #[test]
    fn test_validation_errors_collection() {
        use crate::error::ValidationErrors;

        let mut errors = ValidationErrors::new();
        assert!(!errors.has_errors());
        assert!(errors.get_errors().is_empty());

        errors.add(ValidationError::InvalidEmail("a".to_string()));
        errors.add(ValidationError::InvalidPagination("b".to_string()));

        assert!(errors.has_errors());
        assert_eq!(errors.get_errors().len(), 2);
        assert_eq!(errors.get_errors()[0].code(), "INVALID_EMAIL");
        assert_eq!(errors.get_errors()[1].code(), "INVALID_PAGINATION");
    }

    #[test]
    fn test_validate_date_range_success() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 12, 31)
            .unwrap()
            .and_hms_opt(23, 59, 59)
            .unwrap();
        assert!(validate_date_range(start, end).is_ok());
    }

    #[test]
    fn test_validate_date_range_equal() {
        let dt = NaiveDate::from_ymd_opt(2024, 6, 15)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();
        assert!(validate_date_range(dt, dt).is_ok());
    }

    #[test]
    fn test_validate_date_range_failure() {
        let start = NaiveDate::from_ymd_opt(2024, 12, 31)
            .unwrap()
            .and_hms_opt(23, 59, 59)
            .unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        assert!(validate_date_range(start, end).is_err());
    }
}
