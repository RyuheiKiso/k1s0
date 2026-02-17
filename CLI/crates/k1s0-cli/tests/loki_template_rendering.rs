/// Loki ログ収集テンプレートのレンダリング統合テスト。
use std::fs;
use std::path::Path;

use k1s0_cli::template::context::TemplateContextBuilder;
use k1s0_cli::template::TemplateEngine;
use tempfile::TempDir;

fn template_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("templates")
}

fn render_loki(
    service_name: &str,
    tier: &str,
    server_port: u16,
) -> Option<(TempDir, Vec<String>)> {
    let tpl_dir = template_dir();
    let cat_dir = tpl_dir.join("loki");
    if !cat_dir.exists() {
        return None;
    }

    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let ctx = TemplateContextBuilder::new(service_name, tier, "go", "loki")
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
fn test_loki_file_list() {
    let Some((_, names)) = render_loki("order-api", "service", 8080) else {
        eprintln!("SKIP: loki テンプレートディレクトリが未作成");
        return;
    };

    assert!(
        names.iter().any(|n| n.contains("promtail-config.yaml")),
        "promtail-config.yaml missing. Generated: {:?}",
        names
    );
    assert!(
        names.iter().any(|n| n.contains("log-policy.yaml")),
        "log-policy.yaml missing. Generated: {:?}",
        names
    );
}

// =========================================================================
// Promtail Config テスト
// =========================================================================

#[test]
fn test_promtail_has_service_name() {
    let Some((tmp, _)) = render_loki("order-api", "service", 8080) else {
        eprintln!("SKIP: loki テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "promtail-config.yaml");
    assert!(
        content.contains("order-api"),
        "Promtail config should contain service name\n--- promtail-config.yaml ---\n{}",
        content
    );
}

#[test]
fn test_promtail_has_namespace() {
    let Some((tmp, _)) = render_loki("order-api", "service", 8080) else {
        eprintln!("SKIP: loki テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "promtail-config.yaml");
    assert!(
        content.contains("k1s0-service"),
        "Promtail config should contain namespace\n--- promtail-config.yaml ---\n{}",
        content
    );
}

#[test]
fn test_promtail_system_has_metrics_stage() {
    let Some((tmp, _)) = render_loki("auth-service", "system", 8080) else {
        eprintln!("SKIP: loki テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "promtail-config.yaml");
    assert!(
        content.contains("log_lines_total"),
        "System tier should have metrics stage\n--- promtail-config.yaml ---\n{}",
        content
    );
}

#[test]
fn test_promtail_service_no_metrics_stage() {
    let Some((tmp, _)) = render_loki("order-api", "service", 8080) else {
        eprintln!("SKIP: loki テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "promtail-config.yaml");
    assert!(
        !content.contains("log_lines_total"),
        "Service tier should NOT have metrics stage\n--- promtail-config.yaml ---\n{}",
        content
    );
}

// =========================================================================
// Log Policy テスト
// =========================================================================

#[test]
fn test_log_policy_system_retention() {
    let Some((tmp, _)) = render_loki("auth-service", "system", 8080) else {
        eprintln!("SKIP: loki テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "log-policy.yaml");
    assert!(
        content.contains("retention_days: 90"),
        "System tier should have 90 days retention\n--- log-policy.yaml ---\n{}",
        content
    );
}

#[test]
fn test_log_policy_business_retention() {
    let Some((tmp, _)) = render_loki("order-api", "business", 8080) else {
        eprintln!("SKIP: loki テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "log-policy.yaml");
    assert!(
        content.contains("retention_days: 60"),
        "Business tier should have 60 days retention\n--- log-policy.yaml ---\n{}",
        content
    );
}

#[test]
fn test_log_policy_service_retention() {
    let Some((tmp, _)) = render_loki("order-api", "service", 8080) else {
        eprintln!("SKIP: loki テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "log-policy.yaml");
    assert!(
        content.contains("retention_days: 30"),
        "Service tier should have 30 days retention\n--- log-policy.yaml ---\n{}",
        content
    );
}

// =========================================================================
// Tera 構文残留チェック
// =========================================================================

#[test]
fn test_loki_no_tera_syntax() {
    let Some((tmp, names)) = render_loki("order-api", "service", 8080) else {
        eprintln!("SKIP: loki テンプレートディレクトリが未作成");
        return;
    };

    for name in &names {
        let content = read_output(&tmp, name);
        assert!(!content.contains("{%"), "Tera block syntax found in {}", name);
        assert!(!content.contains("{#"), "Tera comment found in {}", name);
    }
}
