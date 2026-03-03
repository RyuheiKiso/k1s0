use std::collections::HashMap;

use async_trait::async_trait;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Postgres, QueryBuilder, Row};

use crate::domain::entity::file::FileMetadata;
use crate::domain::repository::FileMetadataRepository;

pub struct FileMetadataPostgresRepository {
    pool: PgPool,
    table_name: String,
}

impl FileMetadataPostgresRepository {
    pub fn new(pool: PgPool, schema: &str) -> anyhow::Result<Self> {
        let schema = sanitize_schema(schema)?;
        Ok(Self {
            pool,
            table_name: format!("{}.file_metadata", schema),
        })
    }

    fn map_row(row: PgRow) -> anyhow::Result<FileMetadata> {
        let tags_json: serde_json::Value = row.try_get("tags")?;
        let tags = serde_json::from_value::<HashMap<String, String>>(tags_json).unwrap_or_default();

        let size_i64: i64 = row.try_get("size_bytes")?;
        let size_bytes = u64::try_from(size_i64)
            .map_err(|_| anyhow::anyhow!("invalid size_bytes in DB: {}", size_i64))?;

        Ok(FileMetadata {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            size_bytes,
            mime_type: row.try_get("mime_type")?,
            tenant_id: row.try_get("tenant_id")?,
            owner_id: row.try_get("owner_id")?,
            tags,
            storage_key: row.try_get("storage_key")?,
            checksum_sha256: row.try_get("checksum_sha256")?,
            status: row.try_get("status")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

#[async_trait]
impl FileMetadataRepository for FileMetadataPostgresRepository {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<FileMetadata>> {
        let sql = format!(
            "SELECT id, name, size_bytes, mime_type, tenant_id, owner_id, tags, storage_key, checksum_sha256, status, created_at, updated_at FROM {} WHERE id = $1",
            self.table_name
        );
        let row = sqlx::query(&sql).bind(id).fetch_optional(&self.pool).await?;
        row.map(Self::map_row).transpose()
    }

    async fn find_all(
        &self,
        tenant_id: Option<String>,
        owner_id: Option<String>,
        mime_type: Option<String>,
        tag: Option<(String, String)>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<FileMetadata>, u64)> {
        let page = page.max(1);
        let page_size = page_size.max(1).min(200);
        let offset = i64::from((page - 1) * page_size);

        let mut count_qb =
            QueryBuilder::<Postgres>::new(format!("SELECT COUNT(*) FROM {} WHERE 1=1", self.table_name));
        apply_filters(&mut count_qb, &tenant_id, &owner_id, &mime_type, &tag);
        let total = count_qb.build_query_scalar::<i64>().fetch_one(&self.pool).await?;

        let mut qb = QueryBuilder::<Postgres>::new(format!(
            "SELECT id, name, size_bytes, mime_type, tenant_id, owner_id, tags, storage_key, checksum_sha256, status, created_at, updated_at FROM {} WHERE 1=1",
            self.table_name
        ));
        apply_filters(&mut qb, &tenant_id, &owner_id, &mime_type, &tag);
        qb.push(" ORDER BY created_at DESC LIMIT ");
        qb.push_bind(i64::from(page_size));
        qb.push(" OFFSET ");
        qb.push_bind(offset);

        let rows = qb.build().fetch_all(&self.pool).await?;
        let files = rows
            .into_iter()
            .map(Self::map_row)
            .collect::<anyhow::Result<Vec<_>>>()?;

        Ok((files, u64::try_from(total).unwrap_or(0)))
    }

    async fn create(&self, file: &FileMetadata) -> anyhow::Result<()> {
        let sql = format!(
            "INSERT INTO {} (id, name, size_bytes, mime_type, tenant_id, owner_id, tags, storage_key, checksum_sha256, status, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)",
            self.table_name
        );
        let size_bytes = i64::try_from(file.size_bytes)
            .map_err(|_| anyhow::anyhow!("size_bytes too large: {}", file.size_bytes))?;
        let tags = serde_json::to_value(&file.tags)?;

        sqlx::query(&sql)
            .bind(&file.id)
            .bind(&file.name)
            .bind(size_bytes)
            .bind(&file.mime_type)
            .bind(&file.tenant_id)
            .bind(&file.owner_id)
            .bind(tags)
            .bind(&file.storage_key)
            .bind(&file.checksum_sha256)
            .bind(&file.status)
            .bind(file.created_at)
            .bind(file.updated_at)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn update(&self, file: &FileMetadata) -> anyhow::Result<()> {
        let sql = format!(
            "UPDATE {} SET name = $2, size_bytes = $3, mime_type = $4, tenant_id = $5, owner_id = $6, tags = $7, storage_key = $8, checksum_sha256 = $9, status = $10, updated_at = $11 WHERE id = $1",
            self.table_name
        );
        let size_bytes = i64::try_from(file.size_bytes)
            .map_err(|_| anyhow::anyhow!("size_bytes too large: {}", file.size_bytes))?;
        let tags = serde_json::to_value(&file.tags)?;

        sqlx::query(&sql)
            .bind(&file.id)
            .bind(&file.name)
            .bind(size_bytes)
            .bind(&file.mime_type)
            .bind(&file.tenant_id)
            .bind(&file.owner_id)
            .bind(tags)
            .bind(&file.storage_key)
            .bind(&file.checksum_sha256)
            .bind(&file.status)
            .bind(file.updated_at)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn delete(&self, id: &str) -> anyhow::Result<bool> {
        let sql = format!("DELETE FROM {} WHERE id = $1", self.table_name);
        let result = sqlx::query(&sql).bind(id).execute(&self.pool).await?;
        Ok(result.rows_affected() > 0)
    }
}

fn sanitize_schema(schema: &str) -> anyhow::Result<&str> {
    if schema.is_empty() {
        return Err(anyhow::anyhow!("database schema cannot be empty"));
    }
    if !schema
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(anyhow::anyhow!(
            "database schema contains invalid characters: {}",
            schema
        ));
    }
    Ok(schema)
}

fn apply_filters(
    qb: &mut QueryBuilder<'_, Postgres>,
    tenant_id: &Option<String>,
    owner_id: &Option<String>,
    mime_type: &Option<String>,
    tag: &Option<(String, String)>,
) {
    if let Some(tenant_id) = tenant_id {
        qb.push(" AND tenant_id = ");
        qb.push_bind(tenant_id.clone());
    }
    if let Some(owner_id) = owner_id {
        qb.push(" AND owner_id = ");
        qb.push_bind(owner_id.clone());
    }
    if let Some(mime_type) = mime_type {
        qb.push(" AND mime_type LIKE ");
        qb.push_bind(format!("{}%", mime_type));
    }
    if let Some((tag_key, tag_value)) = tag {
        qb.push(" AND tags ->> ");
        qb.push_bind(tag_key.clone());
        qb.push(" = ");
        qb.push_bind(tag_value.clone());
    }
}
