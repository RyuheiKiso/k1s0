use crate::domain::entity::master_category::{
    CreateMasterCategory, MasterCategory, UpdateMasterCategory,
};
use crate::domain::repository::category_repository::CategoryRepository;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

/// PostgreSQL を使用したカテゴリリポジトリの実装。
/// 各メソッドはジェネリック Executor を受け取り、トランザクション内でも使用可能。
pub struct CategoryPostgresRepository {
    pool: PgPool,
}

impl CategoryPostgresRepository {
    /// 新しいカテゴリリポジトリを生成する。
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 全カテゴリを取得する。active_only が true の場合はアクティブなもののみ返す。
    /// 任意の sqlx Executor を受け取るため、PgPool やトランザクション内から呼び出せる。
    pub async fn find_all_with_executor<'e, E>(
        executor: E,
        active_only: bool,
    ) -> anyhow::Result<Vec<MasterCategory>>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let rows = if active_only {
            sqlx::query_as::<_, CategoryRow>(
                "SELECT * FROM domain_master.master_categories WHERE is_active = true ORDER BY sort_order, code",
            )
            .fetch_all(executor)
            .await?
        } else {
            sqlx::query_as::<_, CategoryRow>(
                "SELECT * FROM domain_master.master_categories ORDER BY sort_order, code",
            )
            .fetch_all(executor)
            .await?
        };
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    /// コードでカテゴリを検索する。任意の sqlx Executor を受け取る。
    pub async fn find_by_code_with_executor<'e, E>(
        executor: E,
        code: &str,
    ) -> anyhow::Result<Option<MasterCategory>>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let row = sqlx::query_as::<_, CategoryRow>(
            "SELECT * FROM domain_master.master_categories WHERE code = $1",
        )
        .bind(code)
        .fetch_optional(executor)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    /// IDでカテゴリを検索する。任意の sqlx Executor を受け取る。
    pub async fn find_by_id_with_executor<'e, E>(
        executor: E,
        id: Uuid,
    ) -> anyhow::Result<Option<MasterCategory>>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let row = sqlx::query_as::<_, CategoryRow>(
            "SELECT * FROM domain_master.master_categories WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(executor)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    /// カテゴリを新規作成する。任意の sqlx Executor を受け取る。
    pub async fn create_with_executor<'e, E>(
        executor: E,
        input: &CreateMasterCategory,
        created_by: &str,
    ) -> anyhow::Result<MasterCategory>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let row = sqlx::query_as::<_, CategoryRow>(
            r#"INSERT INTO domain_master.master_categories
               (code, display_name, description, validation_schema, is_active, sort_order, created_by)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               RETURNING *"#,
        )
        .bind(&input.code)
        .bind(&input.display_name)
        .bind(&input.description)
        .bind(&input.validation_schema)
        .bind(input.is_active.unwrap_or(true))
        .bind(input.sort_order.unwrap_or(0))
        .bind(created_by)
        .fetch_one(executor)
        .await?;
        Ok(row.into())
    }

    /// カテゴリを更新する。任意の sqlx Executor を受け取る。
    pub async fn update_with_executor<'e, E>(
        executor: E,
        code: &str,
        input: &UpdateMasterCategory,
    ) -> anyhow::Result<MasterCategory>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let row = sqlx::query_as::<_, CategoryRow>(
            r#"UPDATE domain_master.master_categories SET
               display_name = COALESCE($2, display_name),
               description = COALESCE($3, description),
               validation_schema = COALESCE($4, validation_schema),
               is_active = COALESCE($5, is_active),
               sort_order = COALESCE($6, sort_order),
               updated_at = now()
               WHERE code = $1 RETURNING *"#,
        )
        .bind(code)
        .bind(&input.display_name)
        .bind(&input.description)
        .bind(&input.validation_schema)
        .bind(input.is_active)
        .bind(input.sort_order)
        .fetch_one(executor)
        .await?;
        Ok(row.into())
    }

    /// カテゴリを削除する。任意の sqlx Executor を受け取る。
    pub async fn delete_with_executor<'e, E>(
        executor: E,
        code: &str,
    ) -> anyhow::Result<()>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        sqlx::query("DELETE FROM domain_master.master_categories WHERE code = $1")
            .bind(code)
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
impl CategoryRepository for CategoryPostgresRepository {
    /// トレイト経由の全件取得。内部で pool を Executor として使用する。
    async fn find_all(&self, active_only: bool) -> anyhow::Result<Vec<MasterCategory>> {
        Self::find_all_with_executor(&self.pool, active_only).await
    }

    /// トレイト経由のコード検索。内部で pool を Executor として使用する。
    async fn find_by_code(&self, code: &str) -> anyhow::Result<Option<MasterCategory>> {
        Self::find_by_code_with_executor(&self.pool, code).await
    }

    /// トレイト経由のID検索。内部で pool を Executor として使用する。
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<MasterCategory>> {
        Self::find_by_id_with_executor(&self.pool, id).await
    }

    /// トレイト経由の作成。内部で pool を Executor として使用する。
    async fn create(
        &self,
        input: &CreateMasterCategory,
        created_by: &str,
    ) -> anyhow::Result<MasterCategory> {
        Self::create_with_executor(&self.pool, input, created_by).await
    }

    /// トレイト経由の更新。内部で pool を Executor として使用する。
    async fn update(
        &self,
        code: &str,
        input: &UpdateMasterCategory,
    ) -> anyhow::Result<MasterCategory> {
        Self::update_with_executor(&self.pool, code, input).await
    }

    /// トレイト経由の削除。内部で pool を Executor として使用する。
    async fn delete(&self, code: &str) -> anyhow::Result<()> {
        Self::delete_with_executor(&self.pool, code).await
    }
}

#[derive(sqlx::FromRow)]
struct CategoryRow {
    id: Uuid,
    code: String,
    display_name: String,
    description: Option<String>,
    validation_schema: Option<serde_json::Value>,
    is_active: bool,
    sort_order: i32,
    created_by: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<CategoryRow> for MasterCategory {
    fn from(row: CategoryRow) -> Self {
        Self {
            id: row.id,
            code: row.code,
            display_name: row.display_name,
            description: row.description,
            validation_schema: row.validation_schema,
            is_active: row.is_active,
            sort_order: row.sort_order,
            created_by: row.created_by,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
