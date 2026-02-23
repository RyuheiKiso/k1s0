use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::domain::entity::config_schema::ConfigSchema;
use crate::domain::repository::config_schema_repository::ConfigSchemaRepository;

/// ConfigSchemaPostgresRepository は ConfigSchemaRepository の PostgreSQL 実装。
pub struct ConfigSchemaPostgresRepository {
    pool: PgPool,
    metrics: Option<Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl ConfigSchemaPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            metrics: None,
        }
    }

    pub fn with_metrics(pool: PgPool, metrics: Arc<k1s0_telemetry::metrics::Metrics>) -> Self {
        Self {
            pool,
            metrics: Some(metrics),
        }
    }
}

/// PostgreSQL の行から ConfigSchema を構築するヘルパー。
fn row_to_config_schema(row: sqlx::postgres::PgRow) -> Result<ConfigSchema, sqlx::Error> {
    Ok(ConfigSchema {
        id: row.try_get("id")?,
        service_name: row.try_get("service_name")?,
        namespace_prefix: row.try_get("namespace_prefix")?,
        schema_json: row.try_get("schema_json")?,
        updated_by: row.try_get("updated_by")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[async_trait]
impl ConfigSchemaRepository for ConfigSchemaPostgresRepository {
    async fn find_by_service_name(
        &self,
        service_name: &str,
    ) -> anyhow::Result<Option<ConfigSchema>> {
        let start = std::time::Instant::now();
        let row = sqlx::query(
            r#"
            SELECT id, service_name, namespace_prefix, schema_json,
                   updated_by, created_at, updated_at
            FROM config.config_schemas
            WHERE service_name = $1
            "#,
        )
        .bind(service_name)
        .fetch_optional(&self.pool)
        .await?;
        if let Some(ref m) = self.metrics {
            m.record_db_query_duration(
                "find_by_service_name",
                "config_schemas",
                start.elapsed().as_secs_f64(),
            );
        }

        match row {
            Some(row) => Ok(Some(row_to_config_schema(row)?)),
            None => Ok(None),
        }
    }

    async fn upsert(&self, schema: &ConfigSchema) -> anyhow::Result<ConfigSchema> {
        let start = std::time::Instant::now();
        let row = sqlx::query(
            r#"
            INSERT INTO config.config_schemas (id, service_name, namespace_prefix, schema_json, updated_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (service_name) DO UPDATE
            SET namespace_prefix = EXCLUDED.namespace_prefix,
                schema_json = EXCLUDED.schema_json,
                updated_by = EXCLUDED.updated_by,
                updated_at = EXCLUDED.updated_at
            RETURNING id, service_name, namespace_prefix, schema_json, updated_by, created_at, updated_at
            "#,
        )
        .bind(schema.id)
        .bind(&schema.service_name)
        .bind(&schema.namespace_prefix)
        .bind(&schema.schema_json)
        .bind(&schema.updated_by)
        .bind(schema.created_at)
        .bind(schema.updated_at)
        .fetch_one(&self.pool)
        .await?;
        if let Some(ref m) = self.metrics {
            m.record_db_query_duration(
                "upsert",
                "config_schemas",
                start.elapsed().as_secs_f64(),
            );
        }

        Ok(row_to_config_schema(row)?)
    }
}
