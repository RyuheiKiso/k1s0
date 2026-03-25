use sqlx::PgPool;

use super::config::DatabaseConfig;

pub async fn connect(cfg: &DatabaseConfig) -> anyhow::Result<PgPool> {
    // DATABASE_URL 環境変数が設定されている場合は優先する（serde_yaml はシェル変数を展開しないため）
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| cfg.connection_url());
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(cfg.max_open_conns)
        .connect(&url)
        .await?;
    Ok(pool)
}
