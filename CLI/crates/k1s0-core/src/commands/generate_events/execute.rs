use anyhow::{Context, Result};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use tera::Tera;

use super::context::{build_consumer_context, build_single_event_context, build_template_context};
use super::types::EventsConfig;

/// テンプレートディレクトリのベースパス
const TEMPLATE_DIR: &str = "templates/events";

/// コード生成を実行し、生成されたファイルパスの一覧を返す。
///
/// # Errors
///
/// テンプレートの読み込み・レンダリング・ファイル書き込みに失敗した場合にエラーを返す。
pub fn execute_event_codegen(
    config: &EventsConfig,
    output_dir: &Path,
    template_base: &Path,
) -> Result<Vec<PathBuf>> {
    let ctx = build_template_context(config);
    let mut generated = Vec::new();

    // Proto ファイル生成
    for event in &config.events {
        let proto_path = output_dir
            .join("proto")
            .join(&config.domain)
            .join("events")
            .join(format!("v{}", event.version))
            .join(format!("{}.proto", event.name_snake()));
        let event_ctx = build_single_event_context(config, event);
        let rendered = render_template(template_base, "proto/event.proto.tera", &event_ctx)?;
        write_generated_file(&proto_path, &rendered)?;
        generated.push(proto_path);
    }

    // 言語別コード生成
    match config.language.as_str() {
        "rust" => {
            generated.extend(generate_rust(config, output_dir, template_base, &ctx)?);
        }
        "go" => {
            generated.extend(generate_go(config, output_dir, template_base, &ctx)?);
        }
        _ => {}
    }

    // Outbox マイグレーション
    if config.events.iter().any(|e| e.outbox) {
        let migrations_dir = output_dir.join("migrations");
        let next_num = next_migration_number(&migrations_dir);

        let up_path = migrations_dir.join(format!("{next_num:04}_create_outbox.up.sql"));
        let rendered = render_template(template_base, "sql/outbox.up.sql.tera", &ctx)?;
        write_generated_file(&up_path, &rendered)?;
        generated.push(up_path);

        let down_path = migrations_dir.join(format!("{next_num:04}_create_outbox.down.sql"));
        let rendered = render_template(template_base, "sql/outbox.down.sql.tera", &ctx)?;
        write_generated_file(&down_path, &rendered)?;
        generated.push(down_path);
    }

    // Schema Registry 設定
    let sr_path = output_dir.join("config").join("schema-registry.yaml");
    let rendered = render_template(template_base, "config/schema-registry.yaml.tera", &ctx)?;
    write_generated_file(&sr_path, &rendered)?;
    generated.push(sr_path);

    Ok(generated)
}

/// Rust コード生成
fn generate_rust(
    config: &EventsConfig,
    output_dir: &Path,
    template_base: &Path,
    ctx: &tera::Context,
) -> Result<Vec<PathBuf>> {
    let mut generated = Vec::new();
    let events_dir = output_dir.join("src").join("events");

    // mod.rs
    let mod_path = events_dir.join("mod.rs");
    let rendered = render_template(template_base, "rust/mod.rs.tera", ctx)?;
    write_generated_file(&mod_path, &rendered)?;
    generated.push(mod_path);

    // types.rs
    let types_path = events_dir.join("types.rs");
    let rendered = render_template(template_base, "rust/types.rs.tera", ctx)?;
    write_generated_file(&types_path, &rendered)?;
    generated.push(types_path);

    // producer.rs
    let producer_path = events_dir.join("producer.rs");
    let rendered = render_template(template_base, "rust/producer.rs.tera", ctx)?;
    write_generated_file(&producer_path, &rendered)?;
    generated.push(producer_path);

    // consumer ハンドラ (consumers を持つイベントのみ)
    for event in &config.events {
        for consumer in &event.consumers {
            let consumer_path = events_dir
                .join("consumers")
                .join(format!("{}.rs", consumer.handler));
            let consumer_ctx = build_consumer_context(config, event, consumer);
            let rendered = render_template(template_base, "rust/consumer.rs.tera", &consumer_ctx)?;
            write_generated_file(&consumer_path, &rendered)?;
            generated.push(consumer_path);
        }
    }

    Ok(generated)
}

/// Go コード生成
fn generate_go(
    config: &EventsConfig,
    output_dir: &Path,
    template_base: &Path,
    ctx: &tera::Context,
) -> Result<Vec<PathBuf>> {
    let mut generated = Vec::new();
    let events_dir = output_dir.join("internal").join("events");

    // producer.go
    let producer_path = events_dir.join("producer.go");
    let rendered = render_template(template_base, "go/producer.go.tera", ctx)?;
    write_generated_file(&producer_path, &rendered)?;
    generated.push(producer_path);

    // consumer ハンドラ
    for event in &config.events {
        for consumer in &event.consumers {
            let consumer_path = events_dir
                .join("consumers")
                .join(format!("{}.go", consumer.handler));
            let consumer_ctx = build_consumer_context(config, event, consumer);
            let rendered = render_template(template_base, "go/consumer.go.tera", &consumer_ctx)?;
            write_generated_file(&consumer_path, &rendered)?;
            generated.push(consumer_path);
        }
    }

    Ok(generated)
}

