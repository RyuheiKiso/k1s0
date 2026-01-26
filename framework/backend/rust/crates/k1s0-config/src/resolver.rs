//! シークレット解決
//!
//! `*_file` キーで指定されたファイルパスからシークレット値を読み込む。

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{ConfigError, ConfigResult};

/// シークレット解決器
///
/// YAML の `*_file` キーで指定されたファイルパスから値を読み込む。
/// Kubernetes の Secret volume mount パターンに対応。
#[derive(Debug, Clone)]
pub struct SecretResolver {
    /// secrets ディレクトリ
    secrets_dir: PathBuf,
}

impl SecretResolver {
    /// 新しいシークレット解決器を作成
    ///
    /// # Arguments
    ///
    /// * `secrets_dir` - secrets ディレクトリのパス
    pub fn new(secrets_dir: impl Into<PathBuf>) -> Self {
        Self {
            secrets_dir: secrets_dir.into(),
        }
    }

    /// secrets ディレクトリを取得
    pub fn secrets_dir(&self) -> &Path {
        &self.secrets_dir
    }

    /// secrets ディレクトリが存在するか確認
    pub fn secrets_dir_exists(&self) -> bool {
        self.secrets_dir.exists() && self.secrets_dir.is_dir()
    }

    /// `*_file` キーの値からシークレットを解決
    ///
    /// # Arguments
    ///
    /// * `file_value` - YAML の `*_file` キーの値（ファイルパスまたはファイル名）
    /// * `key` - 参照元のキー名（エラーメッセージ用）
    ///
    /// # Returns
    ///
    /// ファイルの内容（末尾の改行は削除）
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let resolver = SecretResolver::new("/var/run/secrets/k1s0");
    ///
    /// // 絶対パスの場合はそのまま読み込む
    /// let password = resolver.resolve("/var/run/secrets/k1s0/db_password", "db.password_file")?;
    ///
    /// // 相対パスの場合は secrets_dir からの相対パス
    /// let token = resolver.resolve("api_token", "api.token_file")?;
    /// ```
    pub fn resolve(&self, file_value: &str, key: &str) -> ConfigResult<String> {
        let path = self.resolve_path(file_value);

        if !path.exists() {
            return Err(ConfigError::secret_file_not_found(&path, key));
        }

        let content = fs::read_to_string(&path).map_err(|source| {
            ConfigError::SecretFileReadError {
                path: path.clone(),
                key: key.to_string(),
                source,
            }
        })?;

        // 末尾の改行を削除（Kubernetes Secret は改行を含むことがある）
        Ok(content.trim_end_matches('\n').trim_end_matches('\r').to_string())
    }

    /// ファイルパスを解決（読み込みなし）
    ///
    /// # Arguments
    ///
    /// * `file_value` - YAML の `*_file` キーの値
    ///
    /// # Returns
    ///
    /// 解決されたファイルパス
    pub fn resolve_path(&self, file_value: &str) -> PathBuf {
        let path = Path::new(file_value);

        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.secrets_dir.join(file_value)
        }
    }

    /// シークレットファイルが存在するか確認
    ///
    /// # Arguments
    ///
    /// * `file_value` - YAML の `*_file` キーの値
    pub fn exists(&self, file_value: &str) -> bool {
        self.resolve_path(file_value).exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_new() {
        let resolver = SecretResolver::new("/var/run/secrets/k1s0");
        assert_eq!(
            resolver.secrets_dir(),
            Path::new("/var/run/secrets/k1s0")
        );
    }

    #[test]
    fn test_resolve_path_absolute() {
        let resolver = SecretResolver::new("/var/run/secrets/k1s0");
        let path = resolver.resolve_path("/absolute/path/to/secret");
        assert_eq!(path, PathBuf::from("/absolute/path/to/secret"));
    }

    #[test]
    fn test_resolve_path_relative() {
        let resolver = SecretResolver::new("/var/run/secrets/k1s0");
        let path = resolver.resolve_path("db_password");
        assert_eq!(path, PathBuf::from("/var/run/secrets/k1s0/db_password"));
    }

    #[test]
    fn test_resolve_success() {
        let dir = tempdir().unwrap();
        let secret_path = dir.path().join("db_password");
        fs::write(&secret_path, "super_secret_password\n").unwrap();

        let resolver = SecretResolver::new(dir.path());
        let result = resolver.resolve("db_password", "db.password_file");

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "super_secret_password");
    }

    #[test]
    fn test_resolve_trims_newlines() {
        let dir = tempdir().unwrap();
        let secret_path = dir.path().join("token");
        fs::write(&secret_path, "my_token\r\n").unwrap();

        let resolver = SecretResolver::new(dir.path());
        let result = resolver.resolve("token", "api.token_file").unwrap();

        assert_eq!(result, "my_token");
    }

    #[test]
    fn test_resolve_file_not_found() {
        let dir = tempdir().unwrap();
        let resolver = SecretResolver::new(dir.path());

        let result = resolver.resolve("nonexistent", "db.password_file");

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("secret ファイルが見つかりません"));
        assert!(err.hint().unwrap().contains("db.password_file"));
    }

    #[test]
    fn test_exists() {
        let dir = tempdir().unwrap();
        let secret_path = dir.path().join("existing_secret");
        fs::write(&secret_path, "value").unwrap();

        let resolver = SecretResolver::new(dir.path());

        assert!(resolver.exists("existing_secret"));
        assert!(!resolver.exists("nonexistent"));
    }

    #[test]
    fn test_secrets_dir_exists() {
        let dir = tempdir().unwrap();
        let resolver = SecretResolver::new(dir.path());
        assert!(resolver.secrets_dir_exists());

        let resolver_nonexistent = SecretResolver::new("/nonexistent/path");
        assert!(!resolver_nonexistent.secrets_dir_exists());
    }
}
