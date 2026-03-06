use sqlx::PgPool;

use super::config::DatabaseConfig;

pub async fn connect(cfg: &DatabaseConfig) -> anyhow::Result<PgPool> {
    let url = cfg.connection_url();
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(cfg.max_open_conns)
        .connect(&url)
        .await?;
    Ok(pool)
}
