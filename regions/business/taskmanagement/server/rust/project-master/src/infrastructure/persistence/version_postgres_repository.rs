// ステータス定義バージョン PostgreSQL リポジトリ実装。
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::status_definition_version::StatusDefinitionVersion;
use crate::domain::repository::version_repository::VersionRepository;

pub struct VersionPostgresRepository {
    pool: PgPool,
}

impl VersionPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct VersionRow {
    id: Uuid,
    status_definition_id: Uuid,
    version_number: i32,
    before_data: Option<serde_json::Value>,
    after_data: Option<serde_json::Value>,
    changed_by: String,
    change_reason: Option<String>,
    created_at: DateTime<Utc>,
}

impl From<VersionRow> for StatusDefinitionVersion {
    fn from(row: VersionRow) -> Self {
        Self {
            id: row.id,
            status_definition_id: row.status_definition_id,
            version_number: row.version_number,
            before_data: row.before_data,
            after_data: row.after_data,
            changed_by: row.changed_by,
            change_reason: row.change_reason,
            created_at: row.created_at,
        }
    }
}

#[async_trait]
impl VersionRepository for VersionPostgresRepository {
    async fn find_by_status_definition(
        &self,
        status_definition_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<StatusDefinitionVersion>> {
        let rows = sqlx::query_as::<_, VersionRow>(
            r#"
            SELECT id, status_definition_id, version_number, before_data, after_data,
                   changed_by, change_reason, created_at
            FROM status_definition_versions
            WHERE status_definition_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(status_definition_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn count_by_status_definition(&self, status_definition_id: Uuid) -> anyhow::Result<i64> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM status_definition_versions WHERE status_definition_id = $1",
        )
        .bind(status_definition_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(count)
    }
}
