use chrono::{TimeZone, Utc};

use k1s0_validation::{
    validate_date_range, validate_email, validate_pagination, validate_tenant_id, validate_url,
    validate_uuid, ValidationError, ValidationErrors,
};

// ===========================================================================
// validate_email
// ===========================================================================

// シンプルな有効メールアドレスの検証が成功することを確認する。
#[test]
fn test_email_valid_simple() {
    assert!(validate_email("email", "user@example.com").is_ok());
}

// サブドメインを含むメールアドレスの検証が成功することを確認する。
#[test]
fn test_email_valid_with_subdomain() {
    assert!(validate_email("email", "user@mail.example.co.jp").is_ok());
}

// プラスタグ付きメールアドレスの検証が成功することを確認する。
#[test]
fn test_email_valid_with_plus_tag() {
    assert!(validate_email("email", "user+tag@example.com").is_ok());
}

// ドットを含むローカルパートのメールアドレスの検証が成功することを確認する。
#[test]
fn test_email_valid_with_dots() {
    assert!(validate_email("email", "first.last@example.com").is_ok());
}

// "@" がないメールアドレスの検証がエラーを返し INVALID_EMAIL コードを持つことを確認する。
#[test]
fn test_email_invalid_missing_at() {
    let err = validate_email("email", "userexample.com").unwrap_err();
    assert_eq!(err.field, "email");
    assert_eq!(err.code, "INVALID_EMAIL");
}

// ドメイン部が空のメールアドレスの検証がエラーを返すことを確認する。
#[test]
fn test_email_invalid_missing_domain() {
    assert!(validate_email("email", "user@").is_err());
}

// ローカルパートが空のメールアドレスの検証がエラーを返すことを確認する。
#[test]
fn test_email_invalid_missing_local_part() {
    assert!(validate_email("email", "@example.com").is_err());
}

// 空文字列のメールアドレス検証がエラーを返すことを確認する。
#[test]
fn test_email_invalid_empty() {
    assert!(validate_email("email", "").is_err());
}

// スペースを含むメールアドレスの検証がエラーを返すことを確認する。
#[test]
fn test_email_invalid_with_spaces() {
    assert!(validate_email("email", "user @example.com").is_err());
}

// TLD がないメールアドレスの検証がエラーを返すことを確認する。
#[test]
fn test_email_invalid_missing_tld() {
    assert!(validate_email("email", "user@example").is_err());
}

// ===========================================================================
// validate_url
// ===========================================================================

// https スキームの URL 検証が成功することを確認する。
#[test]
fn test_url_valid_https() {
    assert!(validate_url("url", "https://example.com").is_ok());
}

// http スキームの URL 検証が成功することを確認する。
#[test]
fn test_url_valid_http() {
    assert!(validate_url("url", "http://example.com").is_ok());
}

// パスを含む URL 検証が成功することを確認する。
#[test]
fn test_url_valid_with_path() {
    assert!(validate_url("url", "https://example.com/path/to/resource").is_ok());
}

// クエリパラメータを含む URL 検証が成功することを確認する。
#[test]
fn test_url_valid_with_query() {
    assert!(validate_url("url", "https://example.com/search?q=test&page=1").is_ok());
}

// ポート番号を含む URL 検証が成功することを確認する。
#[test]
fn test_url_valid_with_port() {
    assert!(validate_url("url", "https://example.com:8080/api").is_ok());
}

// ftp スキームの URL 検証がエラーを返し INVALID_URL コードを持つことを確認する。
#[test]
fn test_url_invalid_ftp_scheme() {
    let err = validate_url("url", "ftp://example.com").unwrap_err();
    assert_eq!(err.code, "INVALID_URL");
    assert!(err.message.contains("unsupported scheme"));
}

// スキームなしの URL 検証がエラーを返すことを確認する。
#[test]
fn test_url_invalid_no_scheme() {
    assert!(validate_url("url", "example.com").is_err());
}

// 空文字列の URL 検証がエラーを返すことを確認する。
#[test]
fn test_url_invalid_empty() {
    assert!(validate_url("url", "").is_err());
}

