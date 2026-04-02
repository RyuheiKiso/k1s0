use sqlx::PgPool;

use super::config::{Config, DatabaseConfig};

#[allow(dead_code)]
pub async fn connect(cfg: &DatabaseConfig) -> anyhow::Result<PgPool> {
    let url = cfg.connection_url();
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(cfg.max_open_conns)
        .connect(&url)
        .await?;
    Ok(pool)
}

/// MED-007 監査対応: Config または DATABASE_URL 環境変数から PostgreSQL に接続を試みる。
/// 接続に失敗した場合は None を返し、in-memory フォールバックを許容する。
pub async fn connect_optional(cfg: &Config) -> Option<PgPool> {
    // cfg.database が存在する場合はそちらの接続 URL を優先する
    let url = if let Some(ref db_cfg) = cfg.database {
        db_cfg.connection_url()
    } else if let Ok(url) = std::env::var("DATABASE_URL") {
        url
    } else {
        return None;
    };

    let max_connections = cfg
        .database
        .as_ref()
        .map(|db| db.max_open_conns)
        .unwrap_or(5);

    match sqlx::postgres::PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(&url)
        .await
    {
        Ok(pool) => Some(pool),
        Err(e) => {
            tracing::warn!(error = %e, "PostgreSQL 接続に失敗しました。in-memory バックエンドを使用します");
            None
        }
    }
}
