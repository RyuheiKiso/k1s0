use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::commands::generate::types::{
    ApiStyle, DetailConfig, Framework, GenerateConfig, Kind, LangFw, Language, Rdbms, Tier,
};
use crate::config::CliConfig;

/// テンプレートマニフェスト (.k1s0-template.yaml)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateManifest {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub kind: String,
    pub metadata: TemplateMetadata,
    pub spec: TemplateSpec,
}

impl TemplateManifest {
    /// テンプレート種別を返す。
    pub fn template_type(&self) -> &str {
        &self.spec.template.template_type
    }

    /// テンプレート言語を返す。
    pub fn language(&self) -> &str {
        &self.spec.template.language
    }

    /// テンプレートバージョンを返す。
    pub fn version(&self) -> &str {
        &self.spec.template.version
    }

    /// テンプレートチェックサムを返す。
    pub fn checksum(&self) -> &str {
        &self.spec.template.checksum
    }

    /// バージョンとチェックサムを更新する。
    pub fn update_template_state(&mut self, version: &str, checksum: &str) {
        self.spec.template.version = version.to_string();
        self.spec.template.checksum = checksum.to_string();
    }

    /// ひな形生成設定と CLI 設定に復元する。
    ///
    /// # Errors
    ///
    /// マニフェストの内容が不正な場合にエラーを返す。
    pub fn to_generate_context(&self) -> Result<(GenerateConfig, CliConfig)> {
        let tier = parse_tier(&self.spec.parameters.tier)?;
        let placement = self.spec.parameters.placement.clone();
        let module_name = self
            .spec
            .parameters
            .module_name
            .clone()
            .or_else(|| self.spec.parameters.service_name.clone())
            .ok_or_else(|| anyhow!("manifest spec.parameters.moduleName is required"))?;

        let kind = parse_kind(self.template_type())?;
        let lang_fw = match kind {
            Kind::Server | Kind::Library => LangFw::Language(parse_language(self.language())?),
            Kind::Client => LangFw::Framework(parse_framework(
                self.spec
                    .parameters
                    .framework
                    .as_deref()
                    .ok_or_else(|| anyhow!("manifest spec.parameters.framework is required"))?,
            )?),
            Kind::Database => LangFw::Database {
                name: module_name.clone(),
                rdbms: parse_rdbms(self.language())?,
            },
        };

        let api_styles = self
            .spec
            .parameters
            .api_styles
            .iter()
            .map(|style| parse_api_style(style))
            .collect::<Result<Vec<_>>>()?;

        let db = match (
            self.spec.parameters.database.clone(),
            self.spec.parameters.database_type.as_deref(),
        ) {
            (Some(name), Some(rdbms)) => Some(crate::commands::generate::types::DbInfo {
                name,
                rdbms: parse_rdbms(rdbms)?,
            }),
            _ => None,
        };

        let bff_language = self
            .spec
            .parameters
            .bff_language
            .as_deref()
            .map(parse_language)
            .transpose()?;

        let config = GenerateConfig {
            kind,
            tier,
            placement,
            lang_fw,
            detail: DetailConfig {
                name: Some(module_name),
                api_styles,
                db,
                kafka: self.spec.parameters.kafka.unwrap_or(false),
                redis: self.spec.parameters.redis.unwrap_or(false),
                bff_language,
            },
        };

        let mut cli_config = CliConfig::default();
        if let Some(docker_registry) = &self.spec.parameters.docker_registry {
            cli_config.docker_registry.clone_from(docker_registry);
        }
        if let Some(go_module_base) = &self.spec.parameters.go_module_base {
            cli_config.go_module_base.clone_from(go_module_base);
        }

        Ok((config, cli_config))
    }

