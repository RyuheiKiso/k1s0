use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::domain::entity::notification_log::NotificationLog;
use crate::domain::repository::NotificationLogRepository;

pub struct NotificationLogPostgresRepository {
    pool: Arc<PgPool>,
}

impl NotificationLogPostgresRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct NotificationLogRow {
    id: String,
    channel_id: String,
    template_id: Option<String>,
    recipient: String,
    subject: String,
    body: String,
    status: String,
    retry_count: i32,
    error_message: Option<String>,
    sent_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<NotificationLogRow> for NotificationLog {
    fn from(r: NotificationLogRow) -> Self {
        NotificationLog {
            id: r.id,
            channel_id: r.channel_id,
            template_id: r.template_id,
            recipient: r.recipient,
            subject: if r.subject.is_empty() {
                None
            } else {
                Some(r.subject)
            },
            body: r.body,
            status: r.status,
            retry_count: r.retry_count.max(0) as u32,
            error_message: r.error_message,
            sent_at: r.sent_at,
            created_at: r.created_at,
        }
    }
}

#[async_trait]
impl NotificationLogRepository for NotificationLogPostgresRepository {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<NotificationLog>> {
        let row: Option<NotificationLogRow> = sqlx::query_as(
            "SELECT id, channel_id, template_id, recipient, subject, body, status, retry_count, error_message, sent_at, created_at, updated_at \
             FROM notification.notification_logs WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(self.pool.as_ref())
        .await?;
        Ok(row.map(Into::into))
    }

    async fn find_by_channel_id(
        &self,
        channel_id: &str,
    ) -> anyhow::Result<Vec<NotificationLog>> {
        let rows: Vec<NotificationLogRow> = sqlx::query_as(
            "SELECT id, channel_id, template_id, recipient, subject, body, status, retry_count, error_message, sent_at, created_at, updated_at \
             FROM notification.notification_logs WHERE channel_id = $1 ORDER BY created_at DESC",
        )
        .bind(channel_id)
        .fetch_all(self.pool.as_ref())
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        channel_id: Option<String>,
        status: Option<String>,
    ) -> anyhow::Result<(Vec<NotificationLog>, u64)> {
        let offset = (page.saturating_sub(1) * page_size) as i64;
        let limit = page_size as i64;

        let mut conditions = Vec::new();
        let mut bind_index = 1u32;

        if channel_id.is_some() {
            conditions.push(format!("channel_id = ${}", bind_index));
            bind_index += 1;
        }
        if status.is_some() {
            conditions.push(format!("status = ${}", bind_index));
            bind_index += 1;
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let count_query = format!(
            "SELECT COUNT(*) FROM notification.notification_logs {}",
            where_clause
        );
        let data_query = format!(
            "SELECT id, channel_id, template_id, recipient, subject, body, status, retry_count, error_message, sent_at, created_at, updated_at \
             FROM notification.notification_logs {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            where_clause, bind_index, bind_index + 1
        );

        let mut count_q = sqlx::query_scalar::<_, i64>(&count_query);
        if let Some(ref v) = channel_id {
            count_q = count_q.bind(v);
        }
        if let Some(ref v) = status {
            count_q = count_q.bind(v);
        }
        let total_count = count_q.fetch_one(self.pool.as_ref()).await?;

        let mut data_q = sqlx::query_as::<_, NotificationLogRow>(&data_query);
        if let Some(ref v) = channel_id {
            data_q = data_q.bind(v);
        }
        if let Some(ref v) = status {
            data_q = data_q.bind(v);
        }
        data_q = data_q.bind(limit);
        data_q = data_q.bind(offset);

        let rows: Vec<NotificationLogRow> = data_q.fetch_all(self.pool.as_ref()).await?;

        Ok((rows.into_iter().map(Into::into).collect(), total_count as u64))
    }

    async fn create(&self, log: &NotificationLog) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO notification.notification_logs \
             (id, channel_id, template_id, recipient, subject, body, status, retry_count, error_message, sent_at, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
        )
        .bind(&log.id)
        .bind(&log.channel_id)
        .bind(&log.template_id)
        .bind(&log.recipient)
        .bind(log.subject.as_deref().unwrap_or(""))
        .bind(&log.body)
        .bind(&log.status)
        .bind(log.retry_count as i32)
        .bind(&log.error_message)
        .bind(log.sent_at)
        .bind(log.created_at)
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }

    async fn update(&self, log: &NotificationLog) -> anyhow::Result<()> {
        sqlx::query(
            "UPDATE notification.notification_logs \
             SET status = $2, retry_count = $3, error_message = $4, sent_at = $5, updated_at = NOW() \
             WHERE id = $1",
        )
        .bind(&log.id)
        .bind(&log.status)
        .bind(log.retry_count as i32)
        .bind(&log.error_message)
        .bind(log.sent_at)
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }
}
