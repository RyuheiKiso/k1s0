// データベース接続プールの生成。
// sqlxを使用してPostgreSQLへの接続プールを作成する。

use sqlx::postgres::{PgPool, PgPoolOptions};

/// プール接続期間を文字列からDurationにパースする。
/// "5m", "30s", "100ms", "1h" 形式をサポート。
fn parse_pool_duration(value: &str) -> Option<std::time::Duration> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(v) = trimmed.strip_suffix("ms") {
        return v.parse::<u64>().ok().map(std::time::Duration::from_millis);
    }
    if let Some(v) = trimmed.strip_suffix('s') {
        return v.parse::<u64>().ok().map(std::time::Duration::from_secs);
    }
    if let Some(v) = trimmed.strip_suffix('m') {
        return v
            .parse::<u64>()
            .ok()
            .map(|mins| std::time::Duration::from_secs(mins * 60));
    }
    if let Some(v) = trimmed.strip_suffix('h') {
        return v
            .parse::<u64>()
            .ok()
            .map(|hours| std::time::Duration::from_secs(hours * 3600));
    }
    trimmed
        .parse::<u64>()
        .ok()
        .map(std::time::Duration::from_secs)
}

/// PostgreSQL接続プールを作成する。
pub async fn create_pool(
    url: &str,
    max_connections: u32,
    max_idle_conns: u32,
    conn_max_lifetime: &str,
) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .min_connections(max_idle_conns.min(max_connections))
        .max_lifetime(parse_pool_duration(conn_max_lifetime))
        .connect(url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?;
    Ok(pool)
}
