use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::entity::event::EventStream;
use crate::domain::repository::EventStreamRepository;

/// StreamPostgresRepository は PostgreSQL 実装のイベントストリームリポジトリ。
pub struct StreamPostgresRepository {
    pool: PgPool,
}

impl StreamPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct EventStreamRow {
    id: String,
    aggregate_type: String,
    current_version: i64,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<EventStreamRow> for EventStream {
    fn from(row: EventStreamRow) -> Self {
        EventStream {
            id: row.id,
            aggregate_type: row.aggregate_type,
            current_version: row.current_version,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[async_trait]
impl EventStreamRepository for StreamPostgresRepository {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<EventStream>> {
        let row = sqlx::query_as::<_, EventStreamRow>(
            r#"
            SELECT id, aggregate_type, current_version, created_at, updated_at
            FROM event_store.event_streams
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Into::into))
    }

    async fn list_all(&self, page: u32, page_size: u32) -> anyhow::Result<(Vec<EventStream>, u64)> {
        let total: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM event_store.event_streams")
                .fetch_one(&self.pool)
                .await?;

        let page = page.max(1);
        let page_size = page_size.max(1).min(200);
        let offset = ((page - 1) * page_size) as i64;

        let rows = sqlx::query_as::<_, EventStreamRow>(
            r#"
            SELECT id, aggregate_type, current_version, created_at, updated_at
            FROM event_store.event_streams
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(page_size as i64)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let streams: Vec<EventStream> = rows.into_iter().map(Into::into).collect();
        Ok((streams, total as u64))
    }

    async fn create(&self, stream: &EventStream) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO event_store.event_streams
                (id, aggregate_type, current_version, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(&stream.id)
        .bind(&stream.aggregate_type)
        .bind(stream.current_version)
        .bind(stream.created_at)
        .bind(stream.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update_version(&self, id: &str, new_version: i64) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE event_store.event_streams
            SET current_version = $2, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(new_version)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete(&self, id: &str) -> anyhow::Result<bool> {
        let result = sqlx::query("DELETE FROM event_store.event_streams WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
