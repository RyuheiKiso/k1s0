use std::path::Path;

use super::{parse_yaml_line, LintResult, Linter, RuleId, Severity, Violation};

/// 機密キーのパターン定義
#[derive(Debug, Clone)]
pub struct SecretKeyPatterns {
    /// 機密キーのパターン（部分一致）
    pub patterns: Vec<&'static str>,
}

impl Default for SecretKeyPatterns {
    fn default() -> Self {
        Self {
            patterns: vec![
                "password",
                "passwd",
                "secret",
                "token",
                "api_key",
                "apikey",
                "api-key",
                "private_key",
                "privatekey",
                "private-key",
                "credential",
                "auth_key",
                "authkey",
                "auth-key",
                "access_key",
                "accesskey",
                "access-key",
                "secret_key",
                "secretkey",
                "secret-key",
                "encryption_key",
                "encryptionkey",
                "encryption-key",
                "signing_key",
                "signingkey",
                "signing-key",
                "client_secret",
                "clientsecret",
                "client-secret",
            ],
        }
    }
}

impl SecretKeyPatterns {
    /// キーが機密パターンにマッチするかチェック
    /// マッチした場合はパターン名を返す
    pub fn matches_secret_key(&self, key: &str) -> Option<&'static str> {
        let key_lower = key.to_lowercase();
        for pattern in &self.patterns {
            if key_lower.contains(pattern) {
                return Some(pattern);
            }
        }
        None
    }
}

impl Linter {
    /// config YAML への機密直書きを検査（K021）
    pub(super) fn check_secret_in_config(&self, path: &Path, result: &mut LintResult) {
        let config_dir = path.join("config");
        if !config_dir.exists() || !config_dir.is_dir() {
            return;
        }

        // config ディレクトリ内の YAML ファイルを検査
        let entries = match std::fs::read_dir(&config_dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_file() {
                let ext = entry_path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if ext == "yaml" || ext == "yml" {
                    self.check_yaml_for_secrets(&entry_path, path, result);
                }
            }
        }
    }

    /// YAML ファイル内の機密直書きを検査
    fn check_yaml_for_secrets(&self, yaml_path: &Path, base_path: &Path, result: &mut LintResult) {
        let content = match std::fs::read_to_string(yaml_path) {
            Ok(c) => c,
            Err(_) => return,
        };

        let relative_path = yaml_path
            .strip_prefix(base_path)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| yaml_path.to_string_lossy().to_string());

        // 機密キーのパターン
        let secret_key_patterns = SecretKeyPatterns::default();

        for (line_num, line) in content.lines().enumerate() {
            // コメント行はスキップ
            let trimmed = line.trim();
            if trimmed.starts_with('#') || trimmed.is_empty() {
                continue;
            }

            // キー: 値 の形式をパース
            if let Some((key, value)) = parse_yaml_line(line) {
                // キーが機密パターンにマッチするかチェック
                if let Some(pattern) = secret_key_patterns.matches_secret_key(&key) {
                    // *_file サフィックスを持つキーは許可
                    if key.ends_with("_file") || key.ends_with("_path") || key.ends_with("_ref") {
                        continue;
                    }

                    // 値が空、null、参照形式でない場合はエラー
                    let value_trimmed = value.trim();
                    if !value_trimmed.is_empty()
                        && value_trimmed != "null"
                        && value_trimmed != "~"
                        && !value_trimmed.starts_with("${")
                        && !value_trimmed.starts_with("{{")
                    {
                        result.add_violation(
                            Violation::new(
                                RuleId::SecretInConfig,
                                Severity::Error,
                                format!(
                                    "機密キー '{}' に値が直接設定されています",
                                    key
                                ),
                            )
                            .with_path(&relative_path)
                            .with_line(line_num + 1)
                            .with_hint(format!(
                                "機密情報は直接書かず、'{}_file' キーでファイルパスを参照してください。例: {}_file: /var/run/secrets/k1s0/{}",
                                pattern, pattern, pattern
                            )),
                        );
                    }
                }
            }
        }
    }
}
