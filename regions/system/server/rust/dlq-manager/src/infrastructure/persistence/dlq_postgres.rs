use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::{DlqMessage, DlqStatus};
use crate::domain::repository::dlq_message_repository::DlqMessageRepository;

/// DlqPostgresRepository は PostgreSQL 実装の DLQ メッセージリポジトリ。
pub struct DlqPostgresRepository {
    pool: PgPool,
}

impl DlqPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DlqMessageRepository for DlqPostgresRepository {
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<DlqMessage>> {
        let row = sqlx::query_as::<_, DlqMessageRow>(
            r#"
            SELECT id, original_topic, error_message, retry_count, max_retries, payload, status, created_at, updated_at, last_retry_at
            FROM dlq.messages
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }

    async fn find_by_topic(
        &self,
        topic: &str,
        page: i32,
        page_size: i32,
    ) -> anyhow::Result<(Vec<DlqMessage>, i64)> {
        let total: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM dlq.messages WHERE original_topic = $1")
                .bind(topic)
                .fetch_one(&self.pool)
                .await?;

        let page = page.max(1);
        let page_size = page_size.max(1);
        let offset = ((page - 1) * page_size) as i64;

        let rows = sqlx::query_as::<_, DlqMessageRow>(
            r#"
            SELECT id, original_topic, error_message, retry_count, max_retries, payload, status, created_at, updated_at, last_retry_at
            FROM dlq.messages
            WHERE original_topic = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(topic)
        .bind(page_size as i64)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let messages: anyhow::Result<Vec<DlqMessage>> =
            rows.into_iter().map(|r| r.try_into()).collect();

        Ok((messages?, total))
    }

    async fn create(&self, message: &DlqMessage) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO dlq.messages
                (id, original_topic, error_message, retry_count, max_retries, payload, status, created_at, updated_at, last_retry_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(message.id)
        .bind(&message.original_topic)
        .bind(&message.error_message)
        .bind(message.retry_count)
        .bind(message.max_retries)
        .bind(&message.payload)
        .bind(message.status.to_string())
        .bind(message.created_at)
        .bind(message.updated_at)
        .bind(message.last_retry_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update(&self, message: &DlqMessage) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE dlq.messages
            SET original_topic = $2, error_message = $3, retry_count = $4, max_retries = $5,
                payload = $6, status = $7, updated_at = $8, last_retry_at = $9
            WHERE id = $1
            "#,
        )
        .bind(message.id)
        .bind(&message.original_topic)
        .bind(&message.error_message)
        .bind(message.retry_count)
        .bind(message.max_retries)
        .bind(&message.payload)
        .bind(message.status.to_string())
        .bind(message.updated_at)
        .bind(message.last_retry_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM dlq.messages WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn count_by_topic(&self, topic: &str) -> anyhow::Result<i64> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM dlq.messages WHERE original_topic = $1")
                .bind(topic)
                .fetch_one(&self.pool)
                .await?;

        Ok(count)
    }
}

/// DlqMessageRow はDB行からのマッピング用。
#[derive(sqlx::FromRow)]
struct DlqMessageRow {
    id: Uuid,
    original_topic: String,
    error_message: String,
    retry_count: i32,
    max_retries: i32,
    payload: Option<serde_json::Value>,
    status: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    last_retry_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl TryFrom<DlqMessageRow> for DlqMessage {
    type Error = anyhow::Error;

    fn try_from(row: DlqMessageRow) -> anyhow::Result<Self> {
        Ok(DlqMessage {
            id: row.id,
            original_topic: row.original_topic,
            error_message: row.error_message,
            retry_count: row.retry_count,
            max_retries: row.max_retries,
            payload: row.payload.unwrap_or(serde_json::Value::Null),
            status: DlqStatus::from_str_value(&row.status)?,
            created_at: row.created_at,
            updated_at: row.updated_at,
            last_retry_at: row.last_retry_at,
        })
    }
}
