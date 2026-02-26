use async_trait::async_trait;
use redis::aio::ConnectionManager;

use crate::domain::repository::QuotaUsageRepository;

/// RedisQuotaUsageRepository は Redis ベースのクォータ使用量リポジトリ。
///
/// カウンターは Redis の INCRBY でアトミックに更新される。
/// TTL は設定せず、リセットはスケジューラが `reset()` を呼ぶ設計。
pub struct RedisQuotaUsageRepository {
    conn: ConnectionManager,
    key_prefix: String,
}

impl RedisQuotaUsageRepository {
    pub fn new(conn: ConnectionManager, key_prefix: String) -> Self {
        Self { conn, key_prefix }
    }

    fn make_key(&self, quota_id: &str) -> String {
        build_key(&self.key_prefix, quota_id)
    }
}

#[async_trait]
impl QuotaUsageRepository for RedisQuotaUsageRepository {
    async fn get_usage(&self, quota_id: &str) -> anyhow::Result<Option<u64>> {
        let key = self.make_key(quota_id);
        let mut conn = self.conn.clone();
        let result: Option<u64> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut conn)
            .await?;
        Ok(result)
    }

    async fn increment(&self, quota_id: &str, amount: u64) -> anyhow::Result<u64> {
        let key = self.make_key(quota_id);
        let mut conn = self.conn.clone();
        let new_total: u64 = redis::cmd("INCRBY")
            .arg(&key)
            .arg(amount)
            .query_async(&mut conn)
            .await?;
        Ok(new_total)
    }

    async fn reset(&self, quota_id: &str) -> anyhow::Result<()> {
        let key = self.make_key(quota_id);
        let mut conn = self.conn.clone();
        redis::cmd("DEL")
            .arg(&key)
            .query_async::<()>(&mut conn)
            .await?;
        Ok(())
    }
}

/// キープレフィックスとクォータIDからRedisキーを生成するヘルパー。
/// テストで安全に呼び出せるようスタンドアロン関数として公開。
fn build_key(prefix: &str, quota_id: &str) -> String {
    format!("{}{}", prefix, quota_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_key_default_prefix() {
        assert_eq!(build_key("quota:", "plan-123"), "quota:plan-123");
    }

    #[test]
    fn test_make_key_custom_prefix() {
        assert_eq!(
            build_key("myapp:quota:usage:", "org-abc"),
            "myapp:quota:usage:org-abc"
        );
    }

    #[test]
    fn test_make_key_empty_prefix() {
        assert_eq!(build_key("", "id-1"), "id-1");
    }
}
