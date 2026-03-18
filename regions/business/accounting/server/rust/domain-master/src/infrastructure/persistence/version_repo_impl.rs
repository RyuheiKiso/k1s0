use crate::domain::entity::master_item_version::MasterItemVersion;
use crate::domain::repository::version_repository::VersionRepository;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

/// PostgreSQL を使用したバージョンリポジトリの実装。
/// 各メソッドはジェネリック Executor を受け取り、トランザクション内でも使用可能。
pub struct VersionPostgresRepository {
    pool: PgPool,
}

impl VersionPostgresRepository {
    /// 新しいバージョンリポジトリを生成する。
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// アイテムIDに紐づくバージョン履歴を取得する。任意の sqlx Executor を受け取る。
    pub async fn find_by_item_with_executor<'e, E>(
        executor: E,
        item_id: Uuid,
    ) -> anyhow::Result<Vec<MasterItemVersion>>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let rows = sqlx::query_as::<_, VersionRow>(
            "SELECT * FROM domain_master.master_item_versions WHERE item_id = $1 ORDER BY version_number DESC",
        )
        .bind(item_id)
        .fetch_all(executor)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    /// アイテムの最新バージョン番号を取得する。任意の sqlx Executor を受け取る。
    pub async fn get_latest_version_number_with_executor<'e, E>(
        executor: E,
        item_id: Uuid,
    ) -> anyhow::Result<i32>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let row: Option<(i32,)> = sqlx::query_as(
            "SELECT COALESCE(MAX(version_number), 0) FROM domain_master.master_item_versions WHERE item_id = $1",
        )
        .bind(item_id)
        .fetch_optional(executor)
        .await?;
        Ok(row.map(|r| r.0).unwrap_or(0))
    }

    /// 新しいバージョンレコードを作成する。任意の sqlx Executor を受け取る。
    pub async fn create_with_executor<'e, E>(
        executor: E,
        item_id: Uuid,
        version_number: i32,
        before_data: Option<serde_json::Value>,
        after_data: Option<serde_json::Value>,
        changed_by: &str,
        change_reason: Option<&str>,
    ) -> anyhow::Result<MasterItemVersion>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let row = sqlx::query_as::<_, VersionRow>(
            r#"INSERT INTO domain_master.master_item_versions
               (item_id, version_number, before_data, after_data, changed_by, change_reason)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING *"#,
        )
        .bind(item_id)
        .bind(version_number)
        .bind(before_data)
        .bind(after_data)
        .bind(changed_by)
        .bind(change_reason)
        .fetch_one(executor)
        .await?;
        Ok(row.into())
    }

    /// トランザクションを開始して返す。ユースケース層で複数リポジトリ操作を束ねるために使用する。
    pub async fn begin_tx(&self) -> anyhow::Result<sqlx::Transaction<'_, sqlx::Postgres>> {
        Ok(self.pool.begin().await?)
    }
}

#[async_trait]
impl VersionRepository for VersionPostgresRepository {
    /// トレイト経由のアイテム別取得。内部で pool を Executor として使用する。
    async fn find_by_item(&self, item_id: Uuid) -> anyhow::Result<Vec<MasterItemVersion>> {
        Self::find_by_item_with_executor(&self.pool, item_id).await
    }

    /// トレイト経由の最新バージョン番号取得。内部で pool を Executor として使用する。
    async fn get_latest_version_number(&self, item_id: Uuid) -> anyhow::Result<i32> {
        Self::get_latest_version_number_with_executor(&self.pool, item_id).await
    }

    /// トレイト経由のバージョン作成。内部で pool を Executor として使用する。
    async fn create<'a>(
        &self,
        item_id: Uuid,
        version_number: i32,
        before_data: Option<serde_json::Value>,
        after_data: Option<serde_json::Value>,
        changed_by: &'a str,
        change_reason: Option<&'a str>,
    ) -> anyhow::Result<MasterItemVersion> {
        Self::create_with_executor(
            &self.pool,
            item_id,
            version_number,
            before_data,
            after_data,
            changed_by,
            change_reason,
        )
        .await
    }
}

#[derive(sqlx::FromRow)]
struct VersionRow {
    id: Uuid,
    item_id: Uuid,
    version_number: i32,
    before_data: Option<serde_json::Value>,
    after_data: Option<serde_json::Value>,
    changed_by: String,
    change_reason: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<VersionRow> for MasterItemVersion {
    fn from(row: VersionRow) -> Self {
        Self {
            id: row.id,
            item_id: row.item_id,
            version_number: row.version_number,
            before_data: row.before_data,
            after_data: row.after_data,
            changed_by: row.changed_by,
            change_reason: row.change_reason,
            created_at: row.created_at,
        }
    }
}
