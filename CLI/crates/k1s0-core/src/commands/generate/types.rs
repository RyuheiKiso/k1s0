use serde::{Serialize, Deserialize};
use std::fmt;

// ============================================================================
// 種別
// ============================================================================

/// 生成する種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Server,
    Client,
    Library,
    Database,
}

impl Kind {
    pub fn label(&self) -> &'static str {
        match self {
            Kind::Server => "サーバー",
            Kind::Client => "クライアント",
            Kind::Library => "ライブラリ",
            Kind::Database => "データベース",
        }
    }

    /// 選択可能なTier一覧を返す。
    pub fn available_tiers(&self) -> Vec<Tier> {
        match self {
            Kind::Server => vec![Tier::System, Tier::Business, Tier::Service],
            Kind::Client => vec![Tier::Business, Tier::Service],
            Kind::Library => vec![Tier::System, Tier::Business],
            Kind::Database => vec![Tier::System, Tier::Business, Tier::Service],
        }
    }
}

pub const KIND_LABELS: &[&str] = &["サーバー", "クライアント", "ライブラリ", "データベース"];
pub const ALL_KINDS: &[Kind] = &[Kind::Server, Kind::Client, Kind::Library, Kind::Database];

// ============================================================================
// Tier
// ============================================================================

/// Tier種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tier {
    System,
    Business,
    Service,
}

impl Tier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Tier::System => "system",
            Tier::Business => "business",
            Tier::Service => "service",
        }
    }

    pub fn label(&self) -> &'static str {
        self.as_str()
    }
}

// ============================================================================
// 言語 / フレームワーク
// ============================================================================

/// サーバー・ライブラリの言語選択。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    Go,
    Rust,
    TypeScript,
    Dart,
}

impl Language {
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::Go => "Go",
            Language::Rust => "Rust",
            Language::TypeScript => "TypeScript",
            Language::Dart => "Dart",
        }
    }

    pub fn dir_name(&self) -> &'static str {
        match self {
            Language::Go => "go",
            Language::Rust => "rust",
            Language::TypeScript => "typescript",
            Language::Dart => "dart",
        }
    }
}

/// クライアントのフレームワーク選択。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Framework {
    React,
    Flutter,
}

impl Framework {
    pub fn as_str(&self) -> &'static str {
        match self {
            Framework::React => "React",
            Framework::Flutter => "Flutter",
        }
    }

    pub fn dir_name(&self) -> &'static str {
        match self {
            Framework::React => "react",
            Framework::Flutter => "flutter",
        }
    }
}

// ============================================================================
// API方式
// ============================================================================

/// API方式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApiStyle {
    Rest,
    Grpc,
    GraphQL,
}

impl ApiStyle {
    pub fn short_label(&self) -> &'static str {
        match self {
            ApiStyle::Rest => "REST",
            ApiStyle::Grpc => "gRPC",
            ApiStyle::GraphQL => "GraphQL",
        }
    }
}

pub const API_LABELS: &[&str] = &["REST (OpenAPI)", "gRPC (protobuf)", "GraphQL"];
pub const ALL_API_STYLES: &[ApiStyle] = &[ApiStyle::Rest, ApiStyle::Grpc, ApiStyle::GraphQL];

// ============================================================================
// RDBMS
// ============================================================================

/// RDBMS種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rdbms {
    PostgreSQL,
    MySQL,
    SQLite,
}

impl Rdbms {
    pub fn as_str(&self) -> &'static str {
        match self {
            Rdbms::PostgreSQL => "PostgreSQL",
            Rdbms::MySQL => "MySQL",
            Rdbms::SQLite => "SQLite",
        }
    }
}

pub const RDBMS_LABELS: &[&str] = &["PostgreSQL", "MySQL", "SQLite"];
pub const ALL_RDBMS: &[Rdbms] = &[Rdbms::PostgreSQL, Rdbms::MySQL, Rdbms::SQLite];

// ============================================================================
// DB情報
// ============================================================================

/// データベース情報。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DbInfo {
    pub name: String,
    pub rdbms: Rdbms,
}

impl fmt::Display for DbInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.rdbms.as_str())
    }
}

// ============================================================================
// 生成設定
// ============================================================================

/// ひな形生成の設定。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenerateConfig {
    /// 種別
    pub kind: Kind,
    /// Tier
    pub tier: Tier,
    /// 配置先 (business: 領域名, service: サービス名)
    pub placement: Option<String>,
    /// 言語・FW の選択結果
    pub lang_fw: LangFw,
    /// 詳細設定
    pub detail: DetailConfig,
}

/// 言語/FW 列挙
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LangFw {
    Language(Language),
    Framework(Framework),
    Database { name: String, rdbms: Rdbms },
}

