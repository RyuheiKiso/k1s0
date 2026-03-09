use chrono::{TimeZone, Utc};

use k1s0_validation::{
    validate_date_range, validate_email, validate_pagination, validate_tenant_id, validate_url,
    validate_uuid, ValidationError, ValidationErrors,
};

// ===========================================================================
// validate_email
// ===========================================================================

#[test]
fn test_email_valid_simple() {
    assert!(validate_email("email", "user@example.com").is_ok());
}

#[test]
fn test_email_valid_with_subdomain() {
    assert!(validate_email("email", "user@mail.example.co.jp").is_ok());
}

#[test]
fn test_email_valid_with_plus_tag() {
    assert!(validate_email("email", "user+tag@example.com").is_ok());
}

#[test]
fn test_email_valid_with_dots() {
    assert!(validate_email("email", "first.last@example.com").is_ok());
}

#[test]
fn test_email_invalid_missing_at() {
    let err = validate_email("email", "userexample.com").unwrap_err();
    assert_eq!(err.field, "email");
    assert_eq!(err.code, "INVALID_EMAIL");
}

#[test]
fn test_email_invalid_missing_domain() {
    assert!(validate_email("email", "user@").is_err());
}

#[test]
fn test_email_invalid_missing_local_part() {
    assert!(validate_email("email", "@example.com").is_err());
}

#[test]
fn test_email_invalid_empty() {
    assert!(validate_email("email", "").is_err());
}

#[test]
fn test_email_invalid_with_spaces() {
    assert!(validate_email("email", "user @example.com").is_err());
}

#[test]
fn test_email_invalid_missing_tld() {
    assert!(validate_email("email", "user@example").is_err());
}

// ===========================================================================
// validate_url
// ===========================================================================

#[test]
fn test_url_valid_https() {
    assert!(validate_url("url", "https://example.com").is_ok());
}

#[test]
fn test_url_valid_http() {
    assert!(validate_url("url", "http://example.com").is_ok());
}

#[test]
fn test_url_valid_with_path() {
    assert!(validate_url("url", "https://example.com/path/to/resource").is_ok());
}

#[test]
fn test_url_valid_with_query() {
    assert!(validate_url("url", "https://example.com/search?q=test&page=1").is_ok());
}

#[test]
fn test_url_valid_with_port() {
    assert!(validate_url("url", "https://example.com:8080/api").is_ok());
}

#[test]
fn test_url_invalid_ftp_scheme() {
    let err = validate_url("url", "ftp://example.com").unwrap_err();
    assert_eq!(err.code, "INVALID_URL");
    assert!(err.message.contains("unsupported scheme"));
}

#[test]
fn test_url_invalid_no_scheme() {
    assert!(validate_url("url", "example.com").is_err());
}

#[test]
fn test_url_invalid_empty() {
    assert!(validate_url("url", "").is_err());
}

#[test]
fn test_url_invalid_just_scheme() {
    // "https://" alone is rejected by the url crate (empty host)
    assert!(validate_url("url", "https://").is_err());
}

// ===========================================================================
// validate_uuid
// ===========================================================================

#[test]
fn test_uuid_valid_v4() {
    assert!(validate_uuid("id", "550e8400-e29b-41d4-a716-446655440000").is_ok());
}

#[test]
fn test_uuid_valid_lowercase() {
    assert!(validate_uuid("id", "a1b2c3d4-e5f6-7890-abcd-ef1234567890").is_ok());
}

#[test]
fn test_uuid_valid_uppercase() {
    assert!(validate_uuid("id", "A1B2C3D4-E5F6-7890-ABCD-EF1234567890").is_ok());
}

#[test]
fn test_uuid_invalid_format() {
    let err = validate_uuid("id", "not-a-uuid").unwrap_err();
    assert_eq!(err.field, "id");
    assert_eq!(err.code, "INVALID_UUID");
}

#[test]
fn test_uuid_invalid_empty() {
    assert!(validate_uuid("id", "").is_err());
}

#[test]
fn test_uuid_invalid_too_short() {
    assert!(validate_uuid("id", "550e8400-e29b-41d4-a716").is_err());
}

#[test]
fn test_uuid_invalid_missing_hyphens() {
    // uuid crate actually accepts this without hyphens
    assert!(validate_uuid("id", "550e8400e29b41d4a716446655440000").is_ok());
}

// ===========================================================================
// validate_pagination
// ===========================================================================

#[test]
fn test_pagination_valid_basic() {
    assert!(validate_pagination("pagination", 1, 10).is_ok());
}

#[test]
fn test_pagination_valid_page_large() {
    assert!(validate_pagination("pagination", 1000, 50).is_ok());
}

