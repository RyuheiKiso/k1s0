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

    /// C-01 監査対応: DB カラム名を正として Rust コードを DB 定義に合わせる
    /// DB カラム: filename, content_type, storage_path, checksum, uploaded_by
    fn map_row(row: PgRow) -> anyhow::Result<FileMetadata> {
        let tags_json: serde_json::Value = row.try_get("tags")?;
        let tags = serde_json::from_value::<HashMap<String, String>>(tags_json).unwrap_or_default();

        let size_i64: i64 = row.try_get("size_bytes")?;
        let size_bytes = u64::try_from(size_i64)
            .map_err(|_| anyhow::anyhow!("invalid size_bytes in DB: {}", size_i64))?;

        Ok(FileMetadata {
            id: row.try_get("id")?,
            filename: row.try_get("filename")?,
            size_bytes,
            content_type: row.try_get("content_type")?,
            uploaded_by: row.try_get("uploaded_by")?,
            tags,
            storage_path: row.try_get("storage_path")?,
            checksum: row.try_get("checksum")?,
            status: row.try_get("status")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

#[async_trait]
impl FileMetadataRepository for FileMetadataPostgresRepository {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<FileMetadata>> {
        // C-01 監査対応: DB カラム名に合わせて SELECT 句を修正
        let sql = format!(
            "SELECT id, filename, size_bytes, content_type, uploaded_by, tags, storage_path, checksum, status, created_at, updated_at FROM {} WHERE id = $1",
            self.table_name
        );
        let row = sqlx::query(&sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        row.map(Self::map_row).transpose()
    }

    async fn find_all(
        &self,
        tenant_id: Option<String>,
        uploaded_by: Option<String>,
        content_type: Option<String>,
        tag: Option<(String, String)>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<FileMetadata>, u64)> {
        let page = page.max(1);
        let page_size = page_size.clamp(1, 200);
        let offset = i64::from((page - 1) * page_size);

        let mut count_qb = QueryBuilder::<Postgres>::new(format!(
            "SELECT COUNT(*) FROM {} WHERE 1=1",
            self.table_name
        ));
        apply_filters(&mut count_qb, &tenant_id, &uploaded_by, &content_type, &tag);
        let total = count_qb
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await?;

        // C-01 監査対応: DB カラム名に合わせて SELECT 句を修正
        let mut qb = QueryBuilder::<Postgres>::new(format!(
            "SELECT id, filename, size_bytes, content_type, uploaded_by, tags, storage_path, checksum, status, created_at, updated_at FROM {} WHERE 1=1",
            self.table_name
        ));
        apply_filters(&mut qb, &tenant_id, &uploaded_by, &content_type, &tag);
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
        // C-01 監査対応: DB カラム名に合わせて INSERT 句を修正（tenant_id 削除）
        let sql = format!(
            "INSERT INTO {} (id, filename, size_bytes, content_type, uploaded_by, tags, storage_path, checksum, status, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)",
            self.table_name
        );
        let size_bytes = i64::try_from(file.size_bytes)
            .map_err(|_| anyhow::anyhow!("size_bytes too large: {}", file.size_bytes))?;
        let tags = serde_json::to_value(&file.tags)?;

        sqlx::query(&sql)
            .bind(&file.id)
            .bind(&file.filename)
            .bind(size_bytes)
            .bind(&file.content_type)
            .bind(&file.uploaded_by)
            .bind(tags)
            .bind(&file.storage_path)
            .bind(&file.checksum)
            .bind(&file.status)
            .bind(file.created_at)
            .bind(file.updated_at)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn update(&self, file: &FileMetadata) -> anyhow::Result<()> {
        // C-01 監査対応: DB カラム名に合わせて UPDATE 句を修正（tenant_id 削除）
        let sql = format!(
            "UPDATE {} SET filename = $2, size_bytes = $3, content_type = $4, uploaded_by = $5, tags = $6, storage_path = $7, checksum = $8, status = $9, updated_at = $10 WHERE id = $1",
            self.table_name
        );
        let size_bytes = i64::try_from(file.size_bytes)
            .map_err(|_| anyhow::anyhow!("size_bytes too large: {}", file.size_bytes))?;
        let tags = serde_json::to_value(&file.tags)?;

        sqlx::query(&sql)
            .bind(&file.id)
            .bind(&file.filename)
            .bind(size_bytes)
            .bind(&file.content_type)
            .bind(&file.uploaded_by)
            .bind(tags)
            .bind(&file.storage_path)
            .bind(&file.checksum)
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

    /// CRIT-01 監査対応: テナントIDと所有者IDを DELETE 条件に追加してアトミックな認可チェックを実現する。
    /// storage_path が tenant_id_prefix で始まる行のみ削除することで、
    /// 認可チェックと削除の間に生じる TOCTOU 競合を防ぐ。
    /// expected_uploader が Some の場合は uploaded_by カラムも条件に追加する。
    async fn delete_with_tenant_check(
        &self,
        id: String,
        tenant_id_prefix: String,
        expected_uploader: Option<String>,
    ) -> anyhow::Result<bool> {
        let result = if let Some(ref uploader) = expected_uploader {
            let sql = format!(
                "DELETE FROM {} WHERE id = $1 AND storage_path LIKE $2 AND uploaded_by = $3",
                self.table_name
            );
            sqlx::query(&sql)
                .bind(&id)
                .bind(format!("{}%", tenant_id_prefix))
                .bind(uploader)
                .execute(&self.pool)
                .await?
        } else {
            let sql = format!(
                "DELETE FROM {} WHERE id = $1 AND storage_path LIKE $2",
                self.table_name
            );
            sqlx::query(&sql)
                .bind(&id)
                .bind(format!("{}%", tenant_id_prefix))
                .execute(&self.pool)
                .await?
        };
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

/// C-01 監査対応: フィルタ条件のカラム名を DB 定義に合わせる
/// DB にはtenant_idカラムが存在しないため、uploaded_by でフィルタする
fn apply_filters(
    qb: &mut QueryBuilder<'_, Postgres>,
    _tenant_id: &Option<String>,
    uploaded_by: &Option<String>,
    content_type: &Option<String>,
    tag: &Option<(String, String)>,
) {
    // tenant_id は DB に存在しないためフィルタを適用しない（将来的にカラム追加時に対応可能）
    if let Some(uploaded_by) = uploaded_by {
        qb.push(" AND uploaded_by = ");
        qb.push_bind(uploaded_by.clone());
    }
    if let Some(content_type) = content_type {
        qb.push(" AND content_type LIKE ");
        qb.push_bind(format!("{}%", content_type));
    }
    if let Some((tag_key, tag_value)) = tag {
        qb.push(" AND tags ->> ");
        qb.push_bind(tag_key.clone());
        qb.push(" = ");
        qb.push_bind(tag_value.clone());
    }
}
