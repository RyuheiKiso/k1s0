//! 規約違反スキャン
//!
//! lint エンジンのパターンを再利用し、既存プロジェクトの規約違反を検出する。

use std::path::Path;

use super::types::{DetectedProjectType, Violation, ViolationSeverity};

/// プロジェクトの規約違反をスキャンする
pub fn scan_violations(path: &Path, project_type: &DetectedProjectType) -> Vec<Violation> {
    let mut violations = Vec::new();

    let src_dir = source_dir(project_type);
    let extensions = file_extensions(project_type);
    let comment_prefixes = comment_prefixes_for(project_type);

    let src_path = path.join(src_dir);
    if src_path.exists() {
        scan_directory(
            &src_path,
            path,
            &extensions,
            &comment_prefixes,
            project_type,
            &mut violations,
        );
    }

    // K021: config YAML の機密情報チェック
    check_secret_in_config(path, &mut violations);

    // K060: Dockerfile ベースイメージ固定チェック
    check_dockerfile(path, &mut violations);

    violations
}

fn source_dir(project_type: &DetectedProjectType) -> &'static str {
    match project_type {
        DetectedProjectType::BackendRust => "src",
        DetectedProjectType::BackendGo => "internal",
        DetectedProjectType::BackendCsharp => "src",
        DetectedProjectType::BackendPython => "src",
        DetectedProjectType::FrontendReact => "src",
        DetectedProjectType::FrontendFlutter => "lib",
        DetectedProjectType::Unknown => "src",
    }
}

fn file_extensions(project_type: &DetectedProjectType) -> Vec<&'static str> {
    match project_type {
        DetectedProjectType::BackendRust => vec!["rs"],
        DetectedProjectType::BackendGo => vec!["go"],
        DetectedProjectType::BackendCsharp => vec!["cs"],
        DetectedProjectType::BackendPython => vec!["py"],
        DetectedProjectType::FrontendReact => vec!["ts", "tsx", "js", "jsx"],
        DetectedProjectType::FrontendFlutter => vec!["dart"],
        DetectedProjectType::Unknown => vec![],
    }
}

fn comment_prefixes_for(project_type: &DetectedProjectType) -> Vec<&'static str> {
    match project_type {
        DetectedProjectType::BackendRust => vec!["//", "///"],
        DetectedProjectType::BackendGo => vec!["//"],
        DetectedProjectType::BackendCsharp => vec!["//"],
        DetectedProjectType::BackendPython => vec!["#"],
        DetectedProjectType::FrontendReact => vec!["//"],
        DetectedProjectType::FrontendFlutter => vec!["//"],
        DetectedProjectType::Unknown => vec!["//"],
    }
}

fn is_comment(trimmed: &str, prefixes: &[&str]) -> bool {
    prefixes.iter().any(|prefix| trimmed.starts_with(prefix))
}

fn scan_directory(
    dir: &Path,
    base_path: &Path,
    extensions: &[&str],
    comment_prefixes: &[&str],
    project_type: &DetectedProjectType,
    violations: &mut Vec<Violation>,
) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let entry_path = entry.path();
        if entry_path.is_dir() {
            scan_directory(
                &entry_path,
                base_path,
                extensions,
                comment_prefixes,
                project_type,
                violations,
            );
        } else if entry_path.is_file() {
            let ext = entry_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            if extensions.contains(&ext) {
                scan_file(
                    &entry_path,
                    base_path,
                    comment_prefixes,
                    project_type,
                    violations,
                );
            }
        }
    }
}

