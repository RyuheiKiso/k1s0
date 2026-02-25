use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::notification_template::NotificationTemplate;
use crate::domain::repository::NotificationTemplateRepository;

pub struct TemplatePostgresRepository {
    pool: Arc<PgPool>,
}

impl TemplatePostgresRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct TemplateRow {
    id: Uuid,
    name: String,
    channel_type: String,
    subject_template: String,
    body_template: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<TemplateRow> for NotificationTemplate {
    fn from(r: TemplateRow) -> Self {
        NotificationTemplate {
            id: r.id,
            name: r.name,
            channel_type: r.channel_type,
            subject_template: if r.subject_template.is_empty() {
                None
            } else {
                Some(r.subject_template)
            },
            body_template: r.body_template,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[async_trait]
impl NotificationTemplateRepository for TemplatePostgresRepository {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<NotificationTemplate>> {
        let row: Option<TemplateRow> = sqlx::query_as(
            "SELECT id, name, channel_type, subject_template, body_template, created_at, updated_at \
             FROM notification.templates WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(self.pool.as_ref())
        .await?;
        Ok(row.map(Into::into))
    }

    async fn find_all(&self) -> anyhow::Result<Vec<NotificationTemplate>> {
        let rows: Vec<TemplateRow> = sqlx::query_as(
            "SELECT id, name, channel_type, subject_template, body_template, created_at, updated_at \
             FROM notification.templates ORDER BY created_at DESC",
        )
        .fetch_all(self.pool.as_ref())
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn create(&self, template: &NotificationTemplate) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO notification.templates \
             (id, name, channel_type, subject_template, body_template, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(template.id)
        .bind(&template.name)
        .bind(&template.channel_type)
        .bind(template.subject_template.as_deref().unwrap_or(""))
        .bind(&template.body_template)
        .bind(template.created_at)
        .bind(template.updated_at)
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }

    async fn update(&self, template: &NotificationTemplate) -> anyhow::Result<()> {
        sqlx::query(
            "UPDATE notification.templates \
             SET name = $2, subject_template = $3, body_template = $4, updated_at = $5 \
             WHERE id = $1",
        )
        .bind(template.id)
        .bind(&template.name)
        .bind(template.subject_template.as_deref().unwrap_or(""))
        .bind(&template.body_template)
        .bind(template.updated_at)
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }

    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool> {
        let result = sqlx::query("DELETE FROM notification.templates WHERE id = $1")
            .bind(id)
            .execute(self.pool.as_ref())
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
