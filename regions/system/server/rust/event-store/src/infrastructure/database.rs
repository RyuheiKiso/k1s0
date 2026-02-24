use sqlx::PgPool;

use super::config::DatabaseConfig;

/// PostgreSQL 接続プールを作成する。
pub async fn connect(cfg: &DatabaseConfig) -> anyhow::Result<PgPool> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(cfg.max_connections)
        .min_connections(cfg.min_connections)
        .acquire_timeout(std::time::Duration::from_secs(cfg.connect_timeout_seconds))
        .connect(&cfg.url)
        .await?;
    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::super::config::DatabaseConfig;

    #[test]
    fn test_database_config_defaults() {
        let cfg = DatabaseConfig {
            url: "postgresql://app:@localhost:5432/k1s0_system".to_string(),
            schema: "event_store".to_string(),
            max_connections: 20,
            min_connections: 5,
            connect_timeout_seconds: 5,
        };
        assert_eq!(cfg.max_connections, 20);
        assert_eq!(cfg.schema, "event_store");
    }
}
