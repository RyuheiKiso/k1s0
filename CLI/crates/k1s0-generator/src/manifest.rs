//! manifest.json の読み書き
//!
//! `.k1s0/manifest.json` の読み込み・書き込み・バリデーションを提供する。

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::{Error, Result};

/// manifest.json の JSON Schema 定義
const MANIFEST_SCHEMA: &str = r#"{
    "$schema": "http://json-schema.org/draft-07/schema#",
    "title": "k1s0 Manifest",
    "description": "k1s0 manifest.json schema",
    "type": "object",
    "required": [
        "schema_version",
        "k1s0_version",
        "template",
        "service",
        "generated_at",
        "managed_paths",
        "protected_paths"
    ],
    "properties": {
        "schema_version": {
            "type": "string",
            "pattern": "^[0-9]+\\.[0-9]+\\.[0-9]+$",
            "description": "スキーマバージョン（セマンティックバージョニング）"
        },
        "k1s0_version": {
            "type": "string",
            "pattern": "^[0-9]+\\.[0-9]+\\.[0-9]+.*$",
            "description": "k1s0 CLI のバージョン"
        },
        "template": {
            "type": "object",
            "required": ["name", "version", "path", "fingerprint"],
            "properties": {
                "name": {
                    "type": "string",
                    "minLength": 1,
                    "description": "テンプレート名"
                },
                "version": {
                    "type": "string",
                    "pattern": "^[0-9]+\\.[0-9]+\\.[0-9]+.*$",
                    "description": "テンプレートバージョン"
                },
                "source": {
                    "type": "string",
                    "enum": ["local", "registry"],
                    "description": "テンプレートソース"
                },
                "path": {
                    "type": "string",
                    "minLength": 1,
                    "description": "テンプレートパス"
                },
                "revision": {
                    "type": ["string", "null"],
                    "description": "Git リビジョン"
                },
                "fingerprint": {
                    "type": "string",
                    "pattern": "^[a-fA-F0-9]+$",
                    "minLength": 8,
                    "description": "テンプレートのフィンガープリント（16進数）"
                }
            }
        },
        "service": {
            "type": "object",
            "required": ["service_name", "language", "type"],
            "properties": {
                "service_name": {
                    "type": "string",
                    "pattern": "^[a-z][a-z0-9-]*[a-z0-9]$|^[a-z]$",
                    "minLength": 1,
                    "maxLength": 63,
                    "description": "サービス名（小文字英数字とハイフン）"
                },
                "language": {
                    "type": "string",
                    "enum": ["rust", "go", "typescript", "python"],
                    "description": "プログラミング言語"
                },
                "type": {
                    "type": "string",
                    "enum": ["backend", "frontend", "bff"],
                    "description": "サービスタイプ"
                },
                "framework": {
                    "type": ["string", "null"],
                    "description": "フレームワーク名"
                }
            }
        },
        "generated_at": {
            "type": "string",
            "format": "date-time",
            "description": "生成日時（ISO 8601形式）"
        },
        "managed_paths": {
            "type": "array",
            "items": {
                "type": "string",
                "minLength": 1
            },
            "description": "CLI が管理するパス"
        },
        "protected_paths": {
            "type": "array",
            "items": {
                "type": "string",
                "minLength": 1
            },
            "description": "CLI が変更しないパス"
        },
        "update_policy": {
            "type": "object",
            "additionalProperties": {
                "type": "string",
                "enum": ["auto", "suggest_only", "protected"]
            },
            "description": "パス別の更新ポリシー"
        },
        "checksums": {
            "type": "object",
            "additionalProperties": {
                "type": "string"
            },
            "description": "ファイルのチェックサム"
        },
        "dependencies": {
            "type": ["object", "null"],
            "properties": {
                "framework_crates": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["name", "version"],
                        "properties": {
                            "name": {"type": "string"},
                            "version": {"type": "string"}
                        }
                    }
                }
            },
            "description": "依存情報"
        }
    }
}"#;

/// manifest.json のスキーマバージョン
pub const SCHEMA_VERSION: &str = "1.0.0";

/// レイヤータイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LayerType {
    /// システム共通基盤層
    Framework,
    /// 業務領域共通層
    Domain,
    /// 個別機能層（デフォルト）
    #[default]
    Feature,
}

