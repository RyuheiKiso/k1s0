//! ドメインレジストリの型定義

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// ドメイン情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainInfo {
    /// ドメイン名
    pub name: String,
    /// バージョン
    pub version: String,
    /// タイプ（backend-rust, backend-go, frontend-react, frontend-flutter）
    #[serde(rename = "type")]
    pub domain_type: String,
    /// 言語
    pub language: String,
    /// パス
    pub path: PathBuf,
    /// 他のドメインへの依存
    pub dependencies: HashMap<String, String>,
    /// 最小 framework バージョン
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_framework_version: Option<String>,
    /// 非推奨情報
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<DeprecatedInfo>,
    /// 破壊的変更
    #[serde(skip_serializing_if = "Option::is_none")]
    pub breaking_changes: Option<HashMap<String, String>>,
}

/// 非推奨情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeprecatedInfo {
    /// 非推奨のメッセージ
    pub message: String,
    /// 代替ドメイン
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alternative: Option<String>,
}

/// feature 情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureInfo {
    /// feature 名
    pub name: String,
    /// タイプ
    #[serde(rename = "type")]
    pub feature_type: String,
    /// パス
    pub path: PathBuf,
    /// ドメイン依存（domain_name -> version_constraint）
    pub domain_dependencies: HashMap<String, String>,
}

/// ドメインサマリー
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainSummary {
    /// 総数
    pub total: usize,
    /// アクティブ数
    pub active: usize,
    /// 非推奨数
    pub deprecated: usize,
    /// 言語別カウント
    pub by_language: HashMap<String, usize>,
    /// タイプ別カウント
    pub by_type: HashMap<String, usize>,
}

/// ドメインカタログ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainCatalog {
    /// ドメイン一覧
    pub domains: Vec<DomainCatalogEntry>,
    /// サマリー
    pub summary: DomainSummary,
}

/// カタログエントリ（依存feature数を含む）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainCatalogEntry {
    /// ドメイン情報
    #[serde(flatten)]
    pub info: DomainInfo,
    /// このドメインに依存している feature 数
    pub dependent_count: usize,
    /// ステータス
    pub status: String,
}

/// ドメインスキャンエラー
#[derive(Debug, thiserror::Error)]
pub enum DomainScanError {
    /// IO エラー
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// manifest パースエラー
    #[error("Manifest parse error at {path}: {message}")]
    ManifestParse {
        path: PathBuf,
        message: String,
    },
}

/// ドメイングラフエラー
#[derive(Debug, thiserror::Error)]
pub enum DomainGraphError {
    /// スキャンエラー
    #[error("Scan error: {0}")]
    Scan(#[from] DomainScanError),

    /// ノードが見つからない
    #[error("Domain not found: {0}")]
    NotFound(String),
}
