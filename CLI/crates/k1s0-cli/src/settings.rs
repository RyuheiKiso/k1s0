//! プロジェクト設定ファイル
//!
//! `.k1s0/settings.yaml` または `.k1s0/settings.local.yaml` からプロジェクト設定を読み込む。
//! `settings.local.yaml` は `.gitignore` に追加することで、個人設定を管理できる。
//!
//! # 設定ファイルの例
//!
//! ```yaml
//! # .k1s0/settings.yaml
//! lint:
//!   rules: [K001, K002, K010]
//!   exclude_rules: [K030]
//!   strict: false
//!   env_var_allowlist:
//!     - "config/local/*.yaml"
//!
//! generate:
//!   default_template: backend-rust
//!   output_dir: ./services
//!
//! registry:
//!   url: "https://registry.example.com"
//!   cache_dir: ~/.k1s0/cache
//! ```

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{CliError, Result};

/// プロジェクト設定
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// lint 設定
    pub lint: LintSettings,

    /// 生成設定
    pub generate: GenerateSettings,

    /// レジストリ設定
    pub registry: RegistrySettings,

    /// プラグイン設定
    pub plugins: PluginsSettings,

    /// カスタム設定（拡張用）
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
}

/// lint 設定
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct LintSettings {
    /// 実行するルール ID（空の場合は全ルール）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<String>>,

    /// 除外するルール ID
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub exclude_rules: Vec<String>,

    /// 警告をエラーとして扱う
    #[serde(default)]
    pub strict: bool,

    /// 環境変数参照を許可するパスパターン
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub env_var_allowlist: Vec<String>,

    /// 自動修正を有効にする
    #[serde(default)]
    pub fix: bool,

    /// AST パースをスキップし grep ベースで高速実行
    #[serde(default)]
    pub fast: bool,

    /// Watch モードのデバウンス間隔（ミリ秒）
    #[serde(default = "default_debounce_ms")]
    pub watch_debounce_ms: u64,
}

fn default_debounce_ms() -> u64 {
    500
}

/// 生成設定
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct GenerateSettings {
    /// デフォルトテンプレート
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_template: Option<String>,

    /// 出力ディレクトリ
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_dir: Option<String>,

    /// テンプレート変数のデフォルト値
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub variables: HashMap<String, String>,
}

/// レジストリ設定
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct RegistrySettings {
    /// レジストリ URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// キャッシュディレクトリ
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_dir: Option<String>,

    /// 認証トークン（環境変数から取得を推奨）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_token: Option<String>,

    /// タイムアウト（秒）
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

fn default_timeout() -> u64 {
    30
}

/// プラグイン設定
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PluginsSettings {
    /// 有効なプラグイン
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub enabled: Vec<String>,

    /// プラグインディレクトリ
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin_dir: Option<String>,

    /// プラグイン設定
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub config: HashMap<String, serde_yaml::Value>,
}

impl Settings {
    /// 設定ファイルを検索して読み込む
    ///
    /// 検索順序:
    /// 1. 引数で指定されたパス
    /// 2. カレントディレクトリから上位に向かって `.k1s0/settings.yaml` を検索
    /// 3. ホームディレクトリの `~/.k1s0/settings.yaml`
    ///
    /// `settings.local.yaml` が存在する場合はマージされる。
    pub fn load(start_path: Option<&Path>) -> Result<Self> {
        let mut settings = Settings::default();

        // 設定ファイルを検索
        if let Some(config_path) = Self::find_config(start_path) {
            // メイン設定ファイルを読み込み
            if config_path.exists() {
                let content = fs::read_to_string(&config_path).map_err(|e| {
                    CliError::io(format!("設定ファイルの読み込みに失敗: {}", e))
                        .with_target(config_path.display().to_string())
                })?;

                settings = serde_yaml::from_str(&content).map_err(|e| {
                    CliError::validation(format!("設定ファイルのパースに失敗: {}", e))
                        .with_target(config_path.display().to_string())
                })?;
            }

            // ローカル設定ファイルをマージ
            let local_path = config_path.with_file_name("settings.local.yaml");
            if local_path.exists() {
                let local_content = fs::read_to_string(&local_path).map_err(|e| {
                    CliError::io(format!("ローカル設定ファイルの読み込みに失敗: {}", e))
                        .with_target(local_path.display().to_string())
                })?;

                let local_settings: Settings = serde_yaml::from_str(&local_content).map_err(|e| {
                    CliError::validation(format!("ローカル設定ファイルのパースに失敗: {}", e))
                        .with_target(local_path.display().to_string())
                })?;

                settings = settings.merge(local_settings);
            }
        }

        // グローバル設定をマージ
        if let Some(global_path) = Self::global_config_path() {
            if global_path.exists() {
                let global_content = fs::read_to_string(&global_path).map_err(|e| {
                    CliError::io(format!("グローバル設定ファイルの読み込みに失敗: {}", e))
                        .with_target(global_path.display().to_string())
                })?;

                let global_settings: Settings =
                    serde_yaml::from_str(&global_content).unwrap_or_default();

                // グローバル設定は優先度が低いので、既存の値を上書きしない
                settings = global_settings.merge(settings);
            }
        }

        Ok(settings)
    }

    /// 設定ファイルを検索する
    fn find_config(start_path: Option<&Path>) -> Option<PathBuf> {
        let start = start_path
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        let mut current = start.as_path();

        loop {
            let config_path = current.join(".k1s0/settings.yaml");
            if config_path.exists() {
                return Some(config_path);
            }

            match current.parent() {
                Some(parent) => current = parent,
                None => break,
            }
        }

        None
    }

    /// グローバル設定ファイルのパスを取得
    fn global_config_path() -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".k1s0/settings.yaml"))
    }

    /// 設定をマージする（self の値を優先）
    pub fn merge(self, other: Settings) -> Self {
        Settings {
            lint: LintSettings {
                rules: self.lint.rules.or(other.lint.rules),
                exclude_rules: if self.lint.exclude_rules.is_empty() {
                    other.lint.exclude_rules
                } else {
                    self.lint.exclude_rules
                },
                strict: self.lint.strict || other.lint.strict,
                env_var_allowlist: if self.lint.env_var_allowlist.is_empty() {
                    other.lint.env_var_allowlist
                } else {
                    self.lint.env_var_allowlist
                },
                fix: self.lint.fix || other.lint.fix,
                fast: self.lint.fast || other.lint.fast,
                watch_debounce_ms: if self.lint.watch_debounce_ms == default_debounce_ms() {
                    other.lint.watch_debounce_ms
                } else {
                    self.lint.watch_debounce_ms
                },
            },
            generate: GenerateSettings {
                default_template: self.generate.default_template.or(other.generate.default_template),
                output_dir: self.generate.output_dir.or(other.generate.output_dir),
                variables: {
                    let mut vars = other.generate.variables;
                    vars.extend(self.generate.variables);
                    vars
                },
            },
            registry: RegistrySettings {
                url: self.registry.url.or(other.registry.url),
                cache_dir: self.registry.cache_dir.or(other.registry.cache_dir),
                auth_token: self.registry.auth_token.or(other.registry.auth_token),
                timeout_secs: if self.registry.timeout_secs == default_timeout() {
                    other.registry.timeout_secs
                } else {
                    self.registry.timeout_secs
                },
            },
            plugins: PluginsSettings {
                enabled: if self.plugins.enabled.is_empty() {
                    other.plugins.enabled
                } else {
                    self.plugins.enabled
                },
                plugin_dir: self.plugins.plugin_dir.or(other.plugins.plugin_dir),
                config: {
                    let mut config = other.plugins.config;
                    config.extend(self.plugins.config);
                    config
                },
            },
            extra: {
                let mut extra = other.extra;
                extra.extend(self.extra);
                extra
            },
        }
    }

    /// 設定をファイルに保存
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = serde_yaml::to_string(self)
            .map_err(|e| CliError::internal(format!("設定のシリアライズに失敗: {}", e)))?;

        // 親ディレクトリを作成
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                CliError::io(format!("ディレクトリの作成に失敗: {}", e))
                    .with_target(parent.display().to_string())
            })?;
        }

        fs::write(path, content).map_err(|e| {
            CliError::io(format!("設定ファイルの書き込みに失敗: {}", e))
                .with_target(path.display().to_string())
        })?;

        Ok(())
    }
}