fn scan_file(
    file_path: &Path,
    base_path: &Path,
    comment_prefixes: &[&str],
    project_type: &DetectedProjectType,
    violations: &mut Vec<Violation>,
) {
    let relative_path = file_path
        .strip_prefix(base_path)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| file_path.to_string_lossy().to_string());

    let content = match std::fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let env_patterns = env_var_patterns(project_type);
    let panic_patterns = panic_patterns(project_type);
    let sql_patterns = sql_injection_patterns(project_type);
    let log_patterns = log_function_patterns(project_type);

    let is_test_file = is_test_file_path(&relative_path, project_type);
    let is_entry = is_entry_point(&relative_path, project_type);

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if is_comment(trimmed, comment_prefixes) {
            continue;
        }

        // K020: 環境変数参照
        for pattern in &env_patterns {
            if line.contains(pattern) {
                violations.push(Violation {
                    rule_id: "K020".to_string(),
                    severity: ViolationSeverity::Error,
                    message: format!("環境変数参照 '{}' が検出されました", pattern),
                    file: Some(relative_path.clone()),
                    line: Some(line_num + 1),
                    auto_fixable: false,
                });
                break;
            }
        }

        // K029: panic/unwrap/expect (test/entry 除外)
        if !is_test_file && !is_entry {
            for pattern in &panic_patterns {
                if line.contains(pattern) {
                    violations.push(Violation {
                        rule_id: "K029".to_string(),
                        severity: ViolationSeverity::Error,
                        message: format!(
                            "本番コードでパニックを起こす可能性があります: '{}'",
                            pattern.trim()
                        ),
                        file: Some(relative_path.clone()),
                        line: Some(line_num + 1),
                        auto_fixable: false,
                    });
                    break;
                }
            }
        }

        // K050: SQL インジェクション
        for pattern in &sql_patterns {
            if line.contains(pattern) {
                violations.push(Violation {
                    rule_id: "K050".to_string(),
                    severity: ViolationSeverity::Error,
                    message: format!(
                        "SQL インジェクションのリスクがあります: '{}'",
                        pattern.trim()
                    ),
                    file: Some(relative_path.clone()),
                    line: Some(line_num + 1),
                    auto_fixable: false,
                });
                break;
            }
        }

        // K053: ログに機密情報
        if contains_log_function(line, &log_patterns) {
            if let Some(keyword) = contains_sensitive_keyword(line) {
                violations.push(Violation {
                    rule_id: "K053".to_string(),
                    severity: ViolationSeverity::Warning,
                    message: format!(
                        "ログ出力に機密情報 '{}' が含まれる可能性があります",
                        keyword
                    ),
                    file: Some(relative_path.clone()),
                    line: Some(line_num + 1),
                    auto_fixable: false,
                });
            }
        }

        // K022: Clean Architecture 依存方向
        check_dependency_direction(
            line,
            &relative_path,
            line_num + 1,
            project_type,
            violations,
        );
    }
}

fn env_var_patterns(project_type: &DetectedProjectType) -> Vec<&'static str> {
    match project_type {
        DetectedProjectType::BackendRust => vec![
            "std::env::var",
            "std::env::var_os",
            "std::env::vars",
            "std::env::vars_os",
            "std::env::set_var",
            "std::env::remove_var",
            "env::var(",
            "env::var_os(",
            "env::vars(",
            "env::set_var(",
            "env::remove_var(",
            "dotenv",
            "dotenvy",
        ],
        DetectedProjectType::BackendGo => vec![
            "os.Getenv",
            "os.LookupEnv",
            "os.Setenv",
            "os.Unsetenv",
            "os.Environ",
            "godotenv",
        ],
        DetectedProjectType::BackendCsharp => vec![
            "Environment.GetEnvironmentVariable",
            "Environment.GetEnvironmentVariables",
            "Environment.ExpandEnvironmentVariables",
            ".AddEnvironmentVariables(",
        ],
        DetectedProjectType::BackendPython => vec![
            "os.environ",
            "os.getenv",
            "os.putenv",
            "os.unsetenv",
            "load_dotenv",
            "from dotenv",
            "import dotenv",
        ],
        DetectedProjectType::FrontendReact => vec!["process.env", "import.meta.env", "dotenv"],
        DetectedProjectType::FrontendFlutter => {
            vec!["Platform.environment", "fromEnvironment", "flutter_dotenv"]
        }
        DetectedProjectType::Unknown => vec![],
    }
}

