// ステータス定義 PostgreSQL リポジトリ実装。
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::status_definition::{
    CreateStatusDefinition, StatusDefinition, StatusDefinitionFilter, UpdateStatusDefinition,
};
use crate::domain::repository::status_definition_repository::StatusDefinitionRepository;

pub struct StatusDefinitionPostgresRepository {
    pool: PgPool,
}

impl StatusDefinitionPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct StatusDefinitionRow {
    id: Uuid,
    project_type_id: Uuid,
    code: String,
    display_name: String,
    description: Option<String>,
    color: Option<String>,
    allowed_transitions: Option<serde_json::Value>,
    is_initial: bool,
    is_terminal: bool,
    sort_order: i32,
    created_by: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<StatusDefinitionRow> for StatusDefinition {
    fn from(row: StatusDefinitionRow) -> Self {
        Self {
            id: row.id,
            project_type_id: row.project_type_id,
            code: row.code,
            display_name: row.display_name,
            description: row.description,
            color: row.color,
            // JSONB カラムから取得した serde_json::Value を Vec<StatusTransition> へ変換する
            // デシリアライズ失敗時は None（空）として扱い、呼び出し元でエラーを起こさない
            allowed_transitions: row.allowed_transitions
                .and_then(|v| serde_json::from_value(v).ok()),
            is_initial: row.is_initial,
            is_terminal: row.is_terminal,
            sort_order: row.sort_order,
            created_by: row.created_by,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[async_trait]
impl StatusDefinitionRepository for StatusDefinitionPostgresRepository {
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<StatusDefinition>> {
        let row = sqlx::query_as::<_, StatusDefinitionRow>(
            r#"
            SELECT id, project_type_id, code, display_name, description, color,
                   allowed_transitions, is_initial, is_terminal, sort_order,
                   created_by, created_at, updated_at
            FROM status_definitions
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(Into::into))
    }

    async fn find_all(&self, filter: &StatusDefinitionFilter) -> anyhow::Result<Vec<StatusDefinition>> {
        let limit = filter.limit.unwrap_or(50);
        let offset = filter.offset.unwrap_or(0);
        let rows = sqlx::query_as::<_, StatusDefinitionRow>(
            r#"
            SELECT id, project_type_id, code, display_name, description, color,
                   allowed_transitions, is_initial, is_terminal, sort_order,
                   created_by, created_at, updated_at
            FROM status_definitions
            WHERE ($1::uuid IS NULL OR project_type_id = $1)
            ORDER BY sort_order ASC, created_at ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(filter.project_type_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn count(&self, filter: &StatusDefinitionFilter) -> anyhow::Result<i64> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM status_definitions WHERE ($1::uuid IS NULL OR project_type_id = $1)",
        )
        .bind(filter.project_type_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(count)
    }

    async fn create(&self, input: &CreateStatusDefinition, created_by: &str) -> anyhow::Result<StatusDefinition> {
        let now = Utc::now();
        let id = Uuid::new_v4();
        // Vec<StatusTransition> を JSONB カラムへ bind できる serde_json::Value に変換する
        let transitions_json: Option<serde_json::Value> = input.allowed_transitions
            .as_ref()
            .map(serde_json::to_value)
            .transpose()
            .map_err(|e| anyhow::anyhow!("allowed_transitions シリアライズ失敗: {}", e))?;
        let row = sqlx::query_as::<_, StatusDefinitionRow>(
            r#"
            INSERT INTO status_definitions
                (id, project_type_id, code, display_name, description, color,
                 allowed_transitions, is_initial, is_terminal, sort_order,
                 created_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING id, project_type_id, code, display_name, description, color,
                      allowed_transitions, is_initial, is_terminal, sort_order,
                      created_by, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(input.project_type_id)
        .bind(&input.code)
        .bind(&input.display_name)
        .bind(&input.description)
        .bind(&input.color)
        .bind(transitions_json)
        .bind(input.is_initial.unwrap_or(false))
        .bind(input.is_terminal.unwrap_or(false))
        .bind(input.sort_order.unwrap_or(0))
        .bind(created_by)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn update(&self, id: Uuid, input: &UpdateStatusDefinition, _updated_by: &str) -> anyhow::Result<StatusDefinition> {
        let now = Utc::now();
        // Vec<StatusTransition> を JSONB カラムへ bind できる serde_json::Value に変換する
        let transitions_json: Option<serde_json::Value> = input.allowed_transitions
            .as_ref()
            .map(serde_json::to_value)
            .transpose()
            .map_err(|e| anyhow::anyhow!("allowed_transitions シリアライズ失敗: {}", e))?;
        let row = sqlx::query_as::<_, StatusDefinitionRow>(
            r#"
            UPDATE status_definitions SET
                display_name = COALESCE($2, display_name),
                description = COALESCE($3, description),
                color = COALESCE($4, color),
                allowed_transitions = COALESCE($5, allowed_transitions),
                is_initial = COALESCE($6, is_initial),
                is_terminal = COALESCE($7, is_terminal),
                sort_order = COALESCE($8, sort_order),
                updated_at = $9
            WHERE id = $1
            RETURNING id, project_type_id, code, display_name, description, color,
                      allowed_transitions, is_initial, is_terminal, sort_order,
                      created_by, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&input.display_name)
        .bind(&input.description)
        .bind(&input.color)
        .bind(transitions_json)
        .bind(input.is_initial)
        .bind(input.is_terminal)
        .bind(input.sort_order)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn delete(&self, id: Uuid) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM status_definitions WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
