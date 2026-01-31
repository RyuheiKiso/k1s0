//! Document Link 機能
//!
//! manifest.json 内のパス値をクリック可能なリンクに変換する。

use std::path::{Path, PathBuf};

use tower_lsp::lsp_types::{DocumentLink, Position, Range, Url};

/// manifest.json 内のパス参照を DocumentLink として返す
///
/// 対象キー:
/// - `template.path` → テンプレートディレクトリ
/// - `managed_paths` / `protected_paths` の各要素 → 実ディレクトリ/ファイル
/// - `checksums` のキー → 実ファイル
/// - `template_snapshot` のキー → 実ファイル
pub fn get_document_links(
    text: &str,
    _uri: &Url,
    workspace_root: Option<&PathBuf>,
) -> Vec<DocumentLink> {
    let mut links = Vec::new();

    let root = match workspace_root {
        Some(r) => r,
        None => return links,
    };

    let json: serde_json::Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(_) => return links,
    };

    // template.path
    if let Some(path_val) = json
        .pointer("/template/path")
        .and_then(|v| v.as_str())
    {
        if let Some(range) = find_value_range(text, "path", path_val) {
            let target_path = root.join(path_val);
            if let Ok(target_uri) = Url::from_file_path(&target_path) {
                links.push(DocumentLink {
                    range,
                    target: Some(target_uri),
                    tooltip: Some(format!("テンプレートパス: {path_val}")),
                    data: None,
                });
            }
        }
    }

    // managed_paths
    collect_array_links(text, &json, "managed_paths", root, &mut links);

    // protected_paths
    collect_array_links(text, &json, "protected_paths", root, &mut links);

    // checksums — キーがファイルパス
    collect_object_key_links(text, &json, "checksums", root, &mut links);

    // template_snapshot — キーがファイルパス
    collect_object_key_links(text, &json, "template_snapshot", root, &mut links);

    links
}

/// 配列内の文字列要素をリンク化
fn collect_array_links(
    text: &str,
    json: &serde_json::Value,
    key: &str,
    root: &Path,
    links: &mut Vec<DocumentLink>,
) {
    if let Some(arr) = json.get(key).and_then(|v| v.as_array()) {
        for item in arr {
            if let Some(path_str) = item.as_str() {
                if let Some(range) = find_string_literal_range(text, path_str) {
                    let target_path = root.join(path_str);
                    if let Ok(target_uri) = Url::from_file_path(&target_path) {
                        links.push(DocumentLink {
                            range,
                            target: Some(target_uri),
                            tooltip: Some(format!("{key}: {path_str}")),
                            data: None,
                        });
                    }
                }
            }
        }
    }
}

/// オブジェクトのキーをリンク化
fn collect_object_key_links(
    text: &str,
    json: &serde_json::Value,
    key: &str,
    root: &Path,
    links: &mut Vec<DocumentLink>,
) {
    if let Some(obj) = json.get(key).and_then(|v| v.as_object()) {
        for file_path in obj.keys() {
            if let Some(range) = find_string_literal_range(text, file_path) {
                let target_path = root.join(file_path);
                if let Ok(target_uri) = Url::from_file_path(&target_path) {
                    links.push(DocumentLink {
                        range,
                        target: Some(target_uri),
                        tooltip: Some(format!("{key}: {file_path}")),
                        data: None,
                    });
                }
            }
        }
    }
}

/// テキスト中の `"key": "value"` パターンから value の Range を取得
fn find_value_range(text: &str, key: &str, value: &str) -> Option<Range> {
    let pattern = format!("\"{key}\"");
    for (line_num, line) in text.lines().enumerate() {
        if let Some(key_pos) = line.find(&pattern) {
            // key の後ろの value を探す
            let after_key = &line[key_pos + pattern.len()..];
            let quoted_value = format!("\"{value}\"");
            if let Some(val_offset) = after_key.find(&quoted_value) {
                let abs_start = key_pos + pattern.len() + val_offset + 1; // skip opening "
                let abs_end = abs_start + value.len();
                return Some(Range {
                    start: Position {
                        line: line_num as u32,
                        character: abs_start as u32,
                    },
                    end: Position {
                        line: line_num as u32,
                        character: abs_end as u32,
                    },
                });
            }
        }
    }
    None
}

/// テキスト中の `"value"` 文字列リテラルの Range を取得（引用符内の部分のみ）
fn find_string_literal_range(text: &str, value: &str) -> Option<Range> {
    let quoted = format!("\"{value}\"");
    for (line_num, line) in text.lines().enumerate() {
        if let Some(pos) = line.find(&quoted) {
            let start = pos + 1; // skip opening "
            let end = start + value.len();
            return Some(Range {
                start: Position {
                    line: line_num as u32,
                    character: start as u32,
                },
                end: Position {
                    line: line_num as u32,
                    character: end as u32,
                },
            });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_value_range() {
        let text = r#"  "path": "CLI/templates/backend-rust/feature""#;
        let range = find_value_range(text, "path", "CLI/templates/backend-rust/feature");
        assert!(range.is_some());
        let range = range.unwrap();
        assert_eq!(range.start.line, 0);
    }

    #[test]
    fn test_find_string_literal_range() {
        let text = r#"    "src/domain/","#;
        let range = find_string_literal_range(text, "src/domain/");
        assert!(range.is_some());
        let range = range.unwrap();
        assert_eq!(range.start.line, 0);
        assert_eq!(range.start.character, 5);
        assert_eq!(range.end.character, 16);
    }

    #[test]
    fn test_get_document_links_empty_json() {
        let text = "{}";
        let uri = Url::parse("file:///test/manifest.json").unwrap();
        let root = PathBuf::from("/test");
        let links = get_document_links(text, &uri, Some(&root));
        assert!(links.is_empty());
    }

    #[test]
    fn test_get_document_links_with_managed_paths() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();
        let text = r#"{
  "managed_paths": ["deploy/", "buf.yaml"],
  "protected_paths": ["src/domain/"]
}"#;
        let uri = Url::from_file_path(root.join(".k1s0/manifest.json")).unwrap();
        let links = get_document_links(text, &uri, Some(&root));
        assert_eq!(links.len(), 3);
    }

    #[test]
    fn test_get_document_links_no_workspace() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let text = r#"{"managed_paths": ["deploy/"]}"#;
        let uri = Url::from_file_path(temp_dir.path().join("manifest.json")).unwrap();
        let links = get_document_links(text, &uri, None);
        assert!(links.is_empty());
    }

    #[test]
    fn test_get_document_links_invalid_json() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();
        let text = "not json";
        let uri = Url::from_file_path(root.join("manifest.json")).unwrap();
        let links = get_document_links(text, &uri, Some(&root));
        assert!(links.is_empty());
    }
}
