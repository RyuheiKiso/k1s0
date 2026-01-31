//! PostgreSQL バックエンドによるリーダー選出。

use async_trait::async_trait;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::LeaderConfig;
use crate::error::{ConsensusError, ConsensusResult};
use crate::leader::metrics::LeaderMetrics;
use crate::leader::{LeaderElector, LeaderEvent, LeaderLease, LeaderWatcher};

/// PostgreSQL を使用したリーダー選出実装。
pub struct DbLeaderElector {
    pool: PgPool,
    node_id: String,
    config: LeaderConfig,
    metrics: LeaderMetrics,
}

impl DbLeaderElector {
    /// 新しい `DbLeaderElector` を作成する。
    #[must_use]
    pub fn new(pool: PgPool, node_id: String, config: LeaderConfig) -> Self {
        Self {
            pool,
            node_id,
            config,
            metrics: LeaderMetrics::new(),
        }
    }

    /// ランダムなノード ID で作成する。
    #[must_use]
    pub fn with_random_node_id(pool: PgPool, config: LeaderConfig) -> Self {
        Self::new(pool, Uuid::new_v4().to_string(), config)
    }

    /// ノード ID を返す。
    #[must_use]
    pub fn node_id(&self) -> &str {
        &self.node_id
    }
}

#[async_trait]
impl LeaderElector for DbLeaderElector {
    async fn try_acquire(&self) -> ConsensusResult<Option<LeaderLease>> {
        let lease_duration =
            chrono::Duration::seconds(i64::from(u32::try_from(self.config.lease_duration_secs).unwrap_or(u32::MAX)));
        let expires_at = Utc::now() + lease_duration;

        // INSERT ON CONFLICT: 期限切れの場合のみ上書き
        let result = sqlx::query_as::<_, (String, u64, chrono::DateTime<Utc>)>(
            r"
            INSERT INTO fw_m_leader_lease (lease_key, holder_id, fence_token, expires_at)
            VALUES ($1, $2, 1, $3)
            ON CONFLICT (lease_key) DO UPDATE
            SET holder_id = $2,
                fence_token = fw_m_leader_lease.fence_token + 1,
                expires_at = $3
            WHERE fw_m_leader_lease.expires_at < NOW()
            RETURNING holder_id, fence_token, expires_at
            ",
        )
        .bind(&self.config.lease_key)
        .bind(&self.node_id)
        .bind(expires_at)
        .fetch_optional(&self.pool)
        .await?;

        match result {
            Some((holder_id, fence_token, expires_at)) if holder_id == self.node_id => {
                self.metrics.record_election(true);
                tracing::info!(
                    lease_key = %self.config.lease_key,
                    holder_id = %self.node_id,
                    fence_token,
                    "leader lease acquired"
                );
                Ok(Some(LeaderLease {
                    lease_key: self.config.lease_key.clone(),
                    holder_id,
                    fence_token,
                    expires_at,
                }))
            }
            _ => {
                self.metrics.record_election(false);
                Ok(None)
            }
        }
    }

    async fn renew(&self, lease: &LeaderLease) -> ConsensusResult<bool> {
        let lease_duration =
            chrono::Duration::seconds(i64::from(u32::try_from(self.config.lease_duration_secs).unwrap_or(u32::MAX)));
        let new_expires_at = Utc::now() + lease_duration;

        let result = sqlx::query(
            r"
            UPDATE fw_m_leader_lease
            SET expires_at = $1
            WHERE lease_key = $2
              AND holder_id = $3
              AND expires_at > NOW()
            ",
        )
        .bind(new_expires_at)
        .bind(&lease.lease_key)
        .bind(&self.node_id)
        .execute(&self.pool)
        .await?;

        let renewed = result.rows_affected() > 0;
        if !renewed {
            tracing::warn!(
                lease_key = %lease.lease_key,
                holder_id = %self.node_id,
                "failed to renew leader lease"
            );
        }
        Ok(renewed)
    }

    async fn release(&self, lease: &LeaderLease) -> ConsensusResult<()> {
        sqlx::query(
            r"
            DELETE FROM fw_m_leader_lease
            WHERE lease_key = $1
              AND holder_id = $2
            ",
        )
        .bind(&lease.lease_key)
        .bind(&self.node_id)
        .execute(&self.pool)
        .await?;

        tracing::info!(
            lease_key = %lease.lease_key,
            holder_id = %self.node_id,
            "leader lease released"
        );
        Ok(())
    }

    async fn current_leader(&self) -> ConsensusResult<Option<LeaderLease>> {
        let result = sqlx::query_as::<_, (String, String, u64, chrono::DateTime<Utc>)>(
            r"
            SELECT lease_key, holder_id, fence_token, expires_at
            FROM fw_m_leader_lease
            WHERE lease_key = $1
              AND expires_at > NOW()
            ",
        )
        .bind(&self.config.lease_key)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|(lease_key, holder_id, fence_token, expires_at)| LeaderLease {
            lease_key,
            holder_id,
            fence_token,
            expires_at,
        }))
    }

    async fn watch(&self) -> ConsensusResult<LeaderWatcher> {
        let (tx, rx) = tokio::sync::watch::channel(LeaderEvent::Lost);
        let pool = self.pool.clone();
        let lease_key = self.config.lease_key.clone();
        let node_id = self.node_id.clone();
        let poll_interval = self.config.watch_poll_interval_secs;

        tokio::spawn(async move {
            let mut last_leader: Option<String> = None;
            let mut interval =
                tokio::time::interval(std::time::Duration::from_secs(poll_interval));

            loop {
                interval.tick().await;

                let result = sqlx::query_as::<_, (String,)>(
                    "SELECT holder_id FROM fw_m_leader_lease WHERE lease_key = $1 AND expires_at > NOW()",
                )
                .bind(&lease_key)
                .fetch_optional(&pool)
                .await;

                match result {
                    Ok(Some((holder_id,))) => {
                        let event = if holder_id == node_id {
                            LeaderEvent::Elected { fence_token: 0 }
                        } else {
                            LeaderEvent::Changed {
                                new_leader: holder_id.clone(),
                            }
                        };

                        if last_leader.as_deref() != Some(&holder_id) {
                            last_leader = Some(holder_id);
                            if tx.send(event).is_err() {
                                break;
                            }
                        }
                    }
                    Ok(None) => {
                        if last_leader.is_some() {
                            last_leader = None;
                            if tx.send(LeaderEvent::Lost).is_err() {
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "leader watch poll failed");
                    }
                }
            }
        });

        Ok(LeaderWatcher::new(rx))
    }
}
