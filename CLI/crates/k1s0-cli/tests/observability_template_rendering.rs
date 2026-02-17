/// 可観測性テンプレートのレンダリング統合テスト。
///
/// CLI/templates/observability/ テンプレートファイルを使用し、
/// テンプレートエンジンでレンダリングした結果を検証する。
use std::fs;
use std::path::Path;

use k1s0_cli::template::context::TemplateContextBuilder;
use k1s0_cli::template::TemplateEngine;
use tempfile::TempDir;

fn template_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("templates")
}

fn render_observability(
    service_name: &str,
    tier: &str,
    server_port: u16,
) -> Option<(TempDir, Vec<String>)> {
    let tpl_dir = template_dir();
    let obs_dir = tpl_dir.join("observability");
    if !obs_dir.exists() {
        return None;
    }

    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let ctx = TemplateContextBuilder::new(service_name, tier, "go", "observability")
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
fn test_observability_file_list() {
    let Some((_, names)) = render_observability("order-api", "service", 8080) else {
        eprintln!("SKIP: observability テンプレートディレクトリが未作成");
        return;
    };

    assert!(
        names.iter().any(|n| n.contains("servicemonitor.yaml")),
        "servicemonitor.yaml missing. Generated: {:?}",
        names
    );
    assert!(
        names.iter().any(|n| n.contains("alerts.yaml")),
        "alerts.yaml missing. Generated: {:?}",
        names
    );
}

// =========================================================================
// ServiceMonitor テスト
// =========================================================================

#[test]
fn test_servicemonitor_has_service_name() {
    let Some((tmp, _)) = render_observability("order-api", "service", 8080) else {
        eprintln!("SKIP: observability テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "servicemonitor.yaml");
    assert!(
        content.contains("order-api"),
        "ServiceMonitor should contain service name\n--- servicemonitor.yaml ---\n{}",
        content
    );
}

#[test]
fn test_servicemonitor_system_interval() {
    let Some((tmp, _)) = render_observability("auth-service", "system", 8080) else {
        eprintln!("SKIP: observability テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "servicemonitor.yaml");
    assert!(
        content.contains("interval: 15s"),
        "System tier should have interval: 15s\n--- servicemonitor.yaml ---\n{}",
        content
    );
}

#[test]
fn test_servicemonitor_business_interval() {
    let Some((tmp, _)) = render_observability("order-api", "business", 8080) else {
        eprintln!("SKIP: observability テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "servicemonitor.yaml");
    assert!(
        content.contains("interval: 30s"),
        "Business tier should have interval: 30s\n--- servicemonitor.yaml ---\n{}",
        content
    );
}

#[test]
fn test_servicemonitor_service_interval() {
    let Some((tmp, _)) = render_observability("order-api", "service", 8080) else {
        eprintln!("SKIP: observability テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "servicemonitor.yaml");
    assert!(
        content.contains("interval: 60s"),
        "Service tier should have interval: 60s\n--- servicemonitor.yaml ---\n{}",
        content
    );
}

#[test]
fn test_servicemonitor_has_namespace() {
    let Some((tmp, _)) = render_observability("order-api", "service", 8080) else {
        eprintln!("SKIP: observability テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "servicemonitor.yaml");
    assert!(
        content.contains("k1s0-service"),
        "ServiceMonitor should contain namespace\n--- servicemonitor.yaml ---\n{}",
        content
    );
}

// =========================================================================
// Alerts テスト
// =========================================================================

#[test]
fn test_alerts_has_high_error_rate() {
    let Some((tmp, _)) = render_observability("order-api", "service", 8080) else {
        eprintln!("SKIP: observability テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "alerts.yaml");
    assert!(
        content.contains("OrderApiHighErrorRate"),
        "Alerts should contain HighErrorRate rule\n--- alerts.yaml ---\n{}",
        content
    );
}

#[test]
fn test_alerts_has_high_latency() {
    let Some((tmp, _)) = render_observability("order-api", "service", 8080) else {
        eprintln!("SKIP: observability テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "alerts.yaml");
    assert!(
        content.contains("OrderApiHighLatency"),
        "Alerts should contain HighLatency rule\n--- alerts.yaml ---\n{}",
        content
    );
}

#[test]
fn test_alerts_has_pod_restart() {
    let Some((tmp, _)) = render_observability("order-api", "service", 8080) else {
        eprintln!("SKIP: observability テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "alerts.yaml");
    assert!(
        content.contains("OrderApiPodRestart"),
        "Alerts should contain PodRestart rule\n--- alerts.yaml ---\n{}",
        content
    );
}

#[test]
fn test_alerts_has_namespace() {
    let Some((tmp, _)) = render_observability("order-api", "service", 8080) else {
        eprintln!("SKIP: observability テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "alerts.yaml");
    assert!(
        content.contains("k1s0-service"),
        "Alerts should contain namespace\n--- alerts.yaml ---\n{}",
        content
    );
}

// =========================================================================
// Tera 構文残留チェック
// =========================================================================

#[test]
fn test_observability_no_tera_syntax() {
    let Some((tmp, names)) = render_observability("order-api", "service", 8080) else {
        eprintln!("SKIP: observability テンプレートディレクトリが未作成");
        return;
    };

    for name in &names {
        let content = read_output(&tmp, name);
        assert!(
            !content.contains("{%"),
            "Tera syntax found in {}",
            name
        );
        assert!(
            !content.contains("{#"),
            "Tera comment found in {}",
            name
        );
    }
}
