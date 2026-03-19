use crate::domain::entity::master_item::{CreateMasterItem, MasterItem, UpdateMasterItem};
use crate::domain::repository::item_repository::ItemRepository;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

/// PostgreSQL を使用したアイテムリポジトリの実装。
/// 各メソッドはジェネリック Executor を受け取り、トランザクション内でも使用可能。
pub struct ItemPostgresRepository {
    pool: PgPool,
}

impl ItemPostgresRepository {
    /// 新しいアイテムリポジトリを生成する。
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// カテゴリIDに属するアイテムを取得する。任意の sqlx Executor を受け取る。
    pub async fn find_by_category_with_executor<'e, E>(
        executor: E,
        category_id: Uuid,
        active_only: bool,
    ) -> anyhow::Result<Vec<MasterItem>>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        // 明示的カラム指定によるクエリ安全性の確保
        let rows = if active_only {
            sqlx::query_as::<_, ItemRow>(
                "SELECT id, category_id, code, display_name, description, attributes, parent_item_id, effective_from, effective_until, is_active, sort_order, created_by, created_at, updated_at FROM domain_master.master_items WHERE category_id = $1 AND is_active = true ORDER BY sort_order, code",
            )
            .bind(category_id)
            .fetch_all(executor)
            .await?
        } else {
            sqlx::query_as::<_, ItemRow>(
                "SELECT id, category_id, code, display_name, description, attributes, parent_item_id, effective_from, effective_until, is_active, sort_order, created_by, created_at, updated_at FROM domain_master.master_items WHERE category_id = $1 ORDER BY sort_order, code",
            )
            .bind(category_id)
            .fetch_all(executor)
            .await?
        };
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    /// カテゴリIDとコードでアイテムを検索する。任意の sqlx Executor を受け取る。
    pub async fn find_by_category_and_code_with_executor<'e, E>(
        executor: E,
        category_id: Uuid,
        code: &str,
    ) -> anyhow::Result<Option<MasterItem>>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        // 明示的カラム指定によるクエリ安全性の確保
        let row = sqlx::query_as::<_, ItemRow>(
            "SELECT id, category_id, code, display_name, description, attributes, parent_item_id, effective_from, effective_until, is_active, sort_order, created_by, created_at, updated_at FROM domain_master.master_items WHERE category_id = $1 AND code = $2",
        )
        .bind(category_id)
        .bind(code)
        .fetch_optional(executor)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    /// IDでアイテムを検索する。任意の sqlx Executor を受け取る。
    pub async fn find_by_id_with_executor<'e, E>(
        executor: E,
        id: Uuid,
    ) -> anyhow::Result<Option<MasterItem>>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        // 明示的カラム指定によるクエリ安全性の確保
        let row =
            sqlx::query_as::<_, ItemRow>("SELECT id, category_id, code, display_name, description, attributes, parent_item_id, effective_from, effective_until, is_active, sort_order, created_by, created_at, updated_at FROM domain_master.master_items WHERE id = $1")
                .bind(id)
                .fetch_optional(executor)
                .await?;
        Ok(row.map(|r| r.into()))
    }

    /// アイテムを新規作成する。任意の sqlx Executor を受け取る。
    pub async fn create_with_executor<'e, E>(
        executor: E,
        category_id: Uuid,
        input: &CreateMasterItem,
        created_by: &str,
    ) -> anyhow::Result<MasterItem>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let row = sqlx::query_as::<_, ItemRow>(
            r#"INSERT INTO domain_master.master_items
               (category_id, code, display_name, description, attributes,
                parent_item_id, effective_from, effective_until, is_active, sort_order, created_by)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
               RETURNING *"#,
        )
        .bind(category_id)
        .bind(&input.code)
        .bind(&input.display_name)
        .bind(&input.description)
        .bind(&input.attributes)
        .bind(input.parent_item_id)
        .bind(input.effective_from)
        .bind(input.effective_until)
        .bind(input.is_active.unwrap_or(true))
        .bind(input.sort_order.unwrap_or(0))
        .bind(created_by)
        .fetch_one(executor)
        .await?;
        Ok(row.into())
    }

    /// アイテムを更新する。任意の sqlx Executor を受け取る。
    pub async fn update_with_executor<'e, E>(
        executor: E,
        id: Uuid,
        input: &UpdateMasterItem,
    ) -> anyhow::Result<MasterItem>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let row = sqlx::query_as::<_, ItemRow>(
            r#"UPDATE domain_master.master_items SET
               display_name = COALESCE($2, display_name),
               description = COALESCE($3, description),
               attributes = COALESCE($4, attributes),
               parent_item_id = COALESCE($5, parent_item_id),
               effective_from = COALESCE($6, effective_from),
               effective_until = COALESCE($7, effective_until),
               is_active = COALESCE($8, is_active),
               sort_order = COALESCE($9, sort_order),
               updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id)
        .bind(&input.display_name)
        .bind(&input.description)
        .bind(&input.attributes)
        .bind(input.parent_item_id)
        .bind(input.effective_from)
        .bind(input.effective_until)
        .bind(input.is_active)
        .bind(input.sort_order)
        .fetch_one(executor)
        .await?;
        Ok(row.into())
    }

    /// アイテムを削除する。任意の sqlx Executor を受け取る。
    pub async fn delete_with_executor<'e, E>(executor: E, id: Uuid) -> anyhow::Result<()>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        sqlx::query("DELETE FROM domain_master.master_items WHERE id = $1")
            .bind(id)
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
impl ItemRepository for ItemPostgresRepository {
    /// トレイト経由のカテゴリ別取得。内部で pool を Executor として使用する。
    async fn find_by_category(
        &self,
        category_id: Uuid,
        active_only: bool,
    ) -> anyhow::Result<Vec<MasterItem>> {
        Self::find_by_category_with_executor(&self.pool, category_id, active_only).await
    }

