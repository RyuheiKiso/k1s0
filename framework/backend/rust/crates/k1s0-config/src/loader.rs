//! 設定ローダー
//!
//! YAML ファイルから設定を読み込み、指定された型にデシリアライズする。

use std::fs;
use std::path::Path;

use serde::de::DeserializeOwned;

use crate::error::{ConfigError, ConfigResult};
use crate::options::ConfigOptions;
use crate::resolver::SecretResolver;

/// 設定ローダー
///
/// YAML ファイルから設定を読み込む。
/// シークレット解決機能も提供。
#[derive(Debug)]
pub struct ConfigLoader {
    /// 設定オプション
    options: ConfigOptions,
    /// シークレット解決器
    secret_resolver: SecretResolver,
    /// 設定ファイルの内容（キャッシュ）
    raw_content: Option<String>,
}

impl ConfigLoader {
    /// 新しい設定ローダーを作成
    ///
    /// # Arguments
    ///
    /// * `options` - 設定オプション
    ///
    /// # Returns
    ///
    /// 設定ローダー、またはエラー
    ///
    /// # Errors
    ///
    /// - 設定ファイルが見つからない場合（`require_config_file` が true のとき）
    /// - secrets ディレクトリが見つからない場合（`require_secrets_dir` が true のとき）
    /// - 環境名が不正な場合
    pub fn new(options: ConfigOptions) -> ConfigResult<Self> {
        // 環境名の検証
        if !options.is_valid_env() {
            return Err(ConfigError::invalid_env(&options.env));
        }

        let config_path = options.effective_config_path();

        // 設定ファイルの存在確認
        if options.require_config_file && !config_path.exists() {
            return Err(ConfigError::config_not_found(&config_path));
        }

        // secrets ディレクトリの存在確認
        if options.require_secrets_dir && !options.secrets_dir.exists() {
            return Err(ConfigError::secrets_dir_not_found(&options.secrets_dir));
        }

        let secret_resolver = SecretResolver::new(&options.secrets_dir);

        Ok(Self {
            options,
            secret_resolver,
            raw_content: None,
        })
    }

    /// 設定ファイルを読み込み、指定された型にデシリアライズ
    ///
    /// # Type Parameters
    ///
    /// * `T` - デシリアライズ先の型（`Deserialize` を実装）
    ///
    /// # Returns
    ///
    /// デシリアライズされた設定、またはエラー
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use k1s0_config::{ConfigLoader, ConfigOptions};
    /// use serde::Deserialize;
    ///
    /// #[derive(Debug, Deserialize)]
    /// struct AppConfig {
    ///     server: ServerConfig,
    /// }
    ///
    /// #[derive(Debug, Deserialize)]
    /// struct ServerConfig {
    ///     host: String,
    ///     port: u16,
    /// }
    ///
    /// let options = ConfigOptions::new("dev")
    ///     .with_config_path("config/dev.yaml");
    /// let loader = ConfigLoader::new(options)?;
    /// let config: AppConfig = loader.load()?;
    /// ```
    pub fn load<T: DeserializeOwned>(&self) -> ConfigResult<T> {
        let config_path = self.options.effective_config_path();

        // 設定ファイルが存在しない場合
        if !config_path.exists() {
            if self.options.require_config_file {
                return Err(ConfigError::config_not_found(&config_path));
            }
            // 空の YAML としてデシリアライズを試みる
            return serde_yaml::from_str("{}").map_err(|source| {
                ConfigError::ConfigParseError {
                    path: config_path,
                    source,
                }
            });
        }

        let content = fs::read_to_string(&config_path).map_err(|source| {
            ConfigError::ConfigFileReadError {
                path: config_path.clone(),
                source,
            }
        })?;

        serde_yaml::from_str(&content).map_err(|source| {
            ConfigError::ConfigParseError {
                path: config_path,
                source,
            }
        })
    }

