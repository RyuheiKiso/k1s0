//! Redis バックエンドによる分散ロック。

use async_trait::async_trait;
use chrono::Utc;
use redis::AsyncCommands;
use uuid::Uuid;

use crate::config::{LockConfig, RedisConfig};
use crate::error::{ConsensusError, ConsensusResult};
use crate::lock::{DistributedLock, LockGuard};

/// Redis を使用した分散ロック実装。
pub struct RedisDistributedLock {
    client: redis::Client,
    holder_id: String,
    key_prefix: String,
    config: LockConfig,
}

impl RedisDistributedLock {
    /// 新しい `RedisDistributedLock` を作成する。
    ///
    /// # Errors
    ///
    /// Redis クライアントの作成に失敗した場合にエラーを返す。
    pub fn new(
        redis_config: &RedisConfig,
        holder_id: String,
        config: LockConfig,
    ) -> ConsensusResult<Self> {
        let client = redis::Client::open(redis_config.url.as_str())
            .map_err(|e| ConsensusError::Redis(e.to_string()))?;
        Ok(Self {
            client,
            holder_id,
            key_prefix: redis_config.key_prefix.clone(),
            config,
        })
    }

    /// ランダムなホルダー ID で作成する。
    ///
    /// # Errors
    ///
    /// Redis クライアントの作成に失敗した場合にエラーを返す。
    pub fn with_random_holder(
        redis_config: &RedisConfig,
        config: LockConfig,
    ) -> ConsensusResult<Self> {
        Self::new(redis_config, Uuid::new_v4().to_string(), config)
    }

    fn lock_key(&self, resource: &str) -> String {
        format!("{}lock:{resource}", self.key_prefix)
    }

    fn fence_key(&self, resource: &str) -> String {
        format!("{}fence:{resource}", self.key_prefix)
    }

    async fn get_connection(&self) -> ConsensusResult<redis::aio::MultiplexedConnection> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| ConsensusError::Redis(e.to_string()))
    }
}

/// Lua スクリプト: ホルダー検証付きアンロック
const UNLOCK_SCRIPT: &str = r"
if redis.call('get', KEYS[1]) == ARGV[1] then
    return redis.call('del', KEYS[1])
else
    return 0
end
";

/// Lua スクリプト: ホルダー検証付き TTL 延長
const EXTEND_SCRIPT: &str = r"
if redis.call('get', KEYS[1]) == ARGV[1] then
    return redis.call('pexpire', KEYS[1], ARGV[2])
else
    return 0
end
";

#[async_trait]
impl DistributedLock for RedisDistributedLock {
    async fn try_lock(&self, resource: &str, ttl_secs: u64) -> ConsensusResult<Option<LockGuard>> {
        let mut conn = self.get_connection().await?;
        let lock_key = self.lock_key(resource);
        let ttl_ms = ttl_secs * 1000;

        // SET NX PX でアトミックにロック取得
        let result: Option<String> = redis::cmd("SET")
            .arg(&lock_key)
            .arg(&self.holder_id)
            .arg("NX")
            .arg("PX")
            .arg(ttl_ms)
            .query_async(&mut conn)
            .await
            .map_err(|e| ConsensusError::Redis(e.to_string()))?;

        if result.is_some() {
            // フェンシングトークンをインクリメント
            let fence_key = self.fence_key(resource);
            let fence_token: u64 = conn
                .incr(&fence_key, 1u64)
                .await
                .map_err(|e| ConsensusError::Redis(e.to_string()))?;

            let expires_at =
                Utc::now() + chrono::Duration::seconds(i64::from(u32::try_from(ttl_secs).unwrap_or(u32::MAX)));

            tracing::debug!(resource, holder_id = %self.holder_id, fence_token, "redis lock acquired");
            super::metrics::lock_acquisitions().inc();

            Ok(Some(LockGuard::new(
                resource.to_owned(),
                self.holder_id.clone(),
                fence_token,
                expires_at,
            )))
        } else {
            Ok(None)
        }
    }

    async fn lock(
        &self,
        resource: &str,
        ttl_secs: u64,
        timeout_ms: u64,
    ) -> ConsensusResult<LockGuard> {
        let deadline =
            tokio::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
        let poll_interval = std::time::Duration::from_millis(self.config.poll_interval_ms);

        loop {
            if let Some(guard) = self.try_lock(resource, ttl_secs).await? {
                return Ok(guard);
            }

            if tokio::time::Instant::now() >= deadline {
                return Err(ConsensusError::LockTimeout {
                    resource: resource.to_owned(),
                    elapsed_ms: timeout_ms,
                });
            }

            tokio::time::sleep(poll_interval).await;
        }
    }

    async fn extend(&self, guard: &LockGuard, ttl_secs: u64) -> ConsensusResult<bool> {
        let mut conn = self.get_connection().await?;
        let lock_key = self.lock_key(&guard.resource);
        let ttl_ms = ttl_secs * 1000;

        let result: i32 = redis::Script::new(EXTEND_SCRIPT)
            .key(&lock_key)
            .arg(&guard.holder_id)
            .arg(ttl_ms)
            .invoke_async(&mut conn)
            .await
            .map_err(|e| ConsensusError::Redis(e.to_string()))?;

        Ok(result == 1)
    }

    async fn unlock(&self, guard: &LockGuard) -> ConsensusResult<()> {
        let mut conn = self.get_connection().await?;
        let lock_key = self.lock_key(&guard.resource);

        let _: i32 = redis::Script::new(UNLOCK_SCRIPT)
            .key(&lock_key)
            .arg(&guard.holder_id)
            .invoke_async(&mut conn)
            .await
            .map_err(|e| ConsensusError::Redis(e.to_string()))?;

        tracing::debug!(
            resource = %guard.resource,
            holder_id = %guard.holder_id,
            "redis lock released"
        );
        Ok(())
    }
}