/// LintConfig への変換
impl From<&LintSettings> for k1s0_generator::lint::LintConfig {
    fn from(settings: &LintSettings) -> Self {
        k1s0_generator::lint::LintConfig {
            rules: settings.rules.clone(),
            exclude_rules: settings.exclude_rules.clone(),
            strict: settings.strict,
            env_var_allowlist: settings.env_var_allowlist.clone(),
            fix: settings.fix,
            fast: settings.fast,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();

        assert!(!settings.lint.strict);
        assert!(settings.lint.rules.is_none());
        assert!(settings.lint.exclude_rules.is_empty());
    }

    #[test]
    fn test_settings_merge() {
        let base = Settings {
            lint: LintSettings {
                strict: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let override_settings = Settings {
            lint: LintSettings {
                rules: Some(vec!["K001".to_string()]),
                ..Default::default()
            },
            ..Default::default()
        };

        let merged = override_settings.merge(base);

        assert!(merged.lint.strict); // base から継承
        assert_eq!(merged.lint.rules, Some(vec!["K001".to_string()])); // override が優先
    }

    #[test]
    fn test_settings_save_and_load() {
        let temp = tempdir().unwrap();
        let config_path = temp.path().join(".k1s0/settings.yaml");

        let settings = Settings {
            lint: LintSettings {
                strict: true,
                exclude_rules: vec!["K030".to_string()],
                ..Default::default()
            },
            generate: GenerateSettings {
                default_template: Some("backend-rust".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        settings.save(&config_path).unwrap();

        // ファイルが作成されたことを確認
        assert!(config_path.exists());

        // 読み込みテスト
        let loaded = Settings::load(Some(temp.path())).unwrap();

        assert!(loaded.lint.strict);
        assert_eq!(loaded.lint.exclude_rules, vec!["K030".to_string()]);
        assert_eq!(
            loaded.generate.default_template,
            Some("backend-rust".to_string())
        );
    }

    #[test]
    fn test_lint_config_conversion() {
        let settings = LintSettings {
            rules: Some(vec!["K001".to_string(), "K002".to_string()]),
            exclude_rules: vec!["K030".to_string()],
            strict: true,
            ..Default::default()
        };

        let config: k1s0_generator::lint::LintConfig = (&settings).into();

        assert_eq!(
            config.rules,
            Some(vec!["K001".to_string(), "K002".to_string()])
        );
        assert_eq!(config.exclude_rules, vec!["K030".to_string()]);
        assert!(config.strict);
    }
}
