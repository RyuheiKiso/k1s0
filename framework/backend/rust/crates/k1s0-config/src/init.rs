//! サービス初期化
//!
//! サービスの初期化を簡単にするためのヘルパーを提供する。
//!
//! # 使用例
//!
//! ```ignore
//! use k1s0_config::{ServiceInit, ServiceArgs};
//! use serde::Deserialize;
//!
//! #[derive(Debug, Deserialize)]
//! struct AppConfig {
//!     server: ServerConfig,
//!     db: DbConfig,
//! }
//!
//! // サービス初期化
//! let init = ServiceInit::from_args(&args)?;
//! let config: AppConfig = init.load_config()?;
//!
//! // secrets の解決
//! let db_password = init.resolve_secret(&config.db.password_file)?;
//! ```

use std::path::Path;

use serde::de::DeserializeOwned;

use crate::{args::ServiceArgs, ConfigError, ConfigLoader, ConfigOptions, ConfigResult};

/// サービス初期化コンテキスト
///
/// 設定の読み込みと secrets の解決を一元化する。
#[derive(Debug)]
pub struct ServiceInit {
    /// 環境名
    env: String,
    /// 設定ローダー
    loader: ConfigLoader,
    /// 設定オプション
    options: ConfigOptions,
}

impl ServiceInit {
    /// ServiceArgs から初期化
    pub fn from_args(args: &ServiceArgs) -> ConfigResult<Self> {
        let options = args.to_config_options();
        Self::from_options(options)
    }

    /// ConfigOptions から初期化
    pub fn from_options(options: ConfigOptions) -> ConfigResult<Self> {
        // 環境名のバリデーション
        if !options.is_valid_env() {
            return Err(ConfigError::invalid_env(&options.env));
        }

        let loader = ConfigLoader::new(options.clone())?;

        Ok(Self {
            env: options.env.clone(),
            loader,
            options,
        })
    }

    /// 開発環境用の初期化（ローカル開発向け）
    ///
    /// - config_dir: `./config`
    /// - secrets_dir: `./secrets` (存在しなくてもOK)
    pub fn for_development() -> ConfigResult<Self> {
        let options = ConfigOptions::new("dev")
            .with_config_dir("./config")
            .with_secrets_dir("./secrets")
            .require_secrets_dir(false);

        Self::from_options(options)
    }

    /// 設定ファイルを読み込む
    pub fn load_config<T>(&self) -> ConfigResult<T>
    where
        T: DeserializeOwned,
    {
        self.loader.load()
    }

    /// secrets ファイルを解決
    ///
    /// `*_file` キーの値をファイルから読み込む。
    ///
    /// # Arguments
    ///
    /// * `file_ref` - ファイル参照（ファイル名または相対パス）
    /// * `key` - 設定キー名（エラーメッセージ用）
    pub fn resolve_secret(&self, file_ref: &str, key: &str) -> ConfigResult<String> {
        self.loader.resolve_secret_file(file_ref, key)
    }

    /// 複数の secrets ファイルを解決
    pub fn resolve_secrets(&self, file_refs: &[(&str, &str)]) -> ConfigResult<Vec<String>> {
        file_refs
            .iter()
            .map(|(file_ref, key)| self.resolve_secret(file_ref, key))
            .collect()
    }

    /// 環境名を取得
    pub fn env(&self) -> &str {
        &self.env
    }

    /// 本番環境かどうか
    pub fn is_production(&self) -> bool {
        self.env == "prod"
    }

    /// 開発環境かどうか
    pub fn is_development(&self) -> bool {
        self.env == "dev"
    }

    /// ステージング環境かどうか
    pub fn is_staging(&self) -> bool {
        self.env == "stg"
    }

    /// 設定ファイルのパスを取得
    pub fn config_path(&self) -> std::path::PathBuf {
        self.loader.config_path()
    }

