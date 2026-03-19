use chrono::{DateTime, Utc};
use regex::Regex;

use crate::error::ValidationError;

fn err(field: &str, code: &str, message: String) -> ValidationError {
    ValidationError::new(field, code, message)
}

// メールアドレスを検証する。TLD 2文字以上を必須とする（4言語統一パターン H-18）
pub fn validate_email(field: &str, email: &str) -> Result<(), ValidationError> {
    let re = Regex::new(r"^[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}$")
        .expect("valid email regex");
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

// UUID v4 のみを許可する（4言語統一パターン H-18）
pub fn validate_uuid(field: &str, id: &str) -> Result<(), ValidationError> {
    let re = Regex::new(
        r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-4[0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$",
    )
    .expect("valid uuid v4 regex");
    if re.is_match(id) {
        Ok(())
    } else {
        Err(err(field, "INVALID_UUID", format!("invalid uuid v4: {id}")))
    }
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

// テナントIDを検証する。先頭・末尾は英数字、中間はハイフン許可（4言語統一パターン H-18）
pub fn validate_tenant_id(field: &str, id: &str) -> Result<(), ValidationError> {
    let re = Regex::new(r"^[a-z0-9][a-z0-9-]{1,61}[a-z0-9]$").expect("valid tenant regex");
    if !re.is_match(id) {
        return Err(err(
            field,
            "INVALID_TENANT_ID",
            format!("tenant ID must be 3-63 chars, lowercase alphanumeric and hyphens, no leading/trailing hyphens: {id}"),
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
    // ページネーション上限: 4言語共通で100に統一（H-18）
    if !(1..=100).contains(&per_page) {
        return Err(err(
            field,
            "INVALID_PAGINATION",
            format!("per_page must be 1-100, got {per_page}"),
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
#[allow(clippy::unwrap_used)]
mod tests {
    use chrono::{TimeZone, Utc};

    use crate::{validate, ValidationErrors};

    use super::*;

    // per_page が 100 で成功し 101 でエラーになることを確認する。
    #[test]
    fn test_validate_pagination_range_100() {
        assert!(validate_pagination("pagination", 1, 100).is_ok());
        assert!(validate_pagination("pagination", 1, 101).is_err());
    }

    // UTC 日時の日付範囲検証で順序が正しい場合成功し逆順の場合エラーになることを確認する。
    #[test]
    fn test_validate_date_range_datetime_utc() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap();
        assert!(validate_date_range("date_range", start, end).is_ok());
        assert!(validate_date_range("date_range", end, start).is_err());
    }

    // validate! マクロが複数の無効な入力からエラーを収集することを確認する。
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
