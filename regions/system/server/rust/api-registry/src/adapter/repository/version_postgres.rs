use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::domain::entity::api_registration::{ApiSchemaVersion, SchemaType};
use crate::domain::repository::ApiSchemaVersionRepository;

pub struct VersionPostgresRepository {
    pool: Arc<PgPool>,
}

impl VersionPostgresRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct ApiSchemaVersionRow {
    name: String,
    version: i32,
    schema_type: String,
    content: String,
    content_hash: String,
    breaking_changes: bool,
    registered_by: String,
    created_at: DateTime<Utc>,
}

impl From<ApiSchemaVersionRow> for ApiSchemaVersion {
    fn from(r: ApiSchemaVersionRow) -> Self {
        ApiSchemaVersion {
            name: r.name,
            version: r.version as u32,
            schema_type: SchemaType::from_str(&r.schema_type),
            content: r.content,
            content_hash: r.content_hash,
            breaking_changes: r.breaking_changes,
            breaking_change_details: vec![],
            registered_by: r.registered_by,
            created_at: r.created_at,
        }
    }
}

#[async_trait]
impl ApiSchemaVersionRepository for VersionPostgresRepository {
    async fn find_by_name_and_version(
        &self,
        name: &str,
        version: u32,
    ) -> anyhow::Result<Option<ApiSchemaVersion>> {
        let row: Option<ApiSchemaVersionRow> = sqlx::query_as(
            "SELECT name, version, schema_type, content, content_hash, breaking_changes, registered_by, created_at \
             FROM apiregistry.api_schema_versions WHERE name = $1 AND version = $2",
        )
        .bind(name)
        .bind(version as i32)
        .fetch_optional(self.pool.as_ref())
        .await?;
        Ok(row.map(Into::into))
    }

    async fn find_latest_by_name(&self, name: &str) -> anyhow::Result<Option<ApiSchemaVersion>> {
        let row: Option<ApiSchemaVersionRow> = sqlx::query_as(
            "SELECT name, version, schema_type, content, content_hash, breaking_changes, registered_by, created_at \
             FROM apiregistry.api_schema_versions WHERE name = $1 ORDER BY version DESC LIMIT 1",
        )
        .bind(name)
        .fetch_optional(self.pool.as_ref())
        .await?;
        Ok(row.map(Into::into))
    }

    async fn find_all_by_name(
        &self,
        name: &str,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<ApiSchemaVersion>, u64)> {
        let offset = (page.saturating_sub(1) * page_size) as i64;
        let limit = page_size as i64;
        let rows: Vec<ApiSchemaVersionRow> = sqlx::query_as(
            "SELECT name, version, schema_type, content, content_hash, breaking_changes, registered_by, created_at \
             FROM apiregistry.api_schema_versions WHERE name = $1 ORDER BY version DESC LIMIT $2 OFFSET $3",
        )
        .bind(name)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.as_ref())
        .await?;
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM apiregistry.api_schema_versions WHERE name = $1",
        )
        .bind(name)
        .fetch_one(self.pool.as_ref())
        .await?;
        Ok((rows.into_iter().map(Into::into).collect(), count.0 as u64))
    }

    async fn create(&self, version: &ApiSchemaVersion) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO apiregistry.api_schema_versions \
             (name, version, schema_type, content, content_hash, breaking_changes, registered_by, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(&version.name)
        .bind(version.version as i32)
        .bind(version.schema_type.to_string())
        .bind(&version.content)
        .bind(&version.content_hash)
        .bind(version.breaking_changes)
        .bind(&version.registered_by)
        .bind(version.created_at)
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }

    async fn delete(&self, name: &str, version: u32) -> anyhow::Result<bool> {
        let result = sqlx::query(
            "DELETE FROM apiregistry.api_schema_versions WHERE name = $1 AND version = $2",
        )
        .bind(name)
        .bind(version as i32)
        .execute(self.pool.as_ref())
        .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn count_by_name(&self, name: &str) -> anyhow::Result<u64> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM apiregistry.api_schema_versions WHERE name = $1",
        )
        .bind(name)
        .fetch_one(self.pool.as_ref())
        .await?;
        Ok(count.0 as u64)
    }
}
