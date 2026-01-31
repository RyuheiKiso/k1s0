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
        "domain" => {
            // ドメイン名からドメインディレクトリへジャンプ
            find_domain_definition(&value, workspace_root)
        }
        _ => {
            // dependencies.domain セクション内のキー → ドメインへジャンプ
            if is_in_domain_dependency_section(content, position) {
                find_domain_definition(&key, workspace_root)
            } else {
                None
            }
        }
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

/// Framework crate/package 定義を検索（多言語対応）
fn find_framework_crate_definition(
    crate_name: &str,
    workspace_root: Option<&PathBuf>,
) -> Option<GotoDefinitionResponse> {
    let root = workspace_root?;

    let csproj_name = format!("{}.csproj", crate_name);
    let search_paths: Vec<(PathBuf, &str)> = vec![
        (root.join("framework/backend/rust/crates").join(crate_name), "Cargo.toml"),
        (root.join("framework/backend/go").join(crate_name), "go.mod"),
        (root.join("framework/backend/csharp").join(crate_name), &csproj_name),
        (root.join("framework/backend/python").join(crate_name), "pyproject.toml"),
        (root.join("framework/frontend/react/packages").join(crate_name), "package.json"),
        (root.join("framework/frontend/flutter/packages").join(crate_name), "pubspec.yaml"),
    ];

    for (pkg_dir, entry_file) in &search_paths {
        if pkg_dir.exists() {
            let entry = pkg_dir.join(entry_file);
            let target = if entry.exists() { entry } else { pkg_dir.clone() };
            let uri = Url::from_file_path(&target).ok()?;
            return Some(GotoDefinitionResponse::Scalar(Location {
                uri,
                range: Range {
                    start: Position { line: 0, character: 0 },
                    end: Position { line: 0, character: 0 },
                },
            }));
        }
    }

    None
}

/// dependencies.domain セクション内かどうかを判定
fn is_in_domain_dependency_section(content: &str, position: Position) -> bool {
    let lines: Vec<&str> = content.lines().collect();
    let line_idx = position.line as usize;
    let mut found_domain = false;

    for i in (0..=line_idx).rev() {
        if i >= lines.len() {
            continue;
        }
        let line = lines[i];
        if found_domain {
            if line.contains("\"dependencies\"") {
                return true;
            }
            // 他のキーに到達
            if line.contains('}') && !line.contains('{') {
                return false;
            }
        }
        if line.contains("\"domain\"") && line.contains('{') {
            found_domain = true;
        }
        // トップレベルセクションに到達
        if line.contains("\"template\"") || line.contains("\"service\"") {
            return false;
        }
    }

    false
}