// スキームのみでホストのない URL 検証がエラーを返すことを確認する。
#[test]
fn test_url_invalid_just_scheme() {
    // "https://" alone is rejected by the url crate (empty host)
    assert!(validate_url("url", "https://").is_err());
}

// ===========================================================================
// validate_uuid
// ===========================================================================

// 標準的な UUID v4 形式の検証が成功することを確認する。
#[test]
fn test_uuid_valid_v4() {
    assert!(validate_uuid("id", "550e8400-e29b-41d4-a716-446655440000").is_ok());
}

// 小文字の UUID 検証が成功することを確認する。
#[test]
fn test_uuid_valid_lowercase() {
    assert!(validate_uuid("id", "a1b2c3d4-e5f6-7890-abcd-ef1234567890").is_ok());
}

// 大文字の UUID 検証が成功することを確認する。
#[test]
fn test_uuid_valid_uppercase() {
    assert!(validate_uuid("id", "A1B2C3D4-E5F6-7890-ABCD-EF1234567890").is_ok());
}

// UUID 形式でない文字列の検証がエラーを返し INVALID_UUID コードを持つことを確認する。
#[test]
fn test_uuid_invalid_format() {
    let err = validate_uuid("id", "not-a-uuid").unwrap_err();
    assert_eq!(err.field, "id");
    assert_eq!(err.code, "INVALID_UUID");
}

// 空文字列の UUID 検証がエラーを返すことを確認する。
#[test]
fn test_uuid_invalid_empty() {
    assert!(validate_uuid("id", "").is_err());
}

// 短すぎる UUID 形式の文字列の検証がエラーを返すことを確認する。
#[test]
fn test_uuid_invalid_too_short() {
    assert!(validate_uuid("id", "550e8400-e29b-41d4-a716").is_err());
}

// ハイフンなしの UUID 文字列が uuid クレートに受け入れられることを確認する。
#[test]
fn test_uuid_invalid_missing_hyphens() {
    // uuid crate actually accepts this without hyphens
    assert!(validate_uuid("id", "550e8400e29b41d4a716446655440000").is_ok());
}

// ===========================================================================
// validate_pagination
// ===========================================================================

// 基本的なページネーションパラメータの検証が成功することを確認する。
#[test]
fn test_pagination_valid_basic() {
    assert!(validate_pagination("pagination", 1, 10).is_ok());
}

// 大きなページ番号のページネーション検証が成功することを確認する。
#[test]
fn test_pagination_valid_page_large() {
    assert!(validate_pagination("pagination", 1000, 50).is_ok());
}

// per_page が 1 のページネーション検証が成功することを確認する。
#[test]
fn test_pagination_valid_per_page_1() {
    assert!(validate_pagination("pagination", 1, 1).is_ok());
}

// per_page が上限値の 200 のページネーション検証が成功することを確認する。
#[test]
fn test_pagination_valid_per_page_200() {
    assert!(validate_pagination("pagination", 1, 200).is_ok());
}

// per_page が 0 のとき INVALID_PAGINATION エラーが返されることを確認する。
#[test]
fn test_pagination_invalid_per_page_0() {
    let err = validate_pagination("pagination", 1, 0).unwrap_err();
    assert_eq!(err.code, "INVALID_PAGINATION");
    assert!(err.message.contains("per_page"));
}

// per_page が 201 のとき INVALID_PAGINATION エラーが返されることを確認する。
#[test]
fn test_pagination_invalid_per_page_201() {
    let err = validate_pagination("pagination", 1, 201).unwrap_err();
    assert_eq!(err.code, "INVALID_PAGINATION");
    assert!(err.message.contains("per_page"));
}

// per_page が u32::MAX のときページネーション検証がエラーを返すことを確認する。
#[test]
fn test_pagination_invalid_per_page_max_u32() {
    assert!(validate_pagination("pagination", 1, u32::MAX).is_err());
}

