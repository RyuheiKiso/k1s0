use super::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn create_test_manifest(dir: &Path, template_name: &str) {
    let k1s0_dir = dir.join(".k1s0");
    fs::create_dir_all(&k1s0_dir).unwrap();

    let manifest = crate::manifest::Manifest {
        schema_version: "1.0.0".to_string(),
        k1s0_version: "0.1.0".to_string(),
        template: crate::manifest::TemplateInfo {
            name: template_name.to_string(),
            version: "0.1.0".to_string(),
            source: "local".to_string(),
            path: format!("CLI/templates/{}/feature", template_name),
            revision: None,
            fingerprint: "abcd1234".to_string(),
        },
        service: crate::manifest::ServiceInfo {
            service_name: "test-service".to_string(),
            language: "rust".to_string(),
            service_type: "backend".to_string(),
            framework: None,
        },
        generated_at: "2026-01-26T10:00:00Z".to_string(),
        managed_paths: vec!["deploy/".to_string()],
        protected_paths: vec!["src/domain/".to_string()],
        update_policy: std::collections::HashMap::new(),
        checksums: std::collections::HashMap::new(),
        dependencies: None,
    };

    manifest.save(k1s0_dir.join("manifest.json")).unwrap();
}

fn create_backend_rust_structure(dir: &Path) {
    // ディレクトリ作成
    for d in &[
        "src/domain",
        "src/application",
        "src/infrastructure",
        "src/presentation",
        "config",
        "deploy/base",
        "deploy/overlays/dev",
        "deploy/overlays/stg",
        "deploy/overlays/prod",
    ] {
        fs::create_dir_all(dir.join(d)).unwrap();
    }

    // ファイル作成
    for f in &[
        "Cargo.toml",
        "README.md",
        "src/main.rs",
        "src/domain/mod.rs",
        "src/application/mod.rs",
        "src/infrastructure/mod.rs",
        "src/presentation/mod.rs",
        "config/default.yaml",
        "config/dev.yaml",
        "config/stg.yaml",
        "config/prod.yaml",
        "buf.yaml",
    ] {
        fs::write(dir.join(f), "").unwrap();
    }
}

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
    };
    let linter = Linter::new(config);
    let result = linter.lint(path);

    // 必須ファイル/ディレクトリのチェックはスキップされる
    assert!(result.is_success());
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

#[test]
fn test_required_files_from_template_name() {
    assert!(RequiredFiles::from_template_name("backend-rust").is_some());
    assert!(RequiredFiles::from_template_name("backend-go").is_some());
    assert!(RequiredFiles::from_template_name("frontend-react").is_some());
    assert!(RequiredFiles::from_template_name("frontend-flutter").is_some());
    assert!(RequiredFiles::from_template_name("unknown").is_none());
}

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

