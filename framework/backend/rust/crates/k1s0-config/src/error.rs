//! エラー型
//!
//! 設定読み込みに関するエラーを定義する。
//! エラーには「原因」「対象」「次のアクション」を含める。

use std::path::PathBuf;
use thiserror::Error;

/// 設定エラー
#[derive(Debug, Error)]
pub enum ConfigError {
    /// 設定ファイルが見つからない
    #[error("設定ファイルが見つかりません: {path}")]
    ConfigFileNotFound {
        /// ファイルパス
        path: PathBuf,
        /// 次のアクション
        hint: String,
    },

    /// 設定ファイルの読み込みに失敗
    #[error("設定ファイルの読み込みに失敗しました: {path}")]
    ConfigFileReadError {
        /// ファイルパス
        path: PathBuf,
        /// 原因
        #[source]
        source: std::io::Error,
    },

    /// 設定ファイルのパースに失敗
    #[error("設定ファイルのパースに失敗しました: {path}")]
    ConfigParseError {
        /// ファイルパス
        path: PathBuf,
        /// 原因
        #[source]
        source: serde_yaml::Error,
    },

    /// secrets ディレクトリが見つからない
    #[error("secrets ディレクトリが見つかりません: {path}")]
    SecretsDirNotFound {
        /// ディレクトリパス
        path: PathBuf,
        /// 次のアクション
        hint: String,
    },

    /// secret ファイルが見つからない
    #[error("secret ファイルが見つかりません: {path}")]
    SecretFileNotFound {
        /// ファイルパス
        path: PathBuf,
        /// 参照元のキー
        key: String,
        /// 次のアクション
        hint: String,
    },

    /// secret ファイルの読み込みに失敗
    #[error("secret ファイルの読み込みに失敗しました: {path}")]
    SecretFileReadError {
        /// ファイルパス
        path: PathBuf,
        /// 参照元のキー
        key: String,
        /// 原因
        #[source]
        source: std::io::Error,
    },

    /// 必須設定が不足
    #[error("必須設定が不足しています: {key}")]
    RequiredConfigMissing {
        /// 設定キー
        key: String,
        /// 次のアクション
        hint: String,
    },

    /// 設定値が不正
    #[error("設定値が不正です: {key} = {value}")]
    InvalidConfigValue {
        /// 設定キー
        key: String,
        /// 設定値
        value: String,
        /// 次のアクション
        hint: String,
    },

    /// 環境名が不正
    #[error("環境名が不正です: {env}")]
    InvalidEnvironment {
        /// 環境名
        env: String,
        /// 次のアクション
        hint: String,
    },
}

impl ConfigError {
    /// 設定ファイルが見つからないエラーを作成
    pub fn config_not_found(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        Self::ConfigFileNotFound {
            hint: format!(
                "--config オプションで設定ファイルのパスを指定するか、{} に配置してください",
                path.display()
            ),
            path,
        }
    }

    /// secrets ディレクトリが見つからないエラーを作成
    pub fn secrets_dir_not_found(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        Self::SecretsDirNotFound {
            hint: format!(
                "--secrets-dir オプションで secrets ディレクトリを指定するか、{} を作成してください",
                path.display()
            ),
            path,
        }
    }

    /// secret ファイルが見つからないエラーを作成
    pub fn secret_file_not_found(path: impl Into<PathBuf>, key: impl Into<String>) -> Self {
        let path = path.into();
        let key = key.into();
        Self::SecretFileNotFound {
            hint: format!(
                "YAML の '{}' で指定されたファイル '{}' を配置してください。\n\
                 Kubernetes: Secret を volume mount してください。\n\
                 ローカル: --secrets-dir で指定したディレクトリに配置してください。",
                key,
                path.display()
            ),
            path,
            key,
        }
    }

    /// 必須設定が不足しているエラーを作成
    pub fn required_missing(key: impl Into<String>) -> Self {
        let key = key.into();
        Self::RequiredConfigMissing {
            hint: format!("config/{{env}}.yaml に '{}' を追加してください", key),
            key,
        }
    }

    /// 設定値が不正なエラーを作成
    pub fn invalid_value(
        key: impl Into<String>,
        value: impl Into<String>,
        hint: impl Into<String>,
    ) -> Self {
        Self::InvalidConfigValue {
            key: key.into(),
            value: value.into(),
            hint: hint.into(),
        }
    }

    /// 環境名が不正なエラーを作成
    pub fn invalid_env(env: impl Into<String>) -> Self {
        let env = env.into();
        Self::InvalidEnvironment {
            hint: "有効な環境名: dev, stg, prod".to_string(),
            env,
        }
    }

    /// ヒントを取得
    pub fn hint(&self) -> Option<&str> {
        match self {
            Self::ConfigFileNotFound { hint, .. } => Some(hint),
            Self::SecretsDirNotFound { hint, .. } => Some(hint),
            Self::SecretFileNotFound { hint, .. } => Some(hint),
            Self::RequiredConfigMissing { hint, .. } => Some(hint),
            Self::InvalidConfigValue { hint, .. } => Some(hint),
            Self::InvalidEnvironment { hint, .. } => Some(hint),
            _ => None,
        }
    }
}

/// 設定操作の結果型
pub type ConfigResult<T> = Result<T, ConfigError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_not_found_error() {
        let err = ConfigError::config_not_found("/etc/k1s0/config/dev.yaml");
        assert!(err.to_string().contains("設定ファイルが見つかりません"));
        assert!(err.hint().unwrap().contains("--config"));
    }

    #[test]
    fn test_secret_file_not_found_error() {
        let err = ConfigError::secret_file_not_found(
            "/var/run/secrets/k1s0/db_password",
            "db.password_file",
        );
        assert!(err.to_string().contains("secret ファイルが見つかりません"));
        assert!(err.hint().unwrap().contains("db.password_file"));
        assert!(err.hint().unwrap().contains("Kubernetes"));
    }

    #[test]
    fn test_required_missing_error() {
        let err = ConfigError::required_missing("db.host");
        assert!(err.to_string().contains("必須設定が不足"));
        assert!(err.hint().unwrap().contains("db.host"));
    }

    #[test]
    fn test_invalid_env_error() {
        let err = ConfigError::invalid_env("invalid");
        assert!(err.to_string().contains("環境名が不正"));
        assert!(err.hint().unwrap().contains("dev, stg, prod"));
    }
}
