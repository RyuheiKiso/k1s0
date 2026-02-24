use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::entity::event::{EventMetadata, StoredEvent};
use crate::domain::repository::EventRepository;

/// EventPostgresRepository は PostgreSQL 実装のイベントリポジトリ。
pub struct EventPostgresRepository {
    pool: PgPool,
}

impl EventPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct StoredEventRow {
    stream_id: String,
    sequence: i64,
    event_type: String,
    version: i64,
    payload: serde_json::Value,
    actor_id: Option<String>,
    correlation_id: Option<String>,
    causation_id: Option<String>,
    occurred_at: chrono::DateTime<chrono::Utc>,
    stored_at: chrono::DateTime<chrono::Utc>,
}

impl From<StoredEventRow> for StoredEvent {
    fn from(row: StoredEventRow) -> Self {
        StoredEvent {
            stream_id: row.stream_id,
            sequence: row.sequence as u64,
            event_type: row.event_type,
            version: row.version,
            payload: row.payload,
            metadata: EventMetadata::new(row.actor_id, row.correlation_id, row.causation_id),
            occurred_at: row.occurred_at,
            stored_at: row.stored_at,
        }
    }
}

#[async_trait]
impl EventRepository for EventPostgresRepository {
    async fn append(
        &self,
        stream_id: &str,
        events: Vec<StoredEvent>,
    ) -> anyhow::Result<Vec<StoredEvent>> {
        let mut result = Vec::with_capacity(events.len());

        for event in events {
            let row = sqlx::query_as::<_, StoredEventRow>(
                r#"
                INSERT INTO event_store.stored_events
                    (stream_id, event_type, version, payload, actor_id, correlation_id, causation_id, occurred_at, stored_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())
                RETURNING stream_id, sequence, event_type, version, payload, actor_id, correlation_id, causation_id, occurred_at, stored_at
                "#,
            )
            .bind(stream_id)
            .bind(&event.event_type)
            .bind(event.version)
            .bind(&event.payload)
            .bind(&event.metadata.actor_id)
            .bind(&event.metadata.correlation_id)
            .bind(&event.metadata.causation_id)
            .bind(event.occurred_at)
            .fetch_one(&self.pool)
            .await?;

            result.push(row.into());
        }

        Ok(result)
    }