    /// ひな形生成設定からマニフェストを構築する。
    pub fn from_generate_config(
        config: &GenerateConfig,
        cli_config: &CliConfig,
        version: &str,
        checksum: &str,
    ) -> Self {
        let template_type = match config.kind {
            Kind::Server => "server",
            Kind::Client => "client",
            Kind::Library => "library",
            Kind::Database => "database",
        };

        let language = match &config.lang_fw {
            LangFw::Language(language) => language.dir_name().to_string(),
            LangFw::Framework(Framework::React) => "typescript".to_string(),
            LangFw::Framework(Framework::Flutter) => "dart".to_string(),
            LangFw::Database { rdbms, .. } => match rdbms {
                Rdbms::PostgreSQL => "postgresql".to_string(),
                Rdbms::MySQL => "mysql".to_string(),
                Rdbms::SQLite => "sqlite".to_string(),
            },
        };

        let module_name = config
            .detail
            .name
            .clone()
            .or_else(|| match &config.lang_fw {
                LangFw::Database { name, .. } => Some(name.clone()),
                _ => None,
            });

        let service_name = match config.kind {
            Kind::Server => module_name.clone(),
            _ => None,
        };

        let framework = match &config.lang_fw {
            LangFw::Framework(framework) => Some(match framework {
                Framework::React => "react".to_string(),
                Framework::Flutter => "flutter".to_string(),
            }),
            _ => None,
        };

        let database = config.detail.db.as_ref().map(|db| db.name.clone());
        let database_type = config.detail.db.as_ref().map(|db| match db.rdbms {
            Rdbms::PostgreSQL => "postgresql".to_string(),
            Rdbms::MySQL => "mysql".to_string(),
            Rdbms::SQLite => "sqlite".to_string(),
        });

        let bff_language = config.detail.bff_language.map(|language| match language {
            Language::Go => "go".to_string(),
            Language::Rust => "rust".to_string(),
            Language::TypeScript => "typescript".to_string(),
            Language::Dart => "dart".to_string(),
        });

        Self {
            api_version: "k1s0/v1".to_string(),
            kind: "TemplateInstance".to_string(),
            metadata: TemplateMetadata {
                name: manifest_name(config),
                generated_at: chrono::Utc::now().to_rfc3339(),
                generated_by: format!("k1s0-cli@{}", env!("CARGO_PKG_VERSION")),
            },
            spec: TemplateSpec {
                template: TemplateDescriptor {
                    template_type: template_type.to_string(),
                    language,
                    version: version.to_string(),
                    checksum: checksum.to_string(),
                },
                parameters: TemplateParameters {
                    tier: config.tier.as_str().to_string(),
                    placement: config.placement.clone(),
                    service_name,
                    module_name,
                    framework,
                    api_styles: config
                        .detail
                        .api_styles
                        .iter()
                        .map(|style| match style {
                            ApiStyle::Rest => "rest".to_string(),
                            ApiStyle::Grpc => "grpc".to_string(),
                            ApiStyle::GraphQL => "graphql".to_string(),
                        })
                        .collect(),
                    database,
                    database_type,
                    kafka: Some(config.detail.kafka),
                    redis: Some(config.detail.redis),
                    bff_language,
                    docker_registry: Some(cli_config.docker_registry.clone()),
                    go_module_base: Some(cli_config.go_module_base.clone()),
                },
                customizations: TemplateCustomizations::for_config(config),
            },
        }
    }
}

/// マニフェストのメタデータ。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    pub name: String,
    #[serde(rename = "generatedAt")]
    pub generated_at: String,
    #[serde(rename = "generatedBy")]
    pub generated_by: String,
}

/// マニフェストの spec。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateSpec {
    pub template: TemplateDescriptor,
    pub parameters: TemplateParameters,
    #[serde(default)]
    pub customizations: TemplateCustomizations,
}

/// テンプレート識別情報。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateDescriptor {
    #[serde(rename = "type")]
    pub template_type: String,
    pub language: String,
    pub version: String,
    pub checksum: String,
}

/// ひな形生成に必要なパラメータ群。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateParameters {
    pub tier: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placement: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub framework: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub api_styles: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kafka: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redis: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bff_language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docker_registry: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub go_module_base: Option<String>,
}

/// カスタマイズルール。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateCustomizations {
    #[serde(default)]
    pub ignore_paths: Vec<String>,
    #[serde(default)]
    pub merge_strategy: BTreeMap<String, MergeStrategy>,
}

