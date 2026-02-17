/// Service Mesh (Istio) テンプレートのレンダリング統合テスト。
///
/// CLI/templates/service-mesh/ テンプレートファイルを使用し、
/// テンプレートエンジンでレンダリングした結果が仕様書と一致することを検証する。
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

/// service-mesh テンプレートをレンダリングする。
///
/// service-mesh テンプレートは言語サブディレクトリを持たないフラット構造のため、
/// テンプレートディレクトリは templates/service-mesh/ を直接参照する。
fn render_service_mesh(
    service_name: &str,
    tier: &str,
    api_style: &str,
    server_port: u16,
    grpc_port: u16,
) -> Option<(TempDir, Vec<String>)> {
    let tpl_dir = template_dir();

    // service-mesh テンプレートディレクトリの存在チェック
    let mesh_dir = tpl_dir.join("service-mesh");
    if !mesh_dir.exists() {
        return None;
    }

    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let ctx = TemplateContextBuilder::new(service_name, tier, "go", "service-mesh")
        .api_style(api_style)
        .server_port(server_port)
        .grpc_port(grpc_port)
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

/// service-mesh テンプレートを kind を上書きしてレンダリングする。
///
/// テンプレートディレクトリは templates/service-mesh/ を使うが、
/// コンテキストの kind は kind_override を使う。
/// AuthorizationPolicy の BFF deny テストで使用する。
///
/// TemplateEngine の tera フィールドは pub(crate) のため、
/// 直接 tera::Tera を使用してテンプレートを登録・レンダリングする。
fn render_service_mesh_with_kind(
    service_name: &str,
    tier: &str,
    api_style: &str,
    server_port: u16,
    grpc_port: u16,
    kind_override: &str,
) -> Option<(TempDir, Vec<String>)> {
    let tpl_dir = template_dir();

    // service-mesh テンプレートディレクトリの存在チェック
    let mesh_dir = tpl_dir.join("service-mesh");
    if !mesh_dir.exists() {
        return None;
    }

    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    // kind="service-mesh" でビルダーを作成してコンテキストを生成
    let base_ctx = TemplateContextBuilder::new(service_name, tier, "go", "service-mesh")
        .api_style(api_style)
        .server_port(server_port)
        .grpc_port(grpc_port)
        .build();

    // Tera コンテキストを生成し、kind を上書き
    let mut tera_ctx = base_ctx.to_tera_context();
    tera_ctx.insert("kind", kind_override);

    // Tera エンジンを直接使用してテンプレートを登録・レンダリング
    let mut tera = tera::Tera::default();

    let mut generated_files = Vec::new();
    let mesh_template_dir = tpl_dir.join("service-mesh");

    for entry in walkdir::WalkDir::new(&mesh_template_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str());
        if ext != Some("tera") {
            continue;
        }

        let relative = path
            .strip_prefix(&mesh_template_dir)
            .unwrap()
            .to_path_buf();
        let template_name = relative.to_string_lossy().replace('\\', "/");

        let content = fs::read_to_string(path).unwrap();
        tera.add_raw_template(&template_name, &content).unwrap();

        let rendered = tera.render(&template_name, &tera_ctx).unwrap();

        // .tera 拡張子を除去
        let output_name = template_name.trim_end_matches(".tera");
        let output_path = output_dir.join(output_name);

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        fs::write(&output_path, rendered).unwrap();
        generated_files.push(output_path);
    }

    let names: Vec<String> = generated_files
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
fn test_service_mesh_file_list() {
    let Some((_, names)) = render_service_mesh("auth-service", "system", "grpc", 80, 9090) else {
        eprintln!("SKIP: service-mesh テンプレートディレクトリが未作成");
        return;
    };

    // 4ファイルの存在確認
    assert!(
        names.iter().any(|n| n.contains("virtual-service.yaml")),
        "virtual-service.yaml missing. Generated: {:?}",
        names
    );
    assert!(
        names.iter().any(|n| n.contains("destination-rule.yaml")),
        "destination-rule.yaml missing. Generated: {:?}",
        names
    );
    assert!(
        names.iter().any(|n| n.contains("peer-authentication.yaml")),
        "peer-authentication.yaml missing. Generated: {:?}",
        names
    );
    assert!(
        names.iter().any(|n| n.contains("authorization-policy.yaml")),
        "authorization-policy.yaml missing. Generated: {:?}",
        names
    );
}

// =========================================================================
// VirtualService テスト
// =========================================================================

#[test]
fn test_virtual_service_system_timeout() {
    let Some((tmp, _)) = render_service_mesh("auth-service", "system", "grpc", 80, 9090) else {
        eprintln!("SKIP: service-mesh テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "virtual-service.yaml");
    assert!(
        content.contains("timeout: 5s"),
        "system tier should have timeout: 5s\n--- virtual-service.yaml ---\n{}",
        content
    );
}

#[test]
fn test_virtual_service_business_timeout() {
    let Some((tmp, _)) = render_service_mesh("order-api", "business", "rest", 80, 9090) else {
        eprintln!("SKIP: service-mesh テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "virtual-service.yaml");
    assert!(
        content.contains("timeout: 10s"),
        "business tier should have timeout: 10s\n--- virtual-service.yaml ---\n{}",
        content
    );
}

#[test]
fn test_virtual_service_service_timeout() {
    let Some((tmp, _)) = render_service_mesh("order-api", "service", "rest", 80, 9090) else {
        eprintln!("SKIP: service-mesh テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "virtual-service.yaml");
    assert!(
        content.contains("timeout: 15s"),
        "service tier should have timeout: 15s\n--- virtual-service.yaml ---\n{}",
        content
    );
}

#[test]
fn test_virtual_service_grpc_route() {
    let Some((tmp, _)) = render_service_mesh("auth-service", "system", "grpc", 80, 9090) else {
        eprintln!("SKIP: service-mesh テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "virtual-service.yaml");
    // gRPC 固有ルートが含まれる
    assert!(
        content.contains("port: 9090"),
        "gRPC route should contain port: 9090\n--- virtual-service.yaml ---\n{}",
        content
    );
    assert!(
        content.contains("cancelled,deadline-exceeded,internal,resource-exhausted,unavailable"),
        "gRPC route should have gRPC-specific retryOn\n--- virtual-service.yaml ---\n{}",
        content
    );
}

#[test]
fn test_virtual_service_no_grpc_route() {
    let Some((tmp, _)) = render_service_mesh("order-api", "service", "rest", 80, 9090) else {
        eprintln!("SKIP: service-mesh テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "virtual-service.yaml");
    // REST のみの場合、gRPC ルートが含まれない
    assert!(
        !content.contains("cancelled,deadline-exceeded,internal,resource-exhausted,unavailable"),
        "REST-only should NOT have gRPC-specific retryOn\n--- virtual-service.yaml ---\n{}",
        content
    );
}

// =========================================================================
// DestinationRule テスト
// =========================================================================

#[test]
fn test_destination_rule_system_connections() {
    let Some((tmp, _)) = render_service_mesh("auth-service", "system", "grpc", 80, 9090) else {
        eprintln!("SKIP: service-mesh テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "destination-rule.yaml");
    assert!(
        content.contains("maxConnections: 200"),
        "system tier should have maxConnections: 200\n--- destination-rule.yaml ---\n{}",
        content
    );
}

#[test]
fn test_destination_rule_service_connections() {
    let Some((tmp, _)) = render_service_mesh("order-api", "service", "rest", 80, 9090) else {
        eprintln!("SKIP: service-mesh テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "destination-rule.yaml");
    assert!(
        content.contains("maxConnections: 100"),
        "service tier should have maxConnections: 100\n--- destination-rule.yaml ---\n{}",
        content
    );
}

#[test]
fn test_destination_rule_grpc_h2_upgrade() {
    let Some((tmp, _)) = render_service_mesh("auth-service", "system", "grpc", 80, 9090) else {
        eprintln!("SKIP: service-mesh テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "destination-rule.yaml");
    assert!(
        content.contains("h2UpgradePolicy: UPGRADE"),
        "gRPC should have h2UpgradePolicy: UPGRADE\n--- destination-rule.yaml ---\n{}",
        content
    );
}

#[test]
fn test_destination_rule_no_h2_upgrade() {
    let Some((tmp, _)) = render_service_mesh("order-api", "service", "rest", 80, 9090) else {
        eprintln!("SKIP: service-mesh テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "destination-rule.yaml");
    assert!(
        !content.contains("h2UpgradePolicy"),
        "REST-only should NOT have h2UpgradePolicy\n--- destination-rule.yaml ---\n{}",
        content
    );
}

// =========================================================================
// PeerAuthentication テスト
// =========================================================================

#[test]
fn test_peer_auth_strict() {
    let Some((tmp, _)) = render_service_mesh("auth-service", "system", "grpc", 80, 9090) else {
        eprintln!("SKIP: service-mesh テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "peer-authentication.yaml");
    assert!(
        content.contains("mode: STRICT"),
        "PeerAuthentication should have mode: STRICT\n--- peer-authentication.yaml ---\n{}",
        content
    );
}

#[test]
fn test_peer_auth_grpc_port_level() {
    let Some((tmp, _)) = render_service_mesh("auth-service", "system", "grpc", 80, 9090) else {
        eprintln!("SKIP: service-mesh テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "peer-authentication.yaml");
    assert!(
        content.contains("portLevelMtls"),
        "gRPC should have portLevelMtls\n--- peer-authentication.yaml ---\n{}",
        content
    );
}

// =========================================================================
// AuthorizationPolicy テスト
// =========================================================================

#[test]
fn test_authorization_policy_system() {
    let Some((tmp, _)) = render_service_mesh("auth-service", "system", "grpc", 80, 9090) else {
        eprintln!("SKIP: service-mesh テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "authorization-policy.yaml");
    // system 層: business・service からのアクセス許可
    assert!(
        content.contains("k1s0-business"),
        "system tier should allow access from k1s0-business\n--- authorization-policy.yaml ---\n{}",
        content
    );
    assert!(
        content.contains("k1s0-service"),
        "system tier should allow access from k1s0-service\n--- authorization-policy.yaml ---\n{}",
        content
    );
}

#[test]
fn test_authorization_policy_service() {
    let Some((tmp, _)) = render_service_mesh("order-api", "service", "rest", 80, 9090) else {
        eprintln!("SKIP: service-mesh テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "authorization-policy.yaml");
    // service 層: ingress・同一 Tier からのアクセス許可
    assert!(
        content.contains("ingress"),
        "service tier should allow access from ingress\n--- authorization-policy.yaml ---\n{}",
        content
    );
    assert!(
        content.contains("k1s0-service"),
        "service tier should allow access from k1s0-service\n--- authorization-policy.yaml ---\n{}",
        content
    );
}

#[test]
fn test_authorization_policy_bff_deny() {
    let Some((tmp, _)) =
        render_service_mesh_with_kind("order-bff", "service", "rest", 80, 9090, "bff")
    else {
        eprintln!("SKIP: service-mesh テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "authorization-policy.yaml");
    // BFF deny ポリシーが含まれる
    assert!(
        content.contains("deny-bff"),
        "BFF kind should include deny-bff policy\n--- authorization-policy.yaml ---\n{}",
        content
    );
    assert!(
        content.contains("action: DENY"),
        "BFF kind should include DENY action\n--- authorization-policy.yaml ---\n{}",
        content
    );
    assert!(
        content.contains("bff-sa"),
        "BFF deny policy should reference bff-sa principal\n--- authorization-policy.yaml ---\n{}",
        content
    );
}

// =========================================================================
// Tera 構文残留チェック
// =========================================================================

#[test]
fn test_service_mesh_no_tera_syntax() {
    let Some((tmp, names)) = render_service_mesh("auth-service", "system", "grpc", 80, 9090)
    else {
        eprintln!("SKIP: service-mesh テンプレートディレクトリが未作成");
        return;
    };

    for name in &names {
        let content = read_output(&tmp, name);
        assert!(
            !content.contains("{%"),
            "Tera syntax {{%% found in {}",
            name
        );
        assert!(
            !content.contains("{#"),
            "Tera comment {{# found in {}",
            name
        );
    }
}
