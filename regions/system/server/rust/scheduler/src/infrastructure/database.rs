use sqlx::PgPool;

use super::config::DatabaseConfig;

/// PostgreSQL 接続プールを作成する。
pub async fn connect(cfg: &DatabaseConfig) -> anyhow::Result<PgPool> {
    // DATABASE_URL 環境変数が設定されている場合は優先する（serde_yaml はシェル変数を展開しないため）
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| cfg.connection_url());
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(cfg.max_open_conns)
        .connect(&url)
        .await?;
    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::super::config::DatabaseConfig;

    #[test]
    fn test_connection_url_format() {
        let cfg = DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            name: "k1s0_system".to_string(),
            user: "app".to_string(),
            password: "pass".to_string(),
            ssl_mode: "disable".to_string(),
            max_open_conns: 25,
            max_idle_conns: 5,
            conn_max_lifetime: "5m".to_string(),
        };
        assert!(cfg
            .connection_url()
            .starts_with("postgres://app:pass@localhost"));
    }
}
