//! 定義ジャンプ機能
//!
//! manifest.json 内のリファレンスから定義箇所へジャンプする機能を提供する。
//! - テンプレート参照 → テンプレートディレクトリ
//! - framework crate 参照 → crate 定義

use std::path::PathBuf;
use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Position, Range, Url};

/// 定義へジャンプの結果
#[derive(Debug)]
pub struct DefinitionResult {
    /// 定義の場所
    pub location: Location,
    /// 定義の種類
    pub kind: DefinitionKind,
}

/// 定義の種類
#[derive(Debug, Clone, PartialEq)]
pub enum DefinitionKind {
    /// テンプレートディレクトリ
    TemplateDirectory,
    /// Framework crate
    FrameworkCrate,
    /// 設定ファイル
    ConfigFile,
    /// manifest.json 内のキー
    ManifestKey,
}

/// 位置から定義を検索
pub fn find_definition(
    content: &str,
    position: Position,
    workspace_root: Option<&PathBuf>,
) -> Option<GotoDefinitionResponse> {
    // カーソル位置の行を取得
    let lines: Vec<&str> = content.lines().collect();
    let line_idx = position.line as usize;

    if line_idx >= lines.len() {
        return None;
    }

    let line = lines[line_idx];
    let char_idx = position.character as usize;

    // カーソル位置の文字列を取得
    let (key, value) = extract_key_value_at_position(line, char_idx)?;

    // コンテキストに応じて定義を検索
    match key.as_str() {
        "path" => {
            // テンプレートパスの場合
            find_template_definition(&value, workspace_root)
        }
        "name" => {
            // framework crate 名の場合
            if is_in_framework_crates_section(content, position) {
                find_framework_crate_definition(&value, workspace_root)
            } else if is_in_template_section(content, position) {
                find_template_by_name(&value, workspace_root)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// 行からキーと値を抽出
fn extract_key_value_at_position(line: &str, char_idx: usize) -> Option<(String, String)> {
    // JSON のキー: 値 パターンを検出
    let trimmed = line.trim();

    // "key": "value" または "key": value のパターンを探す
    if let Some(colon_pos) = trimmed.find(':') {
        let key_part = trimmed[..colon_pos].trim();
        let value_part = trimmed[colon_pos + 1..].trim();

        // キー部分から引用符を除去
        let key = key_part.trim_matches('"').to_string();

        // 値部分から引用符とカンマを除去
        let value = value_part
            .trim_end_matches(',')
            .trim_matches('"')
            .to_string();

        // カーソルが値の範囲内かチェック
        if char_idx > colon_pos {
            return Some((key, value));
        }
    }

    None
}

/// framework_crates セクション内かどうかを判定
fn is_in_framework_crates_section(content: &str, position: Position) -> bool {
    let lines: Vec<&str> = content.lines().collect();
    let line_idx = position.line as usize;

    // 前の行を遡って framework_crates を探す
    for i in (0..=line_idx).rev() {
        if i >= lines.len() {
            continue;
        }
        let line = lines[i];
        if line.contains("\"framework_crates\"") {
            return true;
        }
        // 他のセクションに到達したら終了
        if line.contains("\"dependencies\"") && !line.contains("framework_crates") {
            return false;
        }
    }

    false
}

/// template セクション内かどうかを判定
fn is_in_template_section(content: &str, position: Position) -> bool {
    let lines: Vec<&str> = content.lines().collect();
    let line_idx = position.line as usize;

    for i in (0..=line_idx).rev() {
        if i >= lines.len() {
            continue;
        }
        let line = lines[i];
        if line.contains("\"template\"") && line.contains('{') {
            return true;
        }
        // 他のトップレベルセクションに到達したら終了
        if line.contains("\"service\"") || line.contains("\"dependencies\"") {
            return false;
        }
    }

    false
}

/// テンプレート定義を検索
fn find_template_definition(
    path: &str,
    workspace_root: Option<&PathBuf>,
) -> Option<GotoDefinitionResponse> {
    let root = workspace_root?;
    let template_path = root.join(path);

    if template_path.exists() {
        let uri = Url::from_file_path(&template_path).ok()?;

        // ディレクトリの場合は中の manifest.json を探す
        let target_path = if template_path.is_dir() {
            template_path.join(".k1s0").join("manifest.json")
        } else {
            template_path
        };

        if target_path.exists() {
            let target_uri = Url::from_file_path(&target_path).ok()?;
            return Some(GotoDefinitionResponse::Scalar(Location {
                uri: target_uri,
                range: Range {
                    start: Position { line: 0, character: 0 },
                    end: Position { line: 0, character: 0 },
                },
            }));
        }

        // ディレクトリだけでも返す
        return Some(GotoDefinitionResponse::Scalar(Location {
            uri,
            range: Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 0, character: 0 },
            },
        }));
    }

    None
}

/// テンプレート名から定義を検索
fn find_template_by_name(
    name: &str,
    workspace_root: Option<&PathBuf>,
) -> Option<GotoDefinitionResponse> {
    let root = workspace_root?;

    // CLI/templates/{name} を探す
    let template_path = root.join("CLI").join("templates").join(name);

    if template_path.exists() {
        let uri = Url::from_file_path(&template_path).ok()?;
        return Some(GotoDefinitionResponse::Scalar(Location {
            uri,
            range: Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 0, character: 0 },
            },
        }));
    }

    None
}

/// Framework crate 定義を検索
fn find_framework_crate_definition(
    crate_name: &str,
    workspace_root: Option<&PathBuf>,
) -> Option<GotoDefinitionResponse> {
    let root = workspace_root?;

    // framework/backend/rust/crates/{crate_name} を探す
    let crate_path = root
        .join("framework")
        .join("backend")
        .join("rust")
        .join("crates")
        .join(crate_name);

    if crate_path.exists() {
        // Cargo.toml を優先
        let cargo_toml = crate_path.join("Cargo.toml");
        let target = if cargo_toml.exists() {
            cargo_toml
        } else {
            crate_path
        };

        let uri = Url::from_file_path(&target).ok()?;
        return Some(GotoDefinitionResponse::Scalar(Location {
            uri,
            range: Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 0, character: 0 },
            },
        }));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_key_value_at_position() {
        let line = r#"    "name": "backend-rust","#;

        let result = extract_key_value_at_position(line, 15);
        assert!(result.is_some());

        let (key, value) = result.unwrap();
        assert_eq!(key, "name");
        assert_eq!(value, "backend-rust");
    }

    #[test]
    fn test_extract_key_value_before_colon() {
        let line = r#"    "name": "value""#;

        // カーソルがコロンより前
        let result = extract_key_value_at_position(line, 5);
        assert!(result.is_none());
    }

    #[test]
    fn test_is_in_template_section() {
        let content = r#"{
  "template": {
    "name": "backend-rust",
    "version": "0.1.0"
  }
}"#;

        // "name" の行
        assert!(is_in_template_section(content, Position { line: 2, character: 5 }));

        // "version" の行
        assert!(is_in_template_section(content, Position { line: 3, character: 5 }));
    }

    #[test]
    fn test_is_in_framework_crates_section() {
        let content = r#"{
  "dependencies": {
    "framework_crates": [
      { "name": "k1s0-config" }
    ]
  }
}"#;

        // framework_crates 内
        assert!(is_in_framework_crates_section(
            content,
            Position { line: 3, character: 10 }
        ));
    }
}
