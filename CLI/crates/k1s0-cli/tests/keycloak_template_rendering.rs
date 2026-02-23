/// Keycloak テンプレートのレンダリング統合テスト。
///
/// CLI/templates/keycloak/ テンプレートファイルを使用し、
/// テンプレートエンジンでレンダリングした結果を検証する。
use std::fs;
use std::path::Path;

use k1s0_cli::template::context::TemplateContextBuilder;
use k1s0_cli::template::TemplateEngine;
use tempfile::TempDir;

fn template_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("templates")
}

fn render_keycloak(service_name: &str, tier: &str) -> Option<(TempDir, Vec<String>)> {
    let tpl_dir = template_dir();
    let keycloak_dir = tpl_dir.join("keycloak");
    if !keycloak_dir.exists() {
        return None;
    }

    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let ctx = TemplateContextBuilder::new(service_name, tier, "go", "keycloak").build();

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
fn test_keycloak_file_list() {
    let Some((_, names)) = render_keycloak("order-api", "service") else {
        eprintln!("SKIP: keycloak テンプレートディレクトリが未作成");
        return;
    };

    assert!(
        names.iter().any(|n| n.contains("keycloak-client.json")),
        "keycloak-client.json missing. Generated: {names:?}"
    );
}

// =========================================================================
// Keycloak Client テスト
// =========================================================================

#[test]
fn test_keycloak_client_has_client_id() {
    let Some((tmp, _)) = render_keycloak("order-api", "service") else {
        eprintln!("SKIP: keycloak テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "keycloak-client.json");
    assert!(
        content.contains("\"clientId\": \"order-api\""),
        "Keycloak client should have correct clientId\n--- keycloak-client.json ---\n{content}"
    );
}

#[test]
fn test_keycloak_client_has_pascal_name() {
    let Some((tmp, _)) = render_keycloak("order-api", "service") else {
        eprintln!("SKIP: keycloak テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "keycloak-client.json");
    assert!(
        content.contains("OrderApi"),
        "Keycloak client should use PascalCase name\n--- keycloak-client.json ---\n{content}"
    );
}

#[test]
fn test_keycloak_client_has_namespace() {
    let Some((tmp, _)) = render_keycloak("order-api", "service") else {
        eprintln!("SKIP: keycloak テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "keycloak-client.json");
    assert!(
        content.contains("k1s0-service"),
        "Keycloak client should contain namespace\n--- keycloak-client.json ---\n{content}"
    );
}

#[test]
fn test_keycloak_client_has_tier() {
    let Some((tmp, _)) = render_keycloak("order-api", "service") else {
        eprintln!("SKIP: keycloak テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "keycloak-client.json");
    assert!(
        content.contains("service tier"),
        "Keycloak client should mention tier in description\n--- keycloak-client.json ---\n{content}"
    );
}

#[test]
fn test_keycloak_client_openid_connect() {
    let Some((tmp, _)) = render_keycloak("order-api", "service") else {
        eprintln!("SKIP: keycloak テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "keycloak-client.json");
    assert!(
        content.contains("openid-connect"),
        "Keycloak client should use openid-connect protocol\n--- keycloak-client.json ---\n{content}"
    );
}

#[test]
fn test_keycloak_client_service_accounts_enabled() {
    let Some((tmp, _)) = render_keycloak("order-api", "service") else {
        eprintln!("SKIP: keycloak テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "keycloak-client.json");
    assert!(
        content.contains("\"serviceAccountsEnabled\": true"),
        "Keycloak client should have service accounts enabled\n--- keycloak-client.json ---\n{content}"
    );
}

// =========================================================================
// Tera 構文残留チェック
// =========================================================================

#[test]
fn test_keycloak_no_tera_syntax() {
    let Some((tmp, names)) = render_keycloak("order-api", "service") else {
        eprintln!("SKIP: keycloak テンプレートディレクトリが未作成");
        return;
    };

    for name in &names {
        let content = read_output(&tmp, name);
        assert!(!content.contains("{%"), "Tera syntax found in {name}");
        assert!(!content.contains("{#"), "Tera comment found in {name}");
    }
}
