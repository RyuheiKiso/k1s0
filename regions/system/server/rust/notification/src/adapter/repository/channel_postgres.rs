use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::notification_channel::NotificationChannel;
use crate::domain::repository::NotificationChannelRepository;

pub struct ChannelPostgresRepository {
    pool: Arc<PgPool>,
}

impl ChannelPostgresRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct ChannelRow {
    id: Uuid,
    name: String,
    channel_type: String,
    config: serde_json::Value,
    enabled: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<ChannelRow> for NotificationChannel {
    fn from(r: ChannelRow) -> Self {
        NotificationChannel {
            id: r.id,
            name: r.name,
            channel_type: r.channel_type,
            config: r.config,
            enabled: r.enabled,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[async_trait]
impl NotificationChannelRepository for ChannelPostgresRepository {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<NotificationChannel>> {
        let row: Option<ChannelRow> = sqlx::query_as(
            "SELECT id, name, channel_type, config, enabled, created_at, updated_at \
             FROM notification.channels WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(self.pool.as_ref())
        .await?;
        Ok(row.map(Into::into))
    }

    async fn find_all(&self) -> anyhow::Result<Vec<NotificationChannel>> {
        let rows: Vec<ChannelRow> = sqlx::query_as(
            "SELECT id, name, channel_type, config, enabled, created_at, updated_at \
             FROM notification.channels ORDER BY created_at DESC",
        )
        .fetch_all(self.pool.as_ref())
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn create(&self, channel: &NotificationChannel) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO notification.channels \
             (id, name, channel_type, config, enabled, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(channel.id)
        .bind(&channel.name)
        .bind(&channel.channel_type)
        .bind(&channel.config)
        .bind(channel.enabled)
        .bind(channel.created_at)
        .bind(channel.updated_at)
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }

    async fn update(&self, channel: &NotificationChannel) -> anyhow::Result<()> {
        sqlx::query(
            "UPDATE notification.channels \
             SET name = $2, config = $3, enabled = $4, updated_at = $5 \
             WHERE id = $1",
        )
        .bind(channel.id)
        .bind(&channel.name)
        .bind(&channel.config)
        .bind(channel.enabled)
        .bind(channel.updated_at)
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }

    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool> {
        let result = sqlx::query("DELETE FROM notification.channels WHERE id = $1")
            .bind(id)
            .execute(self.pool.as_ref())
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