fn panic_patterns(project_type: &DetectedProjectType) -> Vec<&'static str> {
    match project_type {
        DetectedProjectType::BackendRust => vec![
            ".unwrap()",
            ".expect(",
            "panic!(",
            "todo!(",
            "unimplemented!(",
            "unreachable!(",
        ],
        DetectedProjectType::BackendGo => vec!["panic(", "log.Fatal("],
        DetectedProjectType::BackendCsharp => vec!["Environment.Exit(", "Environment.FailFast("],
        DetectedProjectType::BackendPython => vec!["sys.exit(", "os._exit("],
        DetectedProjectType::FrontendReact => vec!["process.exit("],
        DetectedProjectType::FrontendFlutter => vec!["exit("],
        DetectedProjectType::Unknown => vec![],
    }
}

fn sql_injection_patterns(project_type: &DetectedProjectType) -> Vec<&'static str> {
    match project_type {
        DetectedProjectType::BackendRust => vec![
            "format!(\"SELECT ",
            "format!(\"INSERT ",
            "format!(\"UPDATE ",
            "format!(\"DELETE ",
            "format!(\"select ",
            "format!(\"insert ",
            "format!(\"update ",
            "format!(\"delete ",
        ],
        DetectedProjectType::BackendGo => vec![
            "fmt.Sprintf(\"SELECT ",
            "fmt.Sprintf(\"select ",
            "\"SELECT \" +",
            "\"INSERT \" +",
            "\"UPDATE \" +",
            "\"DELETE \" +",
            "\"select \" +",
            "\"insert \" +",
            "\"update \" +",
            "\"delete \" +",
        ],
        DetectedProjectType::BackendCsharp => vec![
            "$\"SELECT ",
            "$\"INSERT ",
            "$\"UPDATE ",
            "$\"DELETE ",
            "$\"select ",
            "$\"insert ",
            "$\"update ",
            "$\"delete ",
            "\"SELECT \" +",
            "\"INSERT \" +",
            "\"UPDATE \" +",
            "\"DELETE \" +",
        ],
        DetectedProjectType::BackendPython => vec![
            "f\"SELECT ",
            "f\"INSERT ",
            "f\"UPDATE ",
            "f\"DELETE ",
            "f\"select ",
            "f\"insert ",
            "f\"update ",
            "f\"delete ",
            "f'SELECT ",
            "f'INSERT ",
            "f'UPDATE ",
            "f'DELETE ",
            "\"SELECT \" +",
            "\"INSERT \" +",
            "\"UPDATE \" +",
            "\"DELETE \" +",
            "\"SELECT \".format(",
            "\"INSERT \".format(",
            "\"UPDATE \".format(",
            "\"DELETE \".format(",
        ],
        DetectedProjectType::FrontendReact => vec![
            "`SELECT ${",
            "`INSERT ${",
            "`UPDATE ${",
            "`DELETE ${",
            "`select ${",
            "`insert ${",
            "`update ${",
            "`delete ${",
            "\"SELECT \" +",
            "\"INSERT \" +",
            "\"UPDATE \" +",
            "\"DELETE \" +",
        ],
        DetectedProjectType::FrontendFlutter => vec![
            "'SELECT $",
            "\"SELECT $",
            "'INSERT $",
            "\"INSERT $",
            "'UPDATE $",
            "\"UPDATE $",
            "'DELETE $",
            "\"DELETE $",
        ],
        DetectedProjectType::Unknown => vec![],
    }
}