impl LayerType {
    /// 文字列表現を取得
    pub fn as_str(&self) -> &'static str {
        match self {
            LayerType::Framework => "framework",
            LayerType::Domain => "domain",
            LayerType::Feature => "feature",
        }
    }

    /// 文字列からパース
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "framework" => Some(LayerType::Framework),
            "domain" => Some(LayerType::Domain),
            "feature" => Some(LayerType::Feature),
            _ => None,
        }
    }
}

/// 非推奨情報
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Deprecated {
    /// 単純なフラグ
    Flag(bool),
    /// 詳細な非推奨情報
    Info(DeprecatedInfo),
}

impl Deprecated {
    /// 非推奨かどうかを判定
    pub fn is_deprecated(&self) -> bool {
        match self {
            Deprecated::Flag(flag) => *flag,
            Deprecated::Info(_) => true,
        }
    }
}

/// 詳細な非推奨情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeprecatedInfo {
    /// 非推奨化されたバージョン
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,

    /// 移行先の domain 名
    #[serde(skip_serializing_if = "Option::is_none")]
    pub migrate_to: Option<String>,

    /// 削除予定日
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<String>,

    /// 非推奨化の理由
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// manifest.json のルート構造
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// スキーマバージョン
    pub schema_version: String,

    /// k1s0 バージョン
    pub k1s0_version: String,

    /// テンプレート情報
    pub template: TemplateInfo,

    /// サービス情報
    pub service: ServiceInfo,

    /// レイヤータイプ（framework / domain / feature）
    #[serde(default)]
    pub layer: LayerType,

    /// 所属する domain 名（feature 層のみ）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,

    /// domain のバージョン（domain 層のみ、SemVer 形式）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// 依存する domain のバージョン制約（feature 層で domain を指定している場合）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain_version: Option<String>,

    /// 必要な最小 framework バージョン（domain 層推奨）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_framework_version: Option<String>,

    /// 破壊的変更の記録（バージョン -> 説明）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub breaking_changes: Option<HashMap<String, String>>,

    /// 非推奨フラグまたは情報
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<Deprecated>,

    /// 生成日時
    pub generated_at: String,

    /// CLI が管理するパス
    pub managed_paths: Vec<String>,

    /// CLI が変更しないパス
    pub protected_paths: Vec<String>,

    /// パス別の更新ポリシー
    #[serde(default)]
    pub update_policy: HashMap<String, UpdatePolicy>,

    /// ファイルのチェックサム
    #[serde(default)]
    pub checksums: HashMap<String, String>,

    /// framework crate への依存情報
    #[serde(default)]
    pub dependencies: Option<Dependencies>,
}

/// テンプレート情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateInfo {
    /// テンプレート名
    pub name: String,

    /// テンプレートバージョン
    pub version: String,

    /// ソース（local / registry）
    #[serde(default = "default_source")]
    pub source: String,

    /// テンプレートのパス
    pub path: String,

    /// Git リビジョン
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,

    /// fingerprint
    pub fingerprint: String,
}

fn default_source() -> String {
    "local".to_string()
}

/// サービス情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// サービス名
    pub service_name: String,

    /// 言語
    pub language: String,

    /// タイプ（backend / frontend / bff）
    #[serde(rename = "type")]
    pub service_type: String,

    /// フレームワーク
    #[serde(skip_serializing_if = "Option::is_none")]
    pub framework: Option<String>,
}

/// 更新ポリシー
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UpdatePolicy {
    /// 自動更新
    Auto,
    /// 差分提示のみ
    SuggestOnly,
    /// 変更しない
    Protected,
}

/// 依存情報
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Dependencies {
    /// framework crates（詳細形式）
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub framework_crates: Vec<CrateDependency>,

    /// framework crates（簡易形式：名前のみの配列）
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub framework: Vec<String>,

    /// domain 依存（名前 -> バージョン制約のマップ）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<HashMap<String, String>>,
}

impl Dependencies {
    /// 空かどうかを判定
    pub fn is_empty(&self) -> bool {
        self.framework_crates.is_empty() && self.framework.is_empty() && self.domain.is_none()
    }

