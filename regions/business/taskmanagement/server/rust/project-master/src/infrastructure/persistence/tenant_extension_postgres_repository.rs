// テナント拡張 PostgreSQL リポジトリ実装。
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::status_definition::StatusDefinition;
use crate::domain::entity::tenant_project_extension::{
    TenantMergedStatus, TenantProjectExtension, UpsertTenantExtension,
};
use crate::domain::repository::tenant_extension_repository::TenantExtensionRepository;

pub struct TenantExtensionPostgresRepository {
    pool: PgPool,
}

impl TenantExtensionPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct TenantExtensionRow {
    id: Uuid,
    tenant_id: String,
    status_definition_id: Uuid,
    display_name_override: Option<String>,
    attributes_override: Option<serde_json::Value>,
    is_enabled: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<TenantExtensionRow> for TenantProjectExtension {
    fn from(row: TenantExtensionRow) -> Self {
        Self {
            id: row.id,
            tenant_id: row.tenant_id,
            status_definition_id: row.status_definition_id,
            display_name_override: row.display_name_override,
            attributes_override: row.attributes_override,
            is_enabled: row.is_enabled,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[async_trait]
impl TenantExtensionRepository for TenantExtensionPostgresRepository {
    async fn find(&self, tenant_id: &str, status_definition_id: Uuid) -> anyhow::Result<Option<TenantProjectExtension>> {
        let row = sqlx::query_as::<_, TenantExtensionRow>(
            r#"
            SELECT id, tenant_id, status_definition_id, display_name_override,
                   attributes_override, is_enabled, created_at, updated_at
            FROM tenant_project_extensions
            WHERE tenant_id = $1 AND status_definition_id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(status_definition_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(Into::into))
    }

    async fn list_merged(
        &self,
        tenant_id: &str,
        project_type_id: Uuid,
        _active_only: bool,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<TenantMergedStatus>> {
        // 基本ステータス一覧を取得し、テナント拡張をマージする
        let rows = sqlx::query_as::<_, (Uuid, Uuid, String, String, Option<String>, Option<String>, Option<serde_json::Value>, bool, bool, i32, String, DateTime<Utc>, DateTime<Utc>, Option<String>, Option<serde_json::Value>, Option<bool>)>(
            r#"
            SELECT s.id, s.project_type_id, s.code, s.display_name, s.description,
                   s.color, s.allowed_transitions, s.is_initial, s.is_terminal, s.sort_order,
                   s.created_by, s.created_at, s.updated_at,
                   e.display_name_override, e.attributes_override, e.is_enabled
            FROM status_definitions s
            LEFT JOIN tenant_project_extensions e
                ON s.id = e.status_definition_id AND e.tenant_id = $2
            WHERE s.project_type_id = $1
            ORDER BY s.sort_order ASC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(project_type_id)
        .bind(tenant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let merged = rows.into_iter().map(|(id, pt_id, code, display_name, description, color, allowed_transitions, is_initial, is_terminal, sort_order, created_by, created_at, updated_at, display_name_override, attributes_override, is_enabled)| {
            let base = StatusDefinition {
                id,
                project_type_id: pt_id,
                code: code.clone(),
                display_name: display_name.clone(),
                description,
                color,
                allowed_transitions,
                is_initial,
                is_terminal,
                sort_order,
                created_by,
                created_at,
                updated_at,
            };
            let effective_display_name = display_name_override.clone().unwrap_or(display_name);
            TenantMergedStatus {
                base_status: base,
                extension: is_enabled.map(|_| TenantProjectExtension {
                    id: Uuid::nil(),
                    tenant_id: tenant_id.to_string(),
                    status_definition_id: id,
                    display_name_override,
                    attributes_override: attributes_override.clone(),
                    is_enabled: is_enabled.unwrap_or(true),
                    created_at,
                    updated_at,
                }),
                effective_display_name,
                effective_attributes: attributes_override,
            }
        }).collect();
        Ok(merged)
    }

    async fn count_merged(&self, _tenant_id: &str, project_type_id: Uuid) -> anyhow::Result<i64> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM status_definitions WHERE project_type_id = $1",
        )
        .bind(project_type_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(count)
    }

    async fn upsert(&self, input: &UpsertTenantExtension) -> anyhow::Result<TenantProjectExtension> {
        let now = Utc::now();
        let id = Uuid::new_v4();
        let row = sqlx::query_as::<_, TenantExtensionRow>(
            r#"
            INSERT INTO tenant_project_extensions
                (id, tenant_id, status_definition_id, display_name_override,
                 attributes_override, is_enabled, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (tenant_id, status_definition_id) DO UPDATE SET
                display_name_override = EXCLUDED.display_name_override,
                attributes_override = EXCLUDED.attributes_override,
                is_enabled = EXCLUDED.is_enabled,
                updated_at = EXCLUDED.updated_at
            RETURNING id, tenant_id, status_definition_id, display_name_override,
                      attributes_override, is_enabled, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&input.tenant_id)
        .bind(input.status_definition_id)
        .bind(&input.display_name_override)
        .bind(&input.attributes_override)
        .bind(input.is_enabled.unwrap_or(true))
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn delete(&self, tenant_id: &str, status_definition_id: Uuid) -> anyhow::Result<()> {
        sqlx::query(
            "DELETE FROM tenant_project_extensions WHERE tenant_id = $1 AND status_definition_id = $2",
        )
        .bind(tenant_id)
        .bind(status_definition_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
