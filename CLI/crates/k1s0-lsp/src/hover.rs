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
            content.push_str(&format!("`enum`\n\n**有効な値:**\n"));
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
}