    /// 設定ファイルを読み込み、生のYAML文字列として返す
    ///
    /// # Returns
    ///
    /// YAML ファイルの内容、またはエラー
    pub fn load_raw(&mut self) -> ConfigResult<&str> {
        if self.raw_content.is_some() {
            return Ok(self.raw_content.as_ref().unwrap());
        }

        let config_path = self.options.effective_config_path();

        if !config_path.exists() {
            if self.options.require_config_file {
                return Err(ConfigError::config_not_found(&config_path));
            }
            self.raw_content = Some(String::new());
            return Ok(self.raw_content.as_ref().unwrap());
        }

        let content = fs::read_to_string(&config_path).map_err(|source| {
            ConfigError::ConfigFileReadError {
                path: config_path,
                source,
            }
        })?;

        self.raw_content = Some(content);
        Ok(self.raw_content.as_ref().unwrap())
    }

    /// `*_file` キーの値からシークレットを解決
    ///
    /// # Arguments
    ///
    /// * `file_value` - YAML の `*_file` キーの値
    /// * `key` - 参照元のキー名（エラーメッセージ用）
    ///
    /// # Returns
    ///
    /// ファイルの内容
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // config/dev.yaml:
    /// // db:
    /// //   host: localhost
    /// //   password_file: db_password
    ///
    /// let password = loader.resolve_secret_file("db_password", "db.password_file")?;
    /// ```
    pub fn resolve_secret_file(&self, file_value: &str, key: &str) -> ConfigResult<String> {
        self.secret_resolver.resolve(file_value, key)
    }

    /// シークレット解決器への参照を取得
    pub fn secret_resolver(&self) -> &SecretResolver {
        &self.secret_resolver
    }

    /// 設定オプションへの参照を取得
    pub fn options(&self) -> &ConfigOptions {
        &self.options
    }

    /// 環境名を取得
    pub fn env(&self) -> &str {
        &self.options.env
    }

    /// 設定ファイルのパスを取得
    pub fn config_path(&self) -> std::path::PathBuf {
        self.options.effective_config_path()
    }
}

