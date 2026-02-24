use sqlx::postgres::{PgPool, PgPoolOptions};

pub async fn create_pool(url: &str, max_connections: u32) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?;
    Ok(pool)
}
