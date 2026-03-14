// Redisキャッシュの実装。
// redis crateを使用してキャッシュの読み書きを行う（オプション機能）。

use redis::AsyncCommands;

/// Redisキャッシュクライアント。
/// キーバリュー形式でデータをキャッシュする。
pub struct RedisCache {
    /// Redis接続クライアント
    client: redis::Client,
}

impl RedisCache {
    /// 新しいRedisCacheを生成する。
    /// url: Redis接続URL（例: redis://localhost:6379）
    pub fn new(url: &str) -> anyhow::Result<Self> {
        let client = redis::Client::open(url)?;
        Ok(Self { client })
    }

    /// キーに対応する値を取得する。
    /// 存在しない場合やエラー時はNoneを返す。
    pub async fn get(&self, key: &str) -> Option<String> {
        let mut conn = self.client.get_multiplexed_async_connection().await.ok()?;
        let value: Option<String> = conn.get(key).await.ok()?;
        value
    }

    /// キーに値をセットする。
    /// ttl: 有効期限（秒）
    pub async fn set(&self, key: &str, value: &str, ttl: u64) {
        if let Ok(mut conn) = self.client.get_multiplexed_async_connection().await {
            let _: Result<(), _> = conn.set_ex(key, value, ttl).await;
        }
    }
}
