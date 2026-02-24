use async_trait::async_trait;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};

use crate::envelope::EventEnvelope;
use crate::error::EventStoreError;
use crate::store::EventStore;
use crate::stream::StreamId;

/// PostgreSQL-backed event store implementation.
///
/// Stores events in a `events` table with optimistic concurrency control
/// via version checking.
#[derive(Clone)]
pub struct PostgresEventStore {
    pool: PgPool,
}

impl PostgresEventStore {
    /// Create a new PostgresEventStore from a database URL.
    ///
    /// # Arguments
    /// * `database_url` - PostgreSQL connection URL
    ///   (e.g., "postgres://user:pass@localhost:5432/dbname")
    pub async fn new(database_url: &str) -> Result<Self, EventStoreError> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await
            .map_err(|e| EventStoreError::StorageError(e.to_string()))?;
        Ok(Self { pool })
    }

    /// Create a new PostgresEventStore with a custom connection pool.
    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Run the migration to create the events table if it does not exist.
    pub async fn migrate(&self) -> Result<(), EventStoreError> {
        sqlx::query(CREATE_EVENTS_TABLE)
            .execute(&self.pool)
            .await
            .map_err(|e| EventStoreError::StorageError(e.to_string()))?;
        Ok(())
    }
}

const CREATE_EVENTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS events (
    id BIGSERIAL PRIMARY KEY,
    event_id UUID NOT NULL UNIQUE,
    stream_id TEXT NOT NULL,
    version BIGINT NOT NULL,
    event_type TEXT NOT NULL,
    payload JSONB NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (stream_id, version)
);
CREATE INDEX IF NOT EXISTS idx_events_stream_id ON events (stream_id, version);
"#;

#[async_trait]
impl EventStore for PostgresEventStore {
    async fn append(
        &self,
        stream_id: &StreamId,
        events: Vec<EventEnvelope>,
        expected_version: Option<u64>,
    ) -> Result<u64, EventStoreError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| EventStoreError::StorageError(e.to_string()))?;

        // Get current version with row-level lock to prevent concurrent writes
        let current_version: i64 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(version), 0) FROM events WHERE stream_id = $1 FOR UPDATE",
        )
        .bind(stream_id.as_str())
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| EventStoreError::StorageError(e.to_string()))?;

        let current_version = current_version as u64;

        if let Some(expected) = expected_version {
            if expected != current_version {
                return Err(EventStoreError::VersionConflict {
                    expected,
                    actual: current_version,
                });
            }
        }

        let mut version = current_version;
        for event in &events {
            version += 1;
            sqlx::query(
                r#"
                INSERT INTO events (event_id, stream_id, version, event_type, payload, metadata, recorded_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
            )
            .bind(event.event_id)
            .bind(stream_id.as_str())
            .bind(version as i64)
            .bind(&event.event_type)
            .bind(&event.payload)
            .bind(&event.metadata)
            .bind(event.recorded_at)
            .execute(&mut *tx)
            .await
            .map_err(|e| EventStoreError::StorageError(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| EventStoreError::StorageError(e.to_string()))?;

        Ok(version)
    }

    async fn load(&self, stream_id: &StreamId) -> Result<Vec<EventEnvelope>, EventStoreError> {
        let rows = sqlx::query(
            r#"
            SELECT event_id, stream_id, version, event_type, payload, metadata, recorded_at
            FROM events
            WHERE stream_id = $1
            ORDER BY version ASC
            "#,
        )
        .bind(stream_id.as_str())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EventStoreError::StorageError(e.to_string()))?;

        let events = rows
            .iter()
            .map(|row| EventEnvelope {
                event_id: row.get("event_id"),
                stream_id: row.get("stream_id"),
                version: row.get::<i64, _>("version") as u64,
                event_type: row.get("event_type"),
                payload: row.get("payload"),
                metadata: row.get("metadata"),
                recorded_at: row.get("recorded_at"),
            })
            .collect();

        Ok(events)
    }

    async fn load_from(
        &self,
        stream_id: &StreamId,
        from_version: u64,
    ) -> Result<Vec<EventEnvelope>, EventStoreError> {
        let rows = sqlx::query(
            r#"
            SELECT event_id, stream_id, version, event_type, payload, metadata, recorded_at
            FROM events
            WHERE stream_id = $1 AND version >= $2
            ORDER BY version ASC
            "#,
        )
        .bind(stream_id.as_str())
        .bind(from_version as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EventStoreError::StorageError(e.to_string()))?;

        let events = rows
            .iter()
            .map(|row| EventEnvelope {
                event_id: row.get("event_id"),
                stream_id: row.get("stream_id"),
                version: row.get::<i64, _>("version") as u64,
                event_type: row.get("event_type"),
                payload: row.get("payload"),
                metadata: row.get("metadata"),
                recorded_at: row.get("recorded_at"),
            })
            .collect();

        Ok(events)
    }

    async fn exists(&self, stream_id: &StreamId) -> Result<bool, EventStoreError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM events WHERE stream_id = $1 LIMIT 1",
        )
        .bind(stream_id.as_str())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| EventStoreError::StorageError(e.to_string()))?;

        Ok(count > 0)
    }

    async fn current_version(&self, stream_id: &StreamId) -> Result<u64, EventStoreError> {
        let version: i64 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(version), 0) FROM events WHERE stream_id = $1",
        )
        .bind(stream_id.as_str())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| EventStoreError::StorageError(e.to_string()))?;

        Ok(version as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_events_table_sql_is_valid() {
        // Verify the SQL string is non-empty and contains expected keywords
        assert!(CREATE_EVENTS_TABLE.contains("CREATE TABLE IF NOT EXISTS events"));
        assert!(CREATE_EVENTS_TABLE.contains("event_id UUID NOT NULL UNIQUE"));
        assert!(CREATE_EVENTS_TABLE.contains("stream_id TEXT NOT NULL"));
        assert!(CREATE_EVENTS_TABLE.contains("version BIGINT NOT NULL"));
        assert!(CREATE_EVENTS_TABLE.contains("UNIQUE (stream_id, version)"));
    }
}