    /// 全ての framework crate 名を取得
    pub fn all_framework_crates(&self) -> Vec<String> {
        let mut result: Vec<String> = self.framework.clone();
        for crate_dep in &self.framework_crates {
            if !result.contains(&crate_dep.name) {
                result.push(crate_dep.name.clone());
            }
        }
        result
    }
}

/// crate 依存
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateDependency {
    /// crate 名
    pub name: String,
    /// バージョン
    pub version: String,
}

impl Manifest {
    /// manifest.json を読み込む
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(Error::ManifestNotFound(path.display().to_string()));
        }

        let content = std::fs::read_to_string(path)?;
        let manifest: Manifest = serde_json::from_str(&content)?;

        Ok(manifest)
    }

    /// manifest.json を書き込む
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// バリデーションを実行する
    pub fn validate(&self) -> Result<()> {
        // JSON Schema によるバリデーション
        self.validate_schema()?;

        // ビジネスルールのバリデーション
        self.validate_business_rules()?;

        Ok(())
    }

    /// framework 層かどうか
    pub fn is_framework(&self) -> bool {
        self.layer == LayerType::Framework
    }

    /// domain 層かどうか
    pub fn is_domain(&self) -> bool {
        self.layer == LayerType::Domain
    }

    /// feature 層かどうか
    pub fn is_feature(&self) -> bool {
        self.layer == LayerType::Feature
    }

    /// 非推奨かどうか
    pub fn is_deprecated(&self) -> bool {
        self.deprecated.as_ref().is_some_and(|d| d.is_deprecated())
    }

    /// バージョンを SemVer としてパース
    pub fn parsed_version(&self) -> Option<semver::Version> {
        self.version
            .as_ref()
            .and_then(|v| semver::Version::parse(v).ok())
    }

    /// domain_version を SemVer 制約としてパース
    pub fn parsed_domain_version(&self) -> Option<semver::VersionReq> {
        self.domain_version
            .as_ref()
            .and_then(|v| semver::VersionReq::parse(v).ok())
    }

    /// min_framework_version を SemVer としてパース
    pub fn parsed_min_framework_version(&self) -> Option<semver::Version> {
        self.min_framework_version
            .as_ref()
            .and_then(|v| semver::Version::parse(v).ok())
    }

    /// JSON Schema によるバリデーション
    fn validate_schema(&self) -> Result<()> {
        let schema: serde_json::Value = serde_json::from_str(MANIFEST_SCHEMA)
            .map_err(|e| Error::ManifestValidation(format!("Invalid schema: {}", e)))?;

        let validator = jsonschema::validator_for(&schema)
            .map_err(|e| Error::ManifestValidation(format!("Failed to compile schema: {}", e)))?;

        let manifest_value = serde_json::to_value(self)
            .map_err(|e| Error::ManifestValidation(format!("Failed to serialize manifest: {}", e)))?;

        let errors: Vec<String> = validator
            .iter_errors(&manifest_value)
            .map(|e| format!("{} at {}", e, e.instance_path))
            .collect();

        if !errors.is_empty() {
            return Err(Error::ManifestValidation(format!(
                "Schema validation failed:\n  - {}",
                errors.join("\n  - ")
            )));
        }

        Ok(())
    }

    /// ビジネスルールのバリデーション
    fn validate_business_rules(&self) -> Result<()> {
        // managed_paths と protected_paths の重複チェック
        let managed_set: HashSet<&str> = self.managed_paths.iter().map(|s| s.as_str()).collect();
        let protected_set: HashSet<&str> = self.protected_paths.iter().map(|s| s.as_str()).collect();

        let overlap: Vec<&str> = managed_set.intersection(&protected_set).copied().collect();
        if !overlap.is_empty() {
            return Err(Error::ManifestValidation(format!(
                "managed_paths and protected_paths must not overlap: {:?}",
                overlap
            )));
        }

        // fingerprint の形式チェック（16進数であること）
        if !self.template.fingerprint.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(Error::ManifestValidation(format!(
                "template.fingerprint must be hexadecimal: {}",
                self.template.fingerprint
            )));
        }

        // fingerprint の長さチェック（最低8文字）
        if self.template.fingerprint.len() < 8 {
            return Err(Error::ManifestValidation(format!(
                "template.fingerprint must be at least 8 characters: {}",
                self.template.fingerprint
            )));
        }

        // domain 層の場合は version が必須
        if self.layer == LayerType::Domain && self.version.is_none() {
            return Err(Error::ManifestValidation(
                "domain layer requires 'version' field".to_string(),
            ));
        }

        // feature 層で domain が指定されている場合は domain_version が推奨
        if self.layer == LayerType::Feature && self.domain.is_some() && self.domain_version.is_none() {
            // 警告レベル（エラーにはしない）
            // TODO: ログに警告を出力
        }

        // version フィールドが指定されている場合は SemVer 形式かチェック
        if let Some(ref version) = self.version {
            if semver::Version::parse(version).is_err() {
                return Err(Error::ManifestValidation(format!(
                    "version must be valid SemVer: {}",
                    version
                )));
            }
        }

        // domain_version フィールドが指定されている場合は SemVer 制約形式かチェック
        if let Some(ref domain_version) = self.domain_version {
            if semver::VersionReq::parse(domain_version).is_err() {
                return Err(Error::ManifestValidation(format!(
                    "domain_version must be valid SemVer constraint: {}",
                    domain_version
                )));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn create_test_manifest() -> Manifest {
        Manifest {
            schema_version: SCHEMA_VERSION.to_string(),
            k1s0_version: "0.1.0".to_string(),
            template: TemplateInfo {
                name: "backend-rust".to_string(),
                version: "0.1.0".to_string(),
                source: "local".to_string(),
                path: "CLI/templates/backend-rust/feature".to_string(),
                revision: None,
                fingerprint: "abcd1234".to_string(),
            },
            service: ServiceInfo {
                service_name: "test-service".to_string(),
                language: "rust".to_string(),
                service_type: "backend".to_string(),
                framework: None,
            },
            layer: LayerType::Feature,
            domain: None,
            version: None,
            domain_version: None,
            min_framework_version: None,
            breaking_changes: None,
            deprecated: None,
            generated_at: "2026-01-25T10:00:00Z".to_string(),
            managed_paths: vec!["deploy/".to_string(), "buf.yaml".to_string()],
            protected_paths: vec!["src/domain/".to_string(), "src/application/".to_string()],
            update_policy: HashMap::from([
                ("deploy/".to_string(), UpdatePolicy::Auto),
                ("src/domain/".to_string(), UpdatePolicy::Protected),
            ]),
            checksums: HashMap::new(),
            dependencies: None,
        }
    }

    fn create_test_domain_manifest() -> Manifest {
        Manifest {
            schema_version: SCHEMA_VERSION.to_string(),
            k1s0_version: "0.1.0".to_string(),
            template: TemplateInfo {
                name: "backend-rust".to_string(),
                version: "0.1.0".to_string(),
                source: "local".to_string(),
                path: "CLI/templates/backend-rust/domain".to_string(),
                revision: None,
                fingerprint: "abcd1234".to_string(),
            },
            service: ServiceInfo {
                service_name: "production".to_string(),
                language: "rust".to_string(),
                service_type: "backend".to_string(),
                framework: None,
            },
            layer: LayerType::Domain,
            domain: None,
            version: Some("1.2.0".to_string()),
            domain_version: None,
            min_framework_version: Some("0.1.0".to_string()),
            breaking_changes: Some(HashMap::from([
                ("1.0.0".to_string(), "Initial release".to_string()),
            ])),
            deprecated: None,
            generated_at: "2026-01-25T10:00:00Z".to_string(),
            managed_paths: vec!["deploy/".to_string()],
            protected_paths: vec!["src/".to_string()],
            update_policy: HashMap::new(),
            checksums: HashMap::new(),
            dependencies: Some(Dependencies {
                framework_crates: vec![],
                framework: vec!["k1s0-error".to_string(), "k1s0-config".to_string()],
                domain: None,
            }),
        }
    }

    fn create_test_feature_with_domain_manifest() -> Manifest {
        Manifest {
            schema_version: SCHEMA_VERSION.to_string(),
            k1s0_version: "0.1.0".to_string(),
            template: TemplateInfo {
                name: "backend-rust".to_string(),
                version: "0.1.0".to_string(),
                source: "local".to_string(),
                path: "CLI/templates/backend-rust/feature".to_string(),
                revision: None,
                fingerprint: "abcd1234".to_string(),
            },
            service: ServiceInfo {
                service_name: "work-order".to_string(),
                language: "rust".to_string(),
                service_type: "backend".to_string(),
                framework: None,
            },
            layer: LayerType::Feature,
            domain: Some("production".to_string()),
            version: None,
            domain_version: Some("^1.2.0".to_string()),
            min_framework_version: None,
            breaking_changes: None,
            deprecated: None,
            generated_at: "2026-01-25T10:00:00Z".to_string(),
            managed_paths: vec!["deploy/".to_string()],
            protected_paths: vec!["src/domain/".to_string()],
            update_policy: HashMap::new(),
            checksums: HashMap::new(),
            dependencies: Some(Dependencies {
                framework_crates: vec![],
                framework: vec!["k1s0-error".to_string()],
                domain: Some(HashMap::from([
                    ("production".to_string(), "^1.2.0".to_string()),
                ])),
            }),
        }
    }

    #[test]
    fn test_manifest_save_and_load() {
        let manifest = create_test_manifest();
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        // 保存
        manifest.save(path).unwrap();

        // 読み込み
        let loaded = Manifest::load(path).unwrap();

        // 検証
        assert_eq!(loaded.schema_version, manifest.schema_version);
        assert_eq!(loaded.k1s0_version, manifest.k1s0_version);
        assert_eq!(loaded.template.name, manifest.template.name);
        assert_eq!(loaded.template.fingerprint, manifest.template.fingerprint);
        assert_eq!(loaded.service.service_name, manifest.service.service_name);
        assert_eq!(loaded.managed_paths, manifest.managed_paths);
        assert_eq!(loaded.protected_paths, manifest.protected_paths);
    }

    #[test]
    fn test_manifest_load_not_found() {
        let result = Manifest::load("/nonexistent/path/manifest.json");
        assert!(result.is_err());
        match result {
            Err(Error::ManifestNotFound(_)) => {}
            _ => panic!("Expected ManifestNotFound error"),
        }
    }

    #[test]
    fn test_manifest_validate_empty_service_name() {
        let mut manifest = create_test_manifest();
        manifest.service.service_name = "".to_string();

        let result = manifest.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_manifest_validate_success() {
        let manifest = create_test_manifest();
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_update_policy_serialization() {
        let policy = UpdatePolicy::Auto;
        let serialized = serde_json::to_string(&policy).unwrap();
        assert_eq!(serialized, "\"auto\"");

        let policy = UpdatePolicy::SuggestOnly;
        let serialized = serde_json::to_string(&policy).unwrap();
        assert_eq!(serialized, "\"suggest_only\"");

        let policy = UpdatePolicy::Protected;
        let serialized = serde_json::to_string(&policy).unwrap();
        assert_eq!(serialized, "\"protected\"");
    }

    #[test]
    fn test_manifest_json_roundtrip() {
        let manifest = create_test_manifest();

        // JSON に変換
        let json = serde_json::to_string_pretty(&manifest).unwrap();

        // JSON から復元
        let restored: Manifest = serde_json::from_str(&json).unwrap();

        // 検証
        assert_eq!(restored.schema_version, manifest.schema_version);
        assert_eq!(restored.k1s0_version, manifest.k1s0_version);
        assert_eq!(restored.template.name, manifest.template.name);
        assert_eq!(restored.template.version, manifest.template.version);
        assert_eq!(restored.template.path, manifest.template.path);
        assert_eq!(restored.template.fingerprint, manifest.template.fingerprint);
        assert_eq!(restored.service.service_name, manifest.service.service_name);
        assert_eq!(restored.service.language, manifest.service.language);
        assert_eq!(restored.service.service_type, manifest.service.service_type);
    }

    #[test]
    fn test_load_generated_manifest() {
        // 実際に生成された manifest.json を読み込むテスト
        // このテストは feature/backend/rust/test-service/.k1s0/manifest.json が存在する場合のみ有効
        let manifest_path = std::path::Path::new("../../feature/backend/rust/test-service/.k1s0/manifest.json");
        if manifest_path.exists() {
            let manifest = Manifest::load(manifest_path).unwrap();

            // フェーズ13の要件を検証
            assert!(!manifest.k1s0_version.is_empty(), "k1s0_version は必須");
            assert!(!manifest.template.name.is_empty(), "template.name は必須");
            assert!(!manifest.template.version.is_empty(), "template.version は必須");
            assert!(!manifest.template.path.is_empty(), "template.path は必須");
            assert!(!manifest.template.fingerprint.is_empty(), "template.fingerprint は必須");
            assert!(!manifest.managed_paths.is_empty(), "managed_paths は必須");
            assert!(!manifest.protected_paths.is_empty(), "protected_paths は必須");

            // バリデーション
            assert!(manifest.validate().is_ok());
        }
    }

    #[test]
    fn test_validate_schema_valid_manifest() {
        let manifest = create_test_manifest();
        let result = manifest.validate_schema();
        if let Err(e) = &result {
            eprintln!("Validation error: {:?}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_schema_invalid_service_name_pattern() {
        let mut manifest = create_test_manifest();
        // 大文字を含む無効なサービス名
        manifest.service.service_name = "TestService".to_string();

        let result = manifest.validate_schema();
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(error.contains("service_name") || error.contains("pattern"));
    }

    #[test]
    fn test_validate_schema_invalid_language() {
        let mut manifest = create_test_manifest();
        manifest.service.language = "invalid_language".to_string();

        let result = manifest.validate_schema();
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_schema_invalid_service_type() {
        let mut manifest = create_test_manifest();
        manifest.service.service_type = "invalid_type".to_string();

        let result = manifest.validate_schema();
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_business_rules_overlapping_paths() {
        let mut manifest = create_test_manifest();
        // managed_paths と protected_paths に重複を追加
        manifest.managed_paths.push("src/domain/".to_string());

        let result = manifest.validate_business_rules();
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(error.contains("overlap"));
    }

    #[test]
    fn test_validate_business_rules_invalid_fingerprint() {
        let mut manifest = create_test_manifest();
        // 非16進数のフィンガープリント
        manifest.template.fingerprint = "xyz12345".to_string();

        let result = manifest.validate_business_rules();
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(error.contains("hexadecimal"));
    }

    #[test]
    fn test_validate_business_rules_short_fingerprint() {
        let mut manifest = create_test_manifest();
        // 8文字未満のフィンガープリント
        manifest.template.fingerprint = "abc123".to_string();

        let result = manifest.validate_business_rules();
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(error.contains("8 characters"));
    }

    #[test]
    fn test_validate_full_valid_manifest() {
        let manifest = create_test_manifest();
        assert!(manifest.validate().is_ok());
    }

    // 3層構造関連のテスト

    #[test]
    fn test_layer_type_default() {
        let json = r#"{
            "schema_version": "1.0.0",
            "k1s0_version": "0.1.0",
            "template": {
                "name": "backend-rust",
                "version": "0.1.0",
                "path": "CLI/templates/backend-rust/feature",
                "fingerprint": "abcd1234"
            },
            "service": {
                "service_name": "test",
                "language": "rust",
                "type": "backend"
            },
            "generated_at": "2026-01-25T10:00:00Z",
            "managed_paths": ["deploy/"],
            "protected_paths": ["src/"]
        }"#;

        let manifest: Manifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.layer, LayerType::Feature);
    }

    #[test]
    fn test_layer_type_serialization() {
        assert_eq!(serde_json::to_string(&LayerType::Framework).unwrap(), "\"framework\"");
        assert_eq!(serde_json::to_string(&LayerType::Domain).unwrap(), "\"domain\"");
        assert_eq!(serde_json::to_string(&LayerType::Feature).unwrap(), "\"feature\"");
    }

    #[test]
    fn test_layer_type_deserialization() {
        let framework: LayerType = serde_json::from_str("\"framework\"").unwrap();
        assert_eq!(framework, LayerType::Framework);

        let domain: LayerType = serde_json::from_str("\"domain\"").unwrap();
        assert_eq!(domain, LayerType::Domain);

        let feature: LayerType = serde_json::from_str("\"feature\"").unwrap();
        assert_eq!(feature, LayerType::Feature);
    }

    #[test]
    fn test_domain_manifest() {
        let manifest = create_test_domain_manifest();
        assert!(manifest.is_domain());
        assert!(!manifest.is_feature());
        assert!(!manifest.is_framework());
        assert_eq!(manifest.version, Some("1.2.0".to_string()));
    }

    #[test]
    fn test_feature_with_domain_manifest() {
        let manifest = create_test_feature_with_domain_manifest();
        assert!(manifest.is_feature());
        assert_eq!(manifest.domain, Some("production".to_string()));
        assert_eq!(manifest.domain_version, Some("^1.2.0".to_string()));
    }

    #[test]
    fn test_parsed_version() {
        let manifest = create_test_domain_manifest();
        let version = manifest.parsed_version().unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 0);
    }

    #[test]
    fn test_parsed_domain_version() {
        let manifest = create_test_feature_with_domain_manifest();
        let version_req = manifest.parsed_domain_version().unwrap();
        assert!(version_req.matches(&semver::Version::new(1, 2, 0)));
        assert!(version_req.matches(&semver::Version::new(1, 3, 0)));
        assert!(!version_req.matches(&semver::Version::new(2, 0, 0)));
    }

    #[test]
    fn test_domain_layer_requires_version() {
        let mut manifest = create_test_domain_manifest();
        manifest.version = None;
        let result = manifest.validate_business_rules();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("version"));
    }

    #[test]
    fn test_deprecated_flag() {
        let mut manifest = create_test_domain_manifest();
        manifest.deprecated = Some(Deprecated::Flag(true));
        assert!(manifest.is_deprecated());

        manifest.deprecated = Some(Deprecated::Flag(false));
        assert!(!manifest.is_deprecated());
    }

    #[test]
    fn test_deprecated_info() {
        let mut manifest = create_test_domain_manifest();
        manifest.deprecated = Some(Deprecated::Info(DeprecatedInfo {
            since: Some("1.5.0".to_string()),
            migrate_to: Some("production-v2".to_string()),
            deadline: Some("2026-12-31".to_string()),
            reason: Some("Merged into production-v2".to_string()),
        }));
        assert!(manifest.is_deprecated());
    }

    #[test]
    fn test_dependencies_all_framework_crates() {
        let deps = Dependencies {
            framework_crates: vec![
                CrateDependency { name: "k1s0-error".to_string(), version: "0.1.0".to_string() },
            ],
            framework: vec!["k1s0-config".to_string(), "k1s0-error".to_string()],
            domain: None,
        };
        let all = deps.all_framework_crates();
        assert_eq!(all.len(), 2);
        assert!(all.contains(&"k1s0-config".to_string()));
        assert!(all.contains(&"k1s0-error".to_string()));
    }

    #[test]
    fn test_manifest_with_domain_dependencies() {
        let json = r#"{
            "schema_version": "1.0.0",
            "k1s0_version": "0.1.0",
            "template": {
                "name": "backend-rust",
                "version": "0.1.0",
                "path": "CLI/templates/backend-rust/feature",
                "fingerprint": "abcd1234"
            },
            "service": {
                "service_name": "work-order",
                "language": "rust",
                "type": "backend"
            },
            "layer": "feature",
            "domain": "production",
            "domain_version": "^1.2.0",
            "generated_at": "2026-01-25T10:00:00Z",
            "managed_paths": ["deploy/"],
            "protected_paths": ["src/"],
            "dependencies": {
                "framework": ["k1s0-error", "k1s0-config"],
                "domain": {
                    "production": "^1.2.0"
                }
            }
        }"#;

        let manifest: Manifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.layer, LayerType::Feature);
        assert_eq!(manifest.domain, Some("production".to_string()));
        assert_eq!(manifest.domain_version, Some("^1.2.0".to_string()));

        let deps = manifest.dependencies.unwrap();
        assert_eq!(deps.framework, vec!["k1s0-error", "k1s0-config"]);
        let domain_deps = deps.domain.unwrap();
        assert_eq!(domain_deps.get("production"), Some(&"^1.2.0".to_string()));
    }

    #[test]
    fn test_invalid_semver_version() {
        let mut manifest = create_test_domain_manifest();
        manifest.version = Some("invalid".to_string());
        let result = manifest.validate_business_rules();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("SemVer"));
    }

    #[test]
    fn test_invalid_domain_version_constraint() {
        let mut manifest = create_test_feature_with_domain_manifest();
        manifest.domain_version = Some("invalid".to_string());
        let result = manifest.validate_business_rules();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("constraint"));
    }
}
