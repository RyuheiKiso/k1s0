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
//! # 優先順位（固定）
//!
//! 1. CLI 引数（参照先指定に限定）
//! 2. YAML（`config/{env}.yaml`。非機密の静的設定）
//! 3. DB（`fw_m_setting`。feature 固有の動的設定）※ 本 crate では未対応
//!
//! # 使用例
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

mod error;
mod loader;
mod options;
mod resolver;

pub use error::{ConfigError, ConfigResult};
pub use loader::{load_from_file, ConfigLoader};
pub use options::ConfigOptions;
pub use resolver::SecretResolver;

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
        assert_eq!(
            config_path("dev", None),
            "/etc/k1s0/config/dev.yaml"
        );
        assert_eq!(
            config_path("prod", None),
            "/etc/k1s0/config/prod.yaml"
        );
    }

    #[test]
    fn test_config_path_custom_dir() {
        assert_eq!(
            config_path("dev", Some("./config")),
            "./config/dev.yaml"
        );
    }
}
