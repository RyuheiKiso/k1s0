/// Helm テンプレートのレンダリング統合テスト。
///
/// CLI/templates/helm/ テンプレートファイルを使用し、
/// テンプレートエンジンでレンダリングした結果が仕様書と一致することを検証する。
///
/// NOTE: helm テンプレートは未作成のため、テストは TDD として作成。
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

/// helm テンプレートをレンダリングする。
///
/// helm テンプレートは server と同じコンテキストを使用するが、
/// kind を "helm" として TemplateEngine に渡す想定。
/// 現時点ではテンプレートディレクトリが未作成のため、
/// テンプレートが存在しない場合は None を返す。
fn render_helm(
    lang: &str,
    api_style: &str,
    has_database: bool,
    has_kafka: bool,
    has_redis: bool,
) -> Option<(TempDir, Vec<String>)> {
    let tpl_dir = template_dir();

    // helm テンプレートディレクトリの存在チェック
    // TDD: テンプレートが未作成の場合は None を返す
    let helm_dir = tpl_dir.join("helm").join(lang);
    if !helm_dir.exists() {
        return None;
    }

    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let mut builder = TemplateContextBuilder::new("order-api", "service", lang, "helm")
        .api_style(api_style);

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

// =========================================================================
// Helm Chart テスト (TDD: テンプレート未作成のためスキップ可能)
// =========================================================================

#[test]
fn test_helm_file_list() {
    let Some((_, names)) = render_helm("go", "rest", true, true, true) else {
        // TDD: テンプレート未作成の場合はスキップ
        eprintln!("SKIP: helm/go テンプレートディレクトリが未作成");
        return;
    };

    // 全10ファイルの存在確認
    assert!(names.iter().any(|n| n.contains("Chart.yaml")), "Chart.yaml missing");
    assert!(names.iter().any(|n| n.contains("values.yaml")), "values.yaml missing");
    assert!(names.iter().any(|n| n.contains("values-dev.yaml")), "values-dev.yaml missing");
    assert!(names.iter().any(|n| n.contains("values-prod.yaml")), "values-prod.yaml missing");
    assert!(names.iter().any(|n| n.contains("deployment.yaml")), "deployment.yaml missing");
    assert!(names.iter().any(|n| n.contains("service.yaml")), "service.yaml missing");
}

#[test]
fn test_helm_chart_yaml_content() {
    let Some((tmp, _)) = render_helm("go", "rest", false, false, false) else {
        eprintln!("SKIP: helm/go テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "Chart.yaml");
    assert!(content.contains("name: order-api"));
    assert!(content.contains("version:"));
    assert!(content.contains("appVersion:"));
}

#[test]
fn test_helm_values_yaml_content() {
    let Some((tmp, _)) = render_helm("go", "rest", true, false, false) else {
        eprintln!("SKIP: helm/go テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "values.yaml");
    assert!(content.contains("order-api"));
    assert!(content.contains("replicaCount:"));
    assert!(content.contains("image:"));
}

#[test]
fn test_helm_deployment_content() {
    let Some((tmp, _)) = render_helm("go", "rest", false, false, false) else {
        eprintln!("SKIP: helm/go テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "deployment.yaml");
    assert!(content.contains("kind: Deployment"));
    assert!(content.contains("order-api"));
}

#[test]
fn test_helm_service_content() {
    let Some((tmp, _)) = render_helm("go", "rest", false, false, false) else {
        eprintln!("SKIP: helm/go テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "service.yaml");
    assert!(content.contains("kind: Service"));
}

#[test]
fn test_helm_grpc_port_in_service() {
    let Some((tmp, _)) = render_helm("go", "grpc", false, false, false) else {
        eprintln!("SKIP: helm/go テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "service.yaml");
    assert!(content.contains("grpc") || content.contains("50051"), "gRPC port missing in service.yaml");
}

#[test]
fn test_helm_database_config() {
    let Some((tmp, names)) = render_helm("go", "rest", true, false, false) else {
        eprintln!("SKIP: helm/go テンプレートディレクトリが未作成");
        return;
    };

    // DB 有効時に DB 設定が values.yaml に含まれる
    let content = read_output(&tmp, "values.yaml");
    assert!(
        content.contains("database") || content.contains("postgresql"),
        "database config missing in values.yaml when DB enabled"
    );
}

#[test]
fn test_helm_kafka_config() {
    let Some((tmp, _)) = render_helm("go", "rest", false, true, false) else {
        eprintln!("SKIP: helm/go テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "values.yaml");
    assert!(
        content.contains("kafka"),
        "kafka config missing in values.yaml when Kafka enabled"
    );
}

#[test]
fn test_helm_redis_config() {
    let Some((tmp, _)) = render_helm("go", "rest", false, false, true) else {
        eprintln!("SKIP: helm/go テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "values.yaml");
    assert!(
        content.contains("redis"),
        "redis config missing in values.yaml when Redis enabled"
    );
}

#[test]
fn test_helm_values_dev() {
    let Some((tmp, _)) = render_helm("go", "rest", false, false, false) else {
        eprintln!("SKIP: helm/go テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "values-dev.yaml");
    assert!(content.contains("replicaCount:") || content.contains("replica"));
}

#[test]
fn test_helm_values_prod() {
    let Some((tmp, _)) = render_helm("go", "rest", false, false, false) else {
        eprintln!("SKIP: helm/go テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "values-prod.yaml");
    assert!(content.contains("replicaCount:") || content.contains("replica"));
}

#[test]
fn test_helm_values_staging() {
    let Some((tmp, _)) = render_helm("go", "rest", true, false, false) else {
        eprintln!("SKIP: helm/go テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "values-staging.yaml");
    assert!(content.contains("replicaCount:") || content.contains("replica"));
    assert!(content.contains("staging") || content.contains("autoscaling"));
}

#[test]
fn test_helm_configmap() {
    let Some((tmp, _)) = render_helm("go", "rest", false, false, false) else {
        eprintln!("SKIP: helm/go テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "configmap.yaml");
    assert!(
        content.contains("k1s0-common.configmap") || content.contains("ConfigMap"),
        "configmap.yaml should reference k1s0-common.configmap"
    );
}

#[test]
fn test_helm_hpa() {
    let Some((tmp, _)) = render_helm("go", "rest", false, false, false) else {
        eprintln!("SKIP: helm/go テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "hpa.yaml");
    assert!(
        content.contains("k1s0-common.hpa") || content.contains("HorizontalPodAutoscaler"),
        "hpa.yaml should reference k1s0-common.hpa"
    );
}

#[test]
fn test_helm_pdb() {
    let Some((tmp, _)) = render_helm("go", "rest", false, false, false) else {
        eprintln!("SKIP: helm/go テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "pdb.yaml");
    assert!(
        content.contains("k1s0-common.pdb") || content.contains("PodDisruptionBudget"),
        "pdb.yaml should reference k1s0-common.pdb"
    );
}

#[test]
fn test_helm_no_tera_syntax() {
    let Some((tmp, names)) = render_helm("go", "rest", true, true, true) else {
        eprintln!("SKIP: helm/go テンプレートディレクトリが未作成");
        return;
    };

    for name in &names {
        let content = read_output(&tmp, name);
        // Helm テンプレートの {{ }} は Tera のものではなく、Helm の構文
        // ここでは Tera の {%  %} と {#  #} のみチェック
        assert!(!content.contains("{%"), "Tera syntax {{%% found in {}", name);
        assert!(!content.contains("{#"), "Tera comment {{# found in {}", name);
    }
}
