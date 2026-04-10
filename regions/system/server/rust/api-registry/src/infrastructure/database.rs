use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

pub async fn create_pool(
    url: &str,
    max_connections: u32,
    max_idle_conns: u32,
    conn_max_lifetime_secs: u64,
) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .min_connections(max_idle_conns.min(max_connections))
        .max_lifetime(Some(Duration::from_secs(conn_max_lifetime_secs)))
        .connect(url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect to database: {e}"))?;
    Ok(pool)
}