/// Tera テンプレートをレンダリングする。
fn render_template(
    template_base: &Path,
    template_name: &str,
    ctx: &tera::Context,
) -> Result<String> {
    let template_path = template_base.join(template_name);
    let template_content = fs::read_to_string(&template_path)
        .with_context(|| format!("テンプレートを読み込めません: {}", template_path.display()))?;

    let mut tera = Tera::default();
    tera.add_raw_template(template_name, &template_content)
        .with_context(|| format!("テンプレートのパースに失敗: {template_name}"))?;

    tera.render(template_name, ctx)
        .with_context(|| format!("テンプレートのレンダリングに失敗: {template_name}"))
}

/// ファイルを書き込む（親ディレクトリを自動作成）。
fn write_generated_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("ディレクトリを作成できません: {}", parent.display()))?;
    }
    fs::write(path, content)
        .with_context(|| format!("ファイルを書き込めません: {}", path.display()))?;
    Ok(())
}

/// migrations ディレクトリをスキャンして次のマイグレーション番号を返す。
fn next_migration_number(migrations_dir: &Path) -> u32 {
    if !migrations_dir.exists() {
        return 1;
    }

    let max = fs::read_dir(migrations_dir).ok().map_or(0, |entries| {
        entries
            .filter_map(std::result::Result::ok)
            .filter_map(|e| {
                e.file_name()
                    .to_str()
                    .and_then(|name| name.split('_').next())
                    .and_then(|num| num.parse::<u32>().ok())
            })
            .max()
            .unwrap_or(0)
    });

    max + 1
}

/// テンプレートディレクトリのデフォルトパスを返す。
pub fn default_template_dir() -> PathBuf {
    PathBuf::from(TEMPLATE_DIR)
}

/// 生成サマリーを文字列として返す。
pub fn format_generation_summary(config: &EventsConfig) -> String {
    let mut summary = String::new();
    let _ = writeln!(summary, "  イベント数: {}", config.events.len());
    let _ = writeln!(summary, "  言語: {}", config.language);
    let _ = writeln!(
        summary,
        "  ドメイン: {}.{}.{}",
        config.tier, config.domain, config.service_name
    );

    summary.push_str("\n  トピック一覧:\n");
    for event in &config.events {
        let topic = event.topic_name(&config.tier, &config.domain);
        let _ = writeln!(summary, "    - {topic}");
    }

    let consumer_count: usize = config.events.iter().map(|e| e.consumers.len()).sum();
    if consumer_count > 0 {
        let _ = writeln!(summary, "\n  Consumer 数: {consumer_count}");
        for event in &config.events {
            for consumer in &event.consumers {
                let _ = writeln!(
                    summary,
                    "    - {}.{} → {}",
                    consumer.domain, consumer.service_name, consumer.handler
                );
            }
        }
    }

    let has_outbox = config.events.iter().any(|e| e.outbox);
    let _ = writeln!(
        summary,
        "\n  Outbox: {}",
        if has_outbox { "あり" } else { "なし" }
    );

    summary
}

// テストコードでは unwrap() の使用を許可する
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_next_migration_number_empty() {
        let tmp = tempfile::tempdir().unwrap();
        assert_eq!(next_migration_number(&tmp.path().join("nonexistent")), 1);
    }

    #[test]
    fn test_next_migration_number_with_existing() {
        let tmp = tempfile::tempdir().unwrap();
        let migrations = tmp.path().join("migrations");
        fs::create_dir_all(&migrations).unwrap();
        fs::write(migrations.join("0001_init.up.sql"), "").unwrap();
        fs::write(migrations.join("0001_init.down.sql"), "").unwrap();
        fs::write(migrations.join("0002_add_users.up.sql"), "").unwrap();
        assert_eq!(next_migration_number(&migrations), 3);
    }

    #[test]
    fn test_format_generation_summary() {
        use crate::commands::generate_events::types::*;
        let config = EventsConfig {
            domain: "accounting".to_string(),
            tier: "business".to_string(),
            service_name: "domain-master".to_string(),
            language: "rust".to_string(),
            events: vec![EventDefinition {
                name: "master-item.created".to_string(),
                version: 1,
                description: "test".to_string(),
                partition_key: "item_id".to_string(),
                outbox: true,
                schema: EventSchema {
                    fields: vec![SchemaField {
                        name: "item_id".to_string(),
                        field_type: "string".to_string(),
                        number: 1,
                        description: String::new(),
                    }],
                },
                consumers: vec![ConsumerDefinition {
                    domain: "fa".to_string(),
                    service_name: "asset-manager".to_string(),
                    handler: "on_item_created".to_string(),
                }],
            }],
        };
        let summary = format_generation_summary(&config);
        assert!(summary.contains("イベント数: 1"));
        assert!(summary.contains("k1s0.business.accounting.master-item-created.v1"));
        assert!(summary.contains("Consumer 数: 1"));
        assert!(summary.contains("Outbox: あり"));
    }
}
