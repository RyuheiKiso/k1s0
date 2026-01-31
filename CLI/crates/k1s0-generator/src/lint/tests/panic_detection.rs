use super::super::*;
use super::{create_backend_rust_structure, create_test_manifest};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_rule_id_k029() {
    assert_eq!(RuleId::PanicInProductionCode.as_str(), "K029");
    assert_eq!(
        RuleId::PanicInProductionCode.description(),
        "本番コードでパニックを起こす可能性があります"
    );
}

#[test]
fn test_unwrap_detected() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    let bad_code = r#"
fn get_user(id: &str) -> User {
    let user = find_user(id).unwrap();
    user
}
"#;
    fs::write(path.join("src/domain/mod.rs"), bad_code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    assert!(
        result
            .violations
            .iter()
            .any(|v| v.rule == RuleId::PanicInProductionCode),
        "Expected K029 violation, got {:?}",
        result.violations
    );
}

#[test]
fn test_panic_macro_detected() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    let bad_code = r#"
fn validate(x: i32) {
    if x < 0 {
        panic!("negative value");
    }
}
"#;
    fs::write(path.join("src/application/mod.rs"), bad_code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    assert!(
        result
            .violations
            .iter()
            .any(|v| v.rule == RuleId::PanicInProductionCode),
    );
}

#[test]
fn test_test_file_excluded() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    let test_code = r#"
#[test]
fn test_something() {
    let result = do_thing().unwrap();
    assert_eq!(result, 42);
}
"#;
    fs::write(path.join("src/domain/entity_test.rs"), test_code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    assert!(
        !result
            .violations
            .iter()
            .any(|v| v.rule == RuleId::PanicInProductionCode
                && v.path.as_deref() == Some("src/domain/entity_test.rs")),
        "Test files should be excluded from K029",
    );
}

#[test]
fn test_main_rs_excluded() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    let main_code = r#"
fn main() {
    let config = load_config().unwrap();
}
"#;
    fs::write(path.join("src/main.rs"), main_code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    assert!(
        !result
            .violations
            .iter()
            .any(|v| v.rule == RuleId::PanicInProductionCode
                && v.path.as_deref() == Some("src/main.rs")),
        "Entry points should be excluded from K029",
    );
}

#[test]
fn test_comment_line_excluded() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    let code = r#"
// result.unwrap() は避けるべき
/// panic!("example") のような使い方はしない
fn safe_fn() -> Result<(), String> {
    Ok(())
}
"#;
    fs::write(path.join("src/domain/mod.rs"), code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    assert!(
        !result
            .violations
            .iter()
            .any(|v| v.rule == RuleId::PanicInProductionCode),
        "Comment lines should be excluded from K029",
    );
}
