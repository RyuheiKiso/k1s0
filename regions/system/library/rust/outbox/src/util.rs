/// アウトボックスイベント処理で共通的に使用するユーティリティ関数群。
/// ISO 8601 (RFC 3339) 文字列を `chrono::DateTime`<Utc> に変換する。
/// 各サービスの `outbox_poller` で重複していた `parse_datetime` を共通化したもの。
#[must_use] 
pub fn parse_datetime(s: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&chrono::Utc))
}

/// JSON ペイロードから文字列フィールドを取得するヘルパー。
/// 指定キーが存在しない場合や文字列でない場合は空文字列を返す。
#[must_use] 
pub fn json_str(payload: &serde_json::Value, key: &str) -> String {
    payload
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string()
}

/// JSON ペイロードから i64 フィールドを取得するヘルパー。
/// 指定キーが存在しない場合や数値でない場合は 0 を返す。
#[must_use] 
pub fn json_i64(payload: &serde_json::Value, key: &str) -> i64 {
    payload
        .get(key)
        .and_then(serde_json::Value::as_i64)
        .unwrap_or_default()
}

/// JSON ペイロードから i32 フィールドを取得するヘルパー。
/// 指定キーが存在しない場合や数値でない場合は 0 を返す。
#[must_use] 
pub fn json_i32(payload: &serde_json::Value, key: &str) -> i32 {
    json_i64(payload, key) as i32
}

/// JSON ペイロードから日時フィールドを取得するヘルパー。
/// 指定キーの文字列値を `parse_datetime` でパースする。
pub fn json_datetime(
    payload: &serde_json::Value,
    key: &str,
) -> Option<chrono::DateTime<chrono::Utc>> {
    payload
        .get(key)
        .and_then(|v| v.as_str())
        .and_then(parse_datetime)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // RFC 3339 形式の日時文字列が正しくパースされることを確認する
    #[test]
    fn test_parse_datetime_valid() {
        let result = parse_datetime("2024-01-15T10:30:00Z");
        assert!(result.is_some());
        let dt = result.unwrap();
        assert_eq!(dt.to_rfc3339(), "2024-01-15T10:30:00+00:00");
    }

    // 不正な日時文字列の場合 None を返すことを確認する
    #[test]
    fn test_parse_datetime_invalid() {
        assert!(parse_datetime("not-a-date").is_none());
        assert!(parse_datetime("").is_none());
    }

    // タイムゾーンオフセット付きの日時文字列が UTC に変換されることを確認する
    #[test]
    fn test_parse_datetime_with_offset() {
        let result = parse_datetime("2024-01-15T10:30:00+09:00");
        assert!(result.is_some());
        let dt = result.unwrap();
        assert_eq!(dt.to_rfc3339(), "2024-01-15T01:30:00+00:00");
    }

    // json_str が存在するキーの文字列値を返すことを確認する
    #[test]
    fn test_json_str_present() {
        let v = serde_json::json!({"name": "test"});
        assert_eq!(json_str(&v, "name"), "test");
    }

    // json_str が存在しないキーに対して空文字列を返すことを確認する
    #[test]
    fn test_json_str_missing() {
        let v = serde_json::json!({"name": "test"});
        assert_eq!(json_str(&v, "missing"), "");
    }

    // json_i64 が存在するキーの数値を返すことを確認する
    #[test]
    fn test_json_i64_present() {
        let v = serde_json::json!({"amount": 1000});
        assert_eq!(json_i64(&v, "amount"), 1000);
    }

    // json_i64 が存在しないキーに対して 0 を返すことを確認する
    #[test]
    fn test_json_i64_missing() {
        let v = serde_json::json!({"amount": 1000});
        assert_eq!(json_i64(&v, "missing"), 0);
    }

    // json_i32 が i64 を i32 に変換して返すことを確認する
    #[test]
    fn test_json_i32() {
        let v = serde_json::json!({"quantity": 5});
        assert_eq!(json_i32(&v, "quantity"), 5);
    }

    // json_datetime が有効な日時文字列をパースすることを確認する
    #[test]
    fn test_json_datetime_present() {
        let v = serde_json::json!({"created_at": "2024-01-15T10:30:00Z"});
        assert!(json_datetime(&v, "created_at").is_some());
    }

    // json_datetime が存在しないキーに対して None を返すことを確認する
    #[test]
    fn test_json_datetime_missing() {
        let v = serde_json::json!({"other": "value"});
        assert!(json_datetime(&v, "created_at").is_none());
    }
}
