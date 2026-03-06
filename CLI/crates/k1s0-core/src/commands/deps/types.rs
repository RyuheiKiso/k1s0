//! 依存関係マップの型定義。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

/// 解析スコープ。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DepsScope {
    /// 全サービスを対象にする
    All,
    /// 特定のTierを対象にする ("system", "business", "service")
    Tier(String),
    /// 特定のサービスを対象にする
    Services(Vec<String>),
}

/// 出力形式。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DepsOutputFormat {
    /// ターミナルにテキスト出力
    Terminal,
    /// Mermaidファイルに出力
    Mermaid(PathBuf),
    /// ターミナルとMermaidファイルの両方
    Both(PathBuf),
}

/// 依存関係マップの設定。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepsConfig {
    /// 解析スコープ
    pub scope: DepsScope,
    /// 出力形式
    pub output: DepsOutputFormat,
    /// キャッシュを無視するかどうか
    pub no_cache: bool,
}

/// サービス情報。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// サービス名（例: "auth", "order"）
    pub name: String,
    /// Tier ("system", "business", "service")
    pub tier: String,
    /// ドメイン（business/service tier のドメイン名）
    pub domain: Option<String>,
    /// 実装言語 ("rust", "go", "typescript", "dart")
    pub language: String,
    /// サービスのファイルシステムパス
    pub path: PathBuf,
}

/// 依存関係の種類。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DependencyType {
    /// gRPC依存（proto import）
    Grpc,
    /// Kafka依存（publish/subscribe）
    Kafka,
    /// REST API依存
    Rest,
    /// `GraphQL依存`
    GraphQL,
    /// ライブラリ依存
    Library,
}

impl fmt::Display for DependencyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DependencyType::Grpc => write!(f, "gRPC"),
            DependencyType::Kafka => write!(f, "Kafka"),
            DependencyType::Rest => write!(f, "REST"),
            DependencyType::GraphQL => write!(f, "GraphQL"),
            DependencyType::Library => write!(f, "Library"),
        }
    }
}

/// サービス間の依存関係。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// ソースサービス名
    pub source: String,
    /// ソースサービスのTier
    pub source_tier: String,
    /// ターゲットサービス名
    pub target: String,
    /// ターゲットサービスのTier
    pub target_tier: String,
    /// 依存関係の種類
    pub dep_type: DependencyType,
    /// 検出場所（ファイルパスなど）
    pub locations: Vec<String>,
    /// 追加情報（Kafkaトピック名など）
    pub detail: Option<String>,
}

/// 違反の重大度。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// エラー: 禁止された依存関係
    Error,
    /// 警告: 推奨されない依存関係
    Warning,
    /// 情報: 注意が必要な依存関係
    Info,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "ERROR"),
            Severity::Warning => write!(f, "WARNING"),
            Severity::Info => write!(f, "INFO"),
        }
    }
}

/// Tier間ルール違反。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    /// 重大度
    pub severity: Severity,
    /// ソースサービス名
    pub source: String,
    /// ソースサービスのTier
    pub source_tier: String,
    /// ターゲットサービス名
    pub target: String,
    /// ターゲットサービスのTier
    pub target_tier: String,
    /// 依存関係の種類
    pub dep_type: DependencyType,
    /// 違反メッセージ
    pub message: String,
    /// 検出場所
    pub location: Option<String>,
    /// 推奨事項
    pub recommendation: String,
}

/// 依存関係マップの解析結果。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DepsResult {
    /// 検出されたサービス一覧
    pub services: Vec<ServiceInfo>,
    /// 検出された依存関係一覧
    pub dependencies: Vec<Dependency>,
    /// 検出されたルール違反一覧
    pub violations: Vec<Violation>,
}

/// 依存関係マップのキャッシュ。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepsCache {
    /// キャッシュバージョン
    pub version: u32,
    /// キャッシュ生成日時
    pub generated_at: String,
    /// ファイルハッシュ（パス → SHA256ハッシュ）
    pub file_hashes: HashMap<String, String>,
    /// キャッシュされた依存関係
    pub dependencies: Vec<Dependency>,
    /// キャッシュされた違反
    pub violations: Vec<Violation>,
}
