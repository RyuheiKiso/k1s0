/// Docker Compose テンプレートのレンダリング統合テスト。
///
/// CLI/templates/docker-compose/ テンプレートファイルを使用し、
/// テンプレートエンジンでレンダリングした結果が仕様書と一致することを検証する。
use std::fs;
use std::path::Path;

use k1s0_cli::template::context::TemplateContextBuilder;
use k1s0_cli::template::TemplateEngine;
use tempfile::TempDir;

// =========================================================================
// ヘルパー関数
// =========================================================================

fn template_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("templates")
}

/// docker-compose テンプレートをレンダリングする。
///
/// docker-compose テンプレートはフラット構造（言語サブディレクトリなし）のため、
/// templates/docker-compose/ を直接参照する。
/// テンプレートディレクトリが未作成の場合は None を返す。
fn render_docker_compose() -> Option<(TempDir, Vec<String>)> {
    render_docker_compose_with(
        "go",
        8082,
        true,
        "postgresql",
        true,
        true,
    )
}

/// docker-compose テンプレートをカスタムパラメータでレンダリングする。
fn render_docker_compose_with(
    server_lang: &str,
    port: u16,
    has_database: bool,
    database_type: &str,
    has_kafka: bool,
    has_redis: bool,
) -> Option<(TempDir, Vec<String>)> {
    let tpl_dir = template_dir();

    // docker-compose テンプレートディレクトリの存在チェック（フラット構造）
    let dc_dir = tpl_dir.join("docker-compose");
    if !dc_dir.exists() {
        return None;
    }

    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let mut builder =
        TemplateContextBuilder::new("order-api", "service", "go", "docker-compose")
            .server_language(server_lang)
            .server_port(port);

    if has_database {
        builder = builder.with_database(database_type);
    }
    if has_kafka {
        builder = builder.with_kafka();
    }
    if has_redis {
        builder = builder.with_redis();
    }

    let ctx = builder.build();
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

// =========================================================================
// Docker Compose テスト
// =========================================================================

/// 2ファイルの存在確認
#[test]
fn test_docker_compose_file_list() {
    let Some((_, names)) = render_docker_compose() else {
        eprintln!("SKIP: docker-compose テンプレートディレクトリが未作成");
        return;
    };

    assert!(
        names.iter().any(|n| n.contains("docker-compose.yaml")),
        "docker-compose.yaml missing. Generated: {:?}",
        names
    );
    assert!(
        names
            .iter()
            .any(|n| n.contains("docker-compose.override.yaml.example")),
        "docker-compose.override.yaml.example missing. Generated: {:?}",
        names
    );
    assert_eq!(names.len(), 2, "Expected exactly 2 files, got: {:?}", names);
}

/// has_database=true, database_type=postgresql で postgres サービスが含まれる
#[test]
fn test_docker_compose_postgresql_service() {
    let Some((tmp, _)) =
        render_docker_compose_with("go", 8082, true, "postgresql", false, false)
    else {
        eprintln!("SKIP: docker-compose テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "docker-compose.yaml");
    assert!(
        content.contains("postgres:"),
        "PostgreSQL サービスが docker-compose.yaml に含まれるべき"
    );
    assert!(
        content.contains("image: postgres:17"),
        "postgres:17 イメージが指定されるべき"
    );
    assert!(
        !content.contains("mysql:"),
        "MySQL サービスは含まれるべきでない"
    );
}

/// has_database=true, database_type=mysql で mysql サービスが含まれる
#[test]
fn test_docker_compose_mysql_service() {
    let Some((tmp, _)) =
        render_docker_compose_with("go", 8082, true, "mysql", false, false)
    else {
        eprintln!("SKIP: docker-compose テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "docker-compose.yaml");
    assert!(
        content.contains("mysql:"),
        "MySQL サービスが docker-compose.yaml に含まれるべき"
    );
    assert!(
        content.contains("image: mysql:8.4"),
        "mysql:8.4 イメージが指定されるべき"
    );
    assert!(
        !content.contains("postgres:"),
        "PostgreSQL サービスは含まれるべきでない"
    );
}

/// has_redis=true で redis サービスが含まれる
#[test]
fn test_docker_compose_redis_service() {
    let Some((tmp, _)) =
        render_docker_compose_with("go", 8082, false, "", false, true)
    else {
        eprintln!("SKIP: docker-compose テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "docker-compose.yaml");
    assert!(
        content.contains("redis:"),
        "Redis サービスが docker-compose.yaml に含まれるべき"
    );
    assert!(
        content.contains("image: redis:7"),
        "redis:7 イメージが指定されるべき"
    );
}

/// has_kafka=true で kafka, kafka-ui, schema-registry が含まれる
#[test]
fn test_docker_compose_kafka_services() {
    let Some((tmp, _)) =
        render_docker_compose_with("go", 8082, false, "", true, false)
    else {
        eprintln!("SKIP: docker-compose テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "docker-compose.yaml");
    assert!(
        content.contains("kafka:"),
        "Kafka サービスが docker-compose.yaml に含まれるべき"
    );
    assert!(
        content.contains("kafka-ui:"),
        "Kafka UI サービスが docker-compose.yaml に含まれるべき"
    );
    assert!(
        content.contains("schema-registry:"),
        "Schema Registry サービスが docker-compose.yaml に含まれるべき"
    );
}

/// has_kafka=false で kafka が含まれない
#[test]
fn test_docker_compose_no_kafka() {
    let Some((tmp, _)) =
        render_docker_compose_with("go", 8082, false, "", false, false)
    else {
        eprintln!("SKIP: docker-compose テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "docker-compose.yaml");
    assert!(
        !content.contains("kafka:"),
        "Kafka サービスは含まれるべきでない (has_kafka=false)"
    );
    assert!(
        !content.contains("kafka-ui:"),
        "Kafka UI サービスは含まれるべきでない (has_kafka=false)"
    );
    assert!(
        !content.contains("schema-registry:"),
        "Schema Registry サービスは含まれるべきでない (has_kafka=false)"
    );
}

/// keycloak は常に含まれる
#[test]
fn test_docker_compose_keycloak_always() {
    let Some((tmp, _)) =
        render_docker_compose_with("go", 8082, false, "", false, false)
    else {
        eprintln!("SKIP: docker-compose テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "docker-compose.yaml");
    assert!(
        content.contains("keycloak:"),
        "Keycloak サービスは常に含まれるべき"
    );
    assert!(
        content.contains("quay.io/keycloak/keycloak:26.0"),
        "Keycloak イメージが指定されるべき"
    );
}

/// jaeger, prometheus, grafana, loki は常に含まれる
#[test]
fn test_docker_compose_observability_always() {
    let Some((tmp, _)) =
        render_docker_compose_with("go", 8082, false, "", false, false)
    else {
        eprintln!("SKIP: docker-compose テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "docker-compose.yaml");
    assert!(
        content.contains("jaeger:"),
        "Jaeger サービスは常に含まれるべき"
    );
    assert!(
        content.contains("prometheus:"),
        "Prometheus サービスは常に含まれるべき"
    );
    assert!(
        content.contains("grafana:"),
        "Grafana サービスは常に含まれるべき"
    );
    assert!(
        content.contains("loki:"),
        "Loki サービスは常に含まれるべき"
    );
}

/// server_language=go でビルドコンテキストが Go パス
#[test]
fn test_docker_compose_override_go_context() {
    let Some((tmp, _)) =
        render_docker_compose_with("go", 8082, true, "postgresql", false, false)
    else {
        eprintln!("SKIP: docker-compose テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "docker-compose.override.yaml.example");
    assert!(
        content.contains("./regions/service/server/go/order-api"),
        "Go ビルドコンテキストが含まれるべき\n--- content ---\n{}",
        content
    );
}

/// server_language=rust でビルドコンテキストが Rust パス
#[test]
fn test_docker_compose_override_rust_context() {
    let Some((tmp, _)) =
        render_docker_compose_with("rust", 8082, true, "postgresql", false, false)
    else {
        eprintln!("SKIP: docker-compose テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "docker-compose.override.yaml.example");
    assert!(
        content.contains("./regions/service/server/rust/order-api"),
        "Rust ビルドコンテキストが含まれるべき\n--- content ---\n{}",
        content
    );
}

/// server_port=8082 でポートマッピングに反映
#[test]
fn test_docker_compose_override_port() {
    let Some((tmp, _)) =
        render_docker_compose_with("go", 8082, false, "", false, false)
    else {
        eprintln!("SKIP: docker-compose テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "docker-compose.override.yaml.example");
    assert!(
        content.contains("\"8082:8080\""),
        "ポートマッピング 8082:8080 が含まれるべき\n--- content ---\n{}",
        content
    );
}

/// 全ファイルで Tera {%  %}, {#  #} が残っていないこと
#[test]
fn test_docker_compose_no_tera_syntax() {
    let Some((tmp, names)) = render_docker_compose() else {
        eprintln!("SKIP: docker-compose テンプレートディレクトリが未作成");
        return;
    };

    for name in &names {
        let content = read_output(&tmp, name);
        assert!(
            !content.contains("{%"),
            "Tera syntax {{%% found in {}",
            name
        );
        assert!(
            !content.contains("{#"),
            "Tera comment {{# found in {}",
            name
        );
    }
}