fn log_function_patterns(project_type: &DetectedProjectType) -> Vec<&'static str> {
    match project_type {
        DetectedProjectType::BackendRust => vec![
            "tracing::info!(",
            "tracing::warn!(",
            "tracing::error!(",
            "tracing::debug!(",
            "tracing::trace!(",
            "info!(",
            "warn!(",
            "error!(",
            "debug!(",
            "trace!(",
            "log::info!(",
            "log::warn!(",
            "log::error!(",
            "log::debug!(",
            "log::trace!(",
        ],
        DetectedProjectType::BackendGo => vec![
            "log.Print(",
            "log.Printf(",
            "log.Println(",
            "slog.Info(",
            "slog.Warn(",
            "slog.Error(",
            "slog.Debug(",
            "zap.Info(",
            "zap.Warn(",
            "zap.Error(",
            "zap.Debug(",
            "logger.Info(",
            "logger.Warn(",
            "logger.Error(",
            "logger.Debug(",
        ],
        DetectedProjectType::BackendCsharp => vec![
            "logger.LogInformation(",
            "logger.LogWarning(",
            "logger.LogError(",
            "logger.LogDebug(",
            "logger.LogTrace(",
            "_logger.LogInformation(",
            "_logger.LogWarning(",
            "_logger.LogError(",
            "_logger.LogDebug(",
            "Log.Information(",
            "Log.Warning(",
            "Log.Error(",
            "Log.Debug(",
        ],
        DetectedProjectType::BackendPython => vec![
            "logging.info(",
            "logging.warning(",
            "logging.error(",
            "logging.debug(",
            "logger.info(",
            "logger.warning(",
            "logger.error(",
            "logger.debug(",
            "print(",
        ],
        DetectedProjectType::FrontendReact => vec![
            "console.log(",
            "console.warn(",
            "console.error(",
            "console.info(",
            "console.debug(",
            "logger.info(",
            "logger.warn(",
            "logger.error(",
            "logger.debug(",
        ],
        DetectedProjectType::FrontendFlutter => vec![
            "log(",
            "print(",
            "debugPrint(",
            "logger.i(",
            "logger.w(",
            "logger.e(",
            "logger.d(",
        ],
        DetectedProjectType::Unknown => vec![],
    }
}

const SENSITIVE_KEYWORDS: &[&str] = &[
    "password",
    "token",
    "secret",
    "api_key",
    "apikey",
    "credential",
    "private_key",
];

const SAFE_SUFFIXES: &[&str] = &["_hash", "_hashed"];

fn contains_log_function(line: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|f| line.contains(f))
}

fn contains_sensitive_keyword(line: &str) -> Option<&'static str> {
    let lower = line.to_lowercase();
    for keyword in SENSITIVE_KEYWORDS {
        if let Some(pos) = lower.find(keyword) {
            let after = &lower[pos + keyword.len()..];
            if SAFE_SUFFIXES.iter().any(|s| after.starts_with(s)) {
                continue;
            }
            return Some(keyword);
        }
    }
    None
}

fn is_test_file_path(relative_path: &str, project_type: &DetectedProjectType) -> bool {
    match project_type {
        DetectedProjectType::BackendRust => relative_path.ends_with("_test.rs"),
        DetectedProjectType::BackendGo => relative_path.ends_with("_test.go"),
        DetectedProjectType::BackendPython => {
            let file_name = relative_path.rsplit('/').next().unwrap_or(relative_path);
            file_name.starts_with("test_")
        }
        DetectedProjectType::FrontendReact => {
            relative_path.ends_with(".test.ts")
                || relative_path.ends_with(".test.tsx")
                || relative_path.ends_with(".spec.ts")
                || relative_path.ends_with(".spec.tsx")
                || relative_path.ends_with(".test.js")
                || relative_path.ends_with(".spec.js")
        }
        DetectedProjectType::FrontendFlutter => relative_path.ends_with("_test.dart"),
        DetectedProjectType::BackendCsharp | DetectedProjectType::Unknown => false,
    }
}

fn is_entry_point(relative_path: &str, project_type: &DetectedProjectType) -> bool {
    match project_type {
        DetectedProjectType::BackendRust => relative_path == "src/main.rs",
        DetectedProjectType::BackendGo => relative_path == "cmd/main.go",
        _ => false,
    }
}