// K021: config YAML への機密直書き禁止のテスト

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
fn test_parse_yaml_line() {
    // 正常なキー: 値
    assert_eq!(
        parse_yaml_line("key: value"),
        Some(("key".to_string(), "value".to_string()))
    );
    assert_eq!(
        parse_yaml_line("  password: secret123"),
        Some(("password".to_string(), "secret123".to_string()))
    );

    // 値が空
    assert_eq!(
        parse_yaml_line("token:"),
        Some(("token".to_string(), "".to_string()))
    );

    // コメント行
    assert_eq!(parse_yaml_line("# comment"), None);
    assert_eq!(parse_yaml_line("  # indented comment"), None);

    // 空行
    assert_eq!(parse_yaml_line(""), None);
    assert_eq!(parse_yaml_line("   "), None);

    // リスト項目
    assert_eq!(parse_yaml_line("- item"), None);
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

// K022: Clean Architecture 依存方向違反のテスト

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

// K030/K031/K032: gRPC リトライ設定検査のテスト

#[test]
fn test_rule_id_k030() {
    assert_eq!(RuleId::RetryUsageDetected.as_str(), "K030");
    assert_eq!(
        RuleId::RetryUsageDetected.description(),
        "gRPC リトライ設定が検出されました"
    );
}

#[test]
fn test_rule_id_k031() {
    assert_eq!(RuleId::RetryWithoutAdr.as_str(), "K031");
    assert_eq!(
        RuleId::RetryWithoutAdr.description(),
        "gRPC リトライ設定に ADR 参照がありません"
    );
}

#[test]
fn test_rule_id_k032() {
    assert_eq!(RuleId::RetryConfigIncomplete.as_str(), "K032");
    assert_eq!(
        RuleId::RetryConfigIncomplete.description(),
        "gRPC リトライ設定が不完全です"
    );
}

#[test]
fn test_contains_adr_reference() {
    // 有効な ADR 参照
    assert!(contains_adr_reference("ADR-001"));
    assert!(contains_adr_reference("ADR-123"));
    assert!(contains_adr_reference("adr-001"));
    assert!(contains_adr_reference("\"ADR-001\""));
    assert!(contains_adr_reference("// ADR-001: リトライポリシー"));
    assert!(contains_adr_reference("RetryConfig::enabled(\"ADR-001\")"));

    // 無効な ADR 参照
    assert!(!contains_adr_reference("ADR-01"));  // 2桁は不可
    assert!(!contains_adr_reference("ADR-"));    // 数字なし
    assert!(!contains_adr_reference("ADR"));     // ハイフンなし
    assert!(!contains_adr_reference("ADDR-001")); // 異なるプレフィックス
}

#[test]
fn test_lint_retry_usage_detected() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // リトライ設定を使用するコードを追加
    let code = r#"
use k1s0_grpc_client::config::RetryConfig;

fn create_client() {
    // ADR-001: このサービスは冪等なため、リトライを有効にする
    let retry = RetryConfig::enabled("ADR-001")
        .max_attempts(3)
        .build()
        .unwrap();
}
"#;
    fs::write(path.join("src/infrastructure/mod.rs"), code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    // K030 の警告が検出される
    assert!(
        result.violations.iter().any(|v| v.rule == RuleId::RetryUsageDetected),
        "Expected K030 warning, got {:?}",
        result.violations
    );
}

#[test]
fn test_lint_retry_without_adr() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // ADR 参照なしでリトライ設定を使用するコードを追加
    let code = r#"
use k1s0_grpc_client::config::RetryConfig;

fn create_client() {
    let retry = RetryConfig::enabled("no-adr")
        .max_attempts(3)
        .build()
        .unwrap();
}
"#;
    fs::write(path.join("src/infrastructure/mod.rs"), code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    // K031 のエラーが検出される
    assert!(
        result.violations.iter().any(|v| v.rule == RuleId::RetryWithoutAdr),
        "Expected K031 error, got {:?}",
        result.violations
    );
}

#[test]
fn test_lint_retry_with_adr_no_k031() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // 正しい ADR 参照付きのリトライ設定
    let code = r#"
use k1s0_grpc_client::config::RetryConfig;

fn create_client() {
    let retry = RetryConfig::enabled("ADR-001")
        .max_attempts(3)
        .build()
        .unwrap();
}
"#;
    fs::write(path.join("src/infrastructure/mod.rs"), code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    // K031 のエラーが検出されない
    assert!(
        !result.violations.iter().any(|v| v.rule == RuleId::RetryWithoutAdr),
        "Unexpected K031 error: {:?}",
        result.violations
    );
}

#[test]
fn test_lint_retry_with_adr_in_comment() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // コメントに ADR 参照があるリトライ設定
    let code = r#"
use k1s0_grpc_client::config::RetryConfig;

fn create_client() {
    // ADR-002: Get操作は冪等のためリトライ可
    let retry = RetryConfig::enabled("see-comment")
        .max_attempts(3)
        .build()
        .unwrap();
}
"#;
    fs::write(path.join("src/infrastructure/mod.rs"), code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    // K031 のエラーが検出されない（コメントに ADR 参照あり）
    assert!(
        !result.violations.iter().any(|v| v.rule == RuleId::RetryWithoutAdr),
        "Unexpected K031 error (ADR in comment): {:?}",
        result.violations
    );
}

#[test]
fn test_lint_retry_config_incomplete() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // build() を呼ばないリトライ設定
    let code = r#"
use k1s0_grpc_client::config::RetryConfig;

fn create_client() {
    let retry = RetryConfig::enabled("ADR-001");
    // .build() を呼んでいない
}
"#;
    fs::write(path.join("src/infrastructure/mod.rs"), code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    // K032 のエラーが検出される
    assert!(
        result.violations.iter().any(|v| v.rule == RuleId::RetryConfigIncomplete),
        "Expected K032 error, got {:?}",
        result.violations
    );
}

#[test]
fn test_lint_retry_config_complete() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // 完全なリトライ設定
    let code = r#"
use k1s0_grpc_client::config::RetryConfig;

fn create_client() {
    let retry = RetryConfig::enabled("ADR-001")
        .max_attempts(3)
        .initial_backoff_ms(100)
        .max_backoff_ms(5000)
        .build()
        .unwrap();
}
"#;
    fs::write(path.join("src/infrastructure/mod.rs"), code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    // K032 のエラーが検出されない
    assert!(
        !result.violations.iter().any(|v| v.rule == RuleId::RetryConfigIncomplete),
        "Unexpected K032 error: {:?}",
        result.violations
    );
}

#[test]
fn test_lint_retry_no_violation_when_disabled() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // RetryConfig::disabled() を使用（違反なし）
    let code = r#"
use k1s0_grpc_client::config::RetryConfig;

fn create_client() {
    let retry = RetryConfig::disabled();
}
"#;
    fs::write(path.join("src/infrastructure/mod.rs"), code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    // リトライ関連の違反が検出されない
    assert!(
        !result.violations.iter().any(|v| matches!(
            v.rule,
            RuleId::RetryUsageDetected | RuleId::RetryWithoutAdr | RuleId::RetryConfigIncomplete
        )),
        "Unexpected retry violation for disabled retry: {:?}",
        result.violations
    );
}

#[test]
fn test_lint_retry_exclude_k030() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // リトライ設定を使用するコードを追加
    let code = r#"
use k1s0_grpc_client::config::RetryConfig;

fn create_client() {
    let retry = RetryConfig::enabled("ADR-001")
        .max_attempts(3)
        .build()
        .unwrap();
}
"#;
    fs::write(path.join("src/infrastructure/mod.rs"), code).unwrap();

    // K030 を除外
    let config = LintConfig {
        rules: None,
        exclude_rules: vec!["K030".to_string()],
        strict: false,
        env_var_allowlist: vec![],
        fix: false,
    };
    let linter = Linter::new(config);
    let result = linter.lint(path);

    // K030 の違反が検出されない
    assert!(
        !result.violations.iter().any(|v| v.rule == RuleId::RetryUsageDetected),
        "Unexpected K030 warning (rule should be excluded): {:?}",
        result.violations
    );
}

#[test]
fn test_lint_retry_line_number() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // 完全な構造を作成
    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // リトライ設定（6行目）
    let code = "// comment\n// comment\n// comment\n// comment\n// comment\nlet retry = RetryConfig::enabled(\"ADR-001\").build();\n";
    fs::write(path.join("src/infrastructure/mod.rs"), code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    // K030 の違反が検出され、行番号が正しい
    let retry_violation = result
        .violations
        .iter()
        .find(|v| v.rule == RuleId::RetryUsageDetected)
        .unwrap();
    assert_eq!(retry_violation.line, Some(6));
}
