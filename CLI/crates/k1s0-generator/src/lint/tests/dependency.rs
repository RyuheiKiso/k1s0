use super::super::*;
use super::{create_backend_rust_structure, create_test_manifest};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_rule_id_k022() {
    assert_eq!(RuleId::DependencyDirection.as_str(), "K022");
    assert_eq!(
        RuleId::DependencyDirection.description(),
        "Clean Architecture の依存方向に違反しています"
    );
}

#[test]
fn test_lint_dependency_direction_domain_to_infrastructure() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // domain から infrastructure への違反コードを追加
    let bad_code = r#"
// domain layer should not depend on infrastructure
use crate::infrastructure::db::UserRepository;

pub struct User {
    pub id: String,
    pub name: String,
}
"#;
    fs::write(path.join("src/domain/mod.rs"), bad_code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    // K022 の違反が検出される
    assert!(
        result.violations.iter().any(|v| v.rule == RuleId::DependencyDirection),
        "Expected K022 violation, got {:?}",
        result.violations
    );

    // ヒントが含まれている
    let dep_violation = result
        .violations
        .iter()
        .find(|v| v.rule == RuleId::DependencyDirection)
        .unwrap();
    assert!(dep_violation.hint.is_some());
    assert!(dep_violation.hint.as_ref().unwrap().contains("依存"));
}

#[test]
fn test_lint_dependency_direction_domain_to_application() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // domain から application への違反コードを追加
    let bad_code = r#"
// domain layer should not depend on application
use crate::application::services::UserService;

pub struct User {
    pub id: String,
}
"#;
    fs::write(path.join("src/domain/mod.rs"), bad_code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    // K022 の違反が検出される
    assert!(
        result.violations.iter().any(|v| v.rule == RuleId::DependencyDirection),
        "Expected K022 violation for domain->application, got {:?}",
        result.violations
    );
}

#[test]
fn test_lint_dependency_direction_application_to_infrastructure() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // application から infrastructure への違反コードを追加
    let bad_code = r#"
// application layer should not depend on infrastructure directly
use crate::infrastructure::db::UserRepositoryImpl;

pub struct UserService {}
"#;
    fs::write(path.join("src/application/mod.rs"), bad_code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    // K022 の違反が検出される
    assert!(
        result.violations.iter().any(|v| v.rule == RuleId::DependencyDirection),
        "Expected K022 violation for application->infrastructure, got {:?}",
        result.violations
    );
}

#[test]
fn test_lint_dependency_direction_no_violation() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // 正しい依存関係のコード
    let domain_code = r#"
// domain layer - no external dependencies
pub struct User {
    pub id: String,
    pub name: String,
}

pub trait UserRepository {
    fn find_by_id(&self, id: &str) -> Option<User>;
}
"#;
    fs::write(path.join("src/domain/mod.rs"), domain_code).unwrap();

    let application_code = r#"
// application layer - depends only on domain
use crate::domain::User;
use crate::domain::UserRepository;

pub struct UserService<R: UserRepository> {
    repository: R,
}
"#;
    fs::write(path.join("src/application/mod.rs"), application_code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    // K022 の違反が検出されない
    assert!(
        !result.violations.iter().any(|v| v.rule == RuleId::DependencyDirection),
        "Unexpected K022 violation: {:?}",
        result.violations
    );
}

#[test]
fn test_lint_dependency_direction_exclude_rule() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // domain から infrastructure への違反コードを追加
    let bad_code = r#"
use crate::infrastructure::db::UserRepository;
pub struct User {}
"#;
    fs::write(path.join("src/domain/mod.rs"), bad_code).unwrap();

    // K022 を除外
    let config = LintConfig {
        rules: None,
        exclude_rules: vec!["K022".to_string()],
        strict: false,
        env_var_allowlist: vec![],
        fix: false,
    };
    let linter = Linter::new(config);
    let result = linter.lint(path);

    // K022 の違反が検出されない
    assert!(
        !result.violations.iter().any(|v| v.rule == RuleId::DependencyDirection),
        "Unexpected K022 violation (rule should be excluded): {:?}",
        result.violations
    );
}

#[test]
fn test_dependency_rules_rust() {
    let rules = DependencyRules::rust();
    assert!(rules.file_extensions.contains(&"rs"));
    assert!(rules.import_patterns.iter().any(|p| p.contains("use crate::")));
}

#[test]
fn test_dependency_rules_go() {
    let rules = DependencyRules::go();
    assert!(rules.file_extensions.contains(&"go"));
    assert!(rules.import_patterns.iter().any(|p| p.contains("internal/")));
}

#[test]
fn test_dependency_rules_typescript() {
    let rules = DependencyRules::typescript();
    assert!(rules.file_extensions.contains(&"ts"));
    assert!(rules.file_extensions.contains(&"tsx"));
    assert!(rules.import_patterns.iter().any(|p| p.contains("from '../")));
}

#[test]
fn test_dependency_rules_dart() {
    let rules = DependencyRules::dart();
    assert!(rules.file_extensions.contains(&"dart"));
}

#[test]
fn test_lint_dependency_direction_line_number() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // domain から infrastructure への違反コード（3行目）
    let bad_code = "pub mod user;\n\nuse crate::infrastructure::db::Repo;\n\npub struct X {}\n";
    fs::write(path.join("src/domain/mod.rs"), bad_code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    // K022 の違反が検出され、行番号が正しい
    let dep_violation = result
        .violations
        .iter()
        .find(|v| v.rule == RuleId::DependencyDirection)
        .unwrap();
    assert_eq!(dep_violation.line, Some(3));
}