impl TemplateCustomizations {
    fn for_config(config: &GenerateConfig) -> Self {
        let mut merge_strategy = BTreeMap::new();
        let mut ignore_paths = Vec::new();

        match config.kind {
            Kind::Server => {
                ignore_paths.push("src/domain/**".to_string());
                ignore_paths.push("migrations/**".to_string());
                merge_strategy.insert("Cargo.toml".to_string(), MergeStrategy::Merge);
                merge_strategy.insert("go.mod".to_string(), MergeStrategy::Merge);
                merge_strategy.insert("package.json".to_string(), MergeStrategy::Merge);
                merge_strategy.insert("docker-compose.yaml".to_string(), MergeStrategy::Merge);
                merge_strategy.insert(".env.example".to_string(), MergeStrategy::Merge);
                merge_strategy.insert("src/main.rs".to_string(), MergeStrategy::Template);
                merge_strategy.insert("cmd/main.go".to_string(), MergeStrategy::Template);
                merge_strategy.insert("Dockerfile".to_string(), MergeStrategy::Template);
            }
            Kind::Client => {
                merge_strategy.insert("package.json".to_string(), MergeStrategy::Merge);
                merge_strategy.insert("pubspec.yaml".to_string(), MergeStrategy::Merge);
                merge_strategy.insert("src/main.tsx".to_string(), MergeStrategy::Template);
                merge_strategy.insert("lib/main.dart".to_string(), MergeStrategy::Template);
            }
            Kind::Library => {
                merge_strategy.insert("Cargo.toml".to_string(), MergeStrategy::Merge);
                merge_strategy.insert("go.mod".to_string(), MergeStrategy::Merge);
                merge_strategy.insert("package.json".to_string(), MergeStrategy::Merge);
                merge_strategy.insert("pubspec.yaml".to_string(), MergeStrategy::Merge);
            }
            Kind::Database => {
                ignore_paths.push("migrations/**".to_string());
            }
        }

        Self {
            ignore_paths,
            merge_strategy,
        }
    }
}

/// マージ戦略。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum MergeStrategy {
    Merge,
    Template,
    User,
    #[default]
    Ask,
}

/// マイグレーション対象。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationTarget {
    pub path: PathBuf,
    pub manifest: TemplateManifest,
    pub available_version: String,
}

/// マイグレーション計画。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationPlan {
    pub target: MigrationTarget,
    pub changes: Vec<FileChange>,
}

impl MigrationPlan {
    /// コンフリクトを含むか判定する。
    pub fn has_conflicts(&self) -> bool {
        self.changes
            .iter()
            .any(|change| matches!(change.merge_result, MergeResult::Conflict(_)))
    }
}

/// ファイル変更。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: PathBuf,
    pub change_type: ChangeType,
    pub merge_strategy: MergeStrategy,
    pub merge_result: MergeResult,
}

/// 変更種別。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Skipped,
}

/// マージ結果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MergeResult {
    Clean(String),
    Conflict(Vec<ConflictHunk>),
    NoChange,
}

/// コンフリクトハンク。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictHunk {
    pub base: String,
    pub ours: String,
    pub theirs: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_preview: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ours_preview: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theirs_preview: Option<String>,
}

/// コンフリクト解決方針。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolution {
    UseTemplate,
    UseUser,
    Skip,
}

fn manifest_name(config: &GenerateConfig) -> String {
    let module_name = config
        .detail
        .name
        .clone()
        .or_else(|| match &config.lang_fw {
            LangFw::Database { name, .. } => Some(name.clone()),
            _ => None,
        })
        .unwrap_or_else(|| "module".to_string());
    let suffix = match config.kind {
        Kind::Server => "server",
        Kind::Client => "client",
        Kind::Library => "library",
        Kind::Database => "database",
    };
    format!("{module_name}-{suffix}")
}

fn parse_kind(value: &str) -> Result<Kind> {
    match value {
        "server" => Ok(Kind::Server),
        "client" => Ok(Kind::Client),
        "library" => Ok(Kind::Library),
        "database" => Ok(Kind::Database),
        _ => Err(anyhow!("unsupported template type: {value}")),
    }
}

fn parse_tier(value: &str) -> Result<Tier> {
    match value {
        "system" => Ok(Tier::System),
        "business" => Ok(Tier::Business),
        "service" => Ok(Tier::Service),
        _ => Err(anyhow!("unsupported tier: {value}")),
    }
}

fn parse_language(value: &str) -> Result<Language> {
    match value {
        "go" => Ok(Language::Go),
        "rust" => Ok(Language::Rust),
        "typescript" => Ok(Language::TypeScript),
        "dart" => Ok(Language::Dart),
        _ => Err(anyhow!("unsupported language: {value}")),
    }
}

