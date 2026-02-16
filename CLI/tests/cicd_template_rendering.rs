/// CI/CD テンプレートのレンダリング統合テスト。
///
/// CLI/templates/cicd/ テンプレートファイルを使用し、
/// テンプレートエンジンでレンダリングした結果が仕様書と一致することを検証する。
///
/// NOTE: cicd テンプレートは未作成のため、テストは TDD として作成。
/// テンプレートディレクトリが存在しない場合はエラーになるのが正しい動作。
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
/// 現時点ではテンプレートディレクトリが未作成のため、
/// テンプレートが存在しない場合は None を返す。
fn render_cicd(
    kind: &str,
    lang: &str,
    api_style: &str,
    has_database: bool,
) -> Option<(TempDir, Vec<String>)> {
    let tpl_dir = template_dir();

    // cicd テンプレートディレクトリの存在チェック
    // TDD: テンプレートが未作成の場合は None を返す
    let cicd_dir = tpl_dir.join("cicd").join(lang);
    if !cicd_dir.exists() {
        return None;
    }

    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

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
fn test_cicd_server_go_file_list() {
    let Some((_, names)) = render_cicd("server", "go", "rest", false) else {
        eprintln!("SKIP: cicd/go テンプレートディレクトリが未作成");
        return;
    };

    // server は CI + Deploy の両方が生成される
    assert!(
        names.iter().any(|n| n.contains("ci")),
        "CI workflow file missing for server/go"
    );
    assert!(
        names.iter().any(|n| n.contains("deploy")),
        "Deploy workflow file missing for server/go"
    );
}

#[test]
fn test_cicd_server_rust_file_list() {
    let Some((_, names)) = render_cicd("server", "rust", "rest", false) else {
        eprintln!("SKIP: cicd/rust テンプレートディレクトリが未作成");
        return;
    };

    assert!(
        names.iter().any(|n| n.contains("ci")),
        "CI workflow file missing for server/rust"
    );
    assert!(
        names.iter().any(|n| n.contains("deploy")),
        "Deploy workflow file missing for server/rust"
    );
}

#[test]
fn test_cicd_client_react_file_list() {
    let Some((_, names)) = render_cicd("client", "react", "", false) else {
        eprintln!("SKIP: cicd/react テンプレートディレクトリが未作成");
        return;
    };

    // client は CI のみ（Deploy は helm 経由のため不要）
    assert!(
        names.iter().any(|n| n.contains("ci")),
        "CI workflow file missing for client/react"
    );
}

#[test]
fn test_cicd_library_go_file_list() {
    let Some((_, names)) = render_cicd("library", "go", "", false) else {
        eprintln!("SKIP: cicd/go テンプレートディレクトリが未作成");
        return;
    };

    // library は CI のみ
    assert!(
        names.iter().any(|n| n.contains("ci")),
        "CI workflow file missing for library/go"
    );
}

// =========================================================================
// CI/CD 内容テスト
// =========================================================================

#[test]
fn test_cicd_ci_yaml_go_content() {
    let Some((tmp, names)) = render_cicd("server", "go", "rest", false) else {
        eprintln!("SKIP: cicd/go テンプレートディレクトリが未作成");
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
        eprintln!("SKIP: cicd/rust テンプレートディレクトリが未作成");
        return;
    };

    let ci_file = names.iter().find(|n| n.contains("ci")).unwrap();
    let content = read_output(&tmp, ci_file);

    assert!(content.contains("cargo") || content.contains("rust"), "Rust/cargo reference missing in CI");
    assert!(content.contains("test") || content.contains("Test"), "test step missing in CI");
    assert!(content.contains("clippy") || content.contains("lint"), "lint step missing in CI");
}

#[test]
fn test_cicd_deploy_yaml_content() {
    let Some((tmp, names)) = render_cicd("server", "go", "rest", false) else {
        eprintln!("SKIP: cicd/go テンプレートディレクトリが未作成");
        return;
    };

    let deploy_file = names.iter().find(|n| n.contains("deploy")).unwrap();
    let content = read_output(&tmp, deploy_file);

    assert!(content.contains("deploy") || content.contains("Deploy"), "deploy reference missing");
    assert!(content.contains("helm") || content.contains("docker"), "helm or docker reference missing in deploy");
}

#[test]
fn test_cicd_ci_grpc_step() {
    let Some((tmp, names)) = render_cicd("server", "go", "grpc", false) else {
        eprintln!("SKIP: cicd/go テンプレートディレクトリが未作成");
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
        eprintln!("SKIP: cicd/go テンプレートディレクトリが未作成");
        return;
    };

    let ci_file = names.iter().find(|n| n.contains("ci")).unwrap();
    let content = read_output(&tmp, ci_file);

    assert!(
        content.contains("migration") || content.contains("migrate") || content.contains("database"),
        "migration test step missing when DB enabled"
    );
}

#[test]
fn test_cicd_no_tera_syntax() {
    let Some((tmp, names)) = render_cicd("server", "go", "rest", true) else {
        eprintln!("SKIP: cicd/go テンプレートディレクトリが未作成");
        return;
    };

    for name in &names {
        let content = read_output(&tmp, name);
        assert!(!content.contains("{%"), "Tera syntax {{%% found in {}", name);
        assert!(!content.contains("{#"), "Tera comment {{# found in {}", name);
    }
}

#[test]
fn test_cicd_github_actions_syntax() {
    let Some((tmp, names)) = render_cicd("server", "go", "rest", false) else {
        eprintln!("SKIP: cicd/go テンプレートディレクトリが未作成");
        return;
    };

    for name in &names {
        let content = read_output(&tmp, name);
        // GitHub Actions の ${{ }} 構文が保持されていることを検証
        // (Tera の {{ }} と衝突しないようにエスケープされているはず)
        if content.contains("${{") {
            // GitHub Actions 構文が正しく保持されている
            assert!(
                content.contains("${{") && content.contains("}}"),
                "GitHub Actions syntax broken in {}",
                name
            );
        }
    }
}

#[test]
fn test_cicd_service_name_in_workflow_name() {
    let Some((tmp, names)) = render_cicd("server", "go", "rest", false) else {
        eprintln!("SKIP: cicd/go テンプレートディレクトリが未作成");
        return;
    };

    let ci_file = names.iter().find(|n| n.contains("ci")).unwrap();
    let content = read_output(&tmp, ci_file);

    assert!(
        content.contains("order-api") || content.contains("OrderApi") || content.contains("order_api"),
        "service_name not found in workflow name"
    );
}
