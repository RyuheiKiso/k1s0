use std::path::PathBuf;

use super::{FixResult, RuleId, Violation};

/// 自動修正機能
pub struct Fixer {
    /// ベースパス
    base_path: PathBuf,
}

impl Fixer {
    /// 新しい Fixer を作成
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    /// 違反を修正する
    pub fn fix(&self, violation: &Violation) -> Option<FixResult> {
        match violation.rule {
            RuleId::ManifestNotFound => self.fix_manifest_not_found(),
            RuleId::ManifestMissingKey => self.fix_manifest_missing_key(violation),
            RuleId::RequiredDirMissing => self.fix_required_dir_missing(violation),
            RuleId::RequiredFileMissing => self.fix_required_file_missing(violation),
            // その他のルールは自動修正不可
            _ => None,
        }
    }

    /// K001: manifest.json が存在しない場合、作成する
    fn fix_manifest_not_found(&self) -> Option<FixResult> {
        let manifest_path = self.base_path.join(".k1s0/manifest.json");

        // ディレクトリを作成
        if let Some(parent) = manifest_path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return Some(FixResult::failure(
                    &manifest_path,
                    "manifest.json の作成",
                    format!("ディレクトリの作成に失敗: {}", e),
                ));
            }
        }

        // manifest.json のテンプレートを作成
                let manifest_content = r#"{
    "name": "unnamed-feature",
    "version": "0.1.0",
    "description": "Feature description",
    "template": "backend-rust",
    "created_at": "",
    "dependencies": {},
    "settings": {}
}
"#;

        match std::fs::write(&manifest_path, manifest_content) {
            Ok(()) => Some(FixResult::success(
                manifest_path,
                "manifest.json を作成しました",
            )),
            Err(e) => Some(FixResult::failure(
                manifest_path,
                "manifest.json の作成",
                format!("ファイルの書き込みに失敗: {}", e),
            )),
        }
    }

    /// K002: manifest.json の必須キーが不足している場合、追加する
    fn fix_manifest_missing_key(&self, violation: &Violation) -> Option<FixResult> {
        let manifest_path = self.base_path.join(".k1s0/manifest.json");

        // 既存の manifest を読み込む
        let content = match std::fs::read_to_string(&manifest_path) {
            Ok(c) => c,
            Err(e) => {
                return Some(FixResult::failure(
                    &manifest_path,
                    "manifest.json の更新",
                    format!("ファイルの読み込みに失敗: {}", e),
                ));
            }
        };

        // JSON をパース
        let mut json: serde_json::Value = match serde_json::from_str(&content) {
            Ok(j) => j,
            Err(e) => {
                return Some(FixResult::failure(
                    &manifest_path,
                    "manifest.json の更新",
                    format!("JSON のパースに失敗: {}", e),
                ));
            }
        };

        // 不足しているキーを抽出
        let obj = match json.as_object_mut() {
            Some(o) => o,
            None => {
                return Some(FixResult::failure(
                    &manifest_path,
                    "manifest.json の更新",
                    "JSON がオブジェクトではありません",
                ));
            }
        };

        // メッセージからキーを抽出して追加
        let key = violation.message
            .split('\'')
            .nth(1)
            .unwrap_or("");

        let default_value = match key {
            "name" => serde_json::Value::String("unnamed-feature".to_string()),
            "version" => serde_json::Value::String("0.1.0".to_string()),
            "description" => serde_json::Value::String("Feature description".to_string()),
            "template" => serde_json::Value::String("backend-rust".to_string()),
            "dependencies" => serde_json::Value::Object(serde_json::Map::new()),
            "settings" => serde_json::Value::Object(serde_json::Map::new()),
            _ => return None,
        };

        if !key.is_empty() && !obj.contains_key(key) {
            obj.insert(key.to_string(), default_value);
        }

        // 保存
        match std::fs::write(&manifest_path, serde_json::to_string_pretty(&json).unwrap()) {
            Ok(()) => Some(FixResult::success(
                manifest_path,
                format!("manifest.json に '{}' キーを追加しました", key),
            )),
            Err(e) => Some(FixResult::failure(
                manifest_path,
                "manifest.json の更新",
                format!("ファイルの書き込みに失敗: {}", e),
            )),
        }
    }

    /// K010: 必須ディレクトリが存在しない場合、作成する
    fn fix_required_dir_missing(&self, violation: &Violation) -> Option<FixResult> {
        let dir_path = violation.path.as_ref()?;
        let full_path = self.base_path.join(dir_path);

        match std::fs::create_dir_all(&full_path) {
            Ok(()) => Some(FixResult::success(
                full_path,
                format!("ディレクトリ '{}' を作成しました", dir_path),
            )),
            Err(e) => Some(FixResult::failure(
                full_path,
                format!("ディレクトリ '{}' の作成", dir_path),
                format!("作成に失敗: {}", e),
            )),
        }
    }

    /// K011: 必須ファイルが存在しない場合、作成する
    fn fix_required_file_missing(&self, violation: &Violation) -> Option<FixResult> {
        let file_path = violation.path.as_ref()?;
        let full_path = self.base_path.join(file_path);

        // 親ディレクトリを作成
        if let Some(parent) = full_path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return Some(FixResult::failure(
                    &full_path,
                    format!("ファイル '{}' の作成", file_path),
                    format!("親ディレクトリの作成に失敗: {}", e),
                ));
            }
        }

        // ファイルの種類に応じてテンプレートを選択
        let content = self.get_file_template(file_path);

        match std::fs::write(&full_path, content) {
            Ok(()) => Some(FixResult::success(
                full_path,
                format!("ファイル '{}' を作成しました", file_path),
            )),
            Err(e) => Some(FixResult::failure(
                full_path,
                format!("ファイル '{}' の作成", file_path),
                format!("書き込みに失敗: {}", e),
            )),
        }
    }

    /// ファイルパスに応じたテンプレートを取得
    fn get_file_template(&self, file_path: &str) -> &'static str {
        if file_path.ends_with("mod.rs") {
            "//! Module\n"
        } else if file_path.ends_with(".rs") {
            "// TODO: Implement\n"
        } else if file_path.ends_with("main.rs") {
            "fn main() {\n    println!(\"Hello, world!\");\n}\n"
        } else if file_path.ends_with("README.md") {
            "# Feature\n\nDescription here.\n"
        } else if file_path.ends_with(".yaml") || file_path.ends_with(".yml") {
            "# Configuration\n"
        } else if file_path.ends_with("Cargo.toml") {
            "[package]\nname = \"unnamed\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"
        } else if file_path.ends_with("go.mod") {
            "module example.com/unnamed\n\ngo 1.21\n"
        } else if file_path.ends_with("package.json") {
            "{\n  \"name\": \"unnamed\",\n  \"version\": \"0.1.0\"\n}\n"
        } else if file_path.ends_with("pubspec.yaml") {
            "name: unnamed\nversion: 0.1.0\n"
        } else if file_path.ends_with("buf.yaml") {
            "version: v1\nname: unnamed\n"
        } else {
            ""
        }
    }

    /// ルールが自動修正可能かどうか
    pub fn is_fixable(rule: RuleId) -> bool {
        matches!(
            rule,
            RuleId::ManifestNotFound
                | RuleId::ManifestMissingKey
                | RuleId::RequiredDirMissing
                | RuleId::RequiredFileMissing
        )
    }
}
