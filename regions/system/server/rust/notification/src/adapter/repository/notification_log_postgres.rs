use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

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
    id: Uuid,
    channel_id: Uuid,
    template_id: Option<Uuid>,
    recipient: String,
    subject: String,
    body: String,
    status: String,
    error_message: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<NotificationLogRow> for NotificationLog {
    fn from(r: NotificationLogRow) -> Self {
        let sent_at = if r.status == "sent" {
            Some(r.updated_at)
        } else {
            None
        };
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
            error_message: r.error_message,
            sent_at,
            created_at: r.created_at,
        }
    }
}

#[async_trait]
impl NotificationLogRepository for NotificationLogPostgresRepository {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<NotificationLog>> {
        let row: Option<NotificationLogRow> = sqlx::query_as(
            "SELECT id, channel_id, template_id, recipient, subject, body, status, error_message, created_at, updated_at \
             FROM notification.notification_logs WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(self.pool.as_ref())
        .await?;
        Ok(row.map(Into::into))
    }

    async fn find_by_channel_id(
        &self,
        channel_id: &Uuid,
    ) -> anyhow::Result<Vec<NotificationLog>> {
        let rows: Vec<NotificationLogRow> = sqlx::query_as(
            "SELECT id, channel_id, template_id, recipient, subject, body, status, error_message, created_at, updated_at \
             FROM notification.notification_logs WHERE channel_id = $1 ORDER BY created_at DESC",
        )
        .bind(channel_id)
        .fetch_all(self.pool.as_ref())
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn create(&self, log: &NotificationLog) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO notification.notification_logs \
             (id, channel_id, template_id, recipient, subject, body, status, error_message, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(log.id)
        .bind(log.channel_id)
        .bind(log.template_id)
        .bind(&log.recipient)
        .bind(log.subject.as_deref().unwrap_or(""))
        .bind(&log.body)
        .bind(&log.status)
        .bind(&log.error_message)
        .bind(log.created_at)
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }

    async fn update(&self, log: &NotificationLog) -> anyhow::Result<()> {
        sqlx::query(
            "UPDATE notification.notification_logs \
             SET status = $2, error_message = $3, updated_at = NOW() \
             WHERE id = $1",
        )
        .bind(log.id)
        .bind(&log.status)
        .bind(&log.error_message)
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }
}
