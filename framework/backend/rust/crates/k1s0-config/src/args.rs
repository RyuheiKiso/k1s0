//! 起動引数
//!
//! サービスの起動引数を定義する。
//! clap feature を有効にすることで clap との統合が可能。
//!
//! # 設計方針
//!
//! - 環境変数は使用しない（明示的な引数指定のみ）
//! - `--env` は必須（暗黙のデフォルトなし）
//! - `--config` は任意（`{config_dir}/{env}.yaml` がデフォルト）
//! - `--secrets-dir` は任意（`/var/run/secrets/k1s0` がデフォルト）

use crate::{ConfigOptions, DEFAULT_CONFIG_DIR, DEFAULT_SECRETS_DIR};

/// 起動引数
///
/// サービスの起動に必要な最小限の引数を定義する。
/// clap feature を有効にすることで `#[derive(clap::Args)]` が使用可能。
#[derive(Debug, Clone)]
#[cfg_attr(feature = "clap", derive(clap::Args))]
pub struct ServiceArgs {
    /// 環境名（必須: dev, stg, prod）
    #[cfg_attr(feature = "clap", arg(long, short = 'e'))]
    pub env: String,

    /// 設定ファイルのパス（省略時: {config_dir}/{env}.yaml）
    #[cfg_attr(feature = "clap", arg(long, short = 'c'))]
    pub config: Option<String>,

    /// 設定ファイルのディレクトリ（省略時: /etc/k1s0/config）
    #[cfg_attr(feature = "clap", arg(long))]
    pub config_dir: Option<String>,

    /// secrets ディレクトリ（省略時: /var/run/secrets/k1s0）
    #[cfg_attr(feature = "clap", arg(long, short = 's'))]
    pub secrets_dir: Option<String>,
}

impl ServiceArgs {
    /// 新しい起動引数を作成
    pub fn new(env: impl Into<String>) -> Self {
        Self {
            env: env.into(),
            config: None,
            config_dir: None,
            secrets_dir: None,
        }
    }

    /// 設定ファイルのパスを指定
    pub fn with_config(mut self, path: impl Into<String>) -> Self {
        self.config = Some(path.into());
        self
    }

    /// 設定ファイルのディレクトリを指定
    pub fn with_config_dir(mut self, dir: impl Into<String>) -> Self {
        self.config_dir = Some(dir.into());
        self
    }

    /// secrets ディレクトリを指定
    pub fn with_secrets_dir(mut self, dir: impl Into<String>) -> Self {
        self.secrets_dir = Some(dir.into());
        self
    }

    /// ConfigOptions に変換
    pub fn to_config_options(&self) -> ConfigOptions {
        let mut options = ConfigOptions::new(&self.env);

        if let Some(config) = &self.config {
            options = options.with_config_path(config);
        }

        if let Some(config_dir) = &self.config_dir {
            options = options.with_config_dir(config_dir);
        }

        if let Some(secrets_dir) = &self.secrets_dir {
            options = options.with_secrets_dir(secrets_dir);
        }

        options
    }

    /// 環境名が有効かどうかを検証
    pub fn is_valid_env(&self) -> bool {
        matches!(self.env.as_str(), "default" | "dev" | "stg" | "prod")
    }

    /// 有効な設定ディレクトリを取得
    pub fn effective_config_dir(&self) -> &str {
        self.config_dir.as_deref().unwrap_or(DEFAULT_CONFIG_DIR)
    }

    /// 有効な secrets ディレクトリを取得
    pub fn effective_secrets_dir(&self) -> &str {
        self.secrets_dir.as_deref().unwrap_or(DEFAULT_SECRETS_DIR)
    }
}

impl Default for ServiceArgs {
    fn default() -> Self {
        Self::new("dev")
    }
}

/// サービスメインコマンド
///
/// clap feature を有効にすることで `#[derive(clap::Parser)]` が使用可能。
#[derive(Debug, Clone)]
#[cfg_attr(feature = "clap", derive(clap::Parser))]
#[cfg_attr(feature = "clap", command(author, version, about))]
pub struct ServiceCommand {
    /// サービス引数
    #[cfg_attr(feature = "clap", command(flatten))]
    pub args: ServiceArgs,
}

impl ServiceCommand {
    /// 新しいコマンドを作成
    pub fn new(env: impl Into<String>) -> Self {
        Self {
            args: ServiceArgs::new(env),
        }
    }

    /// ConfigOptions に変換
    pub fn to_config_options(&self) -> ConfigOptions {
        self.args.to_config_options()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_args_new() {
        let args = ServiceArgs::new("prod");
        assert_eq!(args.env, "prod");
        assert!(args.config.is_none());
        assert!(args.config_dir.is_none());
        assert!(args.secrets_dir.is_none());
    }

    #[test]
    fn test_service_args_with_config() {
        let args = ServiceArgs::new("dev")
            .with_config("./config/custom.yaml")
            .with_config_dir("./config")
            .with_secrets_dir("./secrets");

        assert_eq!(args.config, Some("./config/custom.yaml".to_string()));
        assert_eq!(args.config_dir, Some("./config".to_string()));
        assert_eq!(args.secrets_dir, Some("./secrets".to_string()));
    }

    #[test]
    fn test_service_args_to_config_options() {
        let args = ServiceArgs::new("prod")
            .with_config("./config/prod.yaml")
            .with_secrets_dir("./secrets/prod");

        let options = args.to_config_options();
        assert_eq!(options.env, "prod");
        assert!(options.config_path.is_some());
    }

    #[test]
    fn test_service_args_is_valid_env() {
        assert!(ServiceArgs::new("dev").is_valid_env());
        assert!(ServiceArgs::new("stg").is_valid_env());
        assert!(ServiceArgs::new("prod").is_valid_env());
        assert!(ServiceArgs::new("default").is_valid_env());
        assert!(!ServiceArgs::new("invalid").is_valid_env());
    }

    #[test]
    fn test_service_args_effective_dirs() {
        let args = ServiceArgs::new("dev");
        assert_eq!(args.effective_config_dir(), DEFAULT_CONFIG_DIR);
        assert_eq!(args.effective_secrets_dir(), DEFAULT_SECRETS_DIR);

        let args = ServiceArgs::new("dev")
            .with_config_dir("./config")
            .with_secrets_dir("./secrets");
        assert_eq!(args.effective_config_dir(), "./config");
        assert_eq!(args.effective_secrets_dir(), "./secrets");
    }

    #[test]
    fn test_service_command_new() {
        let cmd = ServiceCommand::new("prod");
        assert_eq!(cmd.args.env, "prod");
    }

    #[test]
    fn test_service_args_default() {
        let args = ServiceArgs::default();
        assert_eq!(args.env, "dev");
    }
}