    /// secrets ディレクトリを取得
    pub fn secrets_dir(&self) -> &Path {
        &self.options.secrets_dir
    }

    /// ConfigLoader を取得
    pub fn loader(&self) -> &ConfigLoader {
        &self.loader
    }

    /// ConfigOptions を取得
    pub fn options(&self) -> &ConfigOptions {
        &self.options
    }
}

/// サービス設定のトレイト
///
/// 設定構造体に実装することで、バリデーションや初期化処理を統一できる。
pub trait ServiceConfig: Sized + DeserializeOwned {
    /// 設定をバリデーション
    fn validate(&self) -> ConfigResult<()> {
        Ok(())
    }

    /// secrets を解決した設定を作成
    fn resolve_secrets(&mut self, _init: &ServiceInit) -> ConfigResult<()> {
        Ok(())
    }

    /// 設定を読み込んでバリデーション
    fn load(init: &ServiceInit) -> ConfigResult<Self> {
        let config: Self = init.load_config()?;
        config.validate()?;
        Ok(config)
    }

    /// 設定を読み込んで secrets を解決してバリデーション
    fn load_with_secrets(init: &ServiceInit) -> ConfigResult<Self> {
        let mut config: Self = init.load_config()?;
        config.resolve_secrets(init)?;
        config.validate()?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::fs;
    use tempfile::TempDir;

    #[derive(Debug, Deserialize)]
    struct TestConfig {
        name: String,
        port: u16,
    }

    fn create_test_config(dir: &Path, env: &str) -> std::io::Result<()> {
        let config_dir = dir.join("config");
        fs::create_dir_all(&config_dir)?;
        let config_path = config_dir.join(format!("{}.yaml", env));
        fs::write(
            config_path,
            "name: test-service\nport: 8080\n",
        )?;
        Ok(())
    }

    #[test]
    fn test_service_init_from_args() {
        let temp_dir = TempDir::new().unwrap();
        create_test_config(temp_dir.path(), "dev").unwrap();

        let args = ServiceArgs::new("dev")
            .with_config_dir(temp_dir.path().join("config").to_str().unwrap());

        let init = ServiceInit::from_args(&args).unwrap();
        assert_eq!(init.env(), "dev");
        assert!(init.is_development());
    }

    #[test]
    fn test_service_init_load_config() {
        let temp_dir = TempDir::new().unwrap();
        create_test_config(temp_dir.path(), "dev").unwrap();

        let args = ServiceArgs::new("dev")
            .with_config_dir(temp_dir.path().join("config").to_str().unwrap());

        let init = ServiceInit::from_args(&args).unwrap();
        let config: TestConfig = init.load_config().unwrap();

        assert_eq!(config.name, "test-service");
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn test_service_init_env_methods() {
        let temp_dir = TempDir::new().unwrap();

        // dev
        create_test_config(temp_dir.path(), "dev").unwrap();
        let args = ServiceArgs::new("dev")
            .with_config_dir(temp_dir.path().join("config").to_str().unwrap());
        let init = ServiceInit::from_args(&args).unwrap();
        assert!(init.is_development());
        assert!(!init.is_staging());
        assert!(!init.is_production());

        // stg
        create_test_config(temp_dir.path(), "stg").unwrap();
        let args = ServiceArgs::new("stg")
            .with_config_dir(temp_dir.path().join("config").to_str().unwrap());
        let init = ServiceInit::from_args(&args).unwrap();
        assert!(init.is_staging());

        // prod
        create_test_config(temp_dir.path(), "prod").unwrap();
        let args = ServiceArgs::new("prod")
            .with_config_dir(temp_dir.path().join("config").to_str().unwrap());
        let init = ServiceInit::from_args(&args).unwrap();
        assert!(init.is_production());
    }

    #[test]
    fn test_service_init_invalid_env() {
        let args = ServiceArgs::new("invalid");
        let result = ServiceInit::from_args(&args);
        assert!(result.is_err());
    }
}