fn parse_framework(value: &str) -> Result<Framework> {
    match value {
        "react" => Ok(Framework::React),
        "flutter" => Ok(Framework::Flutter),
        _ => Err(anyhow!("unsupported framework: {value}")),
    }
}

fn parse_rdbms(value: &str) -> Result<Rdbms> {
    match value {
        "postgresql" => Ok(Rdbms::PostgreSQL),
        "mysql" => Ok(Rdbms::MySQL),
        "sqlite" => Ok(Rdbms::SQLite),
        _ => Err(anyhow!("unsupported database type: {value}")),
    }
}

fn parse_api_style(value: &str) -> Result<ApiStyle> {
    match value {
        "rest" => Ok(ApiStyle::Rest),
        "grpc" => Ok(ApiStyle::Grpc),
        "graphql" => Ok(ApiStyle::GraphQL),
        _ => Err(anyhow!("unsupported api style: {value}")),
    }
}

/// パス文字列を POSIX 形式に正規化する。
pub fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_manifest_roundtrip_server() {
        let config = GenerateConfig {
            kind: Kind::Server,
            tier: Tier::Service,
            placement: Some("order".to_string()),
            lang_fw: LangFw::Language(Language::Rust),
            detail: DetailConfig {
                name: Some("order".to_string()),
                api_styles: vec![ApiStyle::Rest, ApiStyle::Grpc],
                db: Some(crate::commands::generate::types::DbInfo {
                    name: "order-db".to_string(),
                    rdbms: Rdbms::PostgreSQL,
                }),
                kafka: true,
                redis: true,
                bff_language: Some(Language::Go),
            },
        };
        let cli_config = CliConfig {
            docker_registry: "registry.internal".to_string(),
            go_module_base: "github.com/acme/platform".to_string(),
            ..CliConfig::default()
        };

        let manifest =
            TemplateManifest::from_generate_config(&config, &cli_config, "1.5.0", "sha256:abc");
        let yaml = serde_yaml::to_string(&manifest).unwrap();
        let parsed: TemplateManifest = serde_yaml::from_str(&yaml).unwrap();
        let (restored_config, restored_cli_config) = parsed.to_generate_context().unwrap();

        assert_eq!(restored_config.kind, Kind::Server);
        assert_eq!(restored_config.tier, Tier::Service);
        assert_eq!(restored_config.placement, Some("order".to_string()));
        assert_eq!(
            restored_config.detail.api_styles,
            vec![ApiStyle::Rest, ApiStyle::Grpc]
        );
        assert_eq!(restored_config.detail.db.unwrap().name, "order-db");
        assert!(restored_config.detail.kafka);
        assert!(restored_config.detail.redis);
        assert_eq!(restored_config.detail.bff_language, Some(Language::Go));
        assert_eq!(restored_cli_config.docker_registry, "registry.internal");
        assert_eq!(
            restored_cli_config.go_module_base,
            "github.com/acme/platform"
        );
        assert!(yaml.contains("apiVersion: k1s0/v1"));
        assert!(yaml.contains("kind: TemplateInstance"));
        assert!(yaml.contains("serviceName: order"));
        assert!(yaml.contains("apiStyles:"));
    }

    #[test]
    fn template_manifest_roundtrip_client() {
        let config = GenerateConfig {
            kind: Kind::Client,
            tier: Tier::Business,
            placement: Some("accounting".to_string()),
            lang_fw: LangFw::Framework(Framework::React),
            detail: DetailConfig {
                name: Some("accounting-web".to_string()),
                ..DetailConfig::default()
            },
        };

        let manifest = TemplateManifest::from_generate_config(
            &config,
            &CliConfig::default(),
            "1.5.0",
            "sha256:def",
        );
        let (restored_config, _) = manifest.to_generate_context().unwrap();

        assert_eq!(restored_config.kind, Kind::Client);
        assert_eq!(restored_config.tier, Tier::Business);
        assert_eq!(restored_config.placement, Some("accounting".to_string()));
        assert!(matches!(
            restored_config.lang_fw,
            LangFw::Framework(Framework::React)
        ));
        assert_eq!(
            restored_config.detail.name,
            Some("accounting-web".to_string())
        );
    }

    #[test]
    fn normalize_path_uses_forward_slashes() {
        let path = PathBuf::from("src").join("main.rs");
        assert_eq!(normalize_path(&path), "src/main.rs");
    }
}
