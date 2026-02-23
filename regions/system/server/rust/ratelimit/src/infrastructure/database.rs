use serde::Deserialize;

/// DatabaseConfig はデータベース接続の設定を表す。
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub user: String,
    #[serde(default)]
    pub password: String,
    #[serde(default = "default_ssl_mode")]
    pub ssl_mode: String,
    #[serde(default = "default_max_open_conns")]
    pub max_open_conns: u32,
}

fn default_ssl_mode() -> String {
    "disable".to_string()
}

fn default_max_open_conns() -> u32 {
    25
}

impl DatabaseConfig {
    /// PostgreSQL 接続 URL を生成する。
    pub fn connection_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}?sslmode={}",
            self.user, self.password, self.host, self.port, self.name, self.ssl_mode
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_url() {
        let config = DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            name: "ratelimit_db".to_string(),
            user: "dev".to_string(),
            password: "dev".to_string(),
            ssl_mode: "disable".to_string(),
            max_open_conns: 25,
        };

        assert_eq!(
            config.connection_url(),
            "postgres://dev:dev@localhost:5432/ratelimit_db?sslmode=disable"
        );
    }

    #[test]
    fn test_database_config_defaults() {
        let yaml = r#"
host: "localhost"
port: 5432
name: "ratelimit_db"
user: "dev"
"#;
        let config: DatabaseConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.ssl_mode, "disable");
        assert_eq!(config.max_open_conns, 25);
    }
}
