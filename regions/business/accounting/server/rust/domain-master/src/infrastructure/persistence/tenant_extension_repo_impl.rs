use crate::domain::entity::tenant_master_extension::{
    TenantMasterExtension, UpsertTenantMasterExtension,
};
use crate::domain::repository::tenant_extension_repository::TenantExtensionRepository;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

/// PostgreSQL を使用したテナント拡張リポジトリの実装。
/// 各メソッドはジェネリック Executor を受け取り、トランザクション内でも使用可能。
pub struct TenantExtensionPostgresRepository {
    pool: PgPool,
}

impl TenantExtensionPostgresRepository {
    /// 新しいテナント拡張リポジトリを生成する。
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// テナントIDとアイテムIDで拡張を検索する。任意の sqlx Executor を受け取る。
    pub async fn find_by_tenant_and_item_with_executor<'e, E>(
        executor: E,
        tenant_id: &str,
        item_id: Uuid,
    ) -> anyhow::Result<Option<TenantMasterExtension>>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        // 明示的カラム指定によるクエリ安全性の確保
        let row = sqlx::query_as::<_, TenantExtensionRow>(
            "SELECT id, tenant_id, item_id, display_name_override, attributes_override, is_enabled, created_at, updated_at FROM domain_master.tenant_master_extensions WHERE tenant_id = $1 AND item_id = $2",
        )
        .bind(tenant_id)
        .bind(item_id)
        .fetch_optional(executor)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    /// テナントIDとカテゴリIDで拡張一覧を取得する。任意の sqlx Executor を受け取る。
    pub async fn find_by_tenant_and_category_with_executor<'e, E>(
        executor: E,
        tenant_id: &str,
        category_id: Uuid,
    ) -> anyhow::Result<Vec<TenantMasterExtension>>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let rows = sqlx::query_as::<_, TenantExtensionRow>(
            r#"SELECT te.* FROM domain_master.tenant_master_extensions te
               INNER JOIN domain_master.master_items mi ON te.item_id = mi.id
               WHERE te.tenant_id = $1 AND mi.category_id = $2"#,
        )
        .bind(tenant_id)
        .bind(category_id)
        .fetch_all(executor)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    /// テナント拡張を挿入または更新する。任意の sqlx Executor を受け取る。
    pub async fn upsert_with_executor<'e, E>(
        executor: E,
        tenant_id: &str,
        item_id: Uuid,
        input: &UpsertTenantMasterExtension,
    ) -> anyhow::Result<TenantMasterExtension>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let row = sqlx::query_as::<_, TenantExtensionRow>(
            r#"INSERT INTO domain_master.tenant_master_extensions
               (tenant_id, item_id, display_name_override, attributes_override, is_enabled)
               VALUES ($1, $2, $3, $4, $5)
               ON CONFLICT (tenant_id, item_id) DO UPDATE SET
               display_name_override = COALESCE($3, domain_master.tenant_master_extensions.display_name_override),
               attributes_override = COALESCE($4, domain_master.tenant_master_extensions.attributes_override),
               is_enabled = COALESCE($5, domain_master.tenant_master_extensions.is_enabled),
               updated_at = now()
               RETURNING *"#,
        )
        .bind(tenant_id)
        .bind(item_id)
        .bind(&input.display_name_override)
        .bind(&input.attributes_override)
        .bind(input.is_enabled.unwrap_or(true))
        .fetch_one(executor)
        .await?;
        Ok(row.into())
    }

    /// テナント拡張を削除する。任意の sqlx Executor を受け取る。
    pub async fn delete_with_executor<'e, E>(
        executor: E,
        tenant_id: &str,
        item_id: Uuid,
    ) -> anyhow::Result<()>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        sqlx::query(
            "DELETE FROM domain_master.tenant_master_extensions WHERE tenant_id = $1 AND item_id = $2",
        )
        .bind(tenant_id)
        .bind(item_id)
        .execute(executor)
        .await?;
        Ok(())
    }

    /// トランザクションを開始して返す。ユースケース層で複数リポジトリ操作を束ねるために使用する。
    pub async fn begin_tx(&self) -> anyhow::Result<sqlx::Transaction<'_, sqlx::Postgres>> {
        Ok(self.pool.begin().await?)
    }
}

#[async_trait]
impl TenantExtensionRepository for TenantExtensionPostgresRepository {
    /// トレイト経由のテナント・アイテム検索。内部で pool を Executor として使用する。
    async fn find_by_tenant_and_item(
        &self,
        tenant_id: &str,
        item_id: Uuid,
    ) -> anyhow::Result<Option<TenantMasterExtension>> {
        Self::find_by_tenant_and_item_with_executor(&self.pool, tenant_id, item_id).await
    }

    /// トレイト経由のテナント・カテゴリ検索。内部で pool を Executor として使用する。
    async fn find_by_tenant_and_category(
        &self,
        tenant_id: &str,
        category_id: Uuid,
    ) -> anyhow::Result<Vec<TenantMasterExtension>> {
        Self::find_by_tenant_and_category_with_executor(&self.pool, tenant_id, category_id).await
    }

    /// トレイト経由のupsert。内部で pool を Executor として使用する。
    async fn upsert(
        &self,
        tenant_id: &str,
        item_id: Uuid,
        input: &UpsertTenantMasterExtension,
    ) -> anyhow::Result<TenantMasterExtension> {
        Self::upsert_with_executor(&self.pool, tenant_id, item_id, input).await
    }

    /// トレイト経由の削除。内部で pool を Executor として使用する。
    async fn delete(&self, tenant_id: &str, item_id: Uuid) -> anyhow::Result<()> {
        Self::delete_with_executor(&self.pool, tenant_id, item_id).await
    }
}

#[derive(sqlx::FromRow)]
struct TenantExtensionRow {
    id: Uuid,
    tenant_id: String,
    item_id: Uuid,
    display_name_override: Option<String>,
    attributes_override: Option<serde_json::Value>,
    is_enabled: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<TenantExtensionRow> for TenantMasterExtension {
    fn from(row: TenantExtensionRow) -> Self {
        Self {
            id: row.id,
            tenant_id: row.tenant_id,
            item_id: row.item_id,
            display_name_override: row.display_name_override,
            attributes_override: row.attributes_override,
            is_enabled: row.is_enabled,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
