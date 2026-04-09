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
    #[must_use]
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
            // LOW-008: 安全な型変換（バージョン番号は非負であることが前提）
            latest_version: u32::try_from(r.latest_version).unwrap_or(0),
            // LOW-008: 安全な型変換（バージョン数は非負であることが前提）
            version_count: u32::try_from(r.version_count).unwrap_or(0),
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[async_trait]
impl ApiSchemaRepository for SchemaPostgresRepository {
    // テナントスコープで set_config を設定した後にスキーマ名で検索する。
    // defense-in-depth として WHERE 句にも tenant_id 条件を追加する。
    async fn find_by_name(&self, tenant_id: &str, name: &str) -> anyhow::Result<Option<ApiSchema>> {
        // トランザクション内で RLS 用セッション変数を設定する
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;
        let row: Option<ApiSchemaRow> = sqlx::query_as(
            "SELECT name, description, schema_type, latest_version, version_count, created_at, updated_at \
             FROM apiregistry.api_schemas WHERE tenant_id = $1 AND name = $2",
        )
        .bind(tenant_id)
        .bind(name)
        .fetch_optional(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(row.map(Into::into))
    }

    // テナントスコープで set_config を設定した後にスキーマ一覧を取得する。
    async fn find_all(
        &self,
        tenant_id: &str,
        schema_type: Option<String>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<ApiSchema>, u64)> {
        let offset = i64::from(page.saturating_sub(1) * page_size);
        let limit = i64::from(page_size);

        // トランザクション内で RLS 用セッション変数を設定する
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        // defense-in-depth として WHERE 句にも tenant_id を追加する
        let rows: Vec<ApiSchemaRow> = if let Some(ref st) = schema_type {
            sqlx::query_as(
                "SELECT name, description, schema_type, latest_version, version_count, created_at, updated_at \
                 FROM apiregistry.api_schemas WHERE tenant_id = $1 AND schema_type = $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4",
            )
            .bind(tenant_id)
            .bind(st)
            .bind(limit)
            .bind(offset)
            .fetch_all(&mut *tx)
            .await?
        } else {
            sqlx::query_as(
                "SELECT name, description, schema_type, latest_version, version_count, created_at, updated_at \
                 FROM apiregistry.api_schemas WHERE tenant_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
            )
            .bind(tenant_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&mut *tx)
            .await?
        };

        // カウントクエリにも tenant_id フィルタを適用する
        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM apiregistry.api_schemas WHERE tenant_id = $1")
                .bind(tenant_id)
                .fetch_one(&mut *tx)
                .await?;

        tx.commit().await?;
        // LOW-008: 安全な型変換（COUNT(*) は非負であることが前提）
        Ok((rows.into_iter().map(Into::into).collect(), u64::try_from(count.0).unwrap_or(0)))
    }

    // テナントスコープで set_config を設定した後にスキーマを作成する。
    async fn create(&self, tenant_id: &str, schema: &ApiSchema) -> anyhow::Result<()> {
        // トランザクション内で RLS 用セッション変数を設定する
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;
        // tenant_id カラムにも挿入する
        sqlx::query(
            "INSERT INTO apiregistry.api_schemas \
             (tenant_id, name, description, schema_type, latest_version, version_count, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(tenant_id)
        .bind(&schema.name)
        .bind(&schema.description)
        .bind(schema.schema_type.to_string())
        // LOW-008: 安全な型変換（バージョン番号は i32 範囲内が前提）
        .bind(i32::try_from(schema.latest_version).unwrap_or(i32::MAX))
        // LOW-008: 安全な型変換（バージョン数は i32 範囲内が前提）
        .bind(i32::try_from(schema.version_count).unwrap_or(i32::MAX))
        .bind(schema.created_at)
        .bind(schema.updated_at)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }

    // テナントスコープで set_config を設定した後にスキーマを更新する。
    async fn update(&self, tenant_id: &str, schema: &ApiSchema) -> anyhow::Result<()> {
        // トランザクション内で RLS 用セッション変数を設定する
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;
        // WHERE 句に tenant_id を追加して defense-in-depth を実現する
        sqlx::query(
            "UPDATE apiregistry.api_schemas \
             SET description = $3, latest_version = $4, version_count = $5, updated_at = $6 \
             WHERE tenant_id = $1 AND name = $2",
        )
        .bind(tenant_id)
        .bind(&schema.name)
        .bind(&schema.description)
        // LOW-008: 安全な型変換（バージョン番号は i32 範囲内が前提）
        .bind(i32::try_from(schema.latest_version).unwrap_or(i32::MAX))
        // LOW-008: 安全な型変換（バージョン数は i32 範囲内が前提）
        .bind(i32::try_from(schema.version_count).unwrap_or(i32::MAX))
        .bind(schema.updated_at)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }
}
