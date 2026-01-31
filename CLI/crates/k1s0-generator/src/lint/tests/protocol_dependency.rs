use super::super::*;
use super::{create_backend_rust_structure, create_test_manifest};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_rule_id_k026() {
    assert_eq!(RuleId::ProtocolDependencyInDomain.as_str(), "K026");
    assert_eq!(
        RuleId::ProtocolDependencyInDomain.description(),
        "Domain 層でプロトコル固有の型が使用されています"
    );
}

#[test]
fn test_protocol_dependency_in_domain_detected() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    let bad_code = r#"
pub fn check_status() -> StatusCode {
    StatusCode::OK
}
"#;
    fs::write(path.join("src/domain/mod.rs"), bad_code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    assert!(
        result
            .violations
            .iter()
            .any(|v| v.rule == RuleId::ProtocolDependencyInDomain),
        "Expected K026 violation, got {:?}",
        result.violations
    );
}

#[test]
fn test_protocol_dependency_in_presentation_not_detected() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // domain 層はクリーン
    fs::write(path.join("src/domain/mod.rs"), "pub struct User;").unwrap();

    // presentation 層では OK
    let presentation_code = r#"
use axum::http::StatusCode;

pub fn handler() -> StatusCode {
    StatusCode::OK
}
"#;
    fs::write(path.join("src/presentation/mod.rs"), presentation_code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    assert!(
        !result
            .violations
            .iter()
            .any(|v| v.rule == RuleId::ProtocolDependencyInDomain),
        "Unexpected K026 violation: {:?}",
        result.violations
    );
}

#[test]
fn test_protocol_dependency_in_comment_not_detected() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    let code_with_comment = r#"
// StatusCode::OK はここでは使わない
/// tonic::Status:: の例
pub struct DomainError;
"#;
    fs::write(path.join("src/domain/mod.rs"), code_with_comment).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    assert!(
        !result
            .violations
            .iter()
            .any(|v| v.rule == RuleId::ProtocolDependencyInDomain),
        "Unexpected K026 violation in comments: {:?}",
        result.violations
    );
}
