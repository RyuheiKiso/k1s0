use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::domain::entity::api_registration::{ApiSchemaVersion, BreakingChange, SchemaType};
use crate::domain::repository::ApiSchemaVersionRepository;

pub struct VersionPostgresRepository {
    pool: Arc<PgPool>,
}

impl VersionPostgresRepository {
    #[must_use] 
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
    breaking_change_details: Option<serde_json::Value>,
    registered_by: String,
    created_at: DateTime<Utc>,
}

impl From<ApiSchemaVersionRow> for ApiSchemaVersion {
    fn from(r: ApiSchemaVersionRow) -> Self {
        let breaking_change_details: Vec<BreakingChange> = r
            .breaking_change_details
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();
        ApiSchemaVersion {
            name: r.name,
            version: r.version as u32,
            schema_type: SchemaType::from_str(&r.schema_type),
            content: r.content,
            content_hash: r.content_hash,
            breaking_changes: r.breaking_changes,
            breaking_change_details,
            registered_by: r.registered_by,
            created_at: r.created_at,
        }
    }
}

#[async_trait]
impl ApiSchemaVersionRepository for VersionPostgresRepository {
    // テナントスコープで set_config を設定した後にスキーマ名とバージョンで検索する。
    async fn find_by_name_and_version(
        &self,
        tenant_id: &str,
        name: &str,
        version: u32,
    ) -> anyhow::Result<Option<ApiSchemaVersion>> {
        // トランザクション内で RLS 用セッション変数を設定する
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;
        let row: Option<ApiSchemaVersionRow> = sqlx::query_as(
            "SELECT name, version, schema_type, content, content_hash, breaking_changes, breaking_change_details, registered_by, created_at \
             FROM apiregistry.api_schema_versions WHERE tenant_id = $1 AND name = $2 AND version = $3",
        )
        .bind(tenant_id)
        .bind(name)
        .bind(version as i32)
        .fetch_optional(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(row.map(Into::into))
    }

    // テナントスコープで set_config を設定した後に最新バージョンを取得する。
    async fn find_latest_by_name(&self, tenant_id: &str, name: &str) -> anyhow::Result<Option<ApiSchemaVersion>> {
        // トランザクション内で RLS 用セッション変数を設定する
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;
        let row: Option<ApiSchemaVersionRow> = sqlx::query_as(
            "SELECT name, version, schema_type, content, content_hash, breaking_changes, breaking_change_details, registered_by, created_at \
             FROM apiregistry.api_schema_versions WHERE tenant_id = $1 AND name = $2 ORDER BY version DESC LIMIT 1",
        )
        .bind(tenant_id)
        .bind(name)
        .fetch_optional(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(row.map(Into::into))
    }

    // テナントスコープで set_config を設定した後にバージョン一覧を取得する。
    async fn find_all_by_name(
        &self,
        tenant_id: &str,
        name: &str,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<ApiSchemaVersion>, u64)> {
        let offset = i64::from(page.saturating_sub(1) * page_size);
        let limit = i64::from(page_size);
        // トランザクション内で RLS 用セッション変数を設定する
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;
        let rows: Vec<ApiSchemaVersionRow> = sqlx::query_as(
            "SELECT name, version, schema_type, content, content_hash, breaking_changes, breaking_change_details, registered_by, created_at \
             FROM apiregistry.api_schema_versions WHERE tenant_id = $1 AND name = $2 ORDER BY version DESC LIMIT $3 OFFSET $4",
        )
        .bind(tenant_id)
        .bind(name)
        .bind(limit)
        .bind(offset)
        .fetch_all(&mut *tx)
        .await?;
        // カウントクエリにも tenant_id フィルタを適用する
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM apiregistry.api_schema_versions WHERE tenant_id = $1 AND name = $2",
        )
        .bind(tenant_id)
        .bind(name)
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok((rows.into_iter().map(Into::into).collect(), count.0 as u64))
    }

    // テナントスコープで set_config を設定した後にバージョンを作成する。
    async fn create(&self, tenant_id: &str, version: &ApiSchemaVersion) -> anyhow::Result<()> {
        let breaking_change_details_json = serde_json::to_value(&version.breaking_change_details)
            .unwrap_or(serde_json::Value::Array(vec![]));
        // トランザクション内で RLS 用セッション変数を設定する
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;
        // tenant_id カラムにも挿入する
        sqlx::query(
            "INSERT INTO apiregistry.api_schema_versions \
             (tenant_id, name, version, schema_type, content, content_hash, breaking_changes, breaking_change_details, registered_by, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(tenant_id)
        .bind(&version.name)
        .bind(version.version as i32)
        .bind(version.schema_type.to_string())
        .bind(&version.content)
        .bind(&version.content_hash)
        .bind(version.breaking_changes)
        .bind(&breaking_change_details_json)
        .bind(&version.registered_by)
        .bind(version.created_at)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }

    // テナントスコープで set_config を設定した後にバージョンを削除する。
    async fn delete(&self, tenant_id: &str, name: &str, version: u32) -> anyhow::Result<bool> {
        // トランザクション内で RLS 用セッション変数を設定する
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;
        // WHERE 句に tenant_id を追加して defense-in-depth を実現する
        let result = sqlx::query(
            "DELETE FROM apiregistry.api_schema_versions WHERE tenant_id = $1 AND name = $2 AND version = $3",
        )
        .bind(tenant_id)
        .bind(name)
        .bind(version as i32)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(result.rows_affected() > 0)
    }

    // テナントスコープで set_config を設定した後にスキーマ名のバージョン数を取得する。
    async fn count_by_name(&self, tenant_id: &str, name: &str) -> anyhow::Result<u64> {
        // トランザクション内で RLS 用セッション変数を設定する
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;
        // カウントクエリにも tenant_id フィルタを適用する
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM apiregistry.api_schema_versions WHERE tenant_id = $1 AND name = $2",
        )
        .bind(tenant_id)
        .bind(name)
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(count.0 as u64)
    }
}
