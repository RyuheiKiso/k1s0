//! 参照検索機能
//!
//! manifest.json 内のキーが参照されている箇所を検索する。

use std::collections::HashMap;
use std::path::PathBuf;
use tower_lsp::lsp_types::{Location, Position, Range, Url};

/// 参照検索結果
pub fn find_references(
    uri: &Url,
    content: &str,
    position: Position,
    workspace_root: Option<&PathBuf>,
    include_declaration: bool,
) -> Vec<Location> {
    let mut references = Vec::new();

    // カーソル位置の要素を特定
    let target = extract_target_at_position(content, position);
    if target.is_none() {
        return references;
    }

    let (target_key, target_value) = target.unwrap();

    // 宣言を含める場合、現在位置を追加
    if include_declaration {
        if let Some(range) = find_string_range(content, &target_value, position.line) {
            references.push(Location {
                uri: uri.clone(),
                range,
            });
        }
    }

    // ワークスペース内の他のmanifest.jsonを検索
    if let Some(root) = workspace_root {
        let manifest_files = find_manifest_files(root);

        for manifest_path in manifest_files {
            if let Ok(manifest_content) = std::fs::read_to_string(&manifest_path) {
                if let Ok(manifest_uri) = Url::from_file_path(&manifest_path) {
                    // 同じファイルは宣言として既に追加済み
                    if manifest_uri == *uri {
                        continue;
                    }

                    // 値への参照を検索
                    let refs = find_value_references(&manifest_content, &target_key, &target_value);
                    for r in refs {
                        references.push(Location {
                            uri: manifest_uri.clone(),
                            range: r,
                        });
                    }
                }
            }
        }
    }

    references
}

/// カーソル位置のターゲットを抽出
fn extract_target_at_position(content: &str, position: Position) -> Option<(String, String)> {
    let lines: Vec<&str> = content.lines().collect();
    let line_idx = position.line as usize;

    if line_idx >= lines.len() {
        return None;
    }

    let line = lines[line_idx];
    let trimmed = line.trim();

    // "key": "value" パターン
    if let Some(colon_pos) = trimmed.find(':') {
        let key_part = trimmed[..colon_pos].trim().trim_matches('"');
        let value_part = trimmed[colon_pos + 1..]
            .trim()
            .trim_end_matches(',')
            .trim_matches('"');

        return Some((key_part.to_string(), value_part.to_string()));
    }

    None
}

/// 文字列の範囲を検索
fn find_string_range(content: &str, target: &str, start_line: u32) -> Option<Range> {
    let lines: Vec<&str> = content.lines().collect();
    let line_idx = start_line as usize;

    if line_idx >= lines.len() {
        return None;
    }

    let line = lines[line_idx];
    let quoted_target = format!("\"{}\"", target);

    if let Some(pos) = line.find(&quoted_target) {
        return Some(Range {
            start: Position {
                line: start_line,
                character: pos as u32,
            },
            end: Position {
                line: start_line,
                character: (pos + quoted_target.len()) as u32,
            },
        });
    }

    None
}

/// 値への参照を検索
fn find_value_references(content: &str, key: &str, value: &str) -> Vec<Range> {
    let mut references = Vec::new();
    let quoted_value = format!("\"{}\"", value);

    for (line_idx, line) in content.lines().enumerate() {
        // 同じキーで同じ値を持つ行を検索
        if line.contains(&format!("\"{}\"", key)) && line.contains(&quoted_value) {
            if let Some(pos) = line.find(&quoted_value) {
                references.push(Range {
                    start: Position {
                        line: line_idx as u32,
                        character: pos as u32,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: (pos + quoted_value.len()) as u32,
                    },
                });
            }
        }
    }

    references
}

/// manifest.json ファイルを検索
fn find_manifest_files(root: &PathBuf) -> Vec<PathBuf> {
    let mut files = Vec::new();

    // feature ディレクトリを検索
    let feature_dir = root.join("feature");
    if feature_dir.exists() {
        search_manifest_in_dir(&feature_dir, &mut files);
    }

    // framework ディレクトリを検索
    let framework_dir = root.join("framework");
    if framework_dir.exists() {
        search_manifest_in_dir(&framework_dir, &mut files);
    }

    files
}

/// ディレクトリ内の manifest.json を再帰的に検索
fn search_manifest_in_dir(dir: &PathBuf, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                // .k1s0/manifest.json を確認
                let manifest = path.join(".k1s0").join("manifest.json");
                if manifest.exists() {
                    files.push(manifest);
                }

                // サブディレクトリも検索
                search_manifest_in_dir(&path, files);
            }
        }
    }
}

/// 同じ値を使用しているキーをグループ化
pub fn group_references_by_key(content: &str) -> HashMap<String, Vec<(String, Range)>> {
    let mut groups: HashMap<String, Vec<(String, Range)>> = HashMap::new();

    if let Ok(value) = serde_json::from_str::<serde_json::Value>(content) {
        collect_values(&value, &mut groups, content, "");
    }

    groups
}

/// 値を収集
fn collect_values(
    value: &serde_json::Value,
    groups: &mut HashMap<String, Vec<(String, Range)>>,
    content: &str,
    path: &str,
) {
    match value {
        serde_json::Value::Object(obj) => {
            for (key, val) in obj {
                let new_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", path, key)
                };
                collect_values(val, groups, content, &new_path);
            }
        }
        serde_json::Value::String(s) => {
            // 値をキーとしてグループ化
            let entry = groups.entry(s.clone()).or_default();

            // 位置を検索（簡易実装）
            for (line_idx, line) in content.lines().enumerate() {
                let quoted = format!("\"{}\"", s);
                if line.contains(&quoted) {
                    if let Some(pos) = line.find(&quoted) {
                        entry.push((
                            path.to_string(),
                            Range {
                                start: Position {
                                    line: line_idx as u32,
                                    character: pos as u32,
                                },
                                end: Position {
                                    line: line_idx as u32,
                                    character: (pos + quoted.len()) as u32,
                                },
                            },
                        ));
                        break;
                    }
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_target_at_position() {
        let content = r#"  "name": "test-service""#;

        let result = extract_target_at_position(content, Position { line: 0, character: 15 });
        assert!(result.is_some());

        let (key, value) = result.unwrap();
        assert_eq!(key, "name");
        assert_eq!(value, "test-service");
    }

    #[test]
    fn test_find_string_range() {
        let content = r#"  "template": "backend-rust""#;

        let range = find_string_range(content, "backend-rust", 0);
        assert!(range.is_some());

        let range = range.unwrap();
        assert_eq!(range.start.character, 14);
    }

    #[test]
    fn test_find_value_references() {
        let content = r#"{
  "template": { "name": "backend-rust" },
  "other": { "name": "backend-rust" }
}"#;

        let refs = find_value_references(content, "name", "backend-rust");
        assert_eq!(refs.len(), 2);
    }

    #[test]
    fn test_group_references_by_key() {
        let content = r#"{"a": "shared", "b": "shared", "c": "unique"}"#;

        let groups = group_references_by_key(content);

        assert!(groups.contains_key("shared"));
        // "shared" は2回出現
        assert!(groups.get("shared").unwrap().len() >= 1);
    }
}
