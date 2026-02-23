/// Vault テンプレートのレンダリング統合テスト。
use std::fs;
use std::path::Path;

use k1s0_cli::template::context::TemplateContextBuilder;
use k1s0_cli::template::TemplateEngine;
use tempfile::TempDir;

fn template_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("templates")
}

fn render_vault(
    service_name: &str,
    tier: &str,
    has_database: bool,
    has_kafka: bool,
    has_redis: bool,
) -> Option<(TempDir, Vec<String>)> {
    let tpl_dir = template_dir();
    let cat_dir = tpl_dir.join("vault");
    if !cat_dir.exists() {
        return None;
    }

    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let mut builder = TemplateContextBuilder::new(service_name, tier, "go", "vault");
    if has_database {
        builder = builder.with_database("postgresql");
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

#[test]
fn test_vault_file_list() {
    let Some((_, names)) = render_vault("order-api", "service", true, false, false) else {
        eprintln!("SKIP: vault テンプレートディレクトリが未作成");
        return;
    };

    assert!(
        names
            .iter()
            .any(|n| n.contains("secret-provider-class.yaml")),
        "secret-provider-class.yaml missing. Generated: {names:?}"
    );
    assert!(
        names.iter().any(|n| n.contains("vault-policy.yaml")),
        "vault-policy.yaml missing. Generated: {names:?}"
    );
}

#[test]
fn test_vault_has_service_name() {
    let Some((tmp, _)) = render_vault("order-api", "service", true, false, false) else {
        eprintln!("SKIP: vault テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "secret-provider-class.yaml");
    assert!(
        content.contains("order-api"),
        "Vault config should contain service name\n--- secret-provider-class.yaml ---\n{content}"
    );
}

#[test]
fn test_vault_has_namespace() {
    let Some((tmp, _)) = render_vault("order-api", "service", true, false, false) else {
        eprintln!("SKIP: vault テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "secret-provider-class.yaml");
    assert!(
        content.contains("k1s0-service"),
        "Vault config should contain namespace\n--- secret-provider-class.yaml ---\n{content}"
    );
}

#[test]
fn test_vault_with_database() {
    let Some((tmp, _)) = render_vault("order-api", "service", true, false, false) else {
        eprintln!("SKIP: vault テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "secret-provider-class.yaml");
    assert!(
        content.contains("db-username"),
        "Should include database secrets when has_database=true\n--- secret-provider-class.yaml ---\n{content}"
    );
}

#[test]
fn test_vault_without_database() {
    let Some((tmp, _)) = render_vault("order-api", "service", false, false, false) else {
        eprintln!("SKIP: vault テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "secret-provider-class.yaml");
    assert!(
        !content.contains("db-username"),
        "Should NOT include database secrets when has_database=false\n--- secret-provider-class.yaml ---\n{content}"
    );
}

#[test]
fn test_vault_with_kafka() {
    let Some((tmp, _)) = render_vault("order-api", "service", false, true, false) else {
        eprintln!("SKIP: vault テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "secret-provider-class.yaml");
    assert!(
        content.contains("kafka-password"),
        "Should include kafka secrets when has_kafka=true\n--- secret-provider-class.yaml ---\n{content}"
    );
}

#[test]
fn test_vault_with_redis() {
    let Some((tmp, _)) = render_vault("order-api", "service", false, false, true) else {
        eprintln!("SKIP: vault テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "secret-provider-class.yaml");
    assert!(
        content.contains("redis-password"),
        "Should include redis secrets when has_redis=true\n--- secret-provider-class.yaml ---\n{content}"
    );
}

#[test]
fn test_vault_secret_path_uses_tier() {
    let Some((tmp, _)) = render_vault("order-api", "service", true, false, false) else {
        eprintln!("SKIP: vault テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "secret-provider-class.yaml");
    assert!(
        content.contains("secret/data/k1s0/service/order-api"),
        "Secret path should include tier and service name\n--- secret-provider-class.yaml ---\n{content}"
    );
}

#[test]
fn test_vault_policy_system_shared() {
    let Some((tmp, _)) = render_vault("auth-service", "system", true, false, false) else {
        eprintln!("SKIP: vault テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "vault-policy.yaml");
    assert!(
        content.contains("secret/data/k1s0/system/shared"),
        "System tier should have access to shared secrets\n--- vault-policy.yaml ---\n{content}"
    );
}

#[test]
fn test_vault_policy_service_no_shared() {
    let Some((tmp, _)) = render_vault("order-api", "service", true, false, false) else {
        eprintln!("SKIP: vault テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "vault-policy.yaml");
    assert!(
        !content.contains("shared"),
        "Service tier should NOT have access to shared secrets\n--- vault-policy.yaml ---\n{content}"
    );
}

#[test]
fn test_vault_no_tera_syntax() {
    let Some((tmp, names)) = render_vault("order-api", "service", true, true, true) else {
        eprintln!("SKIP: vault テンプレートディレクトリが未作成");
        return;
    };

    for name in &names {
        let content = read_output(&tmp, name);
        assert!(!content.contains("{%"), "Tera block syntax found in {name}");
        assert!(!content.contains("{#"), "Tera comment found in {name}");
    }
}
