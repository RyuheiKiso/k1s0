use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::service_doc::{DocType, ServiceDoc};
use crate::domain::repository::DocRepository;

#[derive(Debug, Clone, sqlx::FromRow)]
struct DocRow {
    id: Uuid,
    service_id: Uuid,
    title: String,
    url: String,
    doc_type: String,
    created_at: DateTime<Utc>,
}

impl From<DocRow> for ServiceDoc {
    fn from(row: DocRow) -> Self {
        let doc_type = match row.doc_type.as_str() {
            "runbook" => DocType::Runbook,
            "apispec" => DocType::ApiSpec,
            "architecture" => DocType::Architecture,
            "userguide" => DocType::UserGuide,
            _ => DocType::Other,
        };
        ServiceDoc {
            id: row.id,
            service_id: row.service_id,
            title: row.title,
            url: row.url,
            doc_type,
            created_at: row.created_at,
        }
    }
}

/// `DocPostgresRepository` は `PostgreSQL` ベースのドキュメントリポジトリ。
pub struct DocPostgresRepository {
    pool: PgPool,
    metrics: Option<Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl DocPostgresRepository {
    #[allow(dead_code)]
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            metrics: None,
        }
    }

    #[must_use]
    pub fn with_metrics(pool: PgPool, metrics: Arc<k1s0_telemetry::metrics::Metrics>) -> Self {
        Self {
            pool,
            metrics: Some(metrics),
        }
    }
}

#[async_trait]
impl DocRepository for DocPostgresRepository {
    async fn list_by_service(&self, service_id: Uuid) -> anyhow::Result<Vec<ServiceDoc>> {
        let start = std::time::Instant::now();

        let rows = sqlx::query_as::<_, DocRow>(
            "SELECT id, service_id, title, url, doc_type, created_at \
             FROM service_catalog.service_docs WHERE service_id = $1 \
             ORDER BY created_at ASC",
        )
        .bind(service_id)
        .fetch_all(&self.pool)
        .await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration(
                "list_by_service",
                "service_docs",
                start.elapsed().as_secs_f64(),
            );
        }

        Ok(rows.into_iter().map(std::convert::Into::into).collect())
    }

    async fn set_docs(&self, service_id: Uuid, docs: Vec<ServiceDoc>) -> anyhow::Result<()> {
        let start = std::time::Instant::now();
        let mut tx = self.pool.begin().await?;

        // Delete existing docs for this service
        sqlx::query("DELETE FROM service_catalog.service_docs WHERE service_id = $1")
            .bind(service_id)
            .execute(&mut *tx)
            .await?;

        // Insert new docs
        for doc in &docs {
            sqlx::query(
                "INSERT INTO service_catalog.service_docs \
                 (id, service_id, title, url, doc_type, created_at) \
                 VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(doc.id)
            .bind(doc.service_id)
            .bind(&doc.title)
            .bind(&doc.url)
            .bind(doc.doc_type.to_string())
            .bind(doc.created_at)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("set_docs", "service_docs", start.elapsed().as_secs_f64());
        }

        Ok(())
    }
}
