//! 設定オプション
//!
//! CLI 引数から受け取る設定オプションを定義する。

use std::path::PathBuf;

use crate::{DEFAULT_CONFIG_DIR, DEFAULT_SECRETS_DIR};

/// 設定オプション
///
/// CLI 引数 `--env`, `--config`, `--secrets-dir` に対応する。
#[derive(Debug, Clone)]
pub struct ConfigOptions {
    /// 環境名（dev, stg, prod）
    pub env: String,

    /// 設定ファイルのパス
    /// None の場合は `{config_dir}/{env}.yaml` を使用
    pub config_path: Option<PathBuf>,

    /// 設定ファイルのディレクトリ
    /// config_path が None の場合に使用
    pub config_dir: PathBuf,

    /// secrets ディレクトリ
    pub secrets_dir: PathBuf,

    /// 設定ファイルが存在しない場合にエラーにするか
    pub require_config_file: bool,

    /// secrets ディレクトリが存在しない場合にエラーにするか
    pub require_secrets_dir: bool,
}

impl ConfigOptions {
    /// 新しい設定オプションを作成
    ///
    /// # Arguments
    ///
    /// * `env` - 環境名（dev, stg, prod）
    pub fn new(env: impl Into<String>) -> Self {
        Self {
            env: env.into(),
            config_path: None,
            config_dir: PathBuf::from(DEFAULT_CONFIG_DIR),
            secrets_dir: PathBuf::from(DEFAULT_SECRETS_DIR),
            require_config_file: true,
            require_secrets_dir: false,
        }
    }

    /// 設定ファイルのパスを指定
    ///
    /// # Arguments
    ///
    /// * `path` - 設定ファイルのパス
    pub fn with_config_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.config_path = Some(path.into());
        self
    }

    /// 設定ファイルのディレクトリを指定
    ///
    /// # Arguments
    ///
    /// * `dir` - 設定ファイルのディレクトリ
    pub fn with_config_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.config_dir = dir.into();
        self
    }

    /// secrets ディレクトリを指定
    ///
    /// # Arguments
    ///
    /// * `dir` - secrets ディレクトリ
    pub fn with_secrets_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.secrets_dir = dir.into();
        self
    }

    /// 設定ファイルを必須にするかを指定
    pub fn require_config_file(mut self, require: bool) -> Self {
        self.require_config_file = require;
        self
    }

    /// secrets ディレクトリを必須にするかを指定
    pub fn require_secrets_dir(mut self, require: bool) -> Self {
        self.require_secrets_dir = require;
        self
    }

    /// 実際の設定ファイルのパスを取得
    pub fn effective_config_path(&self) -> PathBuf {
        self.config_path
            .clone()
            .unwrap_or_else(|| self.config_dir.join(format!("{}.yaml", self.env)))
    }

    /// 有効な環境名かどうかを検証
    pub fn is_valid_env(&self) -> bool {
        matches!(self.env.as_str(), "default" | "dev" | "stg" | "prod")
    }
}

impl Default for ConfigOptions {
    fn default() -> Self {
        Self::new("dev")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let options = ConfigOptions::new("prod");
        assert_eq!(options.env, "prod");
        assert!(options.config_path.is_none());
        assert_eq!(options.secrets_dir, PathBuf::from(DEFAULT_SECRETS_DIR));
    }

    #[test]
    fn test_with_config_path() {
        let options = ConfigOptions::new("dev").with_config_path("./config/custom.yaml");
        assert_eq!(
            options.config_path,
            Some(PathBuf::from("./config/custom.yaml"))
        );
    }

    #[test]
    fn test_with_secrets_dir() {
        let options = ConfigOptions::new("dev").with_secrets_dir("./secrets/dev");
        assert_eq!(options.secrets_dir, PathBuf::from("./secrets/dev"));
    }

    #[test]
    fn test_effective_config_path_default() {
        let options = ConfigOptions::new("dev");
        assert_eq!(
            options.effective_config_path(),
            PathBuf::from("/etc/k1s0/config/dev.yaml")
        );
    }

    #[test]
    fn test_effective_config_path_custom() {
        let options = ConfigOptions::new("dev").with_config_path("./config/custom.yaml");
        assert_eq!(
            options.effective_config_path(),
            PathBuf::from("./config/custom.yaml")
        );
    }

    #[test]
    fn test_effective_config_path_custom_dir() {
        let options = ConfigOptions::new("prod").with_config_dir("./config");
        assert_eq!(
            options.effective_config_path(),
            PathBuf::from("./config/prod.yaml")
        );
    }

    #[test]
    fn test_is_valid_env() {
        assert!(ConfigOptions::new("dev").is_valid_env());
        assert!(ConfigOptions::new("stg").is_valid_env());
        assert!(ConfigOptions::new("prod").is_valid_env());
        assert!(ConfigOptions::new("default").is_valid_env());
        assert!(!ConfigOptions::new("invalid").is_valid_env());
    }

    #[test]
    fn test_default() {
        let options = ConfigOptions::default();
        assert_eq!(options.env, "dev");
    }
}
