/// Flagger テンプレートのレンダリング統合テスト。
use std::fs;
use std::path::Path;

use k1s0_cli::template::context::TemplateContextBuilder;
use k1s0_cli::template::TemplateEngine;
use tempfile::TempDir;

fn template_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("templates")
}

fn render_flagger(
    service_name: &str,
    tier: &str,
    server_port: u16,
) -> Option<(TempDir, Vec<String>)> {
    let tpl_dir = template_dir();
    let cat_dir = tpl_dir.join("flagger");
    if !cat_dir.exists() {
        return None;
    }

    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let ctx = TemplateContextBuilder::new(service_name, tier, "go", "flagger")
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

#[test]
fn test_flagger_file_list() {
    let Some((_, names)) = render_flagger("order-api", "service", 8080) else {
        eprintln!("SKIP: flagger テンプレートディレクトリが未作成");
        return;
    };

    assert!(
        names.iter().any(|n| n.contains("canary.yaml")),
        "canary.yaml missing. Generated: {names:?}"
    );
    assert!(
        names.iter().any(|n| n.contains("metric-template.yaml")),
        "metric-template.yaml missing. Generated: {names:?}"
    );
}

#[test]
fn test_canary_has_service_name() {
    let Some((tmp, _)) = render_flagger("order-api", "service", 8080) else {
        eprintln!("SKIP: flagger テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "canary.yaml");
    assert!(
        content.contains("order-api"),
        "Canary should contain service name\n--- canary.yaml ---\n{content}"
    );
}

#[test]
fn test_canary_has_namespace() {
    let Some((tmp, _)) = render_flagger("order-api", "service", 8080) else {
        eprintln!("SKIP: flagger テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "canary.yaml");
    assert!(
        content.contains("k1s0-service"),
        "Canary should contain namespace\n--- canary.yaml ---\n{content}"
    );
}

#[test]
fn test_canary_has_server_port() {
    let Some((tmp, _)) = render_flagger("order-api", "service", 3000) else {
        eprintln!("SKIP: flagger テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "canary.yaml");
    assert!(
        content.contains("3000"),
        "Canary should contain server_port\n--- canary.yaml ---\n{content}"
    );
}

#[test]
fn test_canary_system_max_weight() {
    let Some((tmp, _)) = render_flagger("auth-service", "system", 8080) else {
        eprintln!("SKIP: flagger テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "canary.yaml");
    assert!(
        content.contains("maxWeight: 30"),
        "System tier should have maxWeight 30\n--- canary.yaml ---\n{content}"
    );
}

#[test]
fn test_canary_business_max_weight() {
    let Some((tmp, _)) = render_flagger("order-api", "business", 8080) else {
        eprintln!("SKIP: flagger テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "canary.yaml");
    assert!(
        content.contains("maxWeight: 50"),
        "Business tier should have maxWeight 50\n--- canary.yaml ---\n{content}"
    );
}

#[test]
fn test_canary_service_max_weight() {
    let Some((tmp, _)) = render_flagger("order-api", "service", 8080) else {
        eprintln!("SKIP: flagger テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "canary.yaml");
    assert!(
        content.contains("maxWeight: 70"),
        "Service tier should have maxWeight 70\n--- canary.yaml ---\n{content}"
    );
}

#[test]
fn test_canary_system_duration_threshold() {
    let Some((tmp, _)) = render_flagger("auth-service", "system", 8080) else {
        eprintln!("SKIP: flagger テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "canary.yaml");
    assert!(
        content.contains("max: 500"),
        "System tier should have duration threshold 500ms\n--- canary.yaml ---\n{content}"
    );
}

#[test]
fn test_metric_template_has_service_name() {
    let Some((tmp, _)) = render_flagger("order-api", "service", 8080) else {
        eprintln!("SKIP: flagger テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "metric-template.yaml");
    assert!(
        content.contains("order-api"),
        "MetricTemplate should contain service name\n--- metric-template.yaml ---\n{content}"
    );
}

#[test]
fn test_flagger_no_tera_syntax() {
    let Some((tmp, names)) = render_flagger("order-api", "service", 8080) else {
        eprintln!("SKIP: flagger テンプレートディレクトリが未作成");
        return;
    };

    for name in &names {
        let content = read_output(&tmp, name);
        assert!(!content.contains("{%"), "Tera block syntax found in {name}");
        assert!(!content.contains("{#"), "Tera comment found in {name}");
    }
}
