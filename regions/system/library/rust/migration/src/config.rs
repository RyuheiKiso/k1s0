use std::path::PathBuf;

use crate::error::MigrationError;

#[derive(Debug, Clone)]
pub struct MigrationConfig {
    pub migrations_dir: PathBuf,
    pub database_url: String,
    pub table_name: String,
}

impl MigrationConfig {
    pub fn new(migrations_dir: PathBuf, database_url: String) -> Self {
        Self {
            migrations_dir,
            database_url,
            table_name: "_migrations".to_string(),
        }
    }

    // M-04監査対応: テーブル名を SQL クエリに直接埋め込むため、SQL インジェクション防止のために正規表現でバリデーションする。
    // 英数字とアンダースコアのみ許可し、先頭は英字またはアンダースコアに限定する。
    pub fn with_table_name(mut self, table_name: impl Into<String>) -> Result<Self, MigrationError> {
        let name = table_name.into();
        let valid = {
            let mut chars = name.chars();
            match chars.next() {
                Some(first) if first.is_ascii_alphabetic() || first == '_' => {
                    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
                }
                _ => false,
            }
        };
        if !valid {
            return Err(MigrationError::InvalidTableName(name));
        }
        self.table_name = name;
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // MigrationConfig のデフォルトテーブル名が "_migrations" であることを確認する。
    #[test]
    fn test_default_table_name() {
        let config = MigrationConfig::new(
            PathBuf::from("./migrations"),
            "postgres://localhost/test".to_string(),
        );
        assert_eq!(config.table_name, "_migrations");
    }

    // with_table_name でカスタムテーブル名を設定できることを確認する。
    #[test]
    fn test_custom_table_name() {
        let config = MigrationConfig::new(
            PathBuf::from("./migrations"),
            "postgres://localhost/test".to_string(),
        )
        .with_table_name("custom_migrations")
        .expect("有効なテーブル名のため Ok が返るはず");
        assert_eq!(config.table_name, "custom_migrations");
    }

    // M-04監査対応: 不正なテーブル名（SQL インジェクション文字含む）でエラーが返ることを確認する。
    #[test]
    fn test_invalid_table_name_returns_error() {
        let result = MigrationConfig::new(
            PathBuf::from("./migrations"),
            "postgres://localhost/test".to_string(),
        )
        .with_table_name("bad; DROP TABLE users--");
        assert!(matches!(result, Err(MigrationError::InvalidTableName(_))));
    }

    // M-04監査対応: 先頭が数字のテーブル名はバリデーション失敗となることを確認する。
    #[test]
    fn test_table_name_starting_with_digit_returns_error() {
        let result = MigrationConfig::new(
            PathBuf::from("./migrations"),
            "postgres://localhost/test".to_string(),
        )
        .with_table_name("1invalid");
        assert!(matches!(result, Err(MigrationError::InvalidTableName(_))));
    }
}
