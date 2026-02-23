use serde_json::Value;

/// テスト用アサーションヘルパー。
pub struct AssertionHelper;

impl AssertionHelper {
    /// JSON 部分一致アサーション。
    ///
    /// `actual` が `expected` の全キー・値を含んでいることを検証する。
    /// `actual` に余分なキーがあっても失敗しない。
    pub fn assert_json_contains(actual: &str, expected: &str) {
        let actual_val: Value =
            serde_json::from_str(actual).expect("actual is not valid JSON");
        let expected_val: Value =
            serde_json::from_str(expected).expect("expected is not valid JSON");
        assert!(
            json_contains(&actual_val, &expected_val),
            "JSON partial match failed.\nActual: {}\nExpected: {}",
            actual,
            expected
        );
    }

    /// イベント一覧に指定タイプのイベントが含まれていることを検証する。
    ///
    /// 各イベントは `{"type": "..."}` の形式であることを想定する。
    pub fn assert_event_emitted(events: &[Value], event_type: &str) {
        let found = events.iter().any(|e| {
            e.get("type")
                .and_then(|t| t.as_str())
                .map(|t| t == event_type)
                .unwrap_or(false)
        });
        assert!(
            found,
            "Event '{}' not found in events: {:?}",
            event_type, events
        );
    }

    /// JSON 値が null でないことを検証する。
    pub fn assert_not_null(json_str: &str, path: &str) {
        let val: Value = serde_json::from_str(json_str).expect("invalid JSON");
        let result = json_path(&val, path);
        assert!(
            result.is_some() && !result.unwrap().is_null(),
            "Expected non-null at path '{}' in: {}",
            path,
            json_str
        );
    }
}

fn json_contains(actual: &Value, expected: &Value) -> bool {
    match (actual, expected) {
        (Value::Object(a), Value::Object(e)) => {
            e.iter().all(|(k, v)| a.get(k).map(|av| json_contains(av, v)).unwrap_or(false))
        }
        (Value::Array(a), Value::Array(e)) => {
            e.iter().all(|ev| a.iter().any(|av| json_contains(av, ev)))
        }
        _ => actual == expected,
    }
}

fn json_path<'a>(val: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = val;
    for key in path.split('.') {
        current = current.get(key)?;
    }
    Some(current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_json_contains_simple() {
        AssertionHelper::assert_json_contains(
            r#"{"id":"1","status":"ok","extra":"ignored"}"#,
            r#"{"id":"1","status":"ok"}"#,
        );
    }

    #[test]
    fn test_json_contains_nested() {
        AssertionHelper::assert_json_contains(
            r#"{"user":{"id":"1","name":"test"},"status":"ok"}"#,
            r#"{"user":{"id":"1"}}"#,
        );
    }

    #[test]
    #[should_panic(expected = "JSON partial match failed")]
    fn test_json_contains_mismatch() {
        AssertionHelper::assert_json_contains(
            r#"{"id":"1"}"#,
            r#"{"id":"2"}"#,
        );
    }

    #[test]
    fn test_event_emitted() {
        let events = vec![
            json!({"type": "created", "id": "1"}),
            json!({"type": "updated", "id": "2"}),
        ];
        AssertionHelper::assert_event_emitted(&events, "created");
        AssertionHelper::assert_event_emitted(&events, "updated");
    }

    #[test]
    #[should_panic(expected = "not found")]
    fn test_event_not_emitted() {
        let events = vec![json!({"type": "created"})];
        AssertionHelper::assert_event_emitted(&events, "deleted");
    }

    #[test]
    fn test_assert_not_null() {
        AssertionHelper::assert_not_null(r#"{"data":{"id":"1"}}"#, "data.id");
    }

    #[test]
    #[should_panic(expected = "non-null")]
    fn test_assert_not_null_missing() {
        AssertionHelper::assert_not_null(r#"{"data":{}}"#, "data.id");
    }

    #[test]
    fn test_json_contains_array() {
        AssertionHelper::assert_json_contains(
            r#"{"items":[{"id":"1"},{"id":"2"},{"id":"3"}]}"#,
            r#"{"items":[{"id":"2"}]}"#,
        );
    }
}
