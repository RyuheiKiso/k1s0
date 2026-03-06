use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entity::event_record::EventRecord;
use crate::domain::repository::EventRecordRepository;

pub struct EventRecordPostgresRepository {
    pool: Arc<PgPool>,
}

impl EventRecordPostgresRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EventRecordRepository for EventRecordPostgresRepository {
    async fn create(&self, record: &EventRecord) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO event_monitor.event_records
                (id, correlation_id, event_type, source, domain, trace_id, timestamp, flow_id, flow_step_index, status, received_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(record.id)
        .bind(&record.correlation_id)
        .bind(&record.event_type)
        .bind(&record.source)
        .bind(&record.domain)
        .bind(&record.trace_id)
        .bind(record.timestamp)
        .bind(record.flow_id)
        .bind(record.flow_step_index)
        .bind(&record.status)
        .bind(record.received_at)
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }

    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<EventRecord>> {
        let row = sqlx::query_as::<_, EventRecordRow>(
            "SELECT id, correlation_id, event_type, source, domain, trace_id, timestamp, flow_id, flow_step_index, status, received_at FROM event_monitor.event_records WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(self.pool.as_ref())
        .await?;
        Ok(row.map(Into::into))
    }

    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        domain: Option<String>,
        event_type: Option<String>,
        source: Option<String>,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        status: Option<String>,
    ) -> anyhow::Result<(Vec<EventRecord>, u64)> {
        let offset = (page.saturating_sub(1) * page_size) as i64;
        let limit = page_size as i64;

        let rows = sqlx::query_as::<_, EventRecordRow>(
            r#"
            SELECT id, correlation_id, event_type, source, domain, trace_id, timestamp, flow_id, flow_step_index, status, received_at
            FROM event_monitor.event_records
            WHERE ($1::text IS NULL OR domain = $1)
              AND ($2::text IS NULL OR event_type = $2)
              AND ($3::text IS NULL OR source = $3)
              AND ($4::timestamptz IS NULL OR timestamp >= $4)
              AND ($5::timestamptz IS NULL OR timestamp <= $5)
              AND ($6::text IS NULL OR status = $6)
            ORDER BY timestamp DESC
            LIMIT $7 OFFSET $8
            "#,
        )
        .bind(&domain)
        .bind(&event_type)
        .bind(&source)
        .bind(from)
        .bind(to)
        .bind(&status)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.as_ref())
        .await?;

        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM event_monitor.event_records
            WHERE ($1::text IS NULL OR domain = $1)
              AND ($2::text IS NULL OR event_type = $2)
              AND ($3::text IS NULL OR source = $3)
              AND ($4::timestamptz IS NULL OR timestamp >= $4)
              AND ($5::timestamptz IS NULL OR timestamp <= $5)
              AND ($6::text IS NULL OR status = $6)
            "#,
        )
        .bind(&domain)
        .bind(&event_type)
        .bind(&source)
        .bind(from)
        .bind(to)
        .bind(&status)
        .fetch_one(self.pool.as_ref())
        .await?;

        Ok((rows.into_iter().map(Into::into).collect(), count.0 as u64))
    }

    async fn find_by_correlation_id(
        &self,
        correlation_id: String,
    ) -> anyhow::Result<Vec<EventRecord>> {
        let rows = sqlx::query_as::<_, EventRecordRow>(
            r#"
            SELECT id, correlation_id, event_type, source, domain, trace_id, timestamp, flow_id, flow_step_index, status, received_at
            FROM event_monitor.event_records
            WHERE correlation_id = $1
            ORDER BY timestamp ASC
            "#,
        )
        .bind(&correlation_id)
        .fetch_all(self.pool.as_ref())
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}

#[derive(sqlx::FromRow)]
struct EventRecordRow {
    id: Uuid,
    correlation_id: String,
    event_type: String,
    source: String,
    domain: String,
    trace_id: String,
    timestamp: DateTime<Utc>,
    flow_id: Option<Uuid>,
    flow_step_index: Option<i32>,
    status: String,
    received_at: DateTime<Utc>,
}

impl From<EventRecordRow> for EventRecord {
    fn from(row: EventRecordRow) -> Self {
        Self {
            id: row.id,
            correlation_id: row.correlation_id,
            event_type: row.event_type,
            source: row.source,
            domain: row.domain,
            trace_id: row.trace_id,
            timestamp: row.timestamp,
            flow_id: row.flow_id,
            flow_step_index: row.flow_step_index,
            status: row.status,
            received_at: row.received_at,
        }
    }
}
