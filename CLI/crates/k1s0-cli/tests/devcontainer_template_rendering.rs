/// Dev Container テンプレートのレンダリング統合テスト。
///
/// CLI/templates/devcontainer/ テンプレートファイルを使用し、
/// テンプレートエンジンでレンダリングした結果が仕様書
/// (docs/テンプレート仕様-devcontainer.md) と一致することを検証する。
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

/// devcontainer テンプレートをレンダリングする。
///
/// devcontainer テンプレートはフラット構造（言語サブディレクトリなし）のため、
/// テンプレートディレクトリは templates/devcontainer/ を直接参照する。
/// テンプレートが存在しない場合は None を返す。
fn render_devcontainer(
    lang: &str,
    fw: &str,
    has_database: bool,
    database_type: &str,
    has_kafka: bool,
    has_redis: bool,
) -> Option<(TempDir, Vec<String>)> {
    let tpl_dir = template_dir();

    // devcontainer テンプレートディレクトリの存在チェック（フラット構造）
    let devcontainer_dir = tpl_dir.join("devcontainer");
    if !devcontainer_dir.exists() {
        return None;
    }

    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let mut builder =
        TemplateContextBuilder::new("order-api", "service", lang, "devcontainer").framework(fw);

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
// ファイル一覧テスト
// =========================================================================

#[test]
fn test_devcontainer_file_list() {
    let Some((_, names)) = render_devcontainer("go", "", false, "", false, false) else {
        eprintln!("SKIP: devcontainer テンプレートディレクトリが未作成");
        return;
    };

    // 3ファイルの存在確認
    assert!(
        names.iter().any(|n| n.contains("devcontainer.json")),
        "devcontainer.json missing"
    );
    assert!(
        names
            .iter()
            .any(|n| n.contains("docker-compose.extend.yaml")),
        "docker-compose.extend.yaml missing"
    );
    assert!(
        names.iter().any(|n| n.contains("post-create.sh")),
        "post-create.sh missing"
    );
    assert_eq!(names.len(), 3, "Expected 3 files, got: {names:?}");
}

// =========================================================================
// 言語別 features テスト
// =========================================================================

#[test]
fn test_devcontainer_go_features() {
    let Some((tmp, _)) = render_devcontainer("go", "", false, "", false, false) else {
        eprintln!("SKIP: devcontainer テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "devcontainer.json");
    assert!(
        content.contains("ghcr.io/devcontainers/features/go:1"),
        "Go feature missing in devcontainer.json"
    );
    assert!(
        content.contains("golang.go"),
        "Go extension missing in devcontainer.json"
    );
    assert!(
        content.contains("golangci-lint"),
        "golangci-lint setting missing in devcontainer.json"
    );
}

#[test]
fn test_devcontainer_rust_features() {
    let Some((tmp, _)) = render_devcontainer("rust", "", false, "", false, false) else {
        eprintln!("SKIP: devcontainer テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "devcontainer.json");
    assert!(
        content.contains("ghcr.io/devcontainers/features/rust:1"),
        "Rust feature missing in devcontainer.json"
    );
    assert!(
        content.contains("rust-lang.rust-analyzer"),
        "Rust extension missing in devcontainer.json"
    );
}

#[test]
fn test_devcontainer_react_features() {
    let Some((tmp, _)) = render_devcontainer("typescript", "react", false, "", false, false) else {
        eprintln!("SKIP: devcontainer テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "devcontainer.json");
    assert!(
        content.contains("ghcr.io/devcontainers/features/node:1"),
        "Node feature missing in devcontainer.json for React"
    );
    assert!(
        content.contains("dbaeumer.vscode-eslint"),
        "ESLint extension missing in devcontainer.json for React"
    );
    assert!(
        content.contains("esbenp.prettier-vscode"),
        "Prettier extension missing in devcontainer.json for React"
    );
}

#[test]
fn test_devcontainer_flutter_features() {
    let Some((tmp, _)) = render_devcontainer("dart", "flutter", false, "", false, false) else {
        eprintln!("SKIP: devcontainer テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "devcontainer.json");
    assert!(
        content.contains("Dart-Code.dart-code"),
        "Dart extension missing in devcontainer.json for Flutter"
    );
    assert!(
        content.contains("Dart-Code.flutter"),
        "Flutter extension missing in devcontainer.json for Flutter"
    );
}

// =========================================================================
// 共通 features テスト
// =========================================================================

#[test]
fn test_devcontainer_common_features() {
    let Some((tmp, _)) = render_devcontainer("go", "", false, "", false, false) else {
        eprintln!("SKIP: devcontainer テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "devcontainer.json");
    assert!(
        content.contains("ghcr.io/devcontainers/features/docker-in-docker:2"),
        "Docker-in-Docker feature missing in devcontainer.json"
    );
    assert!(
        content.contains("ghcr.io/devcontainers/features/kubectl-helm-minikube:1"),
        "kubectl feature missing in devcontainer.json"
    );
}

// =========================================================================
// forwardPorts テスト
// =========================================================================

#[test]
fn test_devcontainer_postgresql_port() {
    let Some((tmp, _)) = render_devcontainer("go", "", true, "postgresql", false, false) else {
        eprintln!("SKIP: devcontainer テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "devcontainer.json");
    assert!(
        content.contains("5432"),
        "PostgreSQL port 5432 missing in devcontainer.json when has_database=true, database_type=postgresql"
    );
}

#[test]
fn test_devcontainer_kafka_ports() {
    let Some((tmp, _)) = render_devcontainer("go", "", false, "", true, false) else {
        eprintln!("SKIP: devcontainer テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "devcontainer.json");
    assert!(
        content.contains("9092"),
        "Kafka port 9092 missing in devcontainer.json when has_kafka=true"
    );
    assert!(
        content.contains("8081"),
        "Schema Registry port 8081 missing in devcontainer.json when has_kafka=true"
    );
    assert!(
        content.contains("8090"),
        "Kafka UI port 8090 missing in devcontainer.json when has_kafka=true"
    );
}

// =========================================================================
// Docker Compose 拡張テスト
// =========================================================================

#[test]
fn test_devcontainer_compose_extends_postgres() {
    let Some((tmp, _)) = render_devcontainer("go", "", true, "postgresql", false, false) else {
        eprintln!("SKIP: devcontainer テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "docker-compose.extend.yaml");
    assert!(
        content.contains("depends_on"),
        "depends_on missing in docker-compose.extend.yaml when has_database=true"
    );
    assert!(
        content.contains("postgres"),
        "postgres missing in depends_on when database_type=postgresql"
    );
}

#[test]
fn test_devcontainer_compose_extends_kafka() {
    let Some((tmp, _)) = render_devcontainer("go", "", false, "", true, false) else {
        eprintln!("SKIP: devcontainer テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "docker-compose.extend.yaml");
    assert!(
        content.contains("depends_on"),
        "depends_on missing in docker-compose.extend.yaml when has_kafka=true"
    );
    assert!(
        content.contains("kafka"),
        "kafka missing in depends_on when has_kafka=true"
    );
}

// =========================================================================
// post-create.sh テスト
// =========================================================================

#[test]
fn test_devcontainer_post_create_go() {
    let Some((tmp, _)) = render_devcontainer("go", "", false, "", false, false) else {
        eprintln!("SKIP: devcontainer テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "post-create.sh");
    assert!(
        content.contains("goimports"),
        "goimports missing in post-create.sh for Go"
    );
    assert!(
        content.contains("golangci-lint"),
        "golangci-lint missing in post-create.sh for Go"
    );
    assert!(
        content.contains("buf"),
        "buf missing in post-create.sh for Go"
    );
}

#[test]
fn test_devcontainer_post_create_rust() {
    let Some((tmp, _)) = render_devcontainer("rust", "", false, "", false, false) else {
        eprintln!("SKIP: devcontainer テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "post-create.sh");
    assert!(
        content.contains("clippy"),
        "clippy missing in post-create.sh for Rust"
    );
    assert!(
        content.contains("rustfmt"),
        "rustfmt missing in post-create.sh for Rust"
    );
}

#[test]
fn test_devcontainer_post_create_flutter() {
    let Some((tmp, _)) = render_devcontainer("dart", "flutter", false, "", false, false) else {
        eprintln!("SKIP: devcontainer テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "post-create.sh");
    assert!(
        content.contains("flutter"),
        "Flutter SDK install missing in post-create.sh for Flutter"
    );
    assert!(
        content.contains("/opt/flutter"),
        "Flutter SDK path missing in post-create.sh for Flutter"
    );
}

// =========================================================================
// Tera 構文残存チェック
// =========================================================================

#[test]
fn test_devcontainer_no_tera_syntax() {
    let Some((tmp, names)) = render_devcontainer("go", "", true, "postgresql", true, true) else {
        eprintln!("SKIP: devcontainer テンプレートディレクトリが未作成");
        return;
    };

    for name in &names {
        let content = read_output(&tmp, name);
        assert!(!content.contains("{%"), "Tera syntax {{%% found in {name}");
        assert!(!content.contains("{#"), "Tera comment {{# found in {name}");
    }
}
