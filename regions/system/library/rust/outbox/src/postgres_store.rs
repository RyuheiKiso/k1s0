//! PostgresOutboxStore: sqlx を使用した OutboxStore 実装。
//! feature = "postgres" で有効化される。

use async_trait::async_trait;
use sqlx::PgPool;

use crate::error::OutboxError;
use crate::message::{OutboxMessage, OutboxStatus};
use crate::store::OutboxStore;

/// PostgresOutboxStore は PostgreSQL を使ったアウトボックスストア実装。
pub struct PostgresOutboxStore {
    pool: PgPool,
}

impl PostgresOutboxStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OutboxStore for PostgresOutboxStore {
    async fn save(&self, message: &OutboxMessage) -> Result<(), OutboxError> {
        sqlx::query(
            r#"INSERT INTO outbox.outbox_messages
               (id, topic, partition_key, payload, status, retry_count, max_retries, last_error, created_at, process_after)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"#,
        )
        .bind(message.id)
        .bind(&message.topic)
        .bind(&message.partition_key)
        .bind(&message.payload)
        .bind(message.status.as_str())
        .bind(message.retry_count as i32)
        .bind(message.max_retries as i32)
        .bind(&message.last_error)
        .bind(message.created_at)
        .bind(message.process_after)
        .execute(&self.pool)
        .await
        .map_err(|e| OutboxError::StoreError(e.to_string()))?;
        Ok(())
    }

    async fn fetch_pending(&self, limit: u32) -> Result<Vec<OutboxMessage>, OutboxError> {
        let rows = sqlx::query_as::<_, OutboxRow>(
            r#"SELECT id, topic, partition_key, payload, status, retry_count, max_retries,
                      last_error, created_at, process_after
               FROM outbox.outbox_messages
               WHERE status IN ('PENDING', 'FAILED')
               AND process_after <= NOW()
               ORDER BY created_at ASC
               LIMIT $1"#,
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| OutboxError::StoreError(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn update(&self, message: &OutboxMessage) -> Result<(), OutboxError> {
        sqlx::query(
            r#"UPDATE outbox.outbox_messages
               SET status = $1, retry_count = $2, last_error = $3, process_after = $4
               WHERE id = $5"#,
        )
        .bind(message.status.as_str())
        .bind(message.retry_count as i32)
        .bind(&message.last_error)
        .bind(message.process_after)
        .bind(message.id)
        .execute(&self.pool)
        .await
        .map_err(|e| OutboxError::StoreError(e.to_string()))?;
        Ok(())
    }

    async fn delete_delivered(&self, older_than_days: u32) -> Result<u64, OutboxError> {
        let result = sqlx::query(
            r#"DELETE FROM outbox.outbox_messages
               WHERE status = 'DELIVERED'
               AND created_at < NOW() - ($1 || ' days')::interval"#,
        )
        .bind(older_than_days as i32)
        .execute(&self.pool)
        .await
        .map_err(|e| OutboxError::StoreError(e.to_string()))?;
        Ok(result.rows_affected())
    }
}

/// DB行と OutboxMessage の変換用中間構造体。
#[derive(sqlx::FromRow)]
struct OutboxRow {
    id: uuid::Uuid,
    topic: String,
    partition_key: String,
    payload: serde_json::Value,
    status: String,
    retry_count: i32,
    max_retries: i32,
    last_error: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    process_after: chrono::DateTime<chrono::Utc>,
}

impl From<OutboxRow> for OutboxMessage {
    fn from(row: OutboxRow) -> Self {
        OutboxMessage {
            id: row.id,
            topic: row.topic,
            partition_key: row.partition_key,
            payload: row.payload,
            status: OutboxStatus::from_str(&row.status),
            retry_count: row.retry_count as u32,
            max_retries: row.max_retries as u32,
            last_error: row.last_error,
            created_at: row.created_at,
            process_after: row.process_after,
        }
    }
}
