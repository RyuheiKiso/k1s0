/// Alertmanager テンプレートのレンダリング統合テスト。
use std::fs;
use std::path::Path;

use k1s0_cli::template::context::TemplateContextBuilder;
use k1s0_cli::template::TemplateEngine;
use tempfile::TempDir;

fn template_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("templates")
}

fn render_alertmanager(
    service_name: &str,
    tier: &str,
    server_port: u16,
) -> Option<(TempDir, Vec<String>)> {
    let tpl_dir = template_dir();
    let cat_dir = tpl_dir.join("alertmanager");
    if !cat_dir.exists() {
        return None;
    }

    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let ctx = TemplateContextBuilder::new(service_name, tier, "go", "alertmanager")
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
fn test_alertmanager_file_list() {
    let Some((_, names)) = render_alertmanager("order-api", "service", 8080) else {
        eprintln!("SKIP: alertmanager テンプレートディレクトリが未作成");
        return;
    };

    assert!(
        names.iter().any(|n| n.contains("alertmanager-config.yaml")),
        "alertmanager-config.yaml missing. Generated: {:?}",
        names
    );
    assert!(
        names.iter().any(|n| n.contains("notification-secret.yaml")),
        "notification-secret.yaml missing. Generated: {:?}",
        names
    );
}

#[test]
fn test_alertmanager_has_service_name() {
    let Some((tmp, _)) = render_alertmanager("order-api", "service", 8080) else {
        eprintln!("SKIP: alertmanager テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "alertmanager-config.yaml");
    assert!(
        content.contains("order-api"),
        "Alertmanager config should contain service name\n--- alertmanager-config.yaml ---\n{}",
        content
    );
}

#[test]
fn test_alertmanager_has_namespace() {
    let Some((tmp, _)) = render_alertmanager("order-api", "service", 8080) else {
        eprintln!("SKIP: alertmanager テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "alertmanager-config.yaml");
    assert!(
        content.contains("k1s0-service"),
        "Alertmanager config should contain namespace\n--- alertmanager-config.yaml ---\n{}",
        content
    );
}

#[test]
fn test_alertmanager_system_group_interval() {
    let Some((tmp, _)) = render_alertmanager("auth-service", "system", 8080) else {
        eprintln!("SKIP: alertmanager テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "alertmanager-config.yaml");
    assert!(
        content.contains("group_interval: 1m"),
        "System tier should have group_interval 1m\n--- alertmanager-config.yaml ---\n{}",
        content
    );
}

#[test]
fn test_alertmanager_service_group_interval() {
    let Some((tmp, _)) = render_alertmanager("order-api", "service", 8080) else {
        eprintln!("SKIP: alertmanager テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "alertmanager-config.yaml");
    assert!(
        content.contains("group_interval: 5m"),
        "Service tier should have group_interval 5m\n--- alertmanager-config.yaml ---\n{}",
        content
    );
}

#[test]
fn test_alertmanager_system_repeat_interval() {
    let Some((tmp, _)) = render_alertmanager("auth-service", "system", 8080) else {
        eprintln!("SKIP: alertmanager テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "alertmanager-config.yaml");
    assert!(
        content.contains("repeat_interval: 4h"),
        "System tier should have repeat_interval 4h\n--- alertmanager-config.yaml ---\n{}",
        content
    );
}

#[test]
fn test_notification_secret_has_service_name() {
    let Some((tmp, _)) = render_alertmanager("order-api", "service", 8080) else {
        eprintln!("SKIP: alertmanager テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "notification-secret.yaml");
    assert!(
        content.contains("order-api"),
        "Notification secret should contain service name\n--- notification-secret.yaml ---\n{}",
        content
    );
}

#[test]
fn test_alertmanager_no_tera_syntax() {
    let Some((tmp, names)) = render_alertmanager("order-api", "service", 8080) else {
        eprintln!("SKIP: alertmanager テンプレートディレクトリが未作成");
        return;
    };

    for name in &names {
        let content = read_output(&tmp, name);
        assert!(!content.contains("{%"), "Tera block syntax found in {}", name);
        assert!(!content.contains("{#"), "Tera comment found in {}", name);
    }
}
