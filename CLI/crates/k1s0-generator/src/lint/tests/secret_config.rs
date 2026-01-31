use super::super::*;
use super::{create_backend_rust_structure, create_test_manifest};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_rule_id_k021() {
    assert_eq!(RuleId::SecretInConfig.as_str(), "K021");
    assert_eq!(
        RuleId::SecretInConfig.description(),
        "config YAML に機密情報が直接書かれています"
    );
}

#[test]
fn test_lint_secret_in_config_detected() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // 機密情報を直接書いた config を作成
    let bad_config = r#"
db:
  host: localhost
  port: 5432
  user: myuser
  password: mysecretpassword123
"#;
    fs::write(path.join("config/default.yaml"), bad_config).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    // K021 の違反が検出される
    assert!(
        result.violations.iter().any(|v| v.rule == RuleId::SecretInConfig),
        "Expected K021 violation, got {:?}",
        result.violations
    );

    // ヒントが含まれている
    let secret_violation = result
        .violations
        .iter()
        .find(|v| v.rule == RuleId::SecretInConfig)
        .unwrap();
    assert!(secret_violation.hint.is_some());
    assert!(secret_violation.hint.as_ref().unwrap().contains("_file"));
}

#[test]
fn test_lint_secret_in_config_file_suffix_allowed() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // *_file サフィックスを使った正しい config を作成
    let good_config = r#"
db:
  host: localhost
  port: 5432
  user: myuser
  password_file: /var/run/secrets/k1s0/db_password
auth:
  jwt_private_key_file: /var/run/secrets/k1s0/jwt_private_key.pem
  jwt_public_key_file: /var/run/secrets/k1s0/jwt_public_key.pem
"#;
    fs::write(path.join("config/default.yaml"), good_config).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    // K021 の違反が検出されない
    assert!(
        !result.violations.iter().any(|v| v.rule == RuleId::SecretInConfig),
        "Unexpected K021 violation: {:?}",
        result.violations
    );
}

#[test]
fn test_lint_secret_in_config_token_detected() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // token を直接書いた config を作成
    let bad_config = r#"
integrations:
  github:
    api_token: ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
"#;
    fs::write(path.join("config/dev.yaml"), bad_config).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    // K021 の違反が検出される
    assert!(
        result.violations.iter().any(|v| v.rule == RuleId::SecretInConfig),
        "Expected K021 violation for token, got {:?}",
        result.violations
    );
}

#[test]
fn test_lint_secret_in_config_empty_value_allowed() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // 空値やnullは許可
    let config = r#"
db:
  password: null
  secret: ~
  token:
"#;
    fs::write(path.join("config/default.yaml"), config).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    // K021 の違反が検出されない
    assert!(
        !result.violations.iter().any(|v| v.rule == RuleId::SecretInConfig),
        "Unexpected K021 violation for empty/null values: {:?}",
        result.violations
    );
}

#[test]
fn test_lint_secret_in_config_exclude_rule() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // 機密情報を直接書いた config を作成
    let bad_config = r#"
db:
  password: mysecretpassword123
"#;
    fs::write(path.join("config/default.yaml"), bad_config).unwrap();

    // K021 を除外
    let config = LintConfig {
        rules: None,
        exclude_rules: vec!["K021".to_string()],
        strict: false,
        env_var_allowlist: vec![],
        fix: false,
        fast: false,
    };
    let linter = Linter::new(config);
    let result = linter.lint(path);

    // K021 の違反が検出されない
    assert!(
        !result.violations.iter().any(|v| v.rule == RuleId::SecretInConfig),
        "Unexpected K021 violation (rule should be excluded): {:?}",
        result.violations
    );
}

#[test]
fn test_secret_key_patterns_matches() {
    let patterns = SecretKeyPatterns::default();

    // マッチするケース
    assert!(patterns.matches_secret_key("password").is_some());
    assert!(patterns.matches_secret_key("db_password").is_some());
    assert!(patterns.matches_secret_key("api_token").is_some());
    assert!(patterns.matches_secret_key("secret_key").is_some());
    assert!(patterns.matches_secret_key("jwt_private_key").is_some());
    assert!(patterns.matches_secret_key("client_secret").is_some());

    // マッチしないケース
    assert!(patterns.matches_secret_key("host").is_none());
    assert!(patterns.matches_secret_key("port").is_none());
    assert!(patterns.matches_secret_key("database_name").is_none());
    assert!(patterns.matches_secret_key("timeout").is_none());
}

#[test]
fn test_lint_secret_in_config_line_number() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // 機密情報を直接書いた config を作成（5行目）
    let bad_config = "# config\ndb:\n  host: localhost\n  port: 5432\n  password: secret123\n";
    fs::write(path.join("config/default.yaml"), bad_config).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    // K021 の違反が検出され、行番号が正しい
    let secret_violation = result
        .violations
        .iter()
        .find(|v| v.rule == RuleId::SecretInConfig)
        .unwrap();
    assert_eq!(secret_violation.line, Some(5));
}