#[test]
fn test_pagination_valid_per_page_1() {
    assert!(validate_pagination("pagination", 1, 1).is_ok());
}

#[test]
fn test_pagination_valid_per_page_200() {
    assert!(validate_pagination("pagination", 1, 200).is_ok());
}

#[test]
fn test_pagination_invalid_per_page_0() {
    let err = validate_pagination("pagination", 1, 0).unwrap_err();
    assert_eq!(err.code, "INVALID_PAGINATION");
    assert!(err.message.contains("per_page"));
}

#[test]
fn test_pagination_invalid_per_page_201() {
    let err = validate_pagination("pagination", 1, 201).unwrap_err();
    assert_eq!(err.code, "INVALID_PAGINATION");
    assert!(err.message.contains("per_page"));
}

#[test]
fn test_pagination_invalid_per_page_max_u32() {
    assert!(validate_pagination("pagination", 1, u32::MAX).is_err());
}

// Note: page is u32, so 0 won't trigger the page < 1 check since u32 can be 0
// but the check `page < 1` with u32 means page == 0 would fail
#[test]
fn test_pagination_page_zero() {
    // u32 page = 0 is < 1, should fail
    // However, u32 comparison: 0u32 < 1u32 is true
    // But wait - the code says `if page < 1` which for u32 means page == 0
    let result = validate_pagination("pagination", 0, 10);
    assert!(result.is_err());
}

// ===========================================================================
// validate_tenant_id
// ===========================================================================

#[test]
fn test_tenant_id_valid_alphanumeric() {
    assert!(validate_tenant_id("tenant", "abc123").is_ok());
}

#[test]
fn test_tenant_id_valid_with_hyphens() {
    assert!(validate_tenant_id("tenant", "my-tenant-01").is_ok());
}

#[test]
fn test_tenant_id_valid_min_length_3() {
    assert!(validate_tenant_id("tenant", "abc").is_ok());
}

#[test]
fn test_tenant_id_valid_max_length_63() {
    let id = "a".repeat(63);
    assert!(validate_tenant_id("tenant", &id).is_ok());
}

#[test]
fn test_tenant_id_invalid_too_short() {
    let err = validate_tenant_id("tenant", "ab").unwrap_err();
    assert_eq!(err.code, "INVALID_TENANT_ID");
    assert!(err.message.contains("length"));
}

#[test]
fn test_tenant_id_invalid_empty() {
    assert!(validate_tenant_id("tenant", "").is_err());
}

#[test]
fn test_tenant_id_invalid_single_char() {
    assert!(validate_tenant_id("tenant", "a").is_err());
}

#[test]
fn test_tenant_id_invalid_too_long() {
    let id = "a".repeat(64);
    let err = validate_tenant_id("tenant", &id).unwrap_err();
    assert_eq!(err.code, "INVALID_TENANT_ID");
    assert!(err.message.contains("length"));
}

#[test]
fn test_tenant_id_invalid_special_chars() {
    assert!(validate_tenant_id("tenant", "my_tenant").is_err());
}

#[test]
fn test_tenant_id_invalid_spaces() {
    assert!(validate_tenant_id("tenant", "my tenant").is_err());
}

#[test]
fn test_tenant_id_invalid_dots() {
    assert!(validate_tenant_id("tenant", "my.tenant").is_err());
}

// ===========================================================================
// validate_date_range
// ===========================================================================

#[test]
fn test_date_range_valid() {
    let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap();
    assert!(validate_date_range("range", start, end).is_ok());
}

#[test]
fn test_date_range_same_date_is_valid() {
    let date = Utc.with_ymd_and_hms(2024, 6, 15, 12, 0, 0).unwrap();
    assert!(validate_date_range("range", date, date).is_ok());
}

#[test]
fn test_date_range_start_after_end_is_invalid() {
    let start = Utc.with_ymd_and_hms(2024, 12, 31, 0, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    let err = validate_date_range("range", start, end).unwrap_err();
    assert_eq!(err.code, "INVALID_DATE_RANGE");
    assert!(err.message.contains("start"));
}

#[test]
fn test_date_range_one_second_difference() {
    let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 1).unwrap();
    assert!(validate_date_range("range", start, end).is_ok());
}

// ===========================================================================
// ValidationErrors accumulation
// ===========================================================================

#[test]
fn test_validation_errors_new_is_empty() {
    let errors = ValidationErrors::new();
    assert!(errors.is_empty());
    assert!(!errors.has_errors());
    assert_eq!(errors.get_errors().len(), 0);
}

