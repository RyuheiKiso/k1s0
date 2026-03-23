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

    /// トランザクション内でイベントを一括INSERTする内部ヘルパー。
    /// 呼び出し元のトランザクション（tx）を受け取り、全件INSERTが成功した場合のみコミットは呼び出し元が行う。
    pub async fn append_in_tx<'a>(
        stream_id: &str,
        events: Vec<StoredEvent>,
        tx: &mut sqlx::Transaction<'a, sqlx::Postgres>,
    ) -> anyhow::Result<Vec<StoredEvent>> {
        let mut result = Vec::with_capacity(events.len());

        for event in events {
            // メタデータをJSONオブジェクトとしてシリアライズする
            let metadata = serde_json::json!({
                "actor_id": event.metadata.actor_id,
                "correlation_id": event.metadata.correlation_id,
                "causation_id": event.metadata.causation_id,
            });
            // トランザクション内でINSERTを実行し、採番されたシーケンスを含む行を返す
            let row = sqlx::query_as::<_, StoredEventRow>(
                r#"
                INSERT INTO eventstore.events
                    (stream_id, event_type, version, payload, metadata, occurred_at, stored_at)
                VALUES ($1, $2, $3, $4, $5, $6, NOW())
                RETURNING stream_id, sequence, event_type, version, payload, metadata, occurred_at, stored_at
                "#,
            )
            .bind(stream_id)
            .bind(&event.event_type)
            .bind(event.version)
            .bind(&event.payload)
            .bind(&metadata)
            .bind(event.occurred_at)
            .fetch_one(&mut **tx)
            .await?;

            result.push(row.into());
        }

        Ok(result)
    }
}

#[derive(sqlx::FromRow)]
struct StoredEventRow {
    stream_id: String,
    sequence: i64,
    event_type: String,
    version: i64,
    payload: serde_json::Value,
    metadata: serde_json::Value,
    occurred_at: chrono::DateTime<chrono::Utc>,
    stored_at: chrono::DateTime<chrono::Utc>,
}

impl From<StoredEventRow> for StoredEvent {
    fn from(row: StoredEventRow) -> Self {
        let actor_id = row
            .metadata
            .get("actor_id")
            .and_then(|v| v.as_str())
            .map(String::from);
        let correlation_id = row
            .metadata
            .get("correlation_id")
            .and_then(|v| v.as_str())
            .map(String::from);
        let causation_id = row
            .metadata
            .get("causation_id")
            .and_then(|v| v.as_str())
            .map(String::from);
        StoredEvent {
            stream_id: row.stream_id,
            sequence: row.sequence as u64,
            event_type: row.event_type,
            version: row.version,
            payload: row.payload,
            metadata: EventMetadata::new(actor_id, correlation_id, causation_id),
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
        // 全イベントのINSERTを単一トランザクションで包み、部分的な書き込みを防止する
        let mut tx = self.pool.begin().await?;

        // トランザクション内でイベントを一括INSERTする
        let result = Self::append_in_tx(stream_id, events, &mut tx).await?;

        // 全件INSERT成功後にコミットする
        tx.commit().await?;

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
        let page_size = page_size.clamp(1, 200);
        let offset = ((page - 1) * page_size) as i64;

        // Build dynamic query for total count
        let total: i64 = if let Some(ref et) = event_type {
            if let Some(tv) = to_version {
                sqlx::query_scalar(
                    r#"SELECT COUNT(*) FROM eventstore.events
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
                    r#"SELECT COUNT(*) FROM eventstore.events
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
                r#"SELECT COUNT(*) FROM eventstore.events
                   WHERE stream_id = $1 AND version >= $2 AND version <= $3"#,
            )
            .bind(stream_id)
            .bind(from_version)
            .bind(tv)
            .fetch_one(&self.pool)
            .await?
        } else {
            sqlx::query_scalar(
                r#"SELECT COUNT(*) FROM eventstore.events
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
                    r#"SELECT stream_id, sequence, event_type, version, payload, metadata, occurred_at, stored_at
                       FROM eventstore.events
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
                    r#"SELECT stream_id, sequence, event_type, version, payload, metadata, occurred_at, stored_at
                       FROM eventstore.events
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
                r#"SELECT stream_id, sequence, event_type, version, payload, metadata, occurred_at, stored_at
                   FROM eventstore.events
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
                r#"SELECT stream_id, sequence, event_type, version, payload, metadata, occurred_at, stored_at
                   FROM eventstore.events
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
        let page_size = page_size.clamp(1, 200);
        let offset = ((page - 1) * page_size) as i64;

        let (total, rows) = if let Some(ref et) = event_type {
            let total: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM eventstore.events WHERE event_type = $1")
                    .bind(et)
                    .fetch_one(&self.pool)
                    .await?;

            let rows = sqlx::query_as::<_, StoredEventRow>(
                r#"SELECT stream_id, sequence, event_type, version, payload, metadata, occurred_at, stored_at
                   FROM eventstore.events
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
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM eventstore.events")
                .fetch_one(&self.pool)
                .await?;

            let rows = sqlx::query_as::<_, StoredEventRow>(
                r#"SELECT stream_id, sequence, event_type, version, payload, metadata, occurred_at, stored_at
                   FROM eventstore.events
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
            SELECT stream_id, sequence, event_type, version, payload, metadata, occurred_at, stored_at
            FROM eventstore.events
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
        let result = sqlx::query("DELETE FROM eventstore.events WHERE stream_id = $1")
            .bind(stream_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }
}