    async fn find_by_stream(
        &self,
        stream_id: &str,
        from_version: i64,
        to_version: Option<i64>,
        event_type: Option<String>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<StoredEvent>, u64)> {
        let page = page.max(1);
        let page_size = page_size.max(1).min(200);
        let offset = ((page - 1) * page_size) as i64;

        // Build dynamic query for total count
        let total: i64 = if let Some(ref et) = event_type {
            if let Some(tv) = to_version {
                sqlx::query_scalar(
                    r#"SELECT COUNT(*) FROM event_store.stored_events
                       WHERE stream_id = $1 AND version >= $2 AND version <= $3 AND event_type = $4"#,
                )
                .bind(stream_id)
                .bind(from_version)
                .bind(tv)
                .bind(et)
                .fetch_one(&self.pool)
                .await?
            } else {
                sqlx::query_scalar(
                    r#"SELECT COUNT(*) FROM event_store.stored_events
                       WHERE stream_id = $1 AND version >= $2 AND event_type = $3"#,
                )
                .bind(stream_id)
                .bind(from_version)
                .bind(et)
                .fetch_one(&self.pool)
                .await?
            }
        } else if let Some(tv) = to_version {
            sqlx::query_scalar(
                r#"SELECT COUNT(*) FROM event_store.stored_events
                   WHERE stream_id = $1 AND version >= $2 AND version <= $3"#,
            )
            .bind(stream_id)
            .bind(from_version)
            .bind(tv)
            .fetch_one(&self.pool)
            .await?
        } else {
            sqlx::query_scalar(
                r#"SELECT COUNT(*) FROM event_store.stored_events
                   WHERE stream_id = $1 AND version >= $2"#,
            )
            .bind(stream_id)
            .bind(from_version)
            .fetch_one(&self.pool)
            .await?
        };

        // Build dynamic query for data
        let rows = if let Some(ref et) = event_type {
            if let Some(tv) = to_version {
                sqlx::query_as::<_, StoredEventRow>(
                    r#"SELECT stream_id, sequence, event_type, version, payload, actor_id, correlation_id, causation_id, occurred_at, stored_at
                       FROM event_store.stored_events
                       WHERE stream_id = $1 AND version >= $2 AND version <= $3 AND event_type = $4
                       ORDER BY sequence ASC
                       LIMIT $5 OFFSET $6"#,
                )
                .bind(stream_id)
                .bind(from_version)
                .bind(tv)
                .bind(et)
                .bind(page_size as i64)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            } else {
                sqlx::query_as::<_, StoredEventRow>(
                    r#"SELECT stream_id, sequence, event_type, version, payload, actor_id, correlation_id, causation_id, occurred_at, stored_at
                       FROM event_store.stored_events
                       WHERE stream_id = $1 AND version >= $2 AND event_type = $3
                       ORDER BY sequence ASC
                       LIMIT $4 OFFSET $5"#,
                )
                .bind(stream_id)
                .bind(from_version)
                .bind(et)
                .bind(page_size as i64)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
        } else if let Some(tv) = to_version {
            sqlx::query_as::<_, StoredEventRow>(
                r#"SELECT stream_id, sequence, event_type, version, payload, actor_id, correlation_id, causation_id, occurred_at, stored_at
                   FROM event_store.stored_events
                   WHERE stream_id = $1 AND version >= $2 AND version <= $3
                   ORDER BY sequence ASC
                   LIMIT $4 OFFSET $5"#,
            )
            .bind(stream_id)
            .bind(from_version)
            .bind(tv)
            .bind(page_size as i64)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, StoredEventRow>(
                r#"SELECT stream_id, sequence, event_type, version, payload, actor_id, correlation_id, causation_id, occurred_at, stored_at
                   FROM event_store.stored_events
                   WHERE stream_id = $1 AND version >= $2
                   ORDER BY sequence ASC
                   LIMIT $3 OFFSET $4"#,
            )
            .bind(stream_id)
            .bind(from_version)
            .bind(page_size as i64)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        };

        let events: Vec<StoredEvent> = rows.into_iter().map(Into::into).collect();
        Ok((events, total as u64))
    }

    async fn find_all(
        &self,
        event_type: Option<String>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<StoredEvent>, u64)> {
        let page = page.max(1);
        let page_size = page_size.max(1).min(200);
        let offset = ((page - 1) * page_size) as i64;

        let (total, rows) = if let Some(ref et) = event_type {
            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM event_store.stored_events WHERE event_type = $1",
            )
            .bind(et)
            .fetch_one(&self.pool)
            .await?;

            let rows = sqlx::query_as::<_, StoredEventRow>(
                r#"SELECT stream_id, sequence, event_type, version, payload, actor_id, correlation_id, causation_id, occurred_at, stored_at
                   FROM event_store.stored_events
                   WHERE event_type = $1
                   ORDER BY sequence DESC
                   LIMIT $2 OFFSET $3"#,
            )
            .bind(et)
            .bind(page_size as i64)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

            (total, rows)
        } else {
            let total: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM event_store.stored_events")
                    .fetch_one(&self.pool)
                    .await?;

            let rows = sqlx::query_as::<_, StoredEventRow>(
                r#"SELECT stream_id, sequence, event_type, version, payload, actor_id, correlation_id, causation_id, occurred_at, stored_at
                   FROM event_store.stored_events
                   ORDER BY sequence DESC
                   LIMIT $1 OFFSET $2"#,
            )
            .bind(page_size as i64)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

            (total, rows)
        };

        let events: Vec<StoredEvent> = rows.into_iter().map(Into::into).collect();
        Ok((events, total as u64))
    }

    async fn find_by_sequence(
        &self,
        stream_id: &str,
        sequence: u64,
    ) -> anyhow::Result<Option<StoredEvent>> {
        let row = sqlx::query_as::<_, StoredEventRow>(
            r#"
            SELECT stream_id, sequence, event_type, version, payload, actor_id, correlation_id, causation_id, occurred_at, stored_at
            FROM event_store.stored_events
            WHERE stream_id = $1 AND sequence = $2
            "#,
        )
        .bind(stream_id)
        .bind(sequence as i64)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Into::into))
    }

    async fn delete_by_stream(&self, stream_id: &str) -> anyhow::Result<u64> {
        let result =
            sqlx::query("DELETE FROM event_store.stored_events WHERE stream_id = $1")
                .bind(stream_id)
                .execute(&self.pool)
                .await?;

        Ok(result.rows_affected())
    }
}