    /// トレイト経由のカテゴリ・コード検索。内部で pool を Executor として使用する。
    async fn find_by_category_and_code(
        &self,
        category_id: Uuid,
        code: &str,
    ) -> anyhow::Result<Option<MasterItem>> {
        Self::find_by_category_and_code_with_executor(&self.pool, category_id, code).await
    }

    /// トレイト経由のID検索。内部で pool を Executor として使用する。
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<MasterItem>> {
        Self::find_by_id_with_executor(&self.pool, id).await
    }

    /// トレイト経由の作成。内部で pool を Executor として使用する。
    async fn create(
        &self,
        category_id: Uuid,
        input: &CreateMasterItem,
        created_by: &str,
    ) -> anyhow::Result<MasterItem> {
        Self::create_with_executor(&self.pool, category_id, input, created_by).await
    }

    /// トレイト経由の更新。内部で pool を Executor として使用する。
    async fn update(&self, id: Uuid, input: &UpdateMasterItem) -> anyhow::Result<MasterItem> {
        Self::update_with_executor(&self.pool, id, input).await
    }

    /// トレイト経由の削除。内部で pool を Executor として使用する。
    async fn delete(&self, id: Uuid) -> anyhow::Result<()> {
        Self::delete_with_executor(&self.pool, id).await
    }
}

#[derive(sqlx::FromRow)]
struct ItemRow {
    id: Uuid,
    category_id: Uuid,
    code: String,
    display_name: String,
    description: Option<String>,
    attributes: Option<serde_json::Value>,
    parent_item_id: Option<Uuid>,
    effective_from: Option<chrono::DateTime<chrono::Utc>>,
    effective_until: Option<chrono::DateTime<chrono::Utc>>,
    is_active: bool,
    sort_order: i32,
    created_by: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<ItemRow> for MasterItem {
    fn from(row: ItemRow) -> Self {
        Self {
            id: row.id,
            category_id: row.category_id,
            code: row.code,
            display_name: row.display_name,
            description: row.description,
            attributes: row.attributes,
            parent_item_id: row.parent_item_id,
            effective_from: row.effective_from,
            effective_until: row.effective_until,
            is_active: row.is_active,
            sort_order: row.sort_order,
            created_by: row.created_by,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
