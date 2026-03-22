// プロジェクトタイプ PostgreSQL リポジトリ実装。
// sqlx を使用して project_master.project_types テーブルにアクセスする。
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::project_type::{
    CreateProjectType, ProjectType, ProjectTypeFilter, UpdateProjectType,
};
use crate::domain::repository::project_type_repository::ProjectTypeRepository;

pub struct ProjectTypePostgresRepository {
    pool: PgPool,
}

impl ProjectTypePostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// プロジェクトタイプの DB 行型
#[derive(sqlx::FromRow)]
struct ProjectTypeRow {
    id: Uuid,
    code: String,
    display_name: String,
    description: Option<String>,
    default_workflow: Option<serde_json::Value>,
    is_active: bool,
    sort_order: i32,
    created_by: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<ProjectTypeRow> for ProjectType {
    fn from(row: ProjectTypeRow) -> Self {
        Self {
            id: row.id,
            code: row.code,
            display_name: row.display_name,
            description: row.description,
            default_workflow: row.default_workflow,
            is_active: row.is_active,
            sort_order: row.sort_order,
            created_by: row.created_by,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[async_trait]
impl ProjectTypeRepository for ProjectTypePostgresRepository {
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<ProjectType>> {
        let row = sqlx::query_as::<_, ProjectTypeRow>(
            r#"
            SELECT id, code, display_name, description, default_workflow,
                   is_active, sort_order, created_by, created_at, updated_at
            FROM project_types
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(Into::into))
    }

    async fn find_by_code(&self, code: &str) -> anyhow::Result<Option<ProjectType>> {
        let row = sqlx::query_as::<_, ProjectTypeRow>(
            r#"
            SELECT id, code, display_name, description, default_workflow,
                   is_active, sort_order, created_by, created_at, updated_at
            FROM project_types
            WHERE code = $1
            "#,
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(Into::into))
    }

    async fn find_all(&self, filter: &ProjectTypeFilter) -> anyhow::Result<Vec<ProjectType>> {
        let limit = filter.limit.unwrap_or(50);
        let offset = filter.offset.unwrap_or(0);
        let rows = sqlx::query_as::<_, ProjectTypeRow>(
            r#"
            SELECT id, code, display_name, description, default_workflow,
                   is_active, sort_order, created_by, created_at, updated_at
            FROM project_types
            WHERE ($1 = FALSE OR is_active = TRUE)
            ORDER BY sort_order ASC, created_at ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(filter.active_only)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn count(&self, filter: &ProjectTypeFilter) -> anyhow::Result<i64> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM project_types WHERE ($1 = FALSE OR is_active = TRUE)",
        )
        .bind(filter.active_only)
        .fetch_one(&self.pool)
        .await?;
        Ok(count)
    }

    async fn create(
        &self,
        input: &CreateProjectType,
        created_by: &str,
    ) -> anyhow::Result<ProjectType> {
        let now = Utc::now();
        let id = Uuid::new_v4();
        let row = sqlx::query_as::<_, ProjectTypeRow>(
            r#"
            INSERT INTO project_types
                (id, code, display_name, description, default_workflow,
                 is_active, sort_order, created_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, code, display_name, description, default_workflow,
                      is_active, sort_order, created_by, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&input.code)
        .bind(&input.display_name)
        .bind(&input.description)
        .bind(&input.default_workflow)
        .bind(input.is_active.unwrap_or(true))
        .bind(input.sort_order.unwrap_or(0))
        .bind(created_by)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn update(
        &self,
        id: Uuid,
        input: &UpdateProjectType,
        _updated_by: &str,
    ) -> anyhow::Result<ProjectType> {
        let now = Utc::now();
        let row = sqlx::query_as::<_, ProjectTypeRow>(
            r#"
            UPDATE project_types SET
                display_name = COALESCE($2, display_name),
                description = COALESCE($3, description),
                default_workflow = COALESCE($4, default_workflow),
                is_active = COALESCE($5, is_active),
                sort_order = COALESCE($6, sort_order),
                updated_at = $7
            WHERE id = $1
            RETURNING id, code, display_name, description, default_workflow,
                      is_active, sort_order, created_by, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&input.display_name)
        .bind(&input.description)
        .bind(&input.default_workflow)
        .bind(input.is_active)
        .bind(input.sort_order)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn delete(&self, id: Uuid) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM project_types WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
