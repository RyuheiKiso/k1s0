use serde_json::Value;

/// テスト用アサーションヘルパー。
pub struct AssertionHelper;

impl AssertionHelper {
    /// JSON 部分一致アサーション。
    ///
    /// `actual` が `expected` の全キー・値を含んでいることを検証する。
    /// `actual` に余分なキーがあっても失敗しない。
    pub fn assert_json_contains(actual: &str, expected: &str) {
        let actual_val: Value = serde_json::from_str(actual).expect("actual is not valid JSON");
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
        // is_some_and で Option を安全にアンラップし、null チェックを行う
        assert!(
            result.is_some_and(|v| !v.is_null()),
            "Expected non-null at path '{}' in: {}",
            path,
            json_str
        );
    }
}

fn json_contains(actual: &Value, expected: &Value) -> bool {
    match (actual, expected) {
        (Value::Object(a), Value::Object(e)) => e
            .iter()
            .all(|(k, v)| a.get(k).map(|av| json_contains(av, v)).unwrap_or(false)),
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

    // 余分なキーを無視して JSON の部分一致アサーションが成功することを確認する。
    #[test]
    fn test_json_contains_simple() {
        AssertionHelper::assert_json_contains(
            r#"{"id":"1","status":"ok","extra":"ignored"}"#,
            r#"{"id":"1","status":"ok"}"#,
        );
    }

    // ネストされた JSON オブジェクトの部分一致が正しく動作することを確認する。
    #[test]
    fn test_json_contains_nested() {
        AssertionHelper::assert_json_contains(
            r#"{"user":{"id":"1","name":"test"},"status":"ok"}"#,
            r#"{"user":{"id":"1"}}"#,
        );
    }

    // 値が一致しない JSON で部分一致アサーションがパニックすることを確認する。
    #[test]
    #[should_panic(expected = "JSON partial match failed")]
    fn test_json_contains_mismatch() {
        AssertionHelper::assert_json_contains(r#"{"id":"1"}"#, r#"{"id":"2"}"#);
    }

    // イベント一覧に指定したタイプのイベントが含まれることを検証できることを確認する。
    #[test]
    fn test_event_emitted() {
        let events = vec![
            json!({"type": "created", "id": "1"}),
            json!({"type": "updated", "id": "2"}),
        ];
        AssertionHelper::assert_event_emitted(&events, "created");
        AssertionHelper::assert_event_emitted(&events, "updated");
    }

    // 存在しないイベントタイプを検証すると "not found" でパニックすることを確認する。
    #[test]
    #[should_panic(expected = "not found")]
    fn test_event_not_emitted() {
        let events = vec![json!({"type": "created"})];
        AssertionHelper::assert_event_emitted(&events, "deleted");
    }

    // 指定パスの値が非 null であることを検証するアサーションが成功することを確認する。
    #[test]
    fn test_assert_not_null() {
        AssertionHelper::assert_not_null(r#"{"data":{"id":"1"}}"#, "data.id");
    }

    // 存在しないパスを検証すると "non-null" でパニックすることを確認する。
    #[test]
    #[should_panic(expected = "non-null")]
    fn test_assert_not_null_missing() {
        AssertionHelper::assert_not_null(r#"{"data":{}}"#, "data.id");
    }

    // JSON 配列を対象とした部分一致アサーションが正しく機能することを確認する。
    #[test]
    fn test_json_contains_array() {
        AssertionHelper::assert_json_contains(
            r#"{"items":[{"id":"1"},{"id":"2"},{"id":"3"}]}"#,
            r#"{"items":[{"id":"2"}]}"#,
        );
    }
}
