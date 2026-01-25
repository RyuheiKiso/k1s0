//! manifest.json の読み書き
//!
//! `.k1s0/manifest.json` の読み込み・書き込み・バリデーションを提供する。

use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::{Error, Result};

/// manifest.json のスキーマバージョン
pub const SCHEMA_VERSION: &str = "1.0.0";

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

    /// 生成日時
    pub generated_at: String,

    /// CLI が管理するパス
    pub managed_paths: Vec<String>,

    /// CLI が変更しないパス
    pub protected_paths: Vec<String>,

    /// パス別の更新ポリシー
    #[serde(default)]
    pub update_policy: std::collections::HashMap<String, UpdatePolicy>,

    /// ファイルのチェックサム
    #[serde(default)]
    pub checksums: std::collections::HashMap<String, String>,

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
    /// framework crates
    #[serde(default)]
    pub framework_crates: Vec<CrateDependency>,
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
        // TODO: JSON Schema を使用したバリデーション
        // 最低限のバリデーション
        if self.service.service_name.is_empty() {
            return Err(Error::ManifestValidation(
                "service_name is required".to_string(),
            ));
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
            generated_at: "2026-01-25T10:00:00Z".to_string(),
            managed_paths: vec!["deploy/".to_string(), "buf.yaml".to_string()],
            protected_paths: vec!["src/domain/".to_string(), "src/application/".to_string()],
            update_policy: std::collections::HashMap::from([
                ("deploy/".to_string(), UpdatePolicy::Auto),
                ("src/domain/".to_string(), UpdatePolicy::Protected),
            ]),
            checksums: std::collections::HashMap::new(),
            dependencies: None,
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
}