/// 設定ファイルから直接読み込むヘルパー関数
///
/// # Arguments
///
/// * `path` - 設定ファイルのパス
///
/// # Type Parameters
///
/// * `T` - デシリアライズ先の型
///
/// # Returns
///
/// デシリアライズされた設定、またはエラー
pub fn load_from_file<T: DeserializeOwned>(path: impl AsRef<Path>) -> ConfigResult<T> {
    let path = path.as_ref();

    if !path.exists() {
        return Err(ConfigError::config_not_found(path));
    }

    let content = fs::read_to_string(path).map_err(|source| {
        ConfigError::ConfigFileReadError {
            path: path.to_path_buf(),
            source,
        }
    })?;

    serde_yaml::from_str(&content).map_err(|source| {
        ConfigError::ConfigParseError {
            path: path.to_path_buf(),
            source,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::fs;
    use tempfile::tempdir;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestConfig {
        server: ServerConfig,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct ServerConfig {
        host: String,
        port: u16,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct ConfigWithSecret {
        db: DbConfig,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct DbConfig {
        host: String,
        password_file: String,
    }

    #[test]
    fn test_new_with_invalid_env() {
        let options = ConfigOptions::new("invalid")
            .require_config_file(false);
        let result = ConfigLoader::new(options);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("環境名が不正"));
    }

    #[test]
    fn test_new_config_not_found() {
        let options = ConfigOptions::new("dev")
            .with_config_path("/nonexistent/config.yaml")
            .require_config_file(true);
        let result = ConfigLoader::new(options);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("設定ファイルが見つかりません"));
    }

    #[test]
    fn test_new_config_not_required() {
        let options = ConfigOptions::new("dev")
            .with_config_path("/nonexistent/config.yaml")
            .require_config_file(false);
        let result = ConfigLoader::new(options);

        assert!(result.is_ok());
    }

    #[test]
    fn test_load_success() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("dev.yaml");
        fs::write(
            &config_path,
            r#"
server:
  host: localhost
  port: 8080
"#,
        )
        .unwrap();

        let options = ConfigOptions::new("dev")
            .with_config_path(&config_path);
        let loader = ConfigLoader::new(options).unwrap();
        let config: TestConfig = loader.load().unwrap();

        assert_eq!(config.server.host, "localhost");
        assert_eq!(config.server.port, 8080);
    }

    #[test]
    fn test_load_parse_error() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("dev.yaml");
        fs::write(&config_path, "invalid: yaml: content:").unwrap();

        let options = ConfigOptions::new("dev")
            .with_config_path(&config_path);
        let loader = ConfigLoader::new(options).unwrap();
        let result: ConfigResult<TestConfig> = loader.load();

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("パースに失敗"));
    }

    #[test]
    fn test_resolve_secret_file() {
        let dir = tempdir().unwrap();

        // 設定ファイル作成
        let config_path = dir.path().join("dev.yaml");
        fs::write(
            &config_path,
            r#"
db:
  host: localhost
  password_file: db_password
"#,
        )
        .unwrap();

        // secrets ディレクトリ作成
        let secrets_dir = dir.path().join("secrets");
        fs::create_dir(&secrets_dir).unwrap();
        fs::write(secrets_dir.join("db_password"), "my_secret_password\n").unwrap();

        let options = ConfigOptions::new("dev")
            .with_config_path(&config_path)
            .with_secrets_dir(&secrets_dir);
        let loader = ConfigLoader::new(options).unwrap();

        // 設定を読み込む
        let config: ConfigWithSecret = loader.load().unwrap();
        assert_eq!(config.db.host, "localhost");
        assert_eq!(config.db.password_file, "db_password");

        // シークレットを解決
        let password = loader
            .resolve_secret_file(&config.db.password_file, "db.password_file")
            .unwrap();
        assert_eq!(password, "my_secret_password");
    }

    #[test]
    fn test_resolve_secret_file_not_found() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("dev.yaml");
        fs::write(&config_path, "dummy: value").unwrap();

        let secrets_dir = dir.path().join("secrets");
        fs::create_dir(&secrets_dir).unwrap();

        let options = ConfigOptions::new("dev")
            .with_config_path(&config_path)
            .with_secrets_dir(&secrets_dir);
        let loader = ConfigLoader::new(options).unwrap();

        let result = loader.resolve_secret_file("nonexistent", "db.password_file");
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(err.to_string().contains("secret ファイルが見つかりません"));

        // hint には参照元キーと Kubernetes の情報が含まれる
        let hint = err.hint().unwrap();
        assert!(hint.contains("db.password_file"));
        assert!(hint.contains("Kubernetes"));
    }

    #[test]
    fn test_load_from_file() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.yaml");
        fs::write(
            &config_path,
            r#"
server:
  host: 127.0.0.1
  port: 3000
"#,
        )
        .unwrap();

        let config: TestConfig = load_from_file(&config_path).unwrap();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 3000);
    }

    #[test]
    fn test_load_from_file_not_found() {
        let result: ConfigResult<TestConfig> = load_from_file("/nonexistent/config.yaml");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("見つかりません"));
    }

    #[test]
    fn test_env_and_config_path() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("prod.yaml");
        fs::write(&config_path, "dummy: value").unwrap();

        let options = ConfigOptions::new("prod")
            .with_config_path(&config_path);
        let loader = ConfigLoader::new(options).unwrap();

        assert_eq!(loader.env(), "prod");
        assert_eq!(loader.config_path(), config_path);
    }

    #[test]
    fn test_load_raw() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("dev.yaml");
        let content = "server:\n  host: localhost\n";
        fs::write(&config_path, content).unwrap();

        let options = ConfigOptions::new("dev")
            .with_config_path(&config_path);
        let mut loader = ConfigLoader::new(options).unwrap();

        let raw = loader.load_raw().unwrap();
        assert_eq!(raw, content);

        // キャッシュからの再取得
        let raw2 = loader.load_raw().unwrap();
        assert_eq!(raw2, content);
    }
}
