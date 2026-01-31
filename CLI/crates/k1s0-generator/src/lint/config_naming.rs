use std::path::Path;

use super::{LintResult, Linter, RuleId, Severity, Violation};

/// 許可される設定ファイル名（拡張子なし）
const VALID_CONFIG_NAMES: &[&str] = &["default", "dev", "stg", "prod"];

impl Linter {
    /// 設定ファイル命名規約を検査（K025）
    pub(super) fn check_config_file_naming(&self, path: &Path, result: &mut LintResult) {
        let config_dir = path.join("config");
        if !config_dir.exists() || !config_dir.is_dir() {
            return;
        }

        let entries = match std::fs::read_dir(&config_dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let entry_path = entry.path();
            if !entry_path.is_file() {
                continue;
            }

            let ext = entry_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            if ext != "yaml" && ext != "yml" {
                continue;
            }

            let stem = entry_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("");

            if !VALID_CONFIG_NAMES.contains(&stem) {
                let relative_path = format!("config/{}", entry_path.file_name().unwrap().to_string_lossy());
                result.add_violation(
                    Violation::new(
                        RuleId::ConfigFileNaming,
                        Severity::Error,
                        format!(
                            "設定ファイル '{}' は命名規約に違反しています",
                            relative_path
                        ),
                    )
                    .with_path(&relative_path)
                    .with_hint(format!(
                        "許可されるファイル名: {}",
                        VALID_CONFIG_NAMES.join(", ")
                    )),
                );
            }
        }
    }
}
