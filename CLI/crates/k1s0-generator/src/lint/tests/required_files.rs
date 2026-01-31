use super::super::*;
use super::create_test_manifest;
use tempfile::TempDir;

#[test]
fn test_lint_required_file_missing() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // manifest だけ作成（必須ファイルなし）
    create_test_manifest(path, "backend-rust");

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    assert!(!result.is_success());
    assert!(result.violations.iter().any(|v| v.rule == RuleId::RequiredFileMissing));
    assert!(result.violations.iter().any(|v| v.rule == RuleId::RequiredDirMissing));
}

#[test]
fn test_lint_with_exclude_rules() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // manifest だけ作成（必須ファイルなし）
    create_test_manifest(path, "backend-rust");

    let config = LintConfig {
        rules: None,
        exclude_rules: vec!["K010".to_string(), "K011".to_string()],
        strict: false,
        env_var_allowlist: vec![],
        fix: false,
        fast: false,
    };
    let linter = Linter::new(config);
    let result = linter.lint(path);

    // 必須ファイル/ディレクトリのチェックはスキップされる
    assert!(result.is_success());
}

#[test]
fn test_required_files_from_template_name() {
    assert!(RequiredFiles::from_template_name("backend-rust").is_some());
    assert!(RequiredFiles::from_template_name("backend-go").is_some());
    assert!(RequiredFiles::from_template_name("frontend-react").is_some());
    assert!(RequiredFiles::from_template_name("frontend-flutter").is_some());
    assert!(RequiredFiles::from_template_name("unknown").is_none());
}

