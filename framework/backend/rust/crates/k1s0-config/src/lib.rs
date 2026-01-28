//! k1s0-config
//!
//! 設定読み込みライブラリ。環境変数を使用せず、YAML ファイルと secrets ファイルから
//! 設定を読み込む。
//!
//! # 設計方針
//!
//! - 環境変数は使用しない（CLI 引数で参照先を指定）
//! - 機密情報は YAML に直接書かず、`*_file` キーでファイルパスを参照
//! - `--secrets-dir` で secrets ファイルの配置先を指定
//!
//! # 起動引数
//!
//! - `--env` / `-e`: 環境名（必須: dev, stg, prod）
//! - `--config` / `-c`: 設定ファイルのパス（省略時: {config_dir}/{env}.yaml）
//! - `--config-dir`: 設定ファイルのディレクトリ（省略時: /etc/k1s0/config）
//! - `--secrets-dir` / `-s`: secrets ディレクトリ（省略時: /var/run/secrets/k1s0）
//!
//! # 優先順位（固定）
//!
//! 1. CLI 引数（参照先指定に限定）
//! 2. YAML（`config/{env}.yaml`。非機密の静的設定）
//! 3. DB（`fw_m_setting`。feature 固有の動的設定）※ `db` feature で有効化
//!
//! # 使用例（基本）
//!
//! ```ignore
//! use k1s0_config::{ConfigLoader, ConfigOptions};
//! use serde::Deserialize;
//!
//! #[derive(Debug, Deserialize)]
//! struct AppConfig {
//!     db: DbConfig,
//! }
//!
//! #[derive(Debug, Deserialize)]
//! struct DbConfig {
//!     host: String,
//!     port: u16,
//!     password_file: String,
//! }
//!
//! let options = ConfigOptions::new("dev")
//!     .with_config_path("config/dev.yaml")
//!     .with_secrets_dir("/var/run/secrets/k1s0");
//!
//! let loader = ConfigLoader::new(options)?;
//! let config: AppConfig = loader.load()?;
//!
//! // *_file キーの値をファイルから読み込む
//! let password = loader.resolve_secret_file(&config.db.password_file)?;
//! ```
//!
//! # 使用例（ServiceInit による簡易初期化）
//!
//! ```ignore
//! use k1s0_config::{ServiceInit, ServiceArgs};
//! use serde::Deserialize;
//!
//! #[derive(Debug, Deserialize)]
//! struct AppConfig {
//!     name: String,
//!     port: u16,
//! }
//!
//! // 起動引数から初期化
//! let args = ServiceArgs::new("dev")
//!     .with_config_dir("./config");
//!
//! let init = ServiceInit::from_args(&args)?;
//! let config: AppConfig = init.load_config()?;
//!
//! // 環境に応じた処理
//! if init.is_production() {
//!     // 本番環境向け処理
//! }
//! ```
//!
//! # 使用例（clap との統合）
//!
//! ```ignore
//! // Cargo.toml で clap feature を有効化:
//! // k1s0-config = { version = "0.1", features = ["clap"] }
//!
//! use clap::Parser;
//! use k1s0_config::{ServiceCommand, ServiceInit};
//!
//! #[derive(Parser)]
//! struct MyService {
//!     #[command(flatten)]
//!     common: ServiceCommand,
//!
//!     #[arg(long)]
//!     custom_option: Option<String>,
//! }
//!
//! let cli = MyService::parse();
//! let init = ServiceInit::from_args(&cli.common.args)?;
//! ```

pub mod args;
mod error;
pub mod init;
mod loader;
mod options;
mod resolver;

#[cfg(feature = "db")]
pub mod db;

pub use args::{ServiceArgs, ServiceCommand};
pub use error::{ConfigError, ConfigResult};
pub use init::{ServiceConfig, ServiceInit};
pub use loader::{load_from_file, ConfigLoader};
pub use options::ConfigOptions;
pub use resolver::SecretResolver;

#[cfg(feature = "db")]
pub use db::{
    DbConfigLoader, DbSettingError, DbSettingRepository, FailureMode, MockDbSettingRepository,
    SettingEntry,
};

/// デフォルトの secrets ディレクトリ
pub const DEFAULT_SECRETS_DIR: &str = "/var/run/secrets/k1s0";

/// デフォルトの config ディレクトリ
pub const DEFAULT_CONFIG_DIR: &str = "/etc/k1s0/config";

/// 設定ファイルのパスを生成する
///
/// # Arguments
///
/// * `env` - 環境名（dev, stg, prod）
/// * `config_dir` - 設定ファイルのディレクトリ（None の場合はデフォルト）
///
/// # Returns
///
/// 設定ファイルのパス（例: `/etc/k1s0/config/dev.yaml`）
pub fn config_path(env: &str, config_dir: Option<&str>) -> String {
    let dir = config_dir.unwrap_or(DEFAULT_CONFIG_DIR);
    format!("{}/{}.yaml", dir, env)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_path_default() {
        assert_eq!(config_path("dev", None), "/etc/k1s0/config/dev.yaml");
        assert_eq!(config_path("prod", None), "/etc/k1s0/config/prod.yaml");
    }

    #[test]
    fn test_config_path_custom_dir() {
        assert_eq!(config_path("dev", Some("./config")), "./config/dev.yaml");
    }
}
