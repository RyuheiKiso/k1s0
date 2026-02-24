use sqlx::PgPool;

/// PostgreSQL 接続プールを作成する（URL 直接指定）。
pub async fn connect(url: &str, max_connections: u32) -> anyhow::Result<PgPool> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(url)
        .await?;
    Ok(pool)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        // database.rs の存在を確認するプレースホルダーテスト
        assert!(true);
    }
}
