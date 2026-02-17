/// OpenTelemetry テンプレートのレンダリング統合テスト。
use std::fs;
use std::path::Path;

use k1s0_cli::template::context::TemplateContextBuilder;
use k1s0_cli::template::TemplateEngine;
use tempfile::TempDir;

fn template_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("templates")
}

fn render_opentelemetry(
    service_name: &str,
    tier: &str,
    server_port: u16,
) -> Option<(TempDir, Vec<String>)> {
    let tpl_dir = template_dir();
    let cat_dir = tpl_dir.join("opentelemetry");
    if !cat_dir.exists() {
        return None;
    }

    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let ctx = TemplateContextBuilder::new(service_name, tier, "go", "opentelemetry")
        .server_port(server_port)
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
fn test_opentelemetry_file_list() {
    let Some((_, names)) = render_opentelemetry("order-api", "service", 8080) else {
        eprintln!("SKIP: opentelemetry テンプレートディレクトリが未作成");
        return;
    };

    assert!(
        names.iter().any(|n| n.contains("collector-config.yaml")),
        "collector-config.yaml missing. Generated: {:?}",
        names
    );
    assert!(
        names.iter().any(|n| n.contains("instrumentation.yaml")),
        "instrumentation.yaml missing. Generated: {:?}",
        names
    );
}

// =========================================================================
// Collector Config テスト
// =========================================================================

#[test]
fn test_collector_has_service_name() {
    let Some((tmp, _)) = render_opentelemetry("order-api", "service", 8080) else {
        eprintln!("SKIP: opentelemetry テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "collector-config.yaml");
    assert!(
        content.contains("order-api"),
        "Collector config should contain service name\n--- collector-config.yaml ---\n{}",
        content
    );
}

#[test]
fn test_collector_has_namespace() {
    let Some((tmp, _)) = render_opentelemetry("order-api", "service", 8080) else {
        eprintln!("SKIP: opentelemetry テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "collector-config.yaml");
    assert!(
        content.contains("k1s0-service"),
        "Collector config should contain namespace\n--- collector-config.yaml ---\n{}",
        content
    );
}

#[test]
fn test_collector_system_batch_timeout() {
    let Some((tmp, _)) = render_opentelemetry("auth-service", "system", 8080) else {
        eprintln!("SKIP: opentelemetry テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "collector-config.yaml");
    assert!(
        content.contains("timeout: 5s"),
        "System tier should have batch timeout 5s\n--- collector-config.yaml ---\n{}",
        content
    );
}

#[test]
fn test_collector_service_batch_timeout() {
    let Some((tmp, _)) = render_opentelemetry("order-api", "service", 8080) else {
        eprintln!("SKIP: opentelemetry テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "collector-config.yaml");
    assert!(
        content.contains("timeout: 10s"),
        "Service tier should have batch timeout 10s\n--- collector-config.yaml ---\n{}",
        content
    );
}

// =========================================================================
// Instrumentation テスト
// =========================================================================

#[test]
fn test_instrumentation_system_sampler() {
    let Some((tmp, _)) = render_opentelemetry("auth-service", "system", 8080) else {
        eprintln!("SKIP: opentelemetry テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "instrumentation.yaml");
    assert!(
        content.contains("\"1.0\""),
        "System tier should have sampler ratio 1.0\n--- instrumentation.yaml ---\n{}",
        content
    );
}

#[test]
fn test_instrumentation_business_sampler() {
    let Some((tmp, _)) = render_opentelemetry("order-api", "business", 8080) else {
        eprintln!("SKIP: opentelemetry テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "instrumentation.yaml");
    assert!(
        content.contains("\"0.5\""),
        "Business tier should have sampler ratio 0.5\n--- instrumentation.yaml ---\n{}",
        content
    );
}

#[test]
fn test_instrumentation_service_sampler() {
    let Some((tmp, _)) = render_opentelemetry("order-api", "service", 8080) else {
        eprintln!("SKIP: opentelemetry テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "instrumentation.yaml");
    assert!(
        content.contains("\"0.1\""),
        "Service tier should have sampler ratio 0.1\n--- instrumentation.yaml ---\n{}",
        content
    );
}

// =========================================================================
// Tera 構文残留チェック
// =========================================================================

#[test]
fn test_opentelemetry_no_tera_syntax() {
    let Some((tmp, names)) = render_opentelemetry("order-api", "service", 8080) else {
        eprintln!("SKIP: opentelemetry テンプレートディレクトリが未作成");
        return;
    };

    for name in &names {
        let content = read_output(&tmp, name);
        assert!(!content.contains("{%"), "Tera block syntax found in {}", name);
        assert!(!content.contains("{#"), "Tera comment found in {}", name);
    }
}
