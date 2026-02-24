use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::entity::event::Snapshot;
use crate::domain::repository::SnapshotRepository;

/// SnapshotPostgresRepository は PostgreSQL 実装のスナップショットリポジトリ。
pub struct SnapshotPostgresRepository {
    pool: PgPool,
}

impl SnapshotPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct SnapshotRow {
    id: String,
    stream_id: String,
    snapshot_version: i64,
    aggregate_type: String,
    state: serde_json::Value,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<SnapshotRow> for Snapshot {
    fn from(row: SnapshotRow) -> Self {
        Snapshot {
            id: row.id,
            stream_id: row.stream_id,
            snapshot_version: row.snapshot_version,
            aggregate_type: row.aggregate_type,
            state: row.state,
            created_at: row.created_at,
        }
    }
}

#[async_trait]
impl SnapshotRepository for SnapshotPostgresRepository {
    async fn create(&self, snapshot: &Snapshot) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO event_store.snapshots
                (id, stream_id, snapshot_version, aggregate_type, state, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(&snapshot.id)
        .bind(&snapshot.stream_id)
        .bind(snapshot.snapshot_version)
        .bind(&snapshot.aggregate_type)
        .bind(&snapshot.state)
        .bind(snapshot.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_latest(&self, stream_id: &str) -> anyhow::Result<Option<Snapshot>> {
        let row = sqlx::query_as::<_, SnapshotRow>(
            r#"
            SELECT id, stream_id, snapshot_version, aggregate_type, state, created_at
            FROM event_store.snapshots
            WHERE stream_id = $1
            ORDER BY snapshot_version DESC
            LIMIT 1
            "#,
        )
        .bind(stream_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Into::into))
    }

    async fn delete_by_stream(&self, stream_id: &str) -> anyhow::Result<u64> {
        let result =
            sqlx::query("DELETE FROM event_store.snapshots WHERE stream_id = $1")
                .bind(stream_id)
                .execute(&self.pool)
                .await?;

        Ok(result.rows_affected())
    }
}
