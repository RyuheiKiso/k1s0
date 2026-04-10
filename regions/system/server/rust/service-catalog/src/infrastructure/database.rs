// secrecy クレートを使用してデータベースパスワードを Secret<String> で保持し、Debug 出力への漏洩を防ぐ（H-1 監査対応）。
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;

/// `DatabaseConfig` はデータベース接続の設定を表す。
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub user: String,
    // パスワードは Secret<String> で保持し、Debug トレイトでは [REDACTED] と表示される
    // パスワードは必須項目のため serde(default) を設定しない（Secret<String> は Default 未実装）
    pub password: Secret<String>,
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
    /// `PostgreSQL` 接続 URL を生成する。
    #[must_use]
    pub fn connection_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}?sslmode={}",
            // expose_secret() でパスワードを取り出し、接続 URL を構築する
            self.user,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.name,
            self.ssl_mode
        )
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use secrecy::Secret;

    use super::*;

    #[test]
    fn test_connection_url() {
        // password フィールドは Secret<String> 型のため Secret::new() でラップする（H-1 監査対応）
        let config = DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            name: "k1s0_system".to_string(),
            user: "app".to_string(),
            password: Secret::new("secret".to_string()),
            ssl_mode: "disable".to_string(),
            max_open_conns: 25,
            max_idle_conns: 5,
            conn_max_lifetime: "5m".to_string(),
        };

        assert_eq!(
            config.connection_url(),
            "postgres://app:secret@localhost:5432/k1s0_system?sslmode=disable"
        );
    }
}
