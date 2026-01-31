//! PostgreSQL バックエンドによる分散ロック。

use async_trait::async_trait;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::LockConfig;
use crate::error::{ConsensusError, ConsensusResult};
use crate::lock::{DistributedLock, LockGuard};

/// PostgreSQL を使用した分散ロック実装。
pub struct DbDistributedLock {
    pool: PgPool,
    holder_id: String,
    config: LockConfig,
}

impl DbDistributedLock {
    /// 新しい `DbDistributedLock` を作成する。
    #[must_use]
    pub fn new(pool: PgPool, holder_id: String, config: LockConfig) -> Self {
        Self {
            pool,
            holder_id,
            config,
        }
    }

    /// ランダムなホルダー ID で作成する。
    #[must_use]
    pub fn with_random_holder(pool: PgPool, config: LockConfig) -> Self {
        Self::new(pool, Uuid::new_v4().to_string(), config)
    }
}

#[async_trait]
impl DistributedLock for DbDistributedLock {
    async fn try_lock(&self, resource: &str, ttl_secs: u64) -> ConsensusResult<Option<LockGuard>> {
        let expires_at =
            Utc::now() + chrono::Duration::seconds(i64::from(u32::try_from(ttl_secs).unwrap_or(u32::MAX)));

        let result = sqlx::query_as::<_, (String, u64, chrono::DateTime<Utc>)>(
            r"
            INSERT INTO fw_m_distributed_lock (resource_name, holder_id, fence_token, expires_at)
            VALUES ($1, $2, 1, $3)
            ON CONFLICT (resource_name) DO UPDATE
            SET holder_id = $2,
                fence_token = fw_m_distributed_lock.fence_token + 1,
                expires_at = $3
            WHERE fw_m_distributed_lock.expires_at < NOW()
            RETURNING holder_id, fence_token, expires_at
            ",
        )
        .bind(resource)
        .bind(&self.holder_id)
        .bind(expires_at)
        .fetch_optional(&self.pool)
        .await?;

        match result {
            Some((holder_id, fence_token, expires_at)) if holder_id == self.holder_id => {
                tracing::debug!(resource, holder_id = %self.holder_id, fence_token, "lock acquired");
                super::metrics::lock_acquisitions().inc();
                Ok(Some(LockGuard::new(
                    resource.to_owned(),
                    holder_id,
                    fence_token,
                    expires_at,
                )))
            }
            _ => Ok(None),
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
        let new_expires_at =
            Utc::now() + chrono::Duration::seconds(i64::from(u32::try_from(ttl_secs).unwrap_or(u32::MAX)));

        let result = sqlx::query(
            r"
            UPDATE fw_m_distributed_lock
            SET expires_at = $1
            WHERE resource_name = $2
              AND holder_id = $3
              AND expires_at > NOW()
            ",
        )
        .bind(new_expires_at)
        .bind(&guard.resource)
        .bind(&guard.holder_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn unlock(&self, guard: &LockGuard) -> ConsensusResult<()> {
        sqlx::query(
            r"
            DELETE FROM fw_m_distributed_lock
            WHERE resource_name = $1
              AND holder_id = $2
            ",
        )
        .bind(&guard.resource)
        .bind(&guard.holder_id)
        .execute(&self.pool)
        .await?;

        tracing::debug!(
            resource = %guard.resource,
            holder_id = %guard.holder_id,
            "lock released"
        );
        Ok(())
    }
}
