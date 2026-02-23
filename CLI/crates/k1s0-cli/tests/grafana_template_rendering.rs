/// Grafana ダッシュボードテンプレートのレンダリング統合テスト。
///
/// CLI/templates/grafana/ テンプレートファイルを使用し、
/// テンプレートエンジンでレンダリングした結果を検証する。
use std::fs;
use std::path::Path;

use k1s0_cli::template::context::TemplateContextBuilder;
use k1s0_cli::template::TemplateEngine;
use tempfile::TempDir;

fn template_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("templates")
}

fn render_grafana(
    service_name: &str,
    tier: &str,
    server_port: u16,
) -> Option<(TempDir, Vec<String>)> {
    let tpl_dir = template_dir();
    let cat_dir = tpl_dir.join("grafana");
    if !cat_dir.exists() {
        return None;
    }

    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let ctx = TemplateContextBuilder::new(service_name, tier, "go", "grafana")
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
fn test_grafana_file_list() {
    let Some((_, names)) = render_grafana("order-api", "service", 8080) else {
        eprintln!("SKIP: grafana テンプレートディレクトリが未作成");
        return;
    };

    assert!(
        names.iter().any(|n| n.contains("dashboard-overview.yaml")),
        "dashboard-overview.yaml missing. Generated: {names:?}"
    );
    assert!(
        names
            .iter()
            .any(|n| n.contains("dashboard-service-detail.yaml")),
        "dashboard-service-detail.yaml missing. Generated: {names:?}"
    );
    assert!(
        names.iter().any(|n| n.contains("dashboard-slo.yaml")),
        "dashboard-slo.yaml missing. Generated: {names:?}"
    );
}

// =========================================================================
// Overview ダッシュボード テスト
// =========================================================================

#[test]
fn test_overview_has_service_name() {
    let Some((tmp, _)) = render_grafana("order-api", "service", 8080) else {
        eprintln!("SKIP: grafana テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "dashboard-overview.yaml");
    assert!(
        content.contains("order-api"),
        "Overview dashboard should contain service name\n--- dashboard-overview.yaml ---\n{content}"
    );
}

#[test]
fn test_overview_has_namespace() {
    let Some((tmp, _)) = render_grafana("order-api", "service", 8080) else {
        eprintln!("SKIP: grafana テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "dashboard-overview.yaml");
    assert!(
        content.contains("k1s0-service"),
        "Overview dashboard should contain namespace\n--- dashboard-overview.yaml ---\n{content}"
    );
}

#[test]
fn test_overview_has_grafana_dashboard_label() {
    let Some((tmp, _)) = render_grafana("order-api", "service", 8080) else {
        eprintln!("SKIP: grafana テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "dashboard-overview.yaml");
    assert!(
        content.contains("grafana_dashboard"),
        "Overview dashboard should have grafana_dashboard label\n--- dashboard-overview.yaml ---\n{content}"
    );
}

#[test]
fn test_overview_system_error_threshold() {
    let Some((tmp, _)) = render_grafana("auth-service", "system", 8080) else {
        eprintln!("SKIP: grafana テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "dashboard-overview.yaml");
    assert!(
        content.contains("0.01"),
        "System tier should have error threshold 0.01 (1%)\n--- dashboard-overview.yaml ---\n{content}"
    );
}

#[test]
fn test_overview_business_error_threshold() {
    let Some((tmp, _)) = render_grafana("order-api", "business", 8080) else {
        eprintln!("SKIP: grafana テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "dashboard-overview.yaml");
    assert!(
        content.contains("0.05"),
        "Business tier should have error threshold 0.05 (5%)\n--- dashboard-overview.yaml ---\n{content}"
    );
}

// =========================================================================
// SLO ダッシュボード テスト
// =========================================================================

#[test]
fn test_slo_has_service_name() {
    let Some((tmp, _)) = render_grafana("order-api", "service", 8080) else {
        eprintln!("SKIP: grafana テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "dashboard-slo.yaml");
    assert!(
        content.contains("order-api"),
        "SLO dashboard should contain service name\n--- dashboard-slo.yaml ---\n{content}"
    );
}

#[test]
fn test_slo_system_availability_target() {
    let Some((tmp, _)) = render_grafana("auth-service", "system", 8080) else {
        eprintln!("SKIP: grafana テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "dashboard-slo.yaml");
    assert!(
        content.contains("0.9995"),
        "System tier should have availability target 0.9995\n--- dashboard-slo.yaml ---\n{content}"
    );
}

#[test]
fn test_slo_business_availability_target() {
    let Some((tmp, _)) = render_grafana("order-api", "business", 8080) else {
        eprintln!("SKIP: grafana テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "dashboard-slo.yaml");
    assert!(
        content.contains("0.999"),
        "Business tier should have availability target 0.999\n--- dashboard-slo.yaml ---\n{content}"
    );
}

#[test]
fn test_slo_system_latency_target() {
    let Some((tmp, _)) = render_grafana("auth-service", "system", 8080) else {
        eprintln!("SKIP: grafana テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "dashboard-slo.yaml");
    assert!(
        content.contains("vector(0.2)"),
        "System tier should have P99 latency target 200ms\n--- dashboard-slo.yaml ---\n{content}"
    );
}

#[test]
fn test_slo_business_latency_target() {
    let Some((tmp, _)) = render_grafana("order-api", "business", 8080) else {
        eprintln!("SKIP: grafana テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "dashboard-slo.yaml");
    assert!(
        content.contains("vector(0.5)"),
        "Business tier should have P99 latency target 500ms\n--- dashboard-slo.yaml ---\n{content}"
    );
}

#[test]
fn test_slo_service_latency_target() {
    let Some((tmp, _)) = render_grafana("order-api", "service", 8080) else {
        eprintln!("SKIP: grafana テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "dashboard-slo.yaml");
    assert!(
        content.contains("vector(1.0)"),
        "Service tier should have P99 latency target 1s\n--- dashboard-slo.yaml ---\n{content}"
    );
}

// =========================================================================
// Service Detail ダッシュボード テスト
// =========================================================================

#[test]
fn test_service_detail_has_service_name() {
    let Some((tmp, _)) = render_grafana("order-api", "service", 8080) else {
        eprintln!("SKIP: grafana テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "dashboard-service-detail.yaml");
    assert!(
        content.contains("order-api"),
        "Service Detail dashboard should contain service name\n--- dashboard-service-detail.yaml ---\n{content}"
    );
}

#[test]
fn test_service_detail_has_server_port() {
    let Some((tmp, _)) = render_grafana("order-api", "service", 3000) else {
        eprintln!("SKIP: grafana テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "dashboard-service-detail.yaml");
    assert!(
        content.contains("3000"),
        "Service Detail dashboard should contain server_port\n--- dashboard-service-detail.yaml ---\n{content}"
    );
}

// =========================================================================
// Tera 構文残留チェック
// =========================================================================

#[test]
fn test_grafana_no_tera_syntax() {
    let Some((tmp, names)) = render_grafana("order-api", "service", 8080) else {
        eprintln!("SKIP: grafana テンプレートディレクトリが未作成");
        return;
    };

    for name in &names {
        let content = read_output(&tmp, name);
        assert!(!content.contains("{%"), "Tera block syntax found in {name}");
        assert!(!content.contains("{#"), "Tera comment found in {name}");
    }
}