/// Clean Architecture 依存方向チェック (K022)
fn check_dependency_direction(
    line: &str,
    relative_path: &str,
    line_number: usize,
    project_type: &DetectedProjectType,
    violations: &mut Vec<Violation>,
) {
    let import_patterns = import_patterns_for(project_type);
    if import_patterns.is_empty() {
        return;
    }

    // ファイルがどの層に属するか判定
    let layer = detect_file_layer(relative_path, project_type);
    let layer = match layer {
        Some(l) => l,
        None => return,
    };

    let forbidden_layers = match layer {
        "domain" => vec!["application", "infrastructure", "presentation"],
        "application" => vec!["infrastructure", "presentation"],
        _ => return,
    };

    for forbidden in &forbidden_layers {
        for pattern_template in &import_patterns {
            let forbidden_pattern = pattern_template.replace("{layer}", forbidden);
            if line.contains(&forbidden_pattern) {
                violations.push(Violation {
                    rule_id: "K022".to_string(),
                    severity: ViolationSeverity::Error,
                    message: format!(
                        "{} 層から {} 層への依存が検出されました",
                        layer, forbidden
                    ),
                    file: Some(relative_path.to_string()),
                    line: Some(line_number),
                    auto_fixable: false,
                });
            }
        }
    }
}

fn detect_file_layer<'a>(relative_path: &'a str, project_type: &DetectedProjectType) -> Option<&'a str> {
    let src_prefix = match project_type {
        DetectedProjectType::BackendRust => "src/",
        DetectedProjectType::BackendGo => "internal/",
        DetectedProjectType::BackendCsharp => "src/",
        DetectedProjectType::BackendPython => "src/",
        DetectedProjectType::FrontendReact => "src/",
        DetectedProjectType::FrontendFlutter => "lib/src/",
        DetectedProjectType::Unknown => return None,
    };

    let after_src = relative_path.strip_prefix(src_prefix)?;

    ["domain", "application", "infrastructure", "presentation"]
        .iter()
        .find(|&layer| after_src.starts_with(layer))
        .copied()
}

fn import_patterns_for(project_type: &DetectedProjectType) -> Vec<String> {
    match project_type {
        DetectedProjectType::BackendRust => vec![
            "use crate::{layer}".to_string(),
            "crate::{layer}::".to_string(),
            "super::super::{layer}".to_string(),
        ],
        DetectedProjectType::BackendGo => vec![
            "\"internal/{layer}".to_string(),
            "/internal/{layer}".to_string(),
        ],
        DetectedProjectType::BackendCsharp => vec![
            "using {layer}".to_string(),
        ],
        DetectedProjectType::BackendPython => vec![
            "from {layer}".to_string(),
            "import {layer}".to_string(),
            "from .{layer}".to_string(),
            "from ..{layer}".to_string(),
        ],
        DetectedProjectType::FrontendReact => vec![
            "from '../{layer}".to_string(),
            "from \"../{layer}".to_string(),
            "from '../../{layer}".to_string(),
            "from \"../../{layer}".to_string(),
            "from '@/{layer}".to_string(),
            "from \"@/{layer}".to_string(),
        ],
        DetectedProjectType::FrontendFlutter => vec![
            "import '../{layer}".to_string(),
            "import '../../{layer}".to_string(),
        ],
        DetectedProjectType::Unknown => vec![],
    }
}

