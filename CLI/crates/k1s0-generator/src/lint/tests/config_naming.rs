use super::super::*;
use super::{create_backend_rust_structure, create_test_manifest};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_rule_id_k025() {
    assert_eq!(RuleId::ConfigFileNaming.as_str(), "K025");
    assert_eq!(
        RuleId::ConfigFileNaming.description(),
        "設定ファイルの命名規約に違反しています"
    );
}

#[test]
fn test_config_naming_valid_files_no_violation() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // default.yaml, dev.yaml, stg.yaml, prod.yaml は既に作成済み
    let linter = Linter::default_linter();
    let result = linter.lint(path);

    assert!(
        !result
            .violations
            .iter()
            .any(|v| v.rule == RuleId::ConfigFileNaming),
        "Unexpected K025 violation: {:?}",
        result.violations
    );
}

#[test]
fn test_config_naming_invalid_file_detected() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // 不正な名前の設定ファイルを追加
    fs::write(path.join("config/local.yaml"), "key: value").unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    assert!(
        result
            .violations
            .iter()
            .any(|v| v.rule == RuleId::ConfigFileNaming),
        "Expected K025 violation, got {:?}",
        result.violations
    );
}

#[test]
fn test_config_naming_no_config_dir_no_violation() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    create_test_manifest(path, "backend-rust");
    // config/ ディレクトリを作成しない（必須ファイルの違反は別ルール）

    let config = LintConfig {
        rules: Some(vec!["K025".to_string()]),
        ..Default::default()
    };
    let linter = Linter::new(config);
    let result = linter.lint(path);

    assert!(
        !result
            .violations
            .iter()
            .any(|v| v.rule == RuleId::ConfigFileNaming),
        "Unexpected K025 violation when config/ doesn't exist: {:?}",
        result.violations
    );
}

#[test]
fn test_config_naming_yml_extension_also_checked() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    fs::write(path.join("config/custom.yml"), "key: value").unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    assert!(
        result
            .violations
            .iter()
            .any(|v| v.rule == RuleId::ConfigFileNaming),
        "Expected K025 violation for .yml file, got {:?}",
        result.violations
    );
}

#[test]
fn test_config_naming_non_yaml_files_ignored() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // 非 YAML ファイルは無視される
    fs::write(path.join("config/README.md"), "# Config").unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    assert!(
        !result
            .violations
            .iter()
            .any(|v| v.rule == RuleId::ConfigFileNaming),
        "Unexpected K025 violation for non-YAML file: {:?}",
        result.violations
    );
}
