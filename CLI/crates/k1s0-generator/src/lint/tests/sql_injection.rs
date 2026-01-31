use super::super::*;
use super::{create_backend_rust_structure, create_test_manifest};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_rule_id_k050() {
    assert_eq!(RuleId::SqlInjectionRisk.as_str(), "K050");
    assert_eq!(
        RuleId::SqlInjectionRisk.description(),
        "SQL インジェクションのリスクがあります"
    );
}

#[test]
fn test_sql_injection_format_detected() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    let bad_code = r#"
fn query_user(id: &str) -> String {
    format!("SELECT * FROM users WHERE id = {}", id)
}
"#;
    fs::write(path.join("src/infrastructure/mod.rs"), bad_code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    assert!(
        result
            .violations
            .iter()
            .any(|v| v.rule == RuleId::SqlInjectionRisk),
        "Expected K050 violation, got {:?}",
        result.violations
    );
}

#[test]
fn test_sql_injection_parameterized_query_no_violation() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    let clean_code = r#"
fn query_user(pool: &Pool, id: &str) {
    sqlx::query("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await;
}
"#;
    fs::write(path.join("src/infrastructure/mod.rs"), clean_code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    assert!(
        !result
            .violations
            .iter()
            .any(|v| v.rule == RuleId::SqlInjectionRisk),
        "Unexpected K050 violation: {:?}",
        result.violations
    );
}

#[test]
fn test_sql_injection_in_comment_not_detected() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    let code_with_comment = r#"
// format!("SELECT * FROM users WHERE id = {}", id) は危険
/// format!("DELETE FROM users WHERE id = {}", id) も同様
fn safe_query() {
    // パラメータバインドを使用
}
"#;
    fs::write(path.join("src/infrastructure/mod.rs"), code_with_comment).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    assert!(
        !result
            .violations
            .iter()
            .any(|v| v.rule == RuleId::SqlInjectionRisk),
        "Unexpected K050 violation in comments: {:?}",
        result.violations
    );
}

#[test]
fn test_sql_injection_multiple_statements() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    let bad_code = r#"
fn bad_queries(name: &str, id: &str) {
    let q1 = format!("INSERT INTO users (name) VALUES ({})", name);
    let q2 = format!("UPDATE users SET name = {} WHERE id = {}", name, id);
    let q3 = format!("DELETE FROM users WHERE id = {}", id);
}
"#;
    fs::write(path.join("src/infrastructure/mod.rs"), bad_code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    let sql_violations: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.rule == RuleId::SqlInjectionRisk)
        .collect();

    assert_eq!(
        sql_violations.len(),
        3,
        "Expected 3 K050 violations, got {:?}",
        sql_violations
    );
}