// Note: page is u32, so 0 won't trigger the page < 1 check since u32 can be 0
// but the check `page < 1` with u32 means page == 0 would fail
// page が 0 のときページネーション検証がエラーを返すことを確認する。
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

// 英数字のみのテナント ID 検証が成功することを確認する。
#[test]
fn test_tenant_id_valid_alphanumeric() {
    assert!(validate_tenant_id("tenant", "abc123").is_ok());
}

// ハイフンを含むテナント ID 検証が成功することを確認する。
#[test]
fn test_tenant_id_valid_with_hyphens() {
    assert!(validate_tenant_id("tenant", "my-tenant-01").is_ok());
}

// 最小長 3 文字のテナント ID 検証が成功することを確認する。
#[test]
fn test_tenant_id_valid_min_length_3() {
    assert!(validate_tenant_id("tenant", "abc").is_ok());
}

// 最大長 63 文字のテナント ID 検証が成功することを確認する。
#[test]
fn test_tenant_id_valid_max_length_63() {
    let id = "a".repeat(63);
    assert!(validate_tenant_id("tenant", &id).is_ok());
}

// 2 文字のテナント ID が INVALID_TENANT_ID エラーを返すことを確認する。
#[test]
fn test_tenant_id_invalid_too_short() {
    let err = validate_tenant_id("tenant", "ab").unwrap_err();
    assert_eq!(err.code, "INVALID_TENANT_ID");
    assert!(err.message.contains("length"));
}

// 空文字列のテナント ID 検証がエラーを返すことを確認する。
#[test]
fn test_tenant_id_invalid_empty() {
    assert!(validate_tenant_id("tenant", "").is_err());
}

// 1 文字のテナント ID 検証がエラーを返すことを確認する。
#[test]
fn test_tenant_id_invalid_single_char() {
    assert!(validate_tenant_id("tenant", "a").is_err());
}

// 64 文字を超えるテナント ID が INVALID_TENANT_ID エラーを返すことを確認する。
#[test]
fn test_tenant_id_invalid_too_long() {
    let id = "a".repeat(64);
    let err = validate_tenant_id("tenant", &id).unwrap_err();
    assert_eq!(err.code, "INVALID_TENANT_ID");
    assert!(err.message.contains("length"));
}

// アンダースコアを含むテナント ID 検証がエラーを返すことを確認する。
#[test]
fn test_tenant_id_invalid_special_chars() {
    assert!(validate_tenant_id("tenant", "my_tenant").is_err());
}

// スペースを含むテナント ID 検証がエラーを返すことを確認する。
#[test]
fn test_tenant_id_invalid_spaces() {
    assert!(validate_tenant_id("tenant", "my tenant").is_err());
}

// ドットを含むテナント ID 検証がエラーを返すことを確認する。
#[test]
fn test_tenant_id_invalid_dots() {
    assert!(validate_tenant_id("tenant", "my.tenant").is_err());
}

// ===========================================================================
// validate_date_range
// ===========================================================================

// 開始日が終了日より前の日付範囲検証が成功することを確認する。
#[test]
fn test_date_range_valid() {
    let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap();
    assert!(validate_date_range("range", start, end).is_ok());
}

// 開始日と終了日が同じ場合の日付範囲検証が成功することを確認する。
#[test]
fn test_date_range_same_date_is_valid() {
    let date = Utc.with_ymd_and_hms(2024, 6, 15, 12, 0, 0).unwrap();
    assert!(validate_date_range("range", date, date).is_ok());
}

// 開始日が終了日より後の日付範囲検証が INVALID_DATE_RANGE エラーを返すことを確認する。
#[test]
fn test_date_range_start_after_end_is_invalid() {
    let start = Utc.with_ymd_and_hms(2024, 12, 31, 0, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    let err = validate_date_range("range", start, end).unwrap_err();
    assert_eq!(err.code, "INVALID_DATE_RANGE");
    assert!(err.message.contains("start"));
}

// 1 秒の差がある日付範囲検証が成功することを確認する。
#[test]
fn test_date_range_one_second_difference() {
    let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 1).unwrap();
    assert!(validate_date_range("range", start, end).is_ok());
}

