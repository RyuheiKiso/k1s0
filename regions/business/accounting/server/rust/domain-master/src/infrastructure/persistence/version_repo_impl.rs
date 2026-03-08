use crate::domain::entity::master_item_version::MasterItemVersion;
use crate::domain::repository::version_repository::VersionRepository;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

pub struct VersionPostgresRepository {
    pool: PgPool,
}

impl VersionPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VersionRepository for VersionPostgresRepository {
    async fn find_by_item(&self, item_id: Uuid) -> anyhow::Result<Vec<MasterItemVersion>> {
        let rows = sqlx::query_as::<_, VersionRow>(
            "SELECT * FROM domain_master.master_item_versions WHERE item_id = $1 ORDER BY version_number DESC",
        )
        .bind(item_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn get_latest_version_number(&self, item_id: Uuid) -> anyhow::Result<i32> {
        let row: Option<(i32,)> = sqlx::query_as(
            "SELECT COALESCE(MAX(version_number), 0) FROM domain_master.master_item_versions WHERE item_id = $1",
        )
        .bind(item_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.0).unwrap_or(0))
    }

    async fn create<'a>(
        &self,
        item_id: Uuid,
        version_number: i32,
        before_data: Option<serde_json::Value>,
        after_data: Option<serde_json::Value>,
        changed_by: &'a str,
        change_reason: Option<&'a str>,
    ) -> anyhow::Result<MasterItemVersion> {
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
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
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
