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

/// MED-007 監査対応: Config または `DATABASE_URL` 環境変数から `PostgreSQL` に接続を試みる。
/// 接続に失敗した場合は None を返し、in-memory フォールバックを許容する。
/// `serde_yaml` `はシェル変数を展開しないため、DATABASE_URL` 環境変数が設定されている場合は常に優先する。
pub async fn connect_optional(cfg: &Config) -> Option<PgPool> {
    // DATABASE_URL 環境変数が設定されている場合は最優先で使用する。
    // config.docker.yaml の password フィールドは serde_yaml がシェル変数を展開しないため
    // "${DB_PASSWORD:-dev}" のような文字列リテラルがそのまま格納される場合がある。
    let url = if let Ok(url) = std::env::var("DATABASE_URL") {
        url
    } else if let Some(ref db_cfg) = cfg.database {
        db_cfg.connection_url()
    } else {
        return None;
    };

    let max_connections = cfg
        .database
        .as_ref()
        .map_or(5, |db| db.max_open_conns);

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
