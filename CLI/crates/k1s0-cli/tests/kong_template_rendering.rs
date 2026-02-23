/// Kong API Gateway テンプレートのレンダリング統合テスト。
///
/// CLI/templates/kong/ テンプレートファイルを使用し、
/// テンプレートエンジンでレンダリングした結果を検証する。
use std::fs;
use std::path::Path;

use k1s0_cli::template::context::TemplateContextBuilder;
use k1s0_cli::template::TemplateEngine;
use tempfile::TempDir;

fn template_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("templates")
}

fn render_kong(
    service_name: &str,
    tier: &str,
    api_style: &str,
    server_port: u16,
    grpc_port: u16,
) -> Option<(TempDir, Vec<String>)> {
    let tpl_dir = template_dir();
    let kong_dir = tpl_dir.join("kong");
    if !kong_dir.exists() {
        return None;
    }

    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let ctx = TemplateContextBuilder::new(service_name, tier, "go", "kong")
        .api_style(api_style)
        .server_port(server_port)
        .grpc_port(grpc_port)
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

// =========================================================================
// ファイル一覧テスト
// =========================================================================

#[test]
fn test_kong_file_list() {
    let Some((_, names)) = render_kong("order-api", "service", "rest", 8080, 50051) else {
        eprintln!("SKIP: kong テンプレートディレクトリが未作成");
        return;
    };

    assert!(
        names.iter().any(|n| n.contains("kong-service.yaml")),
        "kong-service.yaml missing. Generated: {names:?}"
    );
    assert!(
        names.iter().any(|n| n.contains("kong-plugins.yaml")),
        "kong-plugins.yaml missing. Generated: {names:?}"
    );
}

// =========================================================================
// Kong Service テスト
// =========================================================================

#[test]
fn test_kong_service_has_service_name() {
    let Some((tmp, _)) = render_kong("order-api", "service", "rest", 8080, 50051) else {
        eprintln!("SKIP: kong テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "kong-service.yaml");
    assert!(
        content.contains("order-api"),
        "Kong service should contain service name\n--- kong-service.yaml ---\n{content}"
    );
}

#[test]
fn test_kong_service_has_namespace() {
    let Some((tmp, _)) = render_kong("order-api", "service", "rest", 8080, 50051) else {
        eprintln!("SKIP: kong テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "kong-service.yaml");
    assert!(
        content.contains("k1s0-service"),
        "Kong service should contain namespace\n--- kong-service.yaml ---\n{content}"
    );
}

#[test]
fn test_kong_service_grpc_port() {
    let Some((tmp, _)) = render_kong("auth-service", "system", "grpc", 8080, 50051) else {
        eprintln!("SKIP: kong テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "kong-service.yaml");
    assert!(
        content.contains("50051"),
        "Kong service should contain gRPC port for grpc api_style\n--- kong-service.yaml ---\n{content}"
    );
}

#[test]
fn test_kong_service_no_grpc_port_for_rest() {
    let Some((tmp, _)) = render_kong("order-api", "service", "rest", 8080, 50051) else {
        eprintln!("SKIP: kong テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "kong-service.yaml");
    assert!(
        !content.contains("50051"),
        "Kong service should NOT contain gRPC port for REST-only\n--- kong-service.yaml ---\n{content}"
    );
}

// =========================================================================
// Kong Plugins テスト
// =========================================================================

#[test]
fn test_kong_plugins_rate_limit_system() {
    let Some((tmp, _)) = render_kong("auth-service", "system", "grpc", 8080, 50051) else {
        eprintln!("SKIP: kong テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "kong-plugins.yaml");
    assert!(
        content.contains("minute: 120"),
        "System tier should have rate limit minute: 120\n--- kong-plugins.yaml ---\n{content}"
    );
}

#[test]
fn test_kong_plugins_rate_limit_service() {
    let Some((tmp, _)) = render_kong("order-api", "service", "rest", 8080, 50051) else {
        eprintln!("SKIP: kong テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "kong-plugins.yaml");
    assert!(
        content.contains("minute: 30"),
        "Service tier should have rate limit minute: 30\n--- kong-plugins.yaml ---\n{content}"
    );
}

#[test]
fn test_kong_plugins_has_cors() {
    let Some((tmp, _)) = render_kong("order-api", "service", "rest", 8080, 50051) else {
        eprintln!("SKIP: kong テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "kong-plugins.yaml");
    assert!(
        content.contains("plugin: cors"),
        "Kong plugins should include CORS plugin\n--- kong-plugins.yaml ---\n{content}"
    );
}

#[test]
fn test_kong_plugins_has_jwt() {
    let Some((tmp, _)) = render_kong("order-api", "service", "rest", 8080, 50051) else {
        eprintln!("SKIP: kong テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "kong-plugins.yaml");
    assert!(
        content.contains("plugin: jwt"),
        "Kong plugins should include JWT plugin\n--- kong-plugins.yaml ---\n{content}"
    );
}

// =========================================================================
// Tera 構文残留チェック
// =========================================================================

#[test]
fn test_kong_no_tera_syntax() {
    let Some((tmp, names)) = render_kong("order-api", "service", "rest", 8080, 50051) else {
        eprintln!("SKIP: kong テンプレートディレクトリが未作成");
        return;
    };

    for name in &names {
        let content = read_output(&tmp, name);
        assert!(!content.contains("{%"), "Tera syntax found in {name}");
        assert!(!content.contains("{#"), "Tera comment found in {name}");
    }
}
