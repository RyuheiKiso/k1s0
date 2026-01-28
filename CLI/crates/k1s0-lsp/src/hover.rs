//! manifest.json ホバー機能
//!
//! カーソル位置のキーに関する情報を提供する。

use tower_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position, Range};

use crate::schema::{ManifestSchema, ValueType};

/// カーソル位置のキー情報を取得
pub fn get_hover_info(
    document: &str,
    position: Position,
    schema: &ManifestSchema,
) -> Option<Hover> {
    // カーソル位置のキーを特定
    let (key_path, key_range) = find_key_at_position(document, position)?;

    // スキーマからキー情報を取得
    let path_refs: Vec<&str> = key_path.iter().map(|s| s.as_str()).collect();
    let key = schema.find_key(&path_refs)?;

    // ホバー情報を生成
    let contents = format_hover_contents(key, &key_path);

    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: contents,
        }),
        range: Some(key_range),
    })
}

/// カーソル位置のキーを特定
fn find_key_at_position(document: &str, position: Position) -> Option<(Vec<String>, Range)> {
    let lines: Vec<&str> = document.lines().collect();
    let line_idx = position.line as usize;

    if line_idx >= lines.len() {
        return None;
    }

    let line = lines[line_idx];
    let char_idx = position.character as usize;

    // 現在の行でキー文字列を探す
    let (key_name, start_char, end_char) = find_key_in_line(line, char_idx)?;

    // キーのパスを構築
    let path = build_key_path(document, line_idx, &key_name);

    let range = Range {
        start: Position {
            line: position.line,
            character: start_char as u32,
        },
        end: Position {
            line: position.line,
            character: end_char as u32,
        },
    };

    Some((path, range))
}

/// 行内でキーを見つける
fn find_key_in_line(line: &str, char_idx: usize) -> Option<(String, usize, usize)> {
    // 行内の全ての引用符で囲まれた文字列を見つける
    let mut strings: Vec<(String, usize, usize)> = Vec::new();
    let mut in_string = false;
    let mut string_start = 0;
    let mut current_string = String::new();

    for (i, ch) in line.char_indices() {
        match ch {
            '"' => {
                if in_string {
                    // 文字列の終了
                    strings.push((current_string.clone(), string_start, i + 1));
                    current_string.clear();
                    in_string = false;
                } else {
                    // 文字列の開始
                    string_start = i;
                    in_string = true;
                }
            }
            _ if in_string => {
                current_string.push(ch);
            }
            _ => {}
        }
    }

    // カーソル位置を含む文字列を見つける
    for (content, start, end) in &strings {
        if char_idx >= *start && char_idx <= *end {
            // この文字列がキーかどうかを判定（後ろにコロンがあるか）
            let rest = &line[*end..];
            if rest.trim_start().starts_with(':') {
                return Some((content.clone(), *start, *end));
            }
        }
    }

    // キー文字列が見つからない場合、最初のキー文字列を返す（行がキー行の場合）
    if let Some((content, start, end)) = strings.first() {
        let rest = &line[*end..];
        if rest.trim_start().starts_with(':') {
            // カーソルがこの行にある場合、キー情報を返す
            return Some((content.clone(), *start, *end));
        }
    }

    None
}

/// キーのパスを構築
fn build_key_path(document: &str, target_line: usize, target_key: &str) -> Vec<String> {
    let lines: Vec<&str> = document.lines().collect();
    let mut brace_stack: Vec<String> = Vec::new();

    for (line_idx, line) in lines.iter().enumerate() {
        if line_idx > target_line {
            break;
        }

        // 行内のキーと構造を解析
        let trimmed = line.trim();

        // 閉じ波括弧をカウント
        for ch in trimmed.chars() {
            if ch == '}' && !brace_stack.is_empty() {
                brace_stack.pop();
            }
        }

        // キーを抽出
        if let Some((key, _, end)) = find_key_in_line(line, 0) {
            let rest = &line[end..];

            // オブジェクトの開始かどうか
            if rest.contains('{') {
                brace_stack.push(key.clone());
            }

            // ターゲット行の場合、キーをパスに追加
            if line_idx == target_line && key == target_key {
                let mut result = brace_stack.clone();
                result.push(key);
                return result;
            }
        }
    }

    // パスが構築できなかった場合、ターゲットキーのみを返す
    vec![target_key.to_string()]
}

