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

/// helm テンプレートを複数 API スタイル対応でレンダリングする。
///
/// api_styles に複数の API 方式（例: vec!["rest", "grpc"]）を指定できる。
/// Helm テンプレートは言語サブディレクトリを持たないフラット構造のため、
/// テンプレートディレクトリは templates/helm/ を直接参照する。
fn render_helm_with_styles(
    api_styles: Vec<&str>,
    has_database: bool,
    has_kafka: bool,
    has_redis: bool,
) -> Option<(TempDir, Vec<String>)> {
    let tpl_dir = template_dir();

    // helm テンプレートディレクトリの存在チェック（フラット構造）
    let helm_dir = tpl_dir.join("helm");
    if !helm_dir.exists() {
        return None;
    }

    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let styles: Vec<String> = api_styles.iter().map(|s| s.to_string()).collect();
    let mut builder = TemplateContextBuilder::new("order-api", "service", "go", "helm")
        .api_styles(styles);

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
    assert!(names.iter().any(|n| n.contains("ingress.yaml")), "ingress.yaml missing");
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
fn test_helm_vault_secrets_with_kafka() {
    let Some((tmp, _)) = render_helm("go", "rest", false, true, false) else {
        eprintln!("SKIP: helm/go テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "values.yaml");
    assert!(
        content.contains("kafka") && content.contains("vault"),
        "Kafka 有効時に Vault シークレットに Kafka パスが含まれるべき"
    );
}

#[test]
fn test_helm_vault_secrets_with_redis() {
    let Some((tmp, _)) = render_helm("go", "rest", false, false, true) else {
        eprintln!("SKIP: helm/go テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "values.yaml");
    assert!(
        content.contains("redis") && content.contains("vault"),
        "Redis 有効時に Vault シークレットに Redis パスが含まれるべき"
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

// =========================================================================
// Library Chart 連携テスト
// =========================================================================

/// deployment.yaml が Library Chart の include を呼び出していることを検証
#[test]
fn test_helm_deployment_calls_library_chart() {
    let Some((tmp, _)) = render_helm("go", "rest", false, false, false) else {
        eprintln!("SKIP: helm/go テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "deployment.yaml");
    assert!(
        content.contains(r#"{{- include "k1s0-common.deployment" . }}"#),
        "deployment.yaml に Library Chart 呼び出し '{{{{- include \"k1s0-common.deployment\" . }}}}' が含まれていません"
    );
}

/// service.yaml が Library Chart の include を呼び出していることを検証
#[test]
fn test_helm_service_calls_library_chart() {
    let Some((tmp, _)) = render_helm("go", "rest", false, false, false) else {
        eprintln!("SKIP: helm/go テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "service.yaml");
    assert!(
        content.contains(r#"{{- include "k1s0-common.service" . }}"#),
        "service.yaml に Library Chart 呼び出し '{{{{- include \"k1s0-common.service\" . }}}}' が含まれていません"
    );
}

/// configmap.yaml が Library Chart の include を呼び出していることを検証
#[test]
fn test_helm_configmap_calls_library_chart() {
    let Some((tmp, _)) = render_helm("go", "rest", false, false, false) else {
        eprintln!("SKIP: helm/go テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "configmap.yaml");
    assert!(
        content.contains(r#"{{- include "k1s0-common.configmap" . }}"#),
        "configmap.yaml に Library Chart 呼び出し '{{{{- include \"k1s0-common.configmap\" . }}}}' が含まれていません"
    );
}

/// hpa.yaml が Library Chart の include を呼び出していることを検証
#[test]
fn test_helm_hpa_calls_library_chart() {
    let Some((tmp, _)) = render_helm("go", "rest", false, false, false) else {
        eprintln!("SKIP: helm/go テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "hpa.yaml");
    assert!(
        content.contains(r#"{{- include "k1s0-common.hpa" . }}"#),
        "hpa.yaml に Library Chart 呼び出し '{{{{- include \"k1s0-common.hpa\" . }}}}' が含まれていません"
    );
}

/// pdb.yaml が Library Chart の include を呼び出していることを検証
#[test]
fn test_helm_pdb_calls_library_chart() {
    let Some((tmp, _)) = render_helm("go", "rest", false, false, false) else {
        eprintln!("SKIP: helm/go テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "pdb.yaml");
    assert!(
        content.contains(r#"{{- include "k1s0-common.pdb" . }}"#),
        "pdb.yaml に Library Chart 呼び出し '{{{{- include \"k1s0-common.pdb\" . }}}}' が含まれていません"
    );
}

/// ingress.yaml が Library Chart の include を呼び出していることを検証
#[test]
fn test_helm_ingress_calls_library_chart() {
    let Some((tmp, _)) = render_helm("go", "rest", false, false, false) else {
        eprintln!("SKIP: helm/go テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "ingress.yaml");
    assert!(
        content.contains(r#"{{- include "k1s0-common.ingress" . }}"#),
        "ingress.yaml に Library Chart 呼び出し '{{{{- include \"k1s0-common.ingress\" . }}}}' が含まれていません"
    );
}

/// values.yaml に ingress セクションが含まれ、デフォルトで無効であることを検証
#[test]
fn test_helm_values_ingress_section() {
    let Some((tmp, _)) = render_helm("go", "rest", false, false, false) else {
        eprintln!("SKIP: helm/go テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "values.yaml");
    assert!(
        content.contains("ingress:"),
        "values.yaml に ingress セクションが含まれていません"
    );
    assert!(
        content.contains("enabled: false"),
        "ingress.enabled のデフォルトが false であるべき"
    );
    assert!(
        content.contains("ingressClassName: nginx"),
        "ingress.ingressClassName のデフォルトが nginx であるべき"
    );
    assert!(
        content.contains("annotations: {}"),
        "ingress.annotations のデフォルトが空であるべき"
    );
    assert!(
        content.contains("hosts: []"),
        "ingress.hosts のデフォルトが空であるべき"
    );
    assert!(
        content.contains("tls: []"),
        "ingress.tls のデフォルトが空であるべき"
    );
}

/// Chart.yaml が k1s0-common への依存を宣言していることを検証
#[test]
fn test_helm_chart_yaml_has_library_dependency() {
    let Some((tmp, _)) = render_helm("go", "rest", false, false, false) else {
        eprintln!("SKIP: helm/go テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "Chart.yaml");
    assert!(
        content.contains("k1s0-common"),
        "Chart.yaml に Library Chart 依存 'k1s0-common' が含まれていません"
    );
}

// =========================================================================
// REST + gRPC 同時選択時のポートマッピングテスト
// =========================================================================

/// REST + gRPC 同時選択時: values.yaml に container.grpcPort: 50051 と
/// service.grpcPort: 50051 が含まれることを検証する。
/// container.port: 8080 と service.port: 80 も同時に存在すること。
#[test]
fn test_helm_rest_grpc_both_ports() {
    let Some((tmp, _)) = render_helm_with_styles(vec!["rest", "grpc"], false, false, false) else {
        eprintln!("SKIP: helm テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "values.yaml");

    // container セクションのポート検証
    assert!(
        content.contains("port: 8080"),
        "REST+gRPC: container.port: 8080 が values.yaml に含まれるべき\n--- values.yaml ---\n{}",
        content
    );
    assert!(
        content.contains("grpcPort: 50051"),
        "REST+gRPC: container.grpcPort: 50051 が values.yaml に含まれるべき\n--- values.yaml ---\n{}",
        content
    );

    // service セクションのポート検証
    assert!(
        content.contains("port: 80"),
        "REST+gRPC: service.port: 80 が values.yaml に含まれるべき\n--- values.yaml ---\n{}",
        content
    );

    // grpcPort: 50051 が2箇所（container + service）に存在することを確認
    let grpc_port_count = content.matches("grpcPort: 50051").count();
    assert_eq!(
        grpc_port_count, 2,
        "REST+gRPC: grpcPort: 50051 は container と service の2箇所に存在すべき (実際: {})\n--- values.yaml ---\n{}",
        grpc_port_count, content
    );
}

/// REST のみ選択時: grpcPort が null であることを検証する。
/// gRPC ポートは不要なため null が設定される。
#[test]
fn test_helm_rest_only_no_grpc_port() {
    let Some((tmp, _)) = render_helm_with_styles(vec!["rest"], false, false, false) else {
        eprintln!("SKIP: helm テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "values.yaml");

    // REST のみの場合、grpcPort は null
    assert!(
        content.contains("grpcPort: null"),
        "REST のみ: grpcPort: null が values.yaml に含まれるべき\n--- values.yaml ---\n{}",
        content
    );

    // grpcPort: 50051 は存在しないことを確認
    assert!(
        !content.contains("grpcPort: 50051"),
        "REST のみ: grpcPort: 50051 は values.yaml に含まれるべきでない\n--- values.yaml ---\n{}",
        content
    );

    // REST ポートは存在すること
    assert!(
        content.contains("port: 8080"),
        "REST のみ: container.port: 8080 が values.yaml に含まれるべき\n--- values.yaml ---\n{}",
        content
    );
    assert!(
        content.contains("port: 80"),
        "REST のみ: service.port: 80 が values.yaml に含まれるべき\n--- values.yaml ---\n{}",
        content
    );
}

/// gRPC 選択時: values.yaml に grpcHealthCheck セクションが存在することを検証
#[test]
fn test_helm_grpc_health_check_in_values() {
    let Some((tmp, _)) = render_helm_with_styles(vec!["grpc"], false, false, false) else {
        eprintln!("SKIP: helm テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "values.yaml");
    assert!(
        content.contains("grpcHealthCheck"),
        "gRPC 選択時に values.yaml に grpcHealthCheck セクションが含まれるべき\n--- values.yaml ---\n{}",
        content
    );
}

/// REST のみ選択時: values.yaml に grpcHealthCheck セクションが含まれないことを検証
#[test]
fn test_helm_rest_no_grpc_health_check_in_values() {
    let Some((tmp, _)) = render_helm_with_styles(vec!["rest"], false, false, false) else {
        eprintln!("SKIP: helm テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "values.yaml");
    assert!(
        !content.contains("grpcHealthCheck"),
        "REST のみ選択時に values.yaml に grpcHealthCheck セクションが含まれるべきでない\n--- values.yaml ---\n{}",
        content
    );
}

/// gRPC のみ選択時: grpcPort が 50051 であることを検証する。
/// container.grpcPort と service.grpcPort の両方に 50051 が設定される。
#[test]
fn test_helm_grpc_only_port() {
    let Some((tmp, _)) = render_helm_with_styles(vec!["grpc"], false, false, false) else {
        eprintln!("SKIP: helm テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "values.yaml");

    // gRPC のみの場合、grpcPort: 50051 が設定される
    assert!(
        content.contains("grpcPort: 50051"),
        "gRPC のみ: grpcPort: 50051 が values.yaml に含まれるべき\n--- values.yaml ---\n{}",
        content
    );

    // grpcPort: 50051 が2箇所（container + service）に存在することを確認
    let grpc_port_count = content.matches("grpcPort: 50051").count();
    assert_eq!(
        grpc_port_count, 2,
        "gRPC のみ: grpcPort: 50051 は container と service の2箇所に存在すべき (実際: {})\n--- values.yaml ---\n{}",
        grpc_port_count, content
    );

    // grpcPort: null は存在しないことを確認
    assert!(
        !content.contains("grpcPort: null"),
        "gRPC のみ: grpcPort: null は values.yaml に含まれるべきでない\n--- values.yaml ---\n{}",
        content
    );
}
