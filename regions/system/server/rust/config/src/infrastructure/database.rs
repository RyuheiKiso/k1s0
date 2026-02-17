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
    #[serde(default = "default_max_idle_conns")]
    pub max_idle_conns: u32,
    #[serde(default = "default_conn_max_lifetime")]
    pub conn_max_lifetime: String,
}

fn default_ssl_mode() -> String {
    "disable".to_string()
}

fn default_max_open_conns() -> u32 {
    25
}

fn default_max_idle_conns() -> u32 {
    5
}

fn default_conn_max_lifetime() -> String {
    "5m".to_string()
}

impl DatabaseConfig {
    /// PostgreSQL 接続 URL を生成する。
    /// search_path=config を設定し、config スキーマのテーブルに直接アクセスできるようにする。
    pub fn connection_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}?sslmode={}&options=-c search_path%3Dconfig,public",
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
            name: "k1s0_config".to_string(),
            user: "app".to_string(),
            password: "secret".to_string(),
            ssl_mode: "disable".to_string(),
            max_open_conns: 25,
            max_idle_conns: 5,
            conn_max_lifetime: "5m".to_string(),
        };

        assert_eq!(
            config.connection_url(),
            "postgres://app:secret@localhost:5432/k1s0_config?sslmode=disable&options=-c search_path%3Dconfig,public"
        );
    }

    #[test]
    fn test_database_config_deserialization() {
        let yaml = r#"
host: "postgres.k1s0-system.svc.cluster.local"
port: 5432
name: "k1s0_config"
user: "app"
password: "pass"
ssl_mode: "require"
max_open_conns: 50
max_idle_conns: 10
conn_max_lifetime: "10m"
"#;
        let config: DatabaseConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.host, "postgres.k1s0-system.svc.cluster.local");
        assert_eq!(config.port, 5432);
        assert_eq!(config.name, "k1s0_config");
        assert_eq!(config.ssl_mode, "require");
        assert_eq!(config.max_open_conns, 50);
    }

    #[test]
    fn test_database_config_defaults() {
        let yaml = r#"
host: "localhost"
port: 5432
name: "k1s0_config"
user: "app"
"#;
        let config: DatabaseConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.ssl_mode, "disable");
        assert_eq!(config.max_open_conns, 25);
        assert_eq!(config.max_idle_conns, 5);
        assert_eq!(config.conn_max_lifetime, "5m");
    }
}
