// GenerateConfig から TemplateContext への変換を担当するモジュール。
// 生成設定を、テンプレートエンジンが理解する形式に変換する。

use super::types::{ApiStyle, Framework, GenerateConfig, Kind, LangFw, Rdbms, Tier};
use crate::config::CliConfig;
use crate::template::context::{TemplateContext, TemplateContextBuilder};

/// `GenerateConfig` と `CliConfig` から `TemplateContext` を構築する。
///
/// 対応するテンプレートが存在しない `LangFw` パターンの場合は `None` を返す。
pub(crate) fn build_template_context(
    config: &GenerateConfig,
    cli_config: &CliConfig,
) -> Option<TemplateContext> {
    let service_name = config.detail.name.as_deref().unwrap_or("service");
    let tier = config.tier.as_str();

    // kind と言語/DB種別を判定
    let (language, kind) = match config.kind {
        Kind::Server => {
            let lang = match &config.lang_fw {
                LangFw::Language(l) => l.dir_name(),
                _ => return None,
            };
            (lang, "server")
        }
        Kind::Client => {
            let lang = match &config.lang_fw {
                LangFw::Framework(Framework::React) => "typescript",
                LangFw::Framework(Framework::Flutter) => "dart",
                _ => return None,
            };
            (lang, "client")
        }
        Kind::Library => {
            let lang = match &config.lang_fw {
                LangFw::Language(l) => l.dir_name(),
                _ => return None,
            };
            (lang, "library")
        }
        Kind::Database => {
            let db_type = match &config.lang_fw {
                LangFw::Database { rdbms, .. } => match rdbms {
                    Rdbms::PostgreSQL => "postgresql",
                    Rdbms::MySQL => "mysql",
                    Rdbms::SQLite => "sqlite",
                },
                _ => return None,
            };
            (db_type, "database")
        }
    };

    // API スタイルを文字列ベクタに変換
    let api_styles_strs: Vec<String> = config
        .detail
        .api_styles
        .iter()
        .map(|a| match a {
            ApiStyle::Rest => "rest".to_string(),
            ApiStyle::Grpc => "grpc".to_string(),
            ApiStyle::GraphQL => "graphql".to_string(),
        })
        .collect();

    // フレームワーク名を決定
    let fw_name = match &config.lang_fw {
        LangFw::Framework(Framework::React) => "react",
        LangFw::Framework(Framework::Flutter) => "flutter",
        _ => "",
    };

    // ビルダーパターンでコンテキストを組み立て
    let mut builder = TemplateContextBuilder::new(service_name, tier, language, kind)
        .framework(fw_name)
        .api_styles(api_styles_strs)
        .docker_registry(&cli_config.docker_registry)
        .go_module_base(&cli_config.go_module_base);

    // business tier ではドメイン名（業務領域）が必須。placement から取得する。
    if matches!(config.tier, Tier::Business) {
        if let Some(ref domain) = config.placement {
            builder = builder.domain(domain);
        }
    }

    // オプション: データベース接続
    if let Some(ref db) = config.detail.db {
        let db_type = match db.rdbms {
            Rdbms::PostgreSQL => "postgresql",
            Rdbms::MySQL => "mysql",
            Rdbms::SQLite => "sqlite",
        };
        builder = builder.with_database(db_type);
    }

    // オプション: Kafka 連携
    if config.detail.kafka {
        builder = builder.with_kafka();
    }

    // オプション: Redis 連携
    if config.detail.redis {
        builder = builder.with_redis();
    }

    // H-13 監査対応: deprecated な build() の代わりに try_build() を使用して panic を防ぐ。
    // バリデーションエラーが発生した場合は None を返し、呼び出し元でインライン生成にフォールバックする。
    match builder.try_build() {
        Ok(ctx) => Some(ctx),
        Err(e) => {
            eprintln!("テンプレートコンテキストのビルドに失敗しました: {e}");
            None
        }
    }
}
