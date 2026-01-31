use super::super::*;
use super::{create_backend_rust_structure, create_test_manifest};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_rule_id_k053() {
    assert_eq!(RuleId::LoggingSensitiveData.as_str(), "K053");
}

#[test]
fn test_password_in_log_detected() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    let bad_code = r#"
fn login(user: &str, password: &str) {
    tracing::info!("Login attempt: user={}, password={}", user, password);
}
"#;
    fs::write(path.join("src/presentation/mod.rs"), bad_code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    assert!(
        result
            .violations
            .iter()
            .any(|v| v.rule == RuleId::LoggingSensitiveData),
        "Expected K053 violation for password in log",
    );
}

#[test]
fn test_token_in_log_detected() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    let bad_code = r#"
fn refresh(token: &str) {
    info!("Refreshing token={}", token);
}
"#;
    fs::write(path.join("src/application/mod.rs"), bad_code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    assert!(
        result
            .violations
            .iter()
            .any(|v| v.rule == RuleId::LoggingSensitiveData),
    );
}

#[test]
fn test_password_hash_excluded() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    let safe_code = r#"
fn log_user(password_hash: &str) {
    tracing::info!("User password_hash={}", password_hash);
}
"#;
    fs::write(path.join("src/application/mod.rs"), safe_code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    assert!(
        !result
            .violations
            .iter()
            .any(|v| v.rule == RuleId::LoggingSensitiveData),
        "password_hash should not trigger K053",
    );
}

#[test]
fn test_no_log_function_no_violation() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    let safe_code = r#"
fn validate_password(password: &str) -> bool {
    password.len() >= 8
}
"#;
    fs::write(path.join("src/domain/mod.rs"), safe_code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    assert!(
        !result
            .violations
            .iter()
            .any(|v| v.rule == RuleId::LoggingSensitiveData),
        "No log function means no K053 violation",
    );
}

#[test]
fn test_comment_line_excluded() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    let code = r#"
// tracing::info!("password={}", password);
fn safe() {}
"#;
    fs::write(path.join("src/domain/mod.rs"), code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    assert!(
        !result
            .violations
            .iter()
            .any(|v| v.rule == RuleId::LoggingSensitiveData),
    );
}
