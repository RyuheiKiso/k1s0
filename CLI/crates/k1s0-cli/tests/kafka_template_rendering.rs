/// Kafka テンプレートのレンダリング統合テスト。
use std::fs;
use std::path::Path;

use k1s0_cli::template::context::TemplateContextBuilder;
use k1s0_cli::template::TemplateEngine;
use tempfile::TempDir;

fn template_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("templates")
}

fn render_kafka(
    service_name: &str,
    tier: &str,
) -> Option<(TempDir, Vec<String>)> {
    let tpl_dir = template_dir();
    let cat_dir = tpl_dir.join("kafka");
    if !cat_dir.exists() {
        return None;
    }

    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let ctx = TemplateContextBuilder::new(service_name, tier, "go", "kafka")
        .with_kafka()
        .build();

    let mut engine = TemplateEngine::new(&tpl_dir).unwrap();
    let generated = engine.render_to_dir(&ctx, &output_dir).unwrap();

    let names: Vec<String> = generated
        .iter()
        .map(|p| {
            p.strip_prefix(&output_dir)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect();

    Some((tmp, names))
}

fn read_output(tmp: &TempDir, path: &str) -> String {
    fs::read_to_string(tmp.path().join("output").join(path)).unwrap()
}

#[test]
fn test_kafka_file_list() {
    let Some((_, names)) = render_kafka("order-api", "service") else {
        eprintln!("SKIP: kafka テンプレートディレクトリが未作成");
        return;
    };

    assert!(
        names.iter().any(|n| n.contains("kafka-topic.yaml")),
        "kafka-topic.yaml missing. Generated: {:?}",
        names
    );
    assert!(
        names.iter().any(|n| n.contains("kafka-topic-dlq.yaml")),
        "kafka-topic-dlq.yaml missing. Generated: {:?}",
        names
    );
}

#[test]
fn test_kafka_topic_has_service_name() {
    let Some((tmp, _)) = render_kafka("order-api", "service") else {
        eprintln!("SKIP: kafka テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "kafka-topic.yaml");
    assert!(
        content.contains("order_api"),
        "Kafka topic should contain service_name_snake\n--- kafka-topic.yaml ---\n{}",
        content
    );
}

#[test]
fn test_kafka_topic_naming_convention() {
    let Some((tmp, _)) = render_kafka("order-api", "service") else {
        eprintln!("SKIP: kafka テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "kafka-topic.yaml");
    assert!(
        content.contains("k1s0.service.order_api.events.v1"),
        "Kafka topic should follow naming convention\n--- kafka-topic.yaml ---\n{}",
        content
    );
}

#[test]
fn test_kafka_topic_system_partitions() {
    let Some((tmp, _)) = render_kafka("auth-service", "system") else {
        eprintln!("SKIP: kafka テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "kafka-topic.yaml");
    assert!(
        content.contains("partitions: 6"),
        "System tier should have 6 partitions\n--- kafka-topic.yaml ---\n{}",
        content
    );
}

#[test]
fn test_kafka_topic_service_partitions() {
    let Some((tmp, _)) = render_kafka("order-api", "service") else {
        eprintln!("SKIP: kafka テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "kafka-topic.yaml");
    assert!(
        content.contains("partitions: 3"),
        "Service tier should have 3 partitions\n--- kafka-topic.yaml ---\n{}",
        content
    );
}

#[test]
fn test_kafka_dlq_topic_naming() {
    let Some((tmp, _)) = render_kafka("order-api", "service") else {
        eprintln!("SKIP: kafka テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "kafka-topic-dlq.yaml");
    assert!(
        content.contains("k1s0.service.order_api.events.v1.dlq"),
        "DLQ topic should have .dlq suffix\n--- kafka-topic-dlq.yaml ---\n{}",
        content
    );
}

#[test]
fn test_kafka_dlq_has_dlq_label() {
    let Some((tmp, _)) = render_kafka("order-api", "service") else {
        eprintln!("SKIP: kafka テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "kafka-topic-dlq.yaml");
    assert!(
        content.contains("dlq: \"true\""),
        "DLQ topic should have dlq label\n--- kafka-topic-dlq.yaml ---\n{}",
        content
    );
}

#[test]
fn test_kafka_no_tera_syntax() {
    let Some((tmp, names)) = render_kafka("order-api", "service") else {
        eprintln!("SKIP: kafka テンプレートディレクトリが未作成");
        return;
    };

    for name in &names {
        let content = read_output(&tmp, name);
        assert!(!content.contains("{%"), "Tera block syntax found in {}", name);
        assert!(!content.contains("{#"), "Tera comment found in {}", name);
    }
}
