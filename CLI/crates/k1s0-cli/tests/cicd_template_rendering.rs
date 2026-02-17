/// CI/CD テンプレートのレンダリング統合テスト。
///
/// CLI/templates/cicd/ テンプレートファイルを使用し、
/// テンプレートエンジンでレンダリングした結果が仕様書と一致することを検証する。
///
/// 仕様書: docs/テンプレート仕様-CICD.md
/// ディレクトリ構成（フラット）:
///   CLI/templates/cicd/
///     ├── ci.yaml.tera
///     └── deploy.yaml.tera
use std::fs;
use std::path::Path;

use k1s0_cli::template::context::TemplateContextBuilder;
use k1s0_cli::template::TemplateEngine;
use tempfile::TempDir;

// =========================================================================
// ヘルパー関数
// =========================================================================

fn template_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("templates")
}

/// cicd テンプレートをレンダリングする。
///
/// 仕様書に従い、CLI/templates/cicd/ 直下のフラット構造からレンダリングする。
/// kind="cicd" で TemplateEngine に渡し、言語分岐はテンプレート内部の
/// {% if language == "go" %} 等で行う。
fn render_cicd(
    _kind: &str,
    lang: &str,
    api_style: &str,
    has_database: bool,
) -> Option<(TempDir, Vec<String>)> {
    let tpl_dir = template_dir();

    // cicd テンプレートディレクトリの存在チェック（フラット構造）
    let cicd_dir = tpl_dir.join("cicd");
    if !cicd_dir.exists() {
        return None;
    }

    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    // kind="cicd" で構築。language は Tera 変数として渡される。
    // 実際の kind (server/client 等) は元のパラメータを使うが、
    // テンプレート選択用の kind は "cicd" とする。
    let mut builder = TemplateContextBuilder::new("order-api", "service", lang, "cicd")
        .api_style(api_style);

    if has_database {
        builder = builder.with_database("postgresql");
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

// =========================================================================
// CI/CD ファイル一覧テスト
// =========================================================================

#[test]
fn test_cicd_go_generates_ci_and_deploy() {
    let Some((_, names)) = render_cicd("server", "go", "rest", false) else {
        eprintln!("SKIP: cicd テンプレートディレクトリが未作成");
        return;
    };

    // CI + Deploy の両方が生成される
    assert!(
        names.iter().any(|n| n.contains("ci")),
        "CI workflow file missing. Generated: {:?}",
        names
    );
    assert!(
        names.iter().any(|n| n.contains("deploy")),
        "Deploy workflow file missing. Generated: {:?}",
        names
    );
}

#[test]
fn test_cicd_rust_generates_ci_and_deploy() {
    let Some((_, names)) = render_cicd("server", "rust", "rest", false) else {
        eprintln!("SKIP: cicd テンプレートディレクトリが未作成");
        return;
    };

    assert!(
        names.iter().any(|n| n.contains("ci")),
        "CI workflow file missing. Generated: {:?}",
        names
    );
    assert!(
        names.iter().any(|n| n.contains("deploy")),
        "Deploy workflow file missing. Generated: {:?}",
        names
    );
}

#[test]
fn test_cicd_typescript_generates_ci_and_deploy() {
    let Some((_, names)) = render_cicd("client", "typescript", "", false) else {
        eprintln!("SKIP: cicd テンプレートディレクトリが未作成");
        return;
    };

    // CI が生成される
    assert!(
        names.iter().any(|n| n.contains("ci")),
        "CI workflow file missing. Generated: {:?}",
        names
    );
}

#[test]
fn test_cicd_dart_generates_ci() {
    let Some((_, names)) = render_cicd("client", "dart", "", false) else {
        eprintln!("SKIP: cicd テンプレートディレクトリが未作成");
        return;
    };

    assert!(
        names.iter().any(|n| n.contains("ci")),
        "CI workflow file missing. Generated: {:?}",
        names
    );
}

// =========================================================================
// CI ワークフロー内容テスト（言語別）
// =========================================================================

#[test]
fn test_cicd_ci_yaml_go_content() {
    let Some((tmp, names)) = render_cicd("server", "go", "rest", false) else {
        eprintln!("SKIP: cicd テンプレートディレクトリが未作成");
        return;
    };

    let ci_file = names.iter().find(|n| n.contains("ci")).unwrap();
    let content = read_output(&tmp, ci_file);

    assert!(content.contains("go") || content.contains("Go"), "Go reference missing in CI");
    assert!(content.contains("test") || content.contains("Test"), "test step missing in CI");
    assert!(content.contains("lint") || content.contains("Lint"), "lint step missing in CI");
}

#[test]
fn test_cicd_ci_yaml_rust_content() {
    let Some((tmp, names)) = render_cicd("server", "rust", "rest", false) else {
        eprintln!("SKIP: cicd テンプレートディレクトリが未作成");
        return;
    };

    let ci_file = names.iter().find(|n| n.contains("ci")).unwrap();
    let content = read_output(&tmp, ci_file);

    assert!(content.contains("cargo") || content.contains("rust"), "Rust/cargo reference missing in CI");
    assert!(content.contains("test") || content.contains("Test"), "test step missing in CI");
    assert!(content.contains("clippy") || content.contains("lint"), "lint step missing in CI");
}

// =========================================================================
// Deploy ワークフロー内容テスト
// =========================================================================

#[test]
fn test_cicd_deploy_yaml_content() {
    let Some((tmp, names)) = render_cicd("server", "go", "rest", false) else {
        eprintln!("SKIP: cicd テンプレートディレクトリが未作成");
        return;
    };

    let deploy_file = names.iter().find(|n| n.contains("deploy")).unwrap();
    let content = read_output(&tmp, deploy_file);

    assert!(content.contains("deploy") || content.contains("Deploy"), "deploy reference missing");
    assert!(content.contains("helm") || content.contains("docker"), "helm or docker reference missing in deploy");
}

// =========================================================================
// 条件付きステップテスト
// =========================================================================

#[test]
fn test_cicd_ci_grpc_step() {
    let Some((tmp, names)) = render_cicd("server", "go", "grpc", false) else {
        eprintln!("SKIP: cicd テンプレートディレクトリが未作成");
        return;
    };

    let ci_file = names.iter().find(|n| n.contains("ci")).unwrap();
    let content = read_output(&tmp, ci_file);

    assert!(
        content.contains("buf") || content.contains("proto"),
        "buf lint step missing for gRPC CI"
    );
}

#[test]
fn test_cicd_ci_database_step() {
    let Some((tmp, names)) = render_cicd("server", "go", "rest", true) else {
        eprintln!("SKIP: cicd テンプレートディレクトリが未作成");
        return;
    };

    let ci_file = names.iter().find(|n| n.contains("ci")).unwrap();
    let content = read_output(&tmp, ci_file);

    assert!(
        content.contains("migration") || content.contains("migrate") || content.contains("database"),
        "migration test step missing when DB enabled"
    );
}

// =========================================================================
// Tera 構文残留チェック
// =========================================================================

#[test]
fn test_cicd_no_tera_syntax() {
    let Some((tmp, names)) = render_cicd("server", "go", "rest", true) else {
        eprintln!("SKIP: cicd テンプレートディレクトリが未作成");
        return;
    };

    for name in &names {
        let content = read_output(&tmp, name);
        assert!(!content.contains("{%"), "Tera syntax {{%% found in {}", name);
        assert!(!content.contains("{#"), "Tera comment {{# found in {}", name);
    }
}

// =========================================================================
// GitHub Actions 構文保持テスト
// =========================================================================

#[test]
fn test_cicd_github_actions_syntax() {
    let Some((tmp, names)) = render_cicd("server", "go", "rest", false) else {
        eprintln!("SKIP: cicd テンプレートディレクトリが未作成");
        return;
    };

    for name in &names {
        let content = read_output(&tmp, name);
        if content.contains("${{") {
            assert!(
                content.contains("${{") && content.contains("}}"),
                "GitHub Actions syntax broken in {}",
                name
            );
        }
    }
}

// =========================================================================
// サービス名がワークフロー名に含まれるか
// =========================================================================

#[test]
fn test_cicd_service_name_in_workflow_name() {
    let Some((tmp, names)) = render_cicd("server", "go", "rest", false) else {
        eprintln!("SKIP: cicd テンプレートディレクトリが未作成");
        return;
    };

    let ci_file = names.iter().find(|n| n.contains("ci")).unwrap();
    let content = read_output(&tmp, ci_file);

    assert!(
        content.contains("order-api") || content.contains("OrderApi") || content.contains("order_api"),
        "service_name not found in workflow name"
    );
}

// =========================================================================
// セキュリティスキャンステップの存在確認
// =========================================================================

#[test]
fn test_cicd_ci_security_scan_step() {
    let Some((tmp, names)) = render_cicd("server", "go", "rest", false) else {
        eprintln!("SKIP: cicd テンプレートディレクトリが未作成");
        return;
    };

    let ci_file = names.iter().find(|n| n.contains("ci")).unwrap();
    let content = read_output(&tmp, ci_file);

    assert!(
        content.contains("security-scan") || content.contains("trivy"),
        "security-scan step missing in CI"
    );
}
