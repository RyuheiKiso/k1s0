use super::super::*;
use super::{create_backend_rust_structure, create_test_manifest};
use tempfile::TempDir;

#[test]
fn test_lint_success() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    assert!(result.is_success(), "Expected success, got {:?}", result.violations);
    assert_eq!(result.error_count(), 0);
}

#[test]
fn test_lint_manifest_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    assert!(!result.is_success());
    assert!(result.violations.iter().any(|v| v.rule == RuleId::ManifestNotFound));
}

#[test]
fn test_lint_with_specific_rules() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // manifest だけ作成（必須ファイルなし）
    create_test_manifest(path, "backend-rust");

    let config = LintConfig {
        rules: Some(vec!["K001".to_string()]), // manifest 存在チェックのみ
        exclude_rules: vec![],
        strict: false,
        env_var_allowlist: vec![],
        fix: false,
        fast: false,
    };
    let linter = Linter::new(config);
    let result = linter.lint(path);

    // manifest は存在するので成功
    assert!(result.is_success());
}

#[test]
fn test_lint_strict_mode() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // manifest の managed_paths を空にする
    let manifest_path = path.join(".k1s0/manifest.json");
    let mut manifest = crate::manifest::Manifest::load(&manifest_path).unwrap();
    manifest.managed_paths = vec![];
    manifest.save(&manifest_path).unwrap();

    // 通常モード
    let linter = Linter::default_linter();
    let result = linter.lint(path);
    assert!(result.is_success()); // 警告のみなので成功

    // strict モード
    let config = LintConfig {
        rules: None,
        exclude_rules: vec![],
        strict: true,
        env_var_allowlist: vec![],
        fix: false,
        fast: false,
    };
    let linter = Linter::new(config);
    let result = linter.lint(path);
    assert!(!result.is_success()); // 警告がエラーに昇格
}

#[test]
fn test_rule_id_display() {
    assert_eq!(RuleId::ManifestNotFound.as_str(), "K001");
    assert_eq!(RuleId::ManifestMissingKey.as_str(), "K002");
    assert_eq!(RuleId::ManifestInvalidValue.as_str(), "K003");
    assert_eq!(RuleId::RequiredDirMissing.as_str(), "K010");
    assert_eq!(RuleId::RequiredFileMissing.as_str(), "K011");
}