// ===========================================================================
// ValidationErrors accumulation
// ===========================================================================

// 新規 ValidationErrors がエラーなしの空状態であることを確認する。
#[test]
fn test_validation_errors_new_is_empty() {
    let errors = ValidationErrors::new();
    assert!(errors.is_empty());
    assert!(!errors.has_errors());
    assert_eq!(errors.get_errors().len(), 0);
}

// ValidationErrors に 1 件のエラーを追加すると has_errors が true になることを確認する。
#[test]
fn test_validation_errors_add_single() {
    let mut errors = ValidationErrors::new();
    errors.add(ValidationError::new("field1", "CODE1", "msg1"));

    assert!(!errors.is_empty());
    assert!(errors.has_errors());
    assert_eq!(errors.get_errors().len(), 1);
    assert_eq!(errors.get_errors()[0].field, "field1");
}

// 複数のエラーを追加した場合に全件が格納されることを確認する。
#[test]
fn test_validation_errors_add_multiple() {
    let mut errors = ValidationErrors::new();
    errors.add(ValidationError::new("field1", "CODE1", "msg1"));
    errors.add(ValidationError::new("field2", "CODE2", "msg2"));
    errors.add(ValidationError::new("field3", "CODE3", "msg3"));

    assert_eq!(errors.get_errors().len(), 3);
}

// エラーなしの ValidationErrors の表示が "no validation errors" になることを確認する。
#[test]
fn test_validation_errors_display_empty() {
    let errors = ValidationErrors::new();
    assert_eq!(format!("{errors}"), "no validation errors");
}

// エラーを持つ ValidationErrors の表示にフィールド名とコードが含まれることを確認する。
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

// デフォルト構築した ValidationErrors が空であることを確認する。
#[test]
fn test_validation_errors_default() {
    let errors = ValidationErrors::default();
    assert!(errors.is_empty());
}

// ===========================================================================
// ValidationError fields and display
// ===========================================================================

// ValidationError の field、code、message が正しく設定されることを確認する。
#[test]
fn test_validation_error_fields() {
    let err = ValidationError::new("my_field", "MY_CODE", "something went wrong");
    assert_eq!(err.field, "my_field");
    assert_eq!(err.code, "MY_CODE");
    assert_eq!(err.message, "something went wrong");
}

// ValidationError の表示形式が "field (code): message" になることを確認する。
#[test]
fn test_validation_error_display() {
    let err = ValidationError::new("email", "INVALID_EMAIL", "invalid email: bad");
    let display = format!("{err}");
    assert_eq!(display, "email (INVALID_EMAIL): invalid email: bad");
}

// 同じフィールド値を持つ ValidationError が等しく、異なる message を持つと不等になることを確認する。
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

// validate! マクロが複数バリデーターのエラーをすべて収集することを確認する。
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

// すべて有効な入力で validate! マクロを実行するとエラーが収集されないことを確認する。
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

// 一部のバリデーターが失敗した場合に該当エラーのみが収集されることを確認する。
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

// メール検証エラーメッセージに入力値が含まれることを確認する。
#[test]
fn test_email_error_message_contains_input() {
    let err = validate_email("email", "bad-input").unwrap_err();
    assert!(err.message.contains("bad-input"));
}

// UUID 検証エラーメッセージに入力値が含まれることを確認する。
#[test]
fn test_uuid_error_message_contains_input() {
    let err = validate_uuid("id", "bad-uuid").unwrap_err();
    assert!(err.message.contains("bad-uuid"));
}

// URL 検証エラーメッセージに入力値が含まれることを確認する。
#[test]
fn test_url_error_message_contains_input() {
    let err = validate_url("url", "not-a-url").unwrap_err();
    assert!(err.message.contains("not-a-url"));
}

// テナント ID 検証エラーメッセージに実際の長さ情報が含まれることを確認する。
#[test]
fn test_tenant_id_error_message_contains_length_info() {
    let err = validate_tenant_id("tenant", "ab").unwrap_err();
    assert!(err.message.contains("2")); // got length 2
}
