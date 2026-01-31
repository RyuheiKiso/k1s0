use std::path::Path;

use crate::manifest::Manifest;

use super::{LintResult, Linter, RuleId, Severity, Violation};

/// 言語ごとのログ関数パターン
struct LogPatterns {
    file_extensions: Vec<&'static str>,
    log_functions: Vec<&'static str>,
    comment_prefixes: Vec<&'static str>,
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

impl LogPatterns {
    fn rust() -> Self {
        Self {
            file_extensions: vec!["rs"],
            log_functions: vec![
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
            comment_prefixes: vec!["//", "///"],
        }
    }

    fn go() -> Self {
        Self {
            file_extensions: vec!["go"],
            log_functions: vec![
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
            comment_prefixes: vec!["//"],
        }
    }

    fn typescript() -> Self {
        Self {
            file_extensions: vec!["ts", "tsx", "js", "jsx"],
            log_functions: vec![
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
            comment_prefixes: vec!["//"],
        }
    }

    fn csharp() -> Self {
        Self {
            file_extensions: vec!["cs"],
            log_functions: vec![
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
            comment_prefixes: vec!["//"],
        }
    }

    fn python() -> Self {
        Self {
            file_extensions: vec!["py"],
            log_functions: vec![
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
            comment_prefixes: vec!["#"],
        }
    }

    fn dart() -> Self {
        Self {
            file_extensions: vec!["dart"],
            log_functions: vec![
                "log(",
                "print(",
                "debugPrint(",
                "logger.i(",
                "logger.w(",
                "logger.e(",
                "logger.d(",
            ],
            comment_prefixes: vec!["//"],
        }
    }

    fn kotlin() -> Self {
        Self {
            file_extensions: vec!["kt", "kts"],
            log_functions: vec![
                "logger.info(",
                "logger.warn(",
                "logger.error(",
                "logger.debug(",
                "logger.trace(",
                "log.info {",
                "log.warn {",
                "log.error {",
                "log.debug {",
                "Log.d(",
                "Log.e(",
                "Log.i(",
                "Log.w(",
                "Log.v(",
            ],
            comment_prefixes: vec!["//"],
        }
    }
}

fn contains_sensitive_keyword(line: &str) -> Option<&'static str> {
    let lower = line.to_lowercase();
    for keyword in SENSITIVE_KEYWORDS {
        if let Some(pos) = lower.find(keyword) {
            // _hash/_hashed サフィックスチェック
            let after = &lower[pos + keyword.len()..];
            if SAFE_SUFFIXES.iter().any(|s| after.starts_with(s)) {
                continue;
            }
            return Some(keyword);
        }
    }
    None
}

fn contains_log_function(line: &str, patterns: &LogPatterns) -> bool {
    patterns.log_functions.iter().any(|f| line.contains(f))
}

impl Linter {
    /// ログへの機密情報出力を検査（K053）
    pub(super) fn check_sensitive_logging(&self, path: &Path, result: &mut LintResult) {
        let manifest_path = path.join(".k1s0/manifest.json");
        let manifest = match Manifest::load(&manifest_path) {
            Ok(m) => m,
            Err(_) => return,
        };

        let (src_dir, patterns) = match manifest.service.language.as_str() {
            "rust" => ("src", LogPatterns::rust()),
            "go" => ("internal", LogPatterns::go()),
            "typescript" => ("src", LogPatterns::typescript()),
            "csharp" => ("src", LogPatterns::csharp()),
            "python" => ("src", LogPatterns::python()),
            "dart" => ("lib", LogPatterns::dart()),
            "kotlin" => {
                let src_dir = if manifest.service.service_type == "frontend" {
                    "app/src"
                } else {
                    "src"
                };
                (src_dir, LogPatterns::kotlin())
            }
            _ => return,
        };

        let src_path = path.join(src_dir);
        if !src_path.exists() {
            return;
        }

        self.scan_directory_for_sensitive_logging(&src_path, path, &patterns, result);
    }

    fn scan_directory_for_sensitive_logging(
        &self,
        dir: &Path,
        base_path: &Path,
        patterns: &LogPatterns,
        result: &mut LintResult,
    ) {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                self.scan_directory_for_sensitive_logging(&entry_path, base_path, patterns, result);
            } else if entry_path.is_file() {
                let ext = entry_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");
                if patterns.file_extensions.contains(&ext) {
                    self.check_file_for_sensitive_logging(&entry_path, base_path, patterns, result);
                }
            }
        }
    }

    fn check_file_for_sensitive_logging(
        &self,
        file_path: &Path,
        base_path: &Path,
        patterns: &LogPatterns,
        result: &mut LintResult,
    ) {
        let relative_path = file_path
            .strip_prefix(base_path)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| file_path.to_string_lossy().to_string());

        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => return,
        };

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            if patterns
                .comment_prefixes
                .iter()
                .any(|prefix| trimmed.starts_with(prefix))
            {
                continue;
            }

            if contains_log_function(line, patterns) {
                if let Some(keyword) = contains_sensitive_keyword(line) {
                    result.add_violation(
                        Violation::new(
                            RuleId::LoggingSensitiveData,
                            Severity::Warning,
                            format!(
                                "ログ出力に機密情報 '{}' が含まれる可能性があります",
                                keyword
                            ),
                        )
                        .with_path(&relative_path)
                        .with_line(line_num + 1)
                        .with_hint(
                            "機密情報をログに出力しないでください。マスキングまたは除外してください。",
                        ),
                    );
                }
            }
        }
    }
}
