use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::domain::entity::api_registration::{ApiSchema, SchemaType};
use crate::domain::repository::ApiSchemaRepository;

pub struct SchemaPostgresRepository {
    pool: Arc<PgPool>,
}

impl SchemaPostgresRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct ApiSchemaRow {
    name: String,
    description: String,
    schema_type: String,
    latest_version: i32,
    version_count: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<ApiSchemaRow> for ApiSchema {
    fn from(r: ApiSchemaRow) -> Self {
        ApiSchema {
            name: r.name,
            description: r.description,
            schema_type: SchemaType::from_str(&r.schema_type),
            latest_version: r.latest_version as u32,
            version_count: r.version_count as u32,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[async_trait]
impl ApiSchemaRepository for SchemaPostgresRepository {
    async fn find_by_name(&self, name: &str) -> anyhow::Result<Option<ApiSchema>> {
        let row: Option<ApiSchemaRow> = sqlx::query_as(
            "SELECT name, description, schema_type, latest_version, version_count, created_at, updated_at \
             FROM apiregistry.api_schemas WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(self.pool.as_ref())
        .await?;
        Ok(row.map(Into::into))
    }

    async fn find_all(
        &self,
        schema_type: Option<String>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<ApiSchema>, u64)> {
        let offset = (page.saturating_sub(1) * page_size) as i64;
        let limit = page_size as i64;

        let rows: Vec<ApiSchemaRow> = if let Some(ref st) = schema_type {
            sqlx::query_as(
                "SELECT name, description, schema_type, latest_version, version_count, created_at, updated_at \
                 FROM apiregistry.api_schemas WHERE schema_type = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
            )
            .bind(st)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool.as_ref())
            .await?
        } else {
            sqlx::query_as(
                "SELECT name, description, schema_type, latest_version, version_count, created_at, updated_at \
                 FROM apiregistry.api_schemas ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool.as_ref())
            .await?
        };

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM apiregistry.api_schemas")
            .fetch_one(self.pool.as_ref())
            .await?;

        Ok((rows.into_iter().map(Into::into).collect(), count.0 as u64))
    }

    async fn create(&self, schema: &ApiSchema) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO apiregistry.api_schemas \
             (name, description, schema_type, latest_version, version_count, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(&schema.name)
        .bind(&schema.description)
        .bind(schema.schema_type.to_string())
        .bind(schema.latest_version as i32)
        .bind(schema.version_count as i32)
        .bind(schema.created_at)
        .bind(schema.updated_at)
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }

    async fn update(&self, schema: &ApiSchema) -> anyhow::Result<()> {
        sqlx::query(
            "UPDATE apiregistry.api_schemas \
             SET description = $2, latest_version = $3, version_count = $4, updated_at = $5 \
             WHERE name = $1",
        )
        .bind(&schema.name)
        .bind(&schema.description)
        .bind(schema.latest_version as i32)
        .bind(schema.version_count as i32)
        .bind(schema.updated_at)
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }
}
