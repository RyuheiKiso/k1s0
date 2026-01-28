//! シンボル機能
//!
//! manifest.json のシンボル情報を提供する。
//! - ドキュメントシンボル（Document Symbol）
//! - ワークスペースシンボル（Workspace Symbol）

use tower_lsp::lsp_types::{
    DocumentSymbol, Location, Position, Range, SymbolInformation, SymbolKind, Url,
};

/// manifest.json からドキュメントシンボルを抽出
pub fn extract_document_symbols(content: &str) -> Vec<DocumentSymbol> {
    let mut symbols = Vec::new();

    // JSON をパースしてシンボルを抽出
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(content) {
        if let Some(obj) = value.as_object() {
            for (key, val) in obj {
                if let Some(range) = find_key_range(content, key, 0) {
                    let symbol = create_document_symbol(key, val, content, range);
                    symbols.push(symbol);
                }
            }
        }
    }

    symbols
}

/// キーの位置を検索
fn find_key_range(content: &str, key: &str, search_from: usize) -> Option<Range> {
    let search_str = format!("\"{}\"", key);
    let content_from = &content[search_from..];

    if let Some(pos) = content_from.find(&search_str) {
        let absolute_pos = search_from + pos;
        let (line, character) = offset_to_position(content, absolute_pos);
        let end_character = character + search_str.len() as u32;

        Some(Range {
            start: Position { line, character },
            end: Position { line, character: end_character },
        })
    } else {
        None
    }
}

/// バイトオフセットを行・列位置に変換
fn offset_to_position(content: &str, offset: usize) -> (u32, u32) {
    let mut line = 0u32;
    let mut line_start = 0;

    for (i, ch) in content.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            line_start = i + 1;
        }
    }

    let character = (offset - line_start) as u32;
    (line, character)
}

/// DocumentSymbol を作成
fn create_document_symbol(
    name: &str,
    value: &serde_json::Value,
    content: &str,
    range: Range,
) -> DocumentSymbol {
    let kind = match value {
        serde_json::Value::Object(_) => SymbolKind::OBJECT,
        serde_json::Value::Array(_) => SymbolKind::ARRAY,
        serde_json::Value::String(_) => SymbolKind::STRING,
        serde_json::Value::Number(_) => SymbolKind::NUMBER,
        serde_json::Value::Bool(_) => SymbolKind::BOOLEAN,
        serde_json::Value::Null => SymbolKind::NULL,
    };

    let detail = match value {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Number(n) => Some(n.to_string()),
        serde_json::Value::Bool(b) => Some(b.to_string()),
        serde_json::Value::Object(obj) => Some(format!("{} properties", obj.len())),
        serde_json::Value::Array(arr) => Some(format!("{} items", arr.len())),
        serde_json::Value::Null => Some("null".to_string()),
    };

    // 子シンボルを再帰的に抽出
    let children = match value {
        serde_json::Value::Object(obj) => {
            let mut child_symbols = Vec::new();
            for (child_key, child_val) in obj {
                // 簡易的な実装: ネストした位置を推定
                if let Some(child_range) = find_key_range(content, child_key, 0) {
                    let child_symbol = create_document_symbol(child_key, child_val, content, child_range);
                    child_symbols.push(child_symbol);
                }
            }
            if child_symbols.is_empty() {
                None
            } else {
                Some(child_symbols)
            }
        }
        _ => None,
    };

    #[allow(deprecated)] // deprecated field is required for older clients
    DocumentSymbol {
        name: name.to_string(),
        detail,
        kind,
        tags: None,
        deprecated: None,
        range,
        selection_range: range,
        children,
    }
}

/// ワークスペースシンボルを検索
pub fn search_workspace_symbols(
    query: &str,
    manifest_files: &[(Url, String)],
) -> Vec<SymbolInformation> {
    let mut symbols = Vec::new();
    let query_lower = query.to_lowercase();

    for (uri, content) in manifest_files {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(content) {
            if let Some(obj) = value.as_object() {
                collect_matching_symbols(&query_lower, obj, uri, content, &mut symbols, None);
            }
        }
    }

    symbols
}

/// マッチするシンボルを収集
fn collect_matching_symbols(
    query: &str,
    obj: &serde_json::Map<String, serde_json::Value>,
    uri: &Url,
    content: &str,
    symbols: &mut Vec<SymbolInformation>,
    container_name: Option<&str>,
) {
    for (key, value) in obj {
        // クエリにマッチするかチェック
        if query.is_empty() || key.to_lowercase().contains(query) {
            if let Some(range) = find_key_range(content, key, 0) {
                let kind = match value {
                    serde_json::Value::Object(_) => SymbolKind::OBJECT,
                    serde_json::Value::Array(_) => SymbolKind::ARRAY,
                    serde_json::Value::String(_) => SymbolKind::STRING,
                    serde_json::Value::Number(_) => SymbolKind::NUMBER,
                    serde_json::Value::Bool(_) => SymbolKind::BOOLEAN,
                    serde_json::Value::Null => SymbolKind::NULL,
                };

                #[allow(deprecated)] // deprecated field is required for older clients
                symbols.push(SymbolInformation {
                    name: key.clone(),
                    kind,
                    tags: None,
                    deprecated: None,
                    location: Location {
                        uri: uri.clone(),
                        range,
                    },
                    container_name: container_name.map(|s| s.to_string()),
                });
            }
        }

        // 子オブジェクトを再帰的に検索
        if let serde_json::Value::Object(child_obj) = value {
            collect_matching_symbols(query, child_obj, uri, content, symbols, Some(key));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_document_symbols() {
        let content = r#"{
  "schema_version": "1.0.0",
  "template": {
    "name": "backend-rust",
    "version": "0.1.0"
  }
}"#;

        let symbols = extract_document_symbols(content);

        assert!(!symbols.is_empty());
        assert!(symbols.iter().any(|s| s.name == "schema_version"));
        assert!(symbols.iter().any(|s| s.name == "template"));
    }

    #[test]
    fn test_offset_to_position() {
        let content = "line1\nline2\nline3";

        let (line, char) = offset_to_position(content, 0);
        assert_eq!(line, 0);
        assert_eq!(char, 0);

        let (line, char) = offset_to_position(content, 6);
        assert_eq!(line, 1);
        assert_eq!(char, 0);

        let (line, char) = offset_to_position(content, 8);
        assert_eq!(line, 1);
        assert_eq!(char, 2);
    }

    #[test]
    fn test_find_key_range() {
        let content = r#"{"name": "value"}"#;

        let range = find_key_range(content, "name", 0);
        assert!(range.is_some());

        let range = range.unwrap();
        assert_eq!(range.start.line, 0);
        assert_eq!(range.start.character, 1); // After opening brace
    }

    #[test]
    fn test_search_workspace_symbols() {
        let content = r#"{"template": {"name": "test"}}"#;
        let uri = Url::parse("file:///test/manifest.json").unwrap();

        let symbols = search_workspace_symbols("template", &[(uri, content.to_string())]);

        assert!(!symbols.is_empty());
        assert!(symbols.iter().any(|s| s.name == "template"));
    }

    #[test]
    fn test_search_workspace_symbols_empty_query() {
        let content = r#"{"a": 1, "b": 2}"#;
        let uri = Url::parse("file:///test/manifest.json").unwrap();

        let symbols = search_workspace_symbols("", &[(uri, content.to_string())]);

        // 空のクエリはすべてのシンボルを返す
        assert_eq!(symbols.len(), 2);
    }
}
