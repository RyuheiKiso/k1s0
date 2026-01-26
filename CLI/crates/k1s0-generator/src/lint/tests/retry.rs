use super::super::*;
use super::{create_backend_rust_structure, create_test_manifest};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_rule_id_k030() {
    assert_eq!(RuleId::RetryUsageDetected.as_str(), "K030");
    assert_eq!(
        RuleId::RetryUsageDetected.description(),
        "gRPC リトライ設定が検出されました"
    );
}

#[test]
fn test_rule_id_k031() {
    assert_eq!(RuleId::RetryWithoutAdr.as_str(), "K031");
    assert_eq!(
        RuleId::RetryWithoutAdr.description(),
        "gRPC リトライ設定に ADR 参照がありません"
    );
}

#[test]
fn test_rule_id_k032() {
    assert_eq!(RuleId::RetryConfigIncomplete.as_str(), "K032");
    assert_eq!(
        RuleId::RetryConfigIncomplete.description(),
        "gRPC リトライ設定が不完全です"
    );
}

#[test]
fn test_lint_retry_usage_detected() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // リトライ設定を使用するコードを追加
    let code = r#"
use k1s0_grpc_client::config::RetryConfig;

fn create_client() {
    // ADR-001: このサービスは冪等なため、リトライを有効にする
    let retry = RetryConfig::enabled("ADR-001")
        .max_attempts(3)
        .build()
        .unwrap();
}
"#;
    fs::write(path.join("src/infrastructure/mod.rs"), code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    // K030 の警告が検出される
    assert!(
        result.violations.iter().any(|v| v.rule == RuleId::RetryUsageDetected),
        "Expected K030 warning, got {:?}",
        result.violations
    );
}

#[test]
fn test_lint_retry_without_adr() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // ADR 参照なしでリトライ設定を使用するコードを追加
    let code = r#"
use k1s0_grpc_client::config::RetryConfig;

fn create_client() {
    let retry = RetryConfig::enabled("no-adr")
        .max_attempts(3)
        .build()
        .unwrap();
}
"#;
    fs::write(path.join("src/infrastructure/mod.rs"), code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    // K031 のエラーが検出される
    assert!(
        result.violations.iter().any(|v| v.rule == RuleId::RetryWithoutAdr),
        "Expected K031 error, got {:?}",
        result.violations
    );
}

#[test]
fn test_lint_retry_with_adr_no_k031() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // 正しい ADR 参照付きのリトライ設定
    let code = r#"
use k1s0_grpc_client::config::RetryConfig;

fn create_client() {
    let retry = RetryConfig::enabled("ADR-001")
        .max_attempts(3)
        .build()
        .unwrap();
}
"#;
    fs::write(path.join("src/infrastructure/mod.rs"), code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    // K031 のエラーが検出されない
    assert!(
        !result.violations.iter().any(|v| v.rule == RuleId::RetryWithoutAdr),
        "Unexpected K031 error: {:?}",
        result.violations
    );
}

#[test]
fn test_lint_retry_with_adr_in_comment() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // コメントに ADR 参照があるリトライ設定
    let code = r#"
use k1s0_grpc_client::config::RetryConfig;

fn create_client() {
    // ADR-002: Get操作は冪等のためリトライ可
    let retry = RetryConfig::enabled("see-comment")
        .max_attempts(3)
        .build()
        .unwrap();
}
"#;
    fs::write(path.join("src/infrastructure/mod.rs"), code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    // K031 のエラーが検出されない（コメントに ADR 参照あり）
    assert!(
        !result.violations.iter().any(|v| v.rule == RuleId::RetryWithoutAdr),
        "Unexpected K031 error (ADR in comment): {:?}",
        result.violations
    );
}

#[test]
fn test_lint_retry_config_incomplete() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // build() を呼ばないリトライ設定
    let code = r#"
use k1s0_grpc_client::config::RetryConfig;

fn create_client() {
    let retry = RetryConfig::enabled("ADR-001");
    // .build() を呼んでいない
}
"#;
    fs::write(path.join("src/infrastructure/mod.rs"), code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    // K032 のエラーが検出される
    assert!(
        result.violations.iter().any(|v| v.rule == RuleId::RetryConfigIncomplete),
        "Expected K032 error, got {:?}",
        result.violations
    );
}

#[test]
fn test_lint_retry_config_complete() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // 完全なリトライ設定
    let code = r#"
use k1s0_grpc_client::config::RetryConfig;

fn create_client() {
    let retry = RetryConfig::enabled("ADR-001")
        .max_attempts(3)
        .initial_backoff_ms(100)
        .max_backoff_ms(5000)
        .build()
        .unwrap();
}
"#;
    fs::write(path.join("src/infrastructure/mod.rs"), code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    // K032 のエラーが検出されない
    assert!(
        !result.violations.iter().any(|v| v.rule == RuleId::RetryConfigIncomplete),
        "Unexpected K032 error: {:?}",
        result.violations
    );
}

#[test]
fn test_lint_retry_no_violation_when_disabled() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // RetryConfig::disabled() を使用（違反なし）
    let code = r#"
use k1s0_grpc_client::config::RetryConfig;

fn create_client() {
    let retry = RetryConfig::disabled();
}
"#;
    fs::write(path.join("src/infrastructure/mod.rs"), code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    // リトライ関連の違反が検出されない
    assert!(
        !result.violations.iter().any(|v| matches!(
            v.rule,
            RuleId::RetryUsageDetected | RuleId::RetryWithoutAdr | RuleId::RetryConfigIncomplete
        )),
        "Unexpected retry violation for disabled retry: {:?}",
        result.violations
    );
}

#[test]
fn test_lint_retry_exclude_k030() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // リトライ設定を使用するコードを追加
    let code = r#"
use k1s0_grpc_client::config::RetryConfig;

fn create_client() {
    let retry = RetryConfig::enabled("ADR-001")
        .max_attempts(3)
        .build()
        .unwrap();
}
"#;
    fs::write(path.join("src/infrastructure/mod.rs"), code).unwrap();

    // K030 を除外
    let config = LintConfig {
        rules: None,
        exclude_rules: vec!["K030".to_string()],
        strict: false,
        env_var_allowlist: vec![],
        fix: false,
    };
    let linter = Linter::new(config);
    let result = linter.lint(path);

    // K030 の違反が検出されない
    assert!(
        !result.violations.iter().any(|v| v.rule == RuleId::RetryUsageDetected),
        "Unexpected K030 warning (rule should be excluded): {:?}",
        result.violations
    );
}

#[test]
fn test_lint_retry_line_number() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // リトライ設定（6行目）
    let code = "// comment\n// comment\n// comment\n// comment\n// comment\nlet retry = RetryConfig::enabled(\"ADR-001\").build();\n";
    fs::write(path.join("src/infrastructure/mod.rs"), code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    // K030 の違反が検出され、行番号が正しい
    let retry_violation = result
        .violations
        .iter()
        .find(|v| v.rule == RuleId::RetryUsageDetected)
        .unwrap();
    assert_eq!(retry_violation.line, Some(6));
}
