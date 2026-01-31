use super::super::*;
use super::{create_backend_rust_structure, create_test_manifest};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_rule_id_k020() {
    assert_eq!(RuleId::EnvVarUsage.as_str(), "K020");
    assert_eq!(
        RuleId::EnvVarUsage.description(),
        "環境変数の参照は禁止されています"
    );
}

#[test]
fn test_lint_env_var_usage_detected() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // 環境変数を使用するコードを追加
    let bad_code = r#"
use std::env;

fn main() {
    let value = std::env::var("MY_VAR").unwrap();
    println!("{}", value);
}
"#;
    fs::write(path.join("src/main.rs"), bad_code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    // K020 の違反が検出される
    assert!(
        result.violations.iter().any(|v| v.rule == RuleId::EnvVarUsage),
        "Expected K020 violation, got {:?}",
        result.violations
    );

    // ヒントが含まれている
    let env_var_violation = result
        .violations
        .iter()
        .find(|v| v.rule == RuleId::EnvVarUsage)
        .unwrap();
    assert!(env_var_violation.hint.is_some());
    assert!(env_var_violation.hint.as_ref().unwrap().contains("config"));
}

#[test]
fn test_lint_env_var_usage_not_detected_when_clean() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // 環境変数を使用しないコードを追加
    let clean_code = r#"
fn main() {
    println!("Hello, world!");
}
"#;
    fs::write(path.join("src/main.rs"), clean_code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    // K020 の違反が検出されない
    assert!(
        !result.violations.iter().any(|v| v.rule == RuleId::EnvVarUsage),
        "Unexpected K020 violation: {:?}",
        result.violations
    );
}

#[test]
fn test_lint_env_var_allowlist() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // 環境変数を使用するコードを追加
    let bad_code = r#"
use std::env;

fn main() {
    let value = std::env::var("MY_VAR").unwrap();
    println!("{}", value);
}
"#;
    fs::write(path.join("src/main.rs"), bad_code).unwrap();

    // allowlist に追加
    let config = LintConfig {
        rules: None,
        exclude_rules: vec![],
        strict: false,
        env_var_allowlist: vec!["src/main.rs".to_string()],
        fix: false,
        fast: false,
    };
    let linter = Linter::new(config);
    let result = linter.lint(path);

    // allowlist に含まれているので K020 の違反が検出されない
    assert!(
        !result.violations.iter().any(|v| v.rule == RuleId::EnvVarUsage),
        "Unexpected K020 violation (should be allowlisted): {:?}",
        result.violations
    );
}

#[test]
fn test_lint_env_var_allowlist_wildcard() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // 環境変数を使用するコードを追加
    let bad_code = r#"
fn get_config() {
    let value = env::var("CONFIG_VAR").unwrap();
}
"#;
    fs::write(path.join("src/infrastructure/mod.rs"), bad_code).unwrap();

    // ワイルドカード allowlist
    let config = LintConfig {
        rules: None,
        exclude_rules: vec![],
        strict: false,
        env_var_allowlist: vec!["src/infrastructure/*".to_string()],
        fix: false,
        fast: false,
    };
    let linter = Linter::new(config);
    let result = linter.lint(path);

    // allowlist に含まれているので K020 の違反が検出されない
    assert!(
        !result.violations.iter().any(|v| v.rule == RuleId::EnvVarUsage),
        "Unexpected K020 violation (should be allowlisted): {:?}",
        result.violations
    );
}

#[test]
fn test_lint_env_var_dotenv_detected() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // dotenv を使用するコードを追加
    let bad_code = r#"
use dotenv::dotenv;

fn main() {
    dotenv().ok();
}
"#;
    fs::write(path.join("src/main.rs"), bad_code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    // dotenv が検出される
    let dotenv_violation = result
        .violations
        .iter()
        .find(|v| v.rule == RuleId::EnvVarUsage && v.message.contains("dotenv"));
    assert!(
        dotenv_violation.is_some(),
        "Expected dotenv violation, got {:?}",
        result.violations
    );
}

#[test]
fn test_lint_env_var_line_number() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // 環境変数を使用するコードを追加（3行目）
    let bad_code = "fn main() {\n    // comment\n    let x = std::env::var(\"X\").unwrap();\n}\n";
    fs::write(path.join("src/main.rs"), bad_code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    // K020 の違反が検出され、行番号が正しい
    let env_var_violation = result
        .violations
        .iter()
        .find(|v| v.rule == RuleId::EnvVarUsage)
        .unwrap();
    assert_eq!(env_var_violation.line, Some(3));
}

#[test]
fn test_lint_env_var_exclude_rule() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // 環境変数を使用するコードを追加
    let bad_code = r#"
fn main() {
    let value = std::env::var("MY_VAR").unwrap();
}
"#;
    fs::write(path.join("src/main.rs"), bad_code).unwrap();

    // K020 を除外
    let config = LintConfig {
        rules: None,
        exclude_rules: vec!["K020".to_string()],
        strict: false,
        env_var_allowlist: vec![],
        fix: false,
        fast: false,
    };
    let linter = Linter::new(config);
    let result = linter.lint(path);

    // K020 の違反が検出されない
    assert!(
        !result.violations.iter().any(|v| v.rule == RuleId::EnvVarUsage),
        "Unexpected K020 violation (rule should be excluded): {:?}",
        result.violations
    );
}

#[test]
fn test_env_var_patterns_rust() {
    let patterns = EnvVarPatterns::rust();
    assert!(patterns.file_extensions.contains(&"rs"));
    assert!(patterns.patterns.iter().any(|p| p.pattern == "std::env::var"));
    assert!(patterns.patterns.iter().any(|p| p.pattern == "dotenv"));
}

#[test]
fn test_env_var_patterns_go() {
    let patterns = EnvVarPatterns::go();
    assert!(patterns.file_extensions.contains(&"go"));
    assert!(patterns.patterns.iter().any(|p| p.pattern == "os.Getenv"));
}

#[test]
fn test_env_var_patterns_typescript() {
    let patterns = EnvVarPatterns::typescript();
    assert!(patterns.file_extensions.contains(&"ts"));
    assert!(patterns.file_extensions.contains(&"tsx"));
    assert!(patterns.patterns.iter().any(|p| p.pattern == "process.env"));
}

#[test]
fn test_env_var_patterns_dart() {
    let patterns = EnvVarPatterns::dart();
    assert!(patterns.file_extensions.contains(&"dart"));
    assert!(patterns.patterns.iter().any(|p| p.pattern == "Platform.environment"));
}
