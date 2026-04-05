use std::collections::HashMap;

use async_trait::async_trait;
use k1s0_server_common::escape_like_pattern;
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

    /// migration 003 で追加した tenant_id カラムを含む行マッピング
    /// DB カラム: id, tenant_id, filename, content_type, storage_path, checksum, uploaded_by
    fn map_row(row: PgRow) -> anyhow::Result<FileMetadata> {
        let tags_json: serde_json::Value = row.try_get("tags")?;
        let tags = serde_json::from_value::<HashMap<String, String>>(tags_json).unwrap_or_default();

        let size_i64: i64 = row.try_get("size_bytes")?;
        let size_bytes = u64::try_from(size_i64)
            .map_err(|_| anyhow::anyhow!("invalid size_bytes in DB: {}", size_i64))?;

        Ok(FileMetadata {
            id: row.try_get("id")?,
            // テナント分離: migration 003 で追加した tenant_id カラムを読み取る
            tenant_id: row.try_get("tenant_id")?,
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
        // tenant_id カラムを SELECT に含める（migration 003 対応）
        let sql = format!(
            "SELECT id, tenant_id, filename, size_bytes, content_type, uploaded_by, tags, storage_path, checksum, status, created_at, updated_at FROM {} WHERE id = $1",
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

        // RLS テナント境界を設定する。tenant_id が None の場合は空文字列で設定し RLS が全行を拒否する
        let tid = tenant_id.as_deref().unwrap_or("");
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tid)
            .execute(&self.pool)
            .await?;

        let mut count_qb = QueryBuilder::<Postgres>::new(format!(
            "SELECT COUNT(*) FROM {} WHERE 1=1",
            self.table_name
        ));
        apply_filters(&mut count_qb, &tenant_id, &uploaded_by, &content_type, &tag);
        let total = count_qb
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await?;

        // tenant_id カラムを SELECT に含める（migration 003 対応）
        let mut qb = QueryBuilder::<Postgres>::new(format!(
            "SELECT id, tenant_id, filename, size_bytes, content_type, uploaded_by, tags, storage_path, checksum, status, created_at, updated_at FROM {} WHERE 1=1",
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
        // RLS テナント境界を設定してから INSERT する（migration 003 の tenant_isolation ポリシーを通過させる）
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(&file.tenant_id)
            .execute(&self.pool)
            .await?;

        // tenant_id カラムを INSERT 句に追加（migration 003 対応）
        let sql = format!(
            "INSERT INTO {} (id, tenant_id, filename, size_bytes, content_type, uploaded_by, tags, storage_path, checksum, status, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)",
            self.table_name
        );
        let size_bytes = i64::try_from(file.size_bytes)
            .map_err(|_| anyhow::anyhow!("size_bytes too large: {}", file.size_bytes))?;
        let tags = serde_json::to_value(&file.tags)?;

        sqlx::query(&sql)
            .bind(&file.id)
            .bind(&file.tenant_id)
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
        // RLS テナント境界を設定してから UPDATE する（tenant_isolation ポリシーを通過させる）
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(&file.tenant_id)
            .execute(&self.pool)
            .await?;

        // tenant_id は変更不可のため UPDATE 句には含めない（不変の識別子として扱う）
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

    /// テナント分離対応: tenant_id カラムによる明示的なフィルタを追加（migration 003 対応）
    /// RLS set_config + storage_path LIKE + tenant_id = で三重のテナント境界を確保する。
    /// expected_uploader が Some の場合は uploaded_by カラムも条件に追加する。
    async fn delete_with_tenant_check(
        &self,
        id: String,
        tenant_id_prefix: String,
        expected_uploader: Option<String>,
    ) -> anyhow::Result<bool> {
        // RLS テナント境界を設定してから DELETE する（tenant_isolation ポリシーを通過させる）
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(&tenant_id_prefix)
            .execute(&self.pool)
            .await?;

        // RUST-001 監査対応: LIKE パターンの特殊文字（%・_・\）をエスケープし、
        // 意図しないパターンマッチングを防ぐ。ESCAPE '\\' 句で明示的にエスケープ文字を指定する。
        let escaped_prefix = escape_like_pattern(&tenant_id_prefix);
        let result = if let Some(ref uploader) = expected_uploader {
            // tenant_id カラムによる明示的なフィルタを追加（RLS との二重防衛）
            let sql = format!(
                "DELETE FROM {} WHERE id = $1 AND tenant_id = $2 AND storage_path LIKE $3 ESCAPE '\\\\' AND uploaded_by = $4",
                self.table_name
            );
            sqlx::query(&sql)
                .bind(&id)
                .bind(&tenant_id_prefix)
                .bind(format!("{}%", escaped_prefix))
                .bind(uploader)
                .execute(&self.pool)
                .await?
        } else {
            // tenant_id カラムによる明示的なフィルタを追加（RLS との二重防衛）
            let sql = format!(
                "DELETE FROM {} WHERE id = $1 AND tenant_id = $2 AND storage_path LIKE $3 ESCAPE '\\\\'",
                self.table_name
            );
            sqlx::query(&sql)
                .bind(&id)
                .bind(&tenant_id_prefix)
                .bind(format!("{}%", escaped_prefix))
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

/// migration 003 で tenant_id カラムが追加されたためフィルタを有効化する
/// RLS の set_config と組み合わせることで二重のテナント境界を実現する
fn apply_filters(
    qb: &mut QueryBuilder<'_, Postgres>,
    tenant_id: &Option<String>,
    uploaded_by: &Option<String>,
    content_type: &Option<String>,
    tag: &Option<(String, String)>,
) {
    // tenant_id カラムが存在するため明示的なフィルタを追加する（RLS との二重防衛）
    if let Some(ref tid) = tenant_id {
        qb.push(" AND tenant_id = ");
        qb.push_bind(tid.clone());
    }
    if let Some(uploaded_by) = uploaded_by {
        qb.push(" AND uploaded_by = ");
        qb.push_bind(uploaded_by.clone());
    }
    // LIKE 検索のエスケープ: ユーザー入力の % _ \ をエスケープしてパターンマッチの意図しない
    // 拡張（SQLインジェクション様のパターンマッチ）を防ぐ。ESCAPE '\\' で明示的なエスケープ文字を指定する
    if let Some(content_type) = content_type {
        qb.push(" AND content_type LIKE ");
        qb.push_bind(format!("{}%", escape_like_pattern(content_type)));
        qb.push(" ESCAPE '\\\\'");
    }
    if let Some((tag_key, tag_value)) = tag {
        qb.push(" AND tags ->> ");
        qb.push_bind(tag_key.clone());
        qb.push(" = ");
        qb.push_bind(tag_value.clone());
    }
}