/// config YAML 内の機密情報直書きチェック (K021)
fn check_secret_in_config(path: &Path, violations: &mut Vec<Violation>) {
    let config_dir = path.join("config");
    if !config_dir.exists() || !config_dir.is_dir() {
        return;
    }

    let entries = match std::fs::read_dir(&config_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    let secret_patterns: &[&str] = &[
        "password", "passwd", "secret", "token", "api_key", "apikey", "api-key",
        "private_key", "privatekey", "private-key", "credential", "auth_key",
        "authkey", "auth-key", "access_key", "accesskey", "access-key",
        "secret_key", "secretkey", "secret-key", "encryption_key", "signing_key",
        "client_secret", "clientsecret", "client-secret",
    ];

    for entry in entries.flatten() {
        let entry_path = entry.path();
        if entry_path.is_file() {
            let ext = entry_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            if ext == "yaml" || ext == "yml" {
                check_yaml_for_secrets(
                    &entry_path,
                    path,
                    secret_patterns,
                    violations,
                );
            }
        }
    }
}

fn check_yaml_for_secrets(
    yaml_path: &Path,
    base_path: &Path,
    secret_patterns: &[&str],
    violations: &mut Vec<Violation>,
) {
    let content = match std::fs::read_to_string(yaml_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let relative_path = yaml_path
        .strip_prefix(base_path)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| yaml_path.to_string_lossy().to_string());

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }

        if let Some((key, value)) = parse_yaml_line(line) {
            let key_lower = key.to_lowercase();

            // _file / _path / _ref サフィックスは許可
            if key_lower.ends_with("_file")
                || key_lower.ends_with("_path")
                || key_lower.ends_with("_ref")
            {
                continue;
            }

            let matches_secret = secret_patterns
                .iter()
                .any(|pattern| key_lower.contains(pattern));

            if matches_secret {
                let value_trimmed = value.trim();
                if !value_trimmed.is_empty()
                    && value_trimmed != "null"
                    && value_trimmed != "~"
                    && !value_trimmed.starts_with("${")
                    && !value_trimmed.starts_with("{{")
                {
                    violations.push(Violation {
                        rule_id: "K021".to_string(),
                        severity: ViolationSeverity::Error,
                        message: format!(
                            "機密キー '{}' に値が直接設定されています",
                            key
                        ),
                        file: Some(relative_path.clone()),
                        line: Some(line_num + 1),
                        auto_fixable: false,
                    });
                }
            }
        }
    }
}

fn parse_yaml_line(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    let colon_pos = trimmed.find(':')?;
    let key = trimmed[..colon_pos].trim().to_string();
    let value = trimmed[colon_pos + 1..].trim().to_string();
    if key.is_empty() {
        return None;
    }
    Some((key, value))
}

