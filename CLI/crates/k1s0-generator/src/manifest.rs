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
