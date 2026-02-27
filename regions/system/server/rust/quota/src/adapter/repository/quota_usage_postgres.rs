use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::repository::quota_repository::CheckAndIncrementResult;
use crate::domain::repository::QuotaUsageRepository;

pub struct QuotaUsagePostgresRepository {
    pool: Arc<PgPool>,
}

impl QuotaUsagePostgresRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct UsageRow {
    current_usage: i64,
}

#[async_trait]
impl QuotaUsageRepository for QuotaUsagePostgresRepository {
    async fn get_usage(&self, quota_id: &str) -> anyhow::Result<Option<u64>> {
        let uuid = uuid::Uuid::parse_str(quota_id)
            .map_err(|e| anyhow::anyhow!("invalid UUID: {}", e))?;

        let row: Option<UsageRow> = sqlx::query_as(
            "SELECT current_usage FROM quota.quota_usage WHERE policy_id = $1",
        )
        .bind(uuid)
        .fetch_optional(self.pool.as_ref())
        .await?;

        Ok(row.map(|r| r.current_usage as u64))
    }

    async fn increment(&self, quota_id: &str, amount: u64) -> anyhow::Result<u64> {
        let uuid = uuid::Uuid::parse_str(quota_id)
            .map_err(|e| anyhow::anyhow!("invalid UUID: {}", e))?;

        // UPSERT: INSERT する場合は初期値を amount にし、既存の場合は加算する
        let row: (i64,) = sqlx::query_as(
            "INSERT INTO quota.quota_usage (policy_id, current_usage, last_incremented_at) \
             VALUES ($1, $2, NOW()) \
             ON CONFLICT (policy_id, tenant_id) \
             DO UPDATE SET current_usage = quota.quota_usage.current_usage + $2, \
                           last_incremented_at = NOW() \
             RETURNING current_usage",
        )
        .bind(uuid)
        .bind(amount as i64)
        .fetch_one(self.pool.as_ref())
        .await?;

        Ok(row.0 as u64)
    }

    async fn reset(&self, quota_id: &str) -> anyhow::Result<()> {
        let uuid = uuid::Uuid::parse_str(quota_id)
            .map_err(|e| anyhow::anyhow!("invalid UUID: {}", e))?;

        sqlx::query(
            "UPDATE quota.quota_usage \
             SET current_usage = 0, window_start = NOW(), last_incremented_at = NULL \
             WHERE policy_id = $1",
        )
        .bind(uuid)
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    async fn check_and_increment(
        &self,
        quota_id: &str,
        amount: u64,
        limit: u64,
    ) -> anyhow::Result<CheckAndIncrementResult> {
        let uuid = uuid::Uuid::parse_str(quota_id)
            .map_err(|e| anyhow::anyhow!("invalid UUID: {}", e))?;

        // アトミックに current_usage + amount <= limit の場合のみ UPDATE する
        let row: Option<(i64,)> = sqlx::query_as(
            "UPDATE quota.quota_usage \
             SET current_usage = current_usage + $2, last_incremented_at = NOW() \
             WHERE policy_id = $1 AND current_usage + $2 <= $3 \
             RETURNING current_usage",
        )
        .bind(uuid)
        .bind(amount as i64)
        .bind(limit as i64)
        .fetch_optional(self.pool.as_ref())
        .await?;

        match row {
            Some((new_usage,)) => Ok(CheckAndIncrementResult {
                used: new_usage as u64,
                allowed: true,
            }),
            None => {
                // UPDATE が 0 行 → リミット超過。現在の使用量を取得して返す
                let current = self.get_usage(quota_id).await?.unwrap_or(0);
                Ok(CheckAndIncrementResult {
                    used: current,
                    allowed: false,
                })
            }
        }
    }
}
