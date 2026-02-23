/// Consul テンプレートのレンダリング統合テスト。
use std::fs;
use std::path::Path;

use k1s0_cli::template::context::TemplateContextBuilder;
use k1s0_cli::template::TemplateEngine;
use tempfile::TempDir;

fn template_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("templates")
}

fn render_consul(service_name: &str, tier: &str) -> Option<(TempDir, Vec<String>)> {
    let tpl_dir = template_dir();
    let cat_dir = tpl_dir.join("consul");
    if !cat_dir.exists() {
        return None;
    }

    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let ctx = TemplateContextBuilder::new(service_name, tier, "go", "consul").build();

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
fn test_consul_file_list() {
    let Some((_, names)) = render_consul("order-api", "service") else {
        eprintln!("SKIP: consul テンプレートディレクトリが未作成");
        return;
    };

    assert!(
        names.iter().any(|n| n.contains("backend-config.tf")),
        "backend-config.tf missing. Generated: {names:?}"
    );
    assert!(
        names.iter().any(|n| n.contains("service-defaults.yaml")),
        "service-defaults.yaml missing. Generated: {names:?}"
    );
}

#[test]
fn test_consul_backend_has_service_name() {
    let Some((tmp, _)) = render_consul("order-api", "service") else {
        eprintln!("SKIP: consul テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "backend-config.tf");
    assert!(
        content.contains("order-api"),
        "Backend config should contain service name\n--- backend-config.tf ---\n{content}"
    );
}

#[test]
fn test_consul_backend_has_tier_path() {
    let Some((tmp, _)) = render_consul("order-api", "service") else {
        eprintln!("SKIP: consul テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "backend-config.tf");
    assert!(
        content.contains("k1s0/service/order-api"),
        "Backend config should contain tier-based path\n--- backend-config.tf ---\n{content}"
    );
}

#[test]
fn test_consul_service_defaults_has_service_name() {
    let Some((tmp, _)) = render_consul("order-api", "service") else {
        eprintln!("SKIP: consul テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "service-defaults.yaml");
    assert!(
        content.contains("order-api"),
        "Service defaults should contain service name\n--- service-defaults.yaml ---\n{content}"
    );
}

#[test]
fn test_consul_service_defaults_has_namespace() {
    let Some((tmp, _)) = render_consul("order-api", "service") else {
        eprintln!("SKIP: consul テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "service-defaults.yaml");
    assert!(
        content.contains("k1s0-service"),
        "Service defaults should contain namespace\n--- service-defaults.yaml ---\n{content}"
    );
}

#[test]
fn test_consul_no_tera_syntax() {
    let Some((tmp, names)) = render_consul("order-api", "service") else {
        eprintln!("SKIP: consul テンプレートディレクトリが未作成");
        return;
    };

    for name in &names {
        let content = read_output(&tmp, name);
        assert!(!content.contains("{%"), "Tera block syntax found in {name}");
        assert!(!content.contains("{#"), "Tera comment found in {name}");
    }
}
