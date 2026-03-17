//! テストモジュール — unwrap() の使用を許可する
#![allow(clippy::unwrap_used)]
/// Golden Path コンパイル検証テスト。
///
/// テンプレートエンジンで生成した Rust サーバーコードが
/// 実際に `cargo check` を通ることを検証する。
use std::fs;
use std::path::Path;
use std::process::Command;

use k1s0_cli::template::context::TemplateContextBuilder;
use k1s0_cli::template::TemplateEngine;

fn template_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("templates")
}

fn generate_and_check(
    service_name: &str,
    tier: &str,
    api_style: &str,
    has_database: bool,
    database_type: &str,
    has_kafka: bool,
    has_redis: bool,
) {
    let tpl_dir = template_dir();
    // 生成コードはモノリポ内の正しい位置に配置しないと
    // system library への相対パスが解決できない
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent() // crates
        .unwrap()
        .parent() // CLI
        .unwrap()
        .parent() // repo root
        .unwrap();
    let output_dir = match tier {
        "service" => repo_root
            .join("regions/service")
            .join(service_name)
            .join("server/rust"),
        "business" => repo_root
            .join("regions/business/test-domain/server/rust")
            .join(service_name),
        _ => repo_root
            .join("regions/system/server/rust")
            .join(service_name),
    };

    let mut builder =
        TemplateContextBuilder::new(service_name, tier, "rust", "server").api_style(api_style);

    if has_database {
        builder = builder.with_database(database_type);
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

    assert!(!generated.is_empty(), "No files generated");

    // Verify Cargo.toml exists
    assert!(
        output_dir.join("Cargo.toml").exists(),
        "Cargo.toml not generated"
    );

    // Run cargo check
    let output = Command::new("cargo")
        .args(["check", "--message-format=short"])
        .current_dir(&output_dir)
        .output()
        .expect("failed to run cargo check");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Cleanup before assert so it always runs
    let success = output.status.success();
    // Clean up generated directory (go up to service name dir for service tier)
    let cleanup_dir = match tier {
        "service" => repo_root.join("regions/service").join(service_name),
        "business" => repo_root.join("regions/business/test-domain"),
        _ => output_dir.clone(),
    };
    let _ = fs::remove_dir_all(&cleanup_dir);

    assert!(
        success,
        "cargo check failed for {service_name} ({api_style}):\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}"
    );
}

#[test]
#[ignore = "requires network (cargo fetches dependencies)"]
fn test_golden_path_rust_rest_service() {
    generate_and_check(
        "gp-rest-svc",
        "service",
        "rest",
        true,
        "postgresql",
        false,
        false,
    );
}

// Note: gRPC テストは proto ファイル（api/proto/）が必要なため、
// テンプレート生成だけでは cargo check が通らない。
// gRPC の proto 生成は別途 buf generate で行うフロー。

#[test]
#[ignore = "requires network (cargo fetches dependencies)"]
fn test_golden_path_rust_rest_no_db() {
    generate_and_check("gp-rest-nodb", "service", "rest", false, "", false, false);
}

#[test]
#[ignore = "requires network (cargo fetches dependencies)"]
fn test_golden_path_rust_rest_full_stack() {
    generate_and_check(
        "gp-rest-full",
        "service",
        "rest",
        true,
        "postgresql",
        true,
        true,
    );
}

// Note: system tier テストは regions/system/Cargo.toml の workspace に
// 自動的に含まれてしまうため、独立テストとして実行できない。
// 実際の利用フローでは workspace.members に追加して使用する。