/// ドメイン定義を検索
fn find_domain_definition(
    domain_name: &str,
    workspace_root: Option<&PathBuf>,
) -> Option<GotoDefinitionResponse> {
    let root = workspace_root?;
    let search_dirs: &[(&str, &[&str])] = &[
        ("backend", &["rust", "go", "csharp", "python"]),
        ("frontend", &["react", "flutter"]),
    ];

    for (layer, langs) in search_dirs {
        for lang in *langs {
            let path = root.join("domain").join(layer).join(lang).join(domain_name);
            if path.exists() {
                let manifest = path.join(".k1s0").join("manifest.json");
                let target = if manifest.exists() { manifest } else { path };
                let uri = Url::from_file_path(&target).ok()?;
                return Some(GotoDefinitionResponse::Scalar(Location {
                    uri,
                    range: Range::default(),
                }));
            }
        }
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

    #[test]
    fn test_extract_key_value_no_colon() {
        let line = r#"    "name""#;
        let result = extract_key_value_at_position(line, 8);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_key_value_cursor_before_colon() {
        let line = r#"    "name": "value""#;
        // カーソルがコロンより前
        let result = extract_key_value_at_position(line, 5);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_key_value_with_trailing_comma() {
        let line = r#"    "path": "CLI/templates/backend-rust","#;
        let result = extract_key_value_at_position(line, 15);
        assert!(result.is_some());

        let (key, value) = result.unwrap();
        assert_eq!(key, "path");
        assert_eq!(value, "CLI/templates/backend-rust");
    }

    #[test]
    fn test_is_in_template_section_true() {
        let content = r#"{
  "template": {
    "name": "backend-rust"
  }
}"#;

        assert!(is_in_template_section(
            content,
            Position { line: 2, character: 5 }
        ));
    }

    #[test]
    fn test_is_in_template_section_false_in_service() {
        let content = r#"{
  "service": {
    "name": "test"
  }
}"#;

        assert!(!is_in_template_section(
            content,
            Position { line: 2, character: 5 }
        ));
    }

    #[test]
    fn test_is_in_framework_crates_section_false_in_service() {
        let content = r#"{
  "service": {
    "name": "k1s0-config"
  }
}"#;

        assert!(!is_in_framework_crates_section(
            content,
            Position { line: 2, character: 10 }
        ));
    }

    #[test]
    fn test_find_definition_no_workspace() {
        let content = r#"{
  "template": {
    "path": "some/path"
  }
}"#;

        let result = find_definition(content, Position { line: 2, character: 15 }, None);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_definition_unknown_key() {
        let content = r#"{
  "unknown": "value"
}"#;

        let result = find_definition(
            content,
            Position { line: 1, character: 15 },
            Some(&PathBuf::from("C:\\work")),
        );
        assert!(result.is_none());
    }

    #[test]
    fn test_find_definition_beyond_document() {
        let content = "{}";

        let result = find_definition(
            content,
            Position { line: 100, character: 0 },
            Some(&PathBuf::from("C:\\work")),
        );
        assert!(result.is_none());
    }

    #[test]
    fn test_definition_kind_equality() {
        assert_eq!(DefinitionKind::TemplateDirectory, DefinitionKind::TemplateDirectory);
        assert_eq!(DefinitionKind::FrameworkCrate, DefinitionKind::FrameworkCrate);
        assert_ne!(DefinitionKind::TemplateDirectory, DefinitionKind::FrameworkCrate);
    }

    #[test]
    fn test_definition_kind_debug() {
        let kind = DefinitionKind::ConfigFile;
        let debug_str = format!("{:?}", kind);
        assert!(debug_str.contains("ConfigFile"));
    }

    #[test]
    fn test_find_template_definition_with_tempdir() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let template_path = temp_dir.path().join("templates").join("backend-rust");
        std::fs::create_dir_all(&template_path).unwrap();

        let result = find_template_definition(
            "templates/backend-rust",
            Some(&temp_dir.path().to_path_buf()),
        );

        assert!(result.is_some());
    }

    #[test]
    fn test_find_template_definition_nonexistent() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        let result = find_template_definition(
            "nonexistent/path",
            Some(&temp_dir.path().to_path_buf()),
        );

        assert!(result.is_none());
    }

    #[test]
    fn test_find_template_definition_no_workspace() {
        let result = find_template_definition("some/path", None);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_template_by_name_with_tempdir() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let template_path = temp_dir.path().join("CLI").join("templates").join("backend-rust");
        std::fs::create_dir_all(&template_path).unwrap();

        let result = find_template_by_name(
            "backend-rust",
            Some(&temp_dir.path().to_path_buf()),
        );

        assert!(result.is_some());
    }

    #[test]
    fn test_find_template_by_name_nonexistent() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        let result = find_template_by_name(
            "nonexistent-template",
            Some(&temp_dir.path().to_path_buf()),
        );

        assert!(result.is_none());
    }

    #[test]
    fn test_find_framework_crate_definition_with_tempdir() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let crate_path = temp_dir.path()
            .join("framework")
            .join("backend")
            .join("rust")
            .join("crates")
            .join("k1s0-config");
        std::fs::create_dir_all(&crate_path).unwrap();

        // Cargo.toml を作成
        std::fs::write(crate_path.join("Cargo.toml"), "[package]\nname = \"k1s0-config\"").unwrap();

        let result = find_framework_crate_definition(
            "k1s0-config",
            Some(&temp_dir.path().to_path_buf()),
        );

        assert!(result.is_some());
    }

    #[test]
    fn test_find_framework_crate_definition_without_cargo_toml() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let crate_path = temp_dir.path()
            .join("framework")
            .join("backend")
            .join("rust")
            .join("crates")
            .join("k1s0-test");
        std::fs::create_dir_all(&crate_path).unwrap();

        let result = find_framework_crate_definition(
            "k1s0-test",
            Some(&temp_dir.path().to_path_buf()),
        );

        // Cargo.toml がなくてもディレクトリがあれば成功
        assert!(result.is_some());
    }

    #[test]
    fn test_find_framework_crate_definition_nonexistent() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        let result = find_framework_crate_definition(
            "nonexistent-crate",
            Some(&temp_dir.path().to_path_buf()),
        );

        assert!(result.is_none());
    }

    #[test]
    fn test_find_framework_crate_definition_go() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let pkg_path = temp_dir.path()
            .join("framework")
            .join("backend")
            .join("go")
            .join("k1s0-config");
        std::fs::create_dir_all(&pkg_path).unwrap();
        std::fs::write(pkg_path.join("go.mod"), "module k1s0-config").unwrap();

        let result = find_framework_crate_definition(
            "k1s0-config",
            Some(&temp_dir.path().to_path_buf()),
        );

        assert!(result.is_some());
    }

    #[test]
    fn test_find_framework_crate_definition_python() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let pkg_path = temp_dir.path()
            .join("framework")
            .join("backend")
            .join("python")
            .join("k1s0-config");
        std::fs::create_dir_all(&pkg_path).unwrap();
        std::fs::write(pkg_path.join("pyproject.toml"), "[project]").unwrap();

        let result = find_framework_crate_definition(
            "k1s0-config",
            Some(&temp_dir.path().to_path_buf()),
        );

        assert!(result.is_some());
    }

    #[test]
    fn test_find_domain_definition_with_tempdir() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let domain_path = temp_dir.path()
            .join("domain")
            .join("backend")
            .join("rust")
            .join("user-management");
        std::fs::create_dir_all(domain_path.join(".k1s0")).unwrap();
        std::fs::write(domain_path.join(".k1s0/manifest.json"), "{}").unwrap();

        let result = find_domain_definition(
            "user-management",
            Some(&temp_dir.path().to_path_buf()),
        );

        assert!(result.is_some());
    }

    #[test]
    fn test_find_domain_definition_nonexistent() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        let result = find_domain_definition(
            "nonexistent-domain",
            Some(&temp_dir.path().to_path_buf()),
        );

        assert!(result.is_none());
    }

    #[test]
    fn test_find_domain_definition_no_workspace() {
        let result = find_domain_definition("test", None);
        assert!(result.is_none());
    }

    #[test]
    fn test_is_in_domain_dependency_section_true() {
        let content = r#"{
  "dependencies": {
    "domain": {
      "user-management": "^0.1.0"
    }
  }
}"#;

        assert!(is_in_domain_dependency_section(
            content,
            Position { line: 3, character: 10 }
        ));
    }

    #[test]
    fn test_is_in_domain_dependency_section_false() {
        let content = r#"{
  "service": {
    "name": "test"
  }
}"#;

        assert!(!is_in_domain_dependency_section(
            content,
            Position { line: 2, character: 5 }
        ));
    }
}