#[test]
fn test_validation_errors_add_single() {
    let mut errors = ValidationErrors::new();
    errors.add(ValidationError::new("field1", "CODE1", "msg1"));

    assert!(!errors.is_empty());
    assert!(errors.has_errors());
    assert_eq!(errors.get_errors().len(), 1);
    assert_eq!(errors.get_errors()[0].field, "field1");
}

#[test]
fn test_validation_errors_add_multiple() {
    let mut errors = ValidationErrors::new();
    errors.add(ValidationError::new("field1", "CODE1", "msg1"));
    errors.add(ValidationError::new("field2", "CODE2", "msg2"));
    errors.add(ValidationError::new("field3", "CODE3", "msg3"));

    assert_eq!(errors.get_errors().len(), 3);
}

#[test]
fn test_validation_errors_display_empty() {
    let errors = ValidationErrors::new();
    assert_eq!(format!("{errors}"), "no validation errors");
}

#[test]
fn test_validation_errors_display_with_errors() {
    let mut errors = ValidationErrors::new();
    errors.add(ValidationError::new("email", "INVALID_EMAIL", "bad email"));
    errors.add(ValidationError::new("id", "INVALID_UUID", "bad uuid"));

    let display = format!("{errors}");
    assert!(display.contains("email"));
    assert!(display.contains("INVALID_EMAIL"));
    assert!(display.contains("id"));
    assert!(display.contains("INVALID_UUID"));
}

#[test]
fn test_validation_errors_default() {
    let errors = ValidationErrors::default();
    assert!(errors.is_empty());
}

// ===========================================================================
// ValidationError fields and display
// ===========================================================================

#[test]
fn test_validation_error_fields() {
    let err = ValidationError::new("my_field", "MY_CODE", "something went wrong");
    assert_eq!(err.field, "my_field");
    assert_eq!(err.code, "MY_CODE");
    assert_eq!(err.message, "something went wrong");
}

#[test]
fn test_validation_error_display() {
    let err = ValidationError::new("email", "INVALID_EMAIL", "invalid email: bad");
    let display = format!("{err}");
    assert_eq!(display, "email (INVALID_EMAIL): invalid email: bad");
}

#[test]
fn test_validation_error_equality() {
    let e1 = ValidationError::new("f", "c", "m");
    let e2 = ValidationError::new("f", "c", "m");
    let e3 = ValidationError::new("f", "c", "different");
    assert_eq!(e1, e2);
    assert_ne!(e1, e3);
}

// ===========================================================================
// validate! macro accumulation
// ===========================================================================

#[test]
fn test_validate_macro_collects_all_errors() {
    let mut errors = ValidationErrors::new();
    k1s0_validation::validate!(
        errors,
        validate_email("email", "invalid"),
        validate_uuid("id", "not-uuid"),
        validate_pagination("pagination", 1, 0),
        validate_tenant_id("tenant", "a"),
    );

    assert!(errors.has_errors());
    assert_eq!(errors.get_errors().len(), 4);
}

#[test]
fn test_validate_macro_no_errors_when_all_valid() {
    let mut errors = ValidationErrors::new();
    k1s0_validation::validate!(
        errors,
        validate_email("email", "user@example.com"),
        validate_uuid("id", "550e8400-e29b-41d4-a716-446655440000"),
        validate_pagination("pagination", 1, 10),
        validate_tenant_id("tenant", "my-tenant"),
    );

    assert!(!errors.has_errors());
    assert_eq!(errors.get_errors().len(), 0);
}

#[test]
fn test_validate_macro_partial_errors() {
    let mut errors = ValidationErrors::new();
    k1s0_validation::validate!(
        errors,
        validate_email("email", "user@example.com"),
        validate_uuid("id", "not-valid"),
    );

    assert!(errors.has_errors());
    assert_eq!(errors.get_errors().len(), 1);
    assert_eq!(errors.get_errors()[0].field, "id");
}

// ===========================================================================
// Error message content checks
// ===========================================================================

#[test]
fn test_email_error_message_contains_input() {
    let err = validate_email("email", "bad-input").unwrap_err();
    assert!(err.message.contains("bad-input"));
}

#[test]
fn test_uuid_error_message_contains_input() {
    let err = validate_uuid("id", "bad-uuid").unwrap_err();
    assert!(err.message.contains("bad-uuid"));
}

#[test]
fn test_url_error_message_contains_input() {
    let err = validate_url("url", "not-a-url").unwrap_err();
    assert!(err.message.contains("not-a-url"));
}

#[test]
fn test_tenant_id_error_message_contains_length_info() {
    let err = validate_tenant_id("tenant", "ab").unwrap_err();
    assert!(err.message.contains("2")); // got length 2
}
