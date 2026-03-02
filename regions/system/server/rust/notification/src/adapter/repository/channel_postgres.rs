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

    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        channel_type: Option<String>,
        enabled_only: bool,
    ) -> anyhow::Result<(Vec<NotificationChannel>, u64)> {
        let offset = (page.saturating_sub(1) * page_size) as i64;
        let limit = page_size as i64;

        let mut conditions = Vec::new();
        let mut bind_index = 1u32;

        if channel_type.is_some() {
            conditions.push(format!("channel_type = ${}", bind_index));
            bind_index += 1;
        }
        if enabled_only {
            conditions.push("enabled = true".to_string());
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let count_query = format!(
            "SELECT COUNT(*) FROM notification.channels {}",
            where_clause
        );
        let data_query = format!(
            "SELECT id, name, channel_type, config, enabled, created_at, updated_at \
             FROM notification.channels {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            where_clause, bind_index, bind_index + 1
        );

        let mut count_q = sqlx::query_scalar::<_, i64>(&count_query);
        if let Some(ref v) = channel_type {
            count_q = count_q.bind(v);
        }
        let total_count = count_q.fetch_one(self.pool.as_ref()).await?;

        let mut data_q = sqlx::query_as::<_, ChannelRow>(&data_query);
        if let Some(ref v) = channel_type {
            data_q = data_q.bind(v);
        }
        data_q = data_q.bind(limit);
        data_q = data_q.bind(offset);

        let rows: Vec<ChannelRow> = data_q.fetch_all(self.pool.as_ref()).await?;

        Ok((rows.into_iter().map(Into::into).collect(), total_count as u64))
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