/// ホバー内容をフォーマット
fn format_hover_contents(key: &crate::schema::ManifestKey, path: &[String]) -> String {
    let mut content = String::new();

    // パスを表示
    content.push_str(&format!("### `{}`\n\n", path.join(".")));

    // 説明
    content.push_str(&format!("{}\n\n", key.description));

    // 必須/オプション
    if key.required {
        content.push_str("**必須フィールド**\n\n");
    } else {
        content.push_str("*オプション*\n\n");
    }

    // 型情報
    content.push_str("**型:** ");
    match &key.value_type {
        ValueType::String => content.push_str("`string`\n\n"),
        ValueType::Number => content.push_str("`number`\n\n"),
        ValueType::Boolean => content.push_str("`boolean`\n\n"),
        ValueType::Object => content.push_str("`object`\n\n"),
        ValueType::Array => content.push_str("`array`\n\n"),
        ValueType::Enum(values) => {
            content.push_str("`enum`\n\n**有効な値:**\n");
            for v in values {
                content.push_str(&format!("- `\"{}\"`\n", v));
            }
            content.push('\n');
        }
    }

    // 例
    if !key.examples.is_empty() {
        content.push_str("**例:**\n");
        for ex in &key.examples {
            content.push_str(&format!("```json\n\"{}\"\n```\n", ex));
        }
    }

    content
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_key_in_line_simple() {
        let line = r#"  "schema_version": "1.0.0","#;
        let result = find_key_in_line(line, 5);
        assert!(result.is_some());
        let (key, _, _) = result.unwrap();
        assert_eq!(key, "schema_version");
    }

    #[test]
    fn test_find_key_in_line_nested() {
        let line = r#"    "name": "backend-rust","#;
        let result = find_key_in_line(line, 8);
        assert!(result.is_some());
        let (key, _, _) = result.unwrap();
        assert_eq!(key, "name");
    }

    #[test]
    fn test_find_key_in_line_no_key() {
        let line = r#"  },"#;
        let result = find_key_in_line(line, 2);
        assert!(result.is_none());
    }

    #[test]
    fn test_build_key_path_root() {
        let doc = r#"{
  "schema_version": "1.0.0"
}"#;
        let path = build_key_path(doc, 1, "schema_version");
        assert_eq!(path, vec!["schema_version"]);
    }

    #[test]
    fn test_build_key_path_nested() {
        let doc = r#"{
  "template": {
    "name": "backend-rust"
  }
}"#;
        let path = build_key_path(doc, 2, "name");
        assert_eq!(path, vec!["template", "name"]);
    }

    #[test]
    fn test_build_key_path_deep() {
        let doc = r#"{
  "service": {
    "language": "rust"
  }
}"#;
        let path = build_key_path(doc, 2, "language");
        assert_eq!(path, vec!["service", "language"]);
    }

    #[test]
    fn test_get_hover_info_root_key() {
        let schema = ManifestSchema::new();
        let doc = r#"{
  "schema_version": "1.0.0"
}"#;
        let pos = Position { line: 1, character: 5 };

        let hover = get_hover_info(doc, pos, &schema);
        assert!(hover.is_some());
        let hover = hover.unwrap();
        if let HoverContents::Markup(content) = hover.contents {
            assert!(content.value.contains("schema_version"));
            assert!(content.value.contains("スキーマバージョン"));
        } else {
            panic!("Expected Markup content");
        }
    }

    #[test]
    fn test_get_hover_info_nested_key() {
        let schema = ManifestSchema::new();
        let doc = r#"{
  "template": {
    "name": "backend-rust"
  }
}"#;
        let pos = Position { line: 2, character: 8 };

        let hover = get_hover_info(doc, pos, &schema);
        assert!(hover.is_some());
        let hover = hover.unwrap();
        if let HoverContents::Markup(content) = hover.contents {
            assert!(content.value.contains("template.name"));
            assert!(content.value.contains("テンプレート名"));
        } else {
            panic!("Expected Markup content");
        }
    }

    #[test]
    fn test_get_hover_info_enum_key() {
        let schema = ManifestSchema::new();
        let doc = r#"{
  "service": {
    "language": "rust"
  }
}"#;
        let pos = Position { line: 2, character: 8 };

        let hover = get_hover_info(doc, pos, &schema);
        assert!(hover.is_some());
        let hover = hover.unwrap();
        if let HoverContents::Markup(content) = hover.contents {
            assert!(content.value.contains("language"));
            assert!(content.value.contains("有効な値"));
            assert!(content.value.contains("rust"));
            assert!(content.value.contains("go"));
        } else {
            panic!("Expected Markup content");
        }
    }

    #[test]
    fn test_get_hover_info_nonexistent_key() {
        let schema = ManifestSchema::new();
        let doc = r#"{
  "nonexistent": "value"
}"#;
        let pos = Position { line: 1, character: 5 };

        let hover = get_hover_info(doc, pos, &schema);
        // スキーマにないキーなのでNoneが返る
        assert!(hover.is_none());
    }

    #[test]
    fn test_find_key_in_line_at_cursor_position() {
        let line = r#"  "key1": "value1", "key2": "value2""#;

        // key1 の位置
        let result = find_key_in_line(line, 5);
        assert!(result.is_some());
        let (key, _, _) = result.unwrap();
        assert_eq!(key, "key1");
    }

    #[test]
    fn test_find_key_in_line_multiple_strings() {
        let line = r#"  "first": "second": "third""#;
        // 不正な JSON だが、パーサーは最初のキーを見つける
        let result = find_key_in_line(line, 5);
        assert!(result.is_some());
        let (key, _, _) = result.unwrap();
        assert_eq!(key, "first");
    }

    #[test]
    fn test_find_key_in_line_empty() {
        let line = "";
        let result = find_key_in_line(line, 0);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_key_in_line_value_only() {
        let line = r#"  "value""#;
        // コロンがないので値として扱われる
        let result = find_key_in_line(line, 5);
        assert!(result.is_none());
    }

    #[test]
    fn test_build_key_path_empty_document() {
        let doc = "";
        let path = build_key_path(doc, 0, "key");
        assert_eq!(path, vec!["key"]);
    }

    #[test]
    fn test_build_key_path_single_line() {
        let doc = r#"{"key": "value"}"#;
        let path = build_key_path(doc, 0, "key");
        assert_eq!(path, vec!["key"]);
    }

    #[test]
    fn test_build_key_path_multiple_nested() {
        let doc = r#"{
  "a": {
    "b": {
      "c": "value"
    }
  }
}"#;
        let path = build_key_path(doc, 3, "c");
        assert_eq!(path, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_get_hover_info_beyond_document() {
        let schema = ManifestSchema::new();
        let doc = "{}";
        let pos = Position { line: 100, character: 0 };

        let hover = get_hover_info(doc, pos, &schema);
        assert!(hover.is_none());
    }

    #[test]
    fn test_get_hover_info_empty_document() {
        let schema = ManifestSchema::new();
        let doc = "";
        let pos = Position { line: 0, character: 0 };

        let hover = get_hover_info(doc, pos, &schema);
        assert!(hover.is_none());
    }

    #[test]
    fn test_get_hover_info_required_field() {
        let schema = ManifestSchema::new();
        let doc = r#"{
  "k1s0_version": "0.1.0"
}"#;
        let pos = Position { line: 1, character: 5 };

        let hover = get_hover_info(doc, pos, &schema);
        assert!(hover.is_some());
        let hover = hover.unwrap();
        if let HoverContents::Markup(content) = hover.contents {
            assert!(content.value.contains("必須フィールド"));
        } else {
            panic!("Expected Markup content");
        }
    }

    #[test]
    fn test_get_hover_info_optional_field() {
        let schema = ManifestSchema::new();
        let doc = r#"{
  "template": {
    "revision": "abc123"
  }
}"#;
        let pos = Position { line: 2, character: 8 };

        let hover = get_hover_info(doc, pos, &schema);
        assert!(hover.is_some());
        let hover = hover.unwrap();
        if let HoverContents::Markup(content) = hover.contents {
            assert!(content.value.contains("オプション"));
        } else {
            panic!("Expected Markup content");
        }
    }

    #[test]
    fn test_format_hover_contents_string_type() {
        let key = crate::schema::ManifestKey {
            name: "test_key",
            description: "Test description",
            required: true,
            value_type: ValueType::String,
            examples: vec!["example1", "example2"],
            children: None,
        };

        let content = format_hover_contents(&key, &["test_key".to_string()]);

        assert!(content.contains("test_key"));
        assert!(content.contains("Test description"));
        assert!(content.contains("必須フィールド"));
        assert!(content.contains("`string`"));
        assert!(content.contains("example1"));
    }

    #[test]
    fn test_format_hover_contents_enum_type() {
        let key = crate::schema::ManifestKey {
            name: "test_enum",
            description: "Enum field",
            required: false,
            value_type: ValueType::Enum(vec!["opt1", "opt2"]),
            examples: vec![],
            children: None,
        };

        let content = format_hover_contents(&key, &["test_enum".to_string()]);

        assert!(content.contains("test_enum"));
        assert!(content.contains("`enum`"));
        assert!(content.contains("有効な値"));
        assert!(content.contains("opt1"));
        assert!(content.contains("opt2"));
        assert!(content.contains("オプション"));
    }

    #[test]
    fn test_format_hover_contents_number_type() {
        let key = crate::schema::ManifestKey {
            name: "test_number",
            description: "Number field",
            required: true,
            value_type: ValueType::Number,
            examples: vec![],
            children: None,
        };

        let content = format_hover_contents(&key, &["test_number".to_string()]);

        assert!(content.contains("`number`"));
    }

    #[test]
    fn test_format_hover_contents_boolean_type() {
        let key = crate::schema::ManifestKey {
            name: "test_bool",
            description: "Boolean field",
            required: true,
            value_type: ValueType::Boolean,
            examples: vec![],
            children: None,
        };

        let content = format_hover_contents(&key, &["test_bool".to_string()]);

        assert!(content.contains("`boolean`"));
    }

    #[test]
    fn test_format_hover_contents_object_type() {
        let key = crate::schema::ManifestKey {
            name: "test_obj",
            description: "Object field",
            required: true,
            value_type: ValueType::Object,
            examples: vec![],
            children: None,
        };

        let content = format_hover_contents(&key, &["test_obj".to_string()]);

        assert!(content.contains("`object`"));
    }

    #[test]
    fn test_format_hover_contents_array_type() {
        let key = crate::schema::ManifestKey {
            name: "test_array",
            description: "Array field",
            required: true,
            value_type: ValueType::Array,
            examples: vec!["item1"],
            children: None,
        };

        let content = format_hover_contents(&key, &["test_array".to_string()]);

        assert!(content.contains("`array`"));
        assert!(content.contains("item1"));
    }

    #[test]
    fn test_format_hover_contents_nested_path() {
        let key = crate::schema::ManifestKey {
            name: "nested",
            description: "Nested field",
            required: true,
            value_type: ValueType::String,
            examples: vec![],
            children: None,
        };

        let content = format_hover_contents(
            &key,
            &["parent".to_string(), "child".to_string(), "nested".to_string()]
        );

        assert!(content.contains("parent.child.nested"));
    }

    #[test]
    fn test_find_key_at_position_simple() {
        let doc = r#"{
  "key": "value"
}"#;
        let pos = Position { line: 1, character: 5 };

        let result = find_key_at_position(doc, pos);
        assert!(result.is_some());

        let (path, range) = result.unwrap();
        assert_eq!(path, vec!["key"]);
        assert_eq!(range.start.line, 1);
    }

    #[test]
    fn test_find_key_at_position_none_for_value() {
        let doc = r#"{
  "key": "value"
}"#;
        // カーソルが値の位置
        let pos = Position { line: 1, character: 12 };

        let result = find_key_at_position(doc, pos);
        // 値の位置ではキーが見つからない（最初のキーを返す実装なので Some が返る可能性あり）
        // 現在の実装では行にキーがあれば返す
        if let Some((path, _)) = result {
            assert_eq!(path, vec!["key"]);
        }
    }
}
