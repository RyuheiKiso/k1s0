use std::path::PathBuf;

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

    pub fn with_table_name(mut self, table_name: impl Into<String>) -> Self {
        self.table_name = table_name.into();
        self
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
        .with_table_name("custom_migrations");
        assert_eq!(config.table_name, "custom_migrations");
    }
}
