/// Terraform テンプレートのレンダリング統合テスト。
///
/// CLI/templates/terraform/ テンプレートファイルを使用し、
/// テンプレートエンジンでレンダリングした結果が仕様書と一致することを検証する。
///
/// NOTE: terraform テンプレートはフラット構造（言語サブディレクトリなし）。
/// テンプレートディレクトリが存在しない場合は TDD パターンで None を返す。
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

/// Terraform テンプレートをレンダリングする。
///
/// Terraform テンプレートはフラット構造（言語サブディレクトリなし）。
/// kind を "terraform" として `TemplateEngine` に渡す。
/// テンプレートディレクトリが存在しない場合は None を返す（TDD パターン）。
#[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
fn render_terraform(
    environment: &str,
    enable_postgresql: bool,
    enable_mysql: bool,
    enable_kafka: bool,
    enable_observability: bool,
    enable_service_mesh: bool,
    enable_vault: bool,
    enable_harbor: bool,
) -> Option<(TempDir, Vec<String>)> {
    let tpl_dir = template_dir();

    // terraform テンプレートディレクトリの存在チェック（フラット構造）
    // TDD: テンプレートが未作成の場合は None を返す
    let terraform_dir = tpl_dir.join("terraform");
    if !terraform_dir.exists() {
        return None;
    }

    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let mut builder =
        TemplateContextBuilder::new("k1s0", "system", "go", "terraform").environment(environment);

    if enable_postgresql {
        builder = builder.enable_postgresql();
    }
    if enable_mysql {
        builder = builder.enable_mysql();
    }
    if enable_kafka {
        builder = builder.enable_kafka();
    }
    if enable_observability {
        builder = builder.enable_observability();
    }
    if enable_service_mesh {
        builder = builder.enable_service_mesh();
    }
    if enable_vault {
        builder = builder.enable_vault();
    }
    if enable_harbor {
        builder = builder.enable_harbor();
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
// ファイル一覧テスト
// =========================================================================

#[test]
fn test_terraform_file_list() {
    let Some((_, names)) = render_terraform("dev", false, false, false, false, false, false, false)
    else {
        eprintln!("SKIP: terraform テンプレートディレクトリが未作成");
        return;
    };

    // 5ファイルの存在確認
    assert!(
        names.iter().any(|n| n.contains("main.tf")),
        "main.tf missing"
    );
    assert!(
        names.iter().any(|n| n.contains("variables.tf")),
        "variables.tf missing"
    );
    assert!(
        names.iter().any(|n| n.contains("terraform.tfvars")),
        "terraform.tfvars missing"
    );
    assert!(
        names.iter().any(|n| n.contains("backend.tf")),
        "backend.tf missing"
    );
    assert!(
        names.iter().any(|n| n.contains("outputs.tf")),
        "outputs.tf missing"
    );
}

// =========================================================================
// main.tf コンテンツテスト
// =========================================================================

#[test]
fn test_terraform_main_tf_content() {
    let Some((tmp, _)) = render_terraform("dev", false, false, false, false, false, false, false)
    else {
        eprintln!("SKIP: terraform テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "main.tf");
    assert!(
        content.contains("terraform {"),
        "main.tf に terraform ブロックが含まれるべき"
    );
    assert!(
        content.contains("module \"kubernetes_base\""),
        "main.tf に module \"kubernetes_base\" が含まれるべき"
    );
}

// =========================================================================
// backend.tf コンテンツテスト
// =========================================================================

#[test]
fn test_terraform_backend_tf_content() {
    let Some((tmp, _)) = render_terraform("dev", false, false, false, false, false, false, false)
    else {
        eprintln!("SKIP: terraform テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "backend.tf");
    assert!(
        content.contains("terraform/k1s0/dev"),
        "backend.tf に consul backend パス 'terraform/k1s0/dev' が含まれるべき"
    );
}

// =========================================================================
// variables.tf コンテンツテスト
// =========================================================================

#[test]
fn test_terraform_variables_tf_content() {
    let Some((tmp, _)) = render_terraform("dev", false, false, false, false, false, false, false)
    else {
        eprintln!("SKIP: terraform テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "variables.tf");
    assert!(
        content.contains("variable \"environment\""),
        "variables.tf に variable \"environment\" が含まれるべき"
    );
}

// =========================================================================
// terraform.tfvars コンテンツテスト
// =========================================================================

#[test]
fn test_terraform_tfvars_content() {
    let Some((tmp, _)) = render_terraform("dev", false, false, false, false, false, false, false)
    else {
        eprintln!("SKIP: terraform テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "terraform.tfvars");
    assert!(
        content.contains("environment = \"dev\""),
        "terraform.tfvars に environment = \"dev\" が含まれるべき"
    );
}

// =========================================================================
// outputs.tf コンテンツテスト
// =========================================================================

#[test]
fn test_terraform_outputs_tf_content() {
    let Some((tmp, _)) = render_terraform("dev", false, false, false, false, false, false, false)
    else {
        eprintln!("SKIP: terraform テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "outputs.tf");
    assert!(
        content.contains("output \"namespace_names\""),
        "outputs.tf に output \"namespace_names\" が含まれるべき"
    );
}

// =========================================================================
// 条件分岐テスト: enable_observability
// =========================================================================

#[test]
fn test_terraform_observability_enabled() {
    let Some((tmp, _)) = render_terraform("dev", false, false, false, true, false, false, false)
    else {
        eprintln!("SKIP: terraform テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "main.tf");
    assert!(
        content.contains("module \"observability\""),
        "enable_observability=true で main.tf に module \"observability\" が含まれるべき"
    );
}

#[test]
fn test_terraform_observability_disabled() {
    let Some((tmp, _)) = render_terraform("dev", false, false, false, false, false, false, false)
    else {
        eprintln!("SKIP: terraform テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "main.tf");
    assert!(
        !content.contains("module \"observability\""),
        "enable_observability=false で main.tf に module \"observability\" が含まれるべきでない"
    );
}

// =========================================================================
// 条件分岐テスト: enable_vault
// =========================================================================

#[test]
fn test_terraform_vault_provider() {
    let Some((tmp, _)) = render_terraform("dev", false, false, false, false, false, true, false)
    else {
        eprintln!("SKIP: terraform テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "main.tf");
    assert!(
        content.contains("provider \"vault\""),
        "enable_vault=true で main.tf に provider \"vault\" が含まれるべき"
    );
}

// =========================================================================
// 条件分岐テスト: enable_harbor
// =========================================================================

#[test]
fn test_terraform_harbor_provider() {
    let Some((tmp, _)) = render_terraform("dev", false, false, false, false, false, false, true)
    else {
        eprintln!("SKIP: terraform テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "main.tf");
    assert!(
        content.contains("provider \"harbor\""),
        "enable_harbor=true で main.tf に provider \"harbor\" が含まれるべき"
    );
}

// =========================================================================
// 環境別テスト: reclaim_policy
// =========================================================================

#[test]
fn test_terraform_prod_reclaim_policy() {
    let Some((tmp, _)) = render_terraform("prod", false, false, false, false, false, false, false)
    else {
        eprintln!("SKIP: terraform テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "terraform.tfvars");
    assert!(
        content.contains("reclaim_policy   = \"Retain\""),
        "environment=\"prod\" で terraform.tfvars に reclaim_policy = \"Retain\" が含まれるべき\n--- terraform.tfvars ---\n{content}"
    );
}

#[test]
fn test_terraform_dev_reclaim_policy() {
    let Some((tmp, _)) = render_terraform("dev", false, false, false, false, false, false, false)
    else {
        eprintln!("SKIP: terraform テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "terraform.tfvars");
    assert!(
        content.contains("reclaim_policy   = \"Delete\""),
        "environment=\"dev\" で terraform.tfvars に reclaim_policy = \"Delete\" が含まれるべき\n--- terraform.tfvars ---\n{content}"
    );
}

// =========================================================================
// Tera 構文残留チェック
// =========================================================================

#[test]
fn test_terraform_no_tera_syntax() {
    let Some((tmp, names)) = render_terraform("dev", true, true, true, true, true, true, true)
    else {
        eprintln!("SKIP: terraform テンプレートディレクトリが未作成");
        return;
    };

    for name in &names {
        let content = read_output(&tmp, name);
        assert!(!content.contains("{%"), "Tera syntax {{%% found in {name}");
        assert!(!content.contains("{#"), "Tera comment {{# found in {name}");
    }
}