/// Dockerfile ベースイメージ固定チェック (K060)
fn check_dockerfile(path: &Path, violations: &mut Vec<Violation>) {
    let dockerfile_path = path.join("Dockerfile");
    if !dockerfile_path.exists() {
        return;
    }

    let content = match std::fs::read_to_string(&dockerfile_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }

        if !trimmed.starts_with("FROM ") {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }

        let mut image = parts[1];

        if image == "scratch" {
            continue;
        }

        // --platform=... FROM image の場合
        if image.starts_with("--") {
            if parts.len() < 3 {
                continue;
            }
            image = parts[2];
        }

        // sha256 ダイジェスト指定は OK
        if image.contains("@sha256:") {
            continue;
        }

        if let Some(tag_pos) = image.rfind(':') {
            let tag = &image[tag_pos + 1..];
            if tag == "latest" {
                violations.push(Violation {
                    rule_id: "K060".to_string(),
                    severity: ViolationSeverity::Warning,
                    message: format!(
                        "ベースイメージ '{}' が :latest タグを使用しています",
                        image
                    ),
                    file: Some("Dockerfile".to_string()),
                    line: Some(line_num + 1),
                    auto_fixable: false,
                });
            }
        } else {
            violations.push(Violation {
                rule_id: "K060".to_string(),
                severity: ViolationSeverity::Warning,
                message: format!(
                    "ベースイメージ '{}' にタグが指定されていません",
                    image
                ),
                file: Some("Dockerfile".to_string()),
                line: Some(line_num + 1),
                auto_fixable: false,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_env_var_detection_rust() {
        let tmp = tempfile::tempdir().expect("failed to create tempdir");
        let p = tmp.path();
        fs::create_dir_all(p.join("src")).expect("mkdir failed");
        fs::write(
            p.join("src/lib.rs"),
            "fn get_port() -> String { std::env::var(\"PORT\").unwrap() }\n",
        )
        .expect("write failed");

        let violations = scan_violations(p, &DetectedProjectType::BackendRust);
        let env_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "K020").collect();
        assert!(!env_violations.is_empty());
    }

    #[test]
    fn test_secret_in_config() {
        let tmp = tempfile::tempdir().expect("failed to create tempdir");
        let p = tmp.path();
        fs::create_dir_all(p.join("config")).expect("mkdir failed");
        fs::write(
            p.join("config/default.yaml"),
            "database:\n  password: my_secret_password\n",
        )
        .expect("write failed");

        let violations = scan_violations(p, &DetectedProjectType::BackendRust);
        let secret_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "K021").collect();
        assert!(!secret_violations.is_empty());
    }

    #[test]
    fn test_secret_file_ref_allowed() {
        let tmp = tempfile::tempdir().expect("failed to create tempdir");
        let p = tmp.path();
        fs::create_dir_all(p.join("config")).expect("mkdir failed");
        fs::write(
            p.join("config/default.yaml"),
            "database:\n  password_file: /var/run/secrets/db_password\n",
        )
        .expect("write failed");

        let violations = scan_violations(p, &DetectedProjectType::BackendRust);
        let secret_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "K021").collect();
        assert!(secret_violations.is_empty());
    }

    #[test]
    fn test_dockerfile_unpinned() {
        let tmp = tempfile::tempdir().expect("failed to create tempdir");
        let p = tmp.path();
        fs::write(p.join("Dockerfile"), "FROM rust\nRUN cargo build\n").expect("write failed");

        let violations = scan_violations(p, &DetectedProjectType::BackendRust);
        let docker_violations: Vec<_> =
            violations.iter().filter(|v| v.rule_id == "K060").collect();
        assert!(!docker_violations.is_empty());
    }

    #[test]
    fn test_dockerfile_pinned_ok() {
        let tmp = tempfile::tempdir().expect("failed to create tempdir");
        let p = tmp.path();
        fs::write(
            p.join("Dockerfile"),
            "FROM rust:1.85-slim\nRUN cargo build\n",
        )
        .expect("write failed");

        let violations = scan_violations(p, &DetectedProjectType::BackendRust);
        let docker_violations: Vec<_> =
            violations.iter().filter(|v| v.rule_id == "K060").collect();
        assert!(docker_violations.is_empty());
    }

    #[test]
    fn test_panic_detection_skips_test_files() {
        let tmp = tempfile::tempdir().expect("failed to create tempdir");
        let p = tmp.path();
        fs::create_dir_all(p.join("src")).expect("mkdir failed");
        fs::write(
            p.join("src/foo_test.rs"),
            "fn test_thing() { let x = opt.unwrap(); }\n",
        )
        .expect("write failed");

        let violations = scan_violations(p, &DetectedProjectType::BackendRust);
        let panic_violations: Vec<_> =
            violations.iter().filter(|v| v.rule_id == "K029").collect();
        assert!(panic_violations.is_empty());
    }

    #[test]
    fn test_panic_detection_skips_entry_point() {
        let tmp = tempfile::tempdir().expect("failed to create tempdir");
        let p = tmp.path();
        fs::create_dir_all(p.join("src")).expect("mkdir failed");
        fs::write(
            p.join("src/main.rs"),
            "fn main() { let x = opt.unwrap(); }\n",
        )
        .expect("write failed");

        let violations = scan_violations(p, &DetectedProjectType::BackendRust);
        let panic_violations: Vec<_> =
            violations.iter().filter(|v| v.rule_id == "K029").collect();
        assert!(panic_violations.is_empty());
    }
}