/// 詳細設定
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DetailConfig {
    /// サーバー: サービス名 / クライアント: アプリ名 / ライブラリ: ライブラリ名
    pub name: Option<String>,
    /// サーバー: API方式
    pub api_styles: Vec<ApiStyle>,
    /// サーバー: DB設定
    pub db: Option<DbInfo>,
    /// サーバー: Kafka有効
    pub kafka: bool,
    /// サーバー: Redis有効
    pub redis: bool,
    /// サーバー: BFF言語 (service Tier + GraphQL 時のみ)
    pub bff_language: Option<Language>,
}

impl Default for DetailConfig {
    fn default() -> Self {
        Self {
            name: None,
            api_styles: Vec::new(),
            db: None,
            kafka: false,
            redis: false,
            bff_language: None,
        }
    }
}

// ============================================================================
// テスト
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kind_label() {
        assert_eq!(Kind::Server.label(), "サーバー");
        assert_eq!(Kind::Client.label(), "クライアント");
        assert_eq!(Kind::Library.label(), "ライブラリ");
        assert_eq!(Kind::Database.label(), "データベース");
    }

    #[test]
    fn test_kind_available_tiers_server() {
        let tiers = Kind::Server.available_tiers();
        assert_eq!(tiers, vec![Tier::System, Tier::Business, Tier::Service]);
    }

    #[test]
    fn test_kind_available_tiers_client() {
        let tiers = Kind::Client.available_tiers();
        assert_eq!(tiers, vec![Tier::Business, Tier::Service]);
    }

    #[test]
    fn test_kind_available_tiers_library() {
        let tiers = Kind::Library.available_tiers();
        assert_eq!(tiers, vec![Tier::System, Tier::Business]);
    }

    #[test]
    fn test_kind_available_tiers_database() {
        let tiers = Kind::Database.available_tiers();
        assert_eq!(tiers, vec![Tier::System, Tier::Business, Tier::Service]);
    }

    #[test]
    fn test_tier_as_str() {
        assert_eq!(Tier::System.as_str(), "system");
        assert_eq!(Tier::Business.as_str(), "business");
        assert_eq!(Tier::Service.as_str(), "service");
    }

    #[test]
    fn test_language_as_str() {
        assert_eq!(Language::Go.as_str(), "Go");
        assert_eq!(Language::Rust.as_str(), "Rust");
        assert_eq!(Language::TypeScript.as_str(), "TypeScript");
        assert_eq!(Language::Dart.as_str(), "Dart");
    }

    #[test]
    fn test_language_dir_name() {
        assert_eq!(Language::Go.dir_name(), "go");
        assert_eq!(Language::Rust.dir_name(), "rust");
        assert_eq!(Language::TypeScript.dir_name(), "typescript");
        assert_eq!(Language::Dart.dir_name(), "dart");
    }

    #[test]
    fn test_framework_as_str() {
        assert_eq!(Framework::React.as_str(), "React");
        assert_eq!(Framework::Flutter.as_str(), "Flutter");
    }

    #[test]
    fn test_framework_dir_name() {
        assert_eq!(Framework::React.dir_name(), "react");
        assert_eq!(Framework::Flutter.dir_name(), "flutter");
    }

    #[test]
    fn test_api_style_labels() {
        assert_eq!(ApiStyle::Rest.short_label(), "REST");
        assert_eq!(ApiStyle::Grpc.short_label(), "gRPC");
        assert_eq!(ApiStyle::GraphQL.short_label(), "GraphQL");
    }

    #[test]
    fn test_rdbms_as_str() {
        assert_eq!(Rdbms::PostgreSQL.as_str(), "PostgreSQL");
        assert_eq!(Rdbms::MySQL.as_str(), "MySQL");
        assert_eq!(Rdbms::SQLite.as_str(), "SQLite");
    }

    #[test]
    fn test_db_info_display() {
        let db = DbInfo {
            name: "order-db".to_string(),
            rdbms: Rdbms::PostgreSQL,
        };
        assert_eq!(format!("{}", db), "order-db (PostgreSQL)");
    }

    #[test]
    fn test_detail_config_default() {
        let d = DetailConfig::default();
        assert!(d.name.is_none());
        assert!(d.api_styles.is_empty());
        assert!(d.db.is_none());
        assert!(!d.kafka);
        assert!(!d.redis);
        assert!(d.bff_language.is_none());
    }

    #[test]
    fn test_detail_config_default_bff_language_none() {
        // bff_language のデフォルトは None
        let d = DetailConfig::default();
        assert_eq!(d.bff_language, None);
    }

    #[test]
    fn test_generate_config_json_roundtrip() {
        let config = GenerateConfig {
            kind: Kind::Server,
            tier: Tier::System,
            placement: None,
            lang_fw: LangFw::Language(Language::Go),
            detail: DetailConfig {
                name: Some("auth".to_string()),
                api_styles: vec![ApiStyle::Rest],
                db: None,
                kafka: false,
                redis: false,
                bff_language: None,
            },
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: GenerateConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }
}
