use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::domain::entity::config_change_log::ConfigChangeLog;
use crate::domain::entity::config_entry::{
    ConfigEntry, ConfigListResult, Pagination, ServiceConfigEntry, ServiceConfigResult,
};
use crate::domain::repository::ConfigRepository;

/// ConfigPostgresRepository は ConfigRepository の PostgreSQL 実装。
pub struct ConfigPostgresRepository {
    pool: PgPool,
}

impl ConfigPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// テストやマイグレーション用にプールへの参照を返す。
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

/// PostgreSQL の行から ConfigEntry を構築するヘルパー。
fn row_to_config_entry(row: sqlx::postgres::PgRow) -> Result<ConfigEntry, sqlx::Error> {
    Ok(ConfigEntry {
        id: row.try_get("id")?,
        namespace: row.try_get("namespace")?,
        key: row.try_get("key")?,
        value_json: row.try_get("value_json")?,
        version: row.try_get("version")?,
        description: row.try_get("description")?,
        created_by: row.try_get("created_by")?,
        updated_by: row.try_get("updated_by")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

/// PostgreSQL の行から ServiceConfigEntry を構築するヘルパー。
fn row_to_service_config_entry(
    row: sqlx::postgres::PgRow,
) -> Result<ServiceConfigEntry, sqlx::Error> {
    Ok(ServiceConfigEntry {
        namespace: row.try_get("namespace")?,
        key: row.try_get("key")?,
        value: row.try_get("value_json")?,
    })
}

#[async_trait]
impl ConfigRepository for ConfigPostgresRepository {
    async fn find_by_namespace_and_key(
        &self,
        namespace: &str,
        key: &str,
    ) -> anyhow::Result<Option<ConfigEntry>> {
        let row = sqlx::query(
            r#"
            SELECT id, namespace, key, value_json, version, description,
                   created_by, updated_by, created_at, updated_at
            FROM config_entries
            WHERE namespace = $1 AND key = $2
            "#,
        )
        .bind(namespace)
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(row_to_config_entry(row)?)),
            None => Ok(None),
        }
    }

    async fn list_by_namespace(
        &self,
        namespace: &str,
        page: i32,
        page_size: i32,
        search: Option<String>,
    ) -> anyhow::Result<ConfigListResult> {
        let offset = (page - 1) * page_size;

        let (entries, total_count) = if let Some(ref search_term) = search {
            let pattern = format!("%{}%", search_term);
            let rows = sqlx::query(
                r#"
                SELECT id, namespace, key, value_json, version, description,
                       created_by, updated_by, created_at, updated_at
                FROM config_entries
                WHERE namespace = $1 AND key LIKE $2
                ORDER BY key ASC
                LIMIT $3 OFFSET $4
                "#,
            )
            .bind(namespace)
            .bind(&pattern)
            .bind(page_size as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await?;

            let entries: Vec<ConfigEntry> = rows
                .into_iter()
                .map(row_to_config_entry)
                .collect::<Result<Vec<_>, _>>()?;

            let count_row: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM config_entries WHERE namespace = $1 AND key LIKE $2",
            )
            .bind(namespace)
            .bind(&pattern)
            .fetch_one(&self.pool)
            .await?;

            (entries, count_row.0)
        } else {
            let rows = sqlx::query(
                r#"
                SELECT id, namespace, key, value_json, version, description,
                       created_by, updated_by, created_at, updated_at
                FROM config_entries
                WHERE namespace = $1
                ORDER BY key ASC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(namespace)
            .bind(page_size as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await?;

            let entries: Vec<ConfigEntry> = rows
                .into_iter()
                .map(row_to_config_entry)
                .collect::<Result<Vec<_>, _>>()?;

            let count_row: (i64,) =
                sqlx::query_as("SELECT COUNT(*) FROM config_entries WHERE namespace = $1")
                    .bind(namespace)
                    .fetch_one(&self.pool)
                    .await?;

            (entries, count_row.0)
        };

        let has_next = (offset + page_size) < total_count as i32;

        Ok(ConfigListResult {
            entries,
            pagination: Pagination {
                total_count,
                page,
                page_size,
                has_next,
            },
        })
    }

    async fn create(&self, entry: &ConfigEntry) -> anyhow::Result<ConfigEntry> {
        let row = sqlx::query(
            r#"
            INSERT INTO config_entries (id, namespace, key, value_json, version, description, created_by, updated_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, namespace, key, value_json, version, description, created_by, updated_by, created_at, updated_at
            "#,
        )
        .bind(entry.id)
        .bind(&entry.namespace)
        .bind(&entry.key)
        .bind(&entry.value_json)
        .bind(entry.version)
        .bind(&entry.description)
        .bind(&entry.created_by)
        .bind(&entry.updated_by)
        .bind(entry.created_at)
        .bind(entry.updated_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(row_to_config_entry(row)?)
    }

    async fn update(
        &self,
        namespace: &str,
        key: &str,
        value_json: &serde_json::Value,
        expected_version: i32,
        description: Option<String>,
        updated_by: &str,
    ) -> anyhow::Result<ConfigEntry> {
        let now = chrono::Utc::now();
        let new_version = expected_version + 1;

        let row = sqlx::query(
            r#"
            UPDATE config_entries
            SET value_json = $1,
                version = $2,
                description = COALESCE($3, description),
                updated_by = $4,
                updated_at = $5
            WHERE namespace = $6 AND key = $7 AND version = $8
            RETURNING id, namespace, key, value_json, version, description, created_by, updated_by, created_at, updated_at
            "#,
        )
        .bind(value_json)
        .bind(new_version)
        .bind(&description)
        .bind(updated_by)
        .bind(now)
        .bind(namespace)
        .bind(key)
        .bind(expected_version)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(row_to_config_entry(row)?),
            None => {
                // バージョン不一致か、キーが存在しないかを確認
                let exists: Option<(i32,)> = sqlx::query_as(
                    "SELECT version FROM config_entries WHERE namespace = $1 AND key = $2",
                )
                .bind(namespace)
                .bind(key)
                .fetch_optional(&self.pool)
                .await?;

                match exists {
                    Some((current_version,)) => {
                        Err(anyhow::anyhow!(
                            "version conflict: current={}",
                            current_version
                        ))
                    }
                    None => Err(anyhow::anyhow!("config not found: {}/{}", namespace, key)),
                }
            }
        }
    }

    async fn delete(&self, namespace: &str, key: &str) -> anyhow::Result<bool> {
        let result = sqlx::query(
            "DELETE FROM config_entries WHERE namespace = $1 AND key = $2",
        )
        .bind(namespace)
        .bind(key)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn find_by_service_name(
        &self,
        service_name: &str,
    ) -> anyhow::Result<ServiceConfigResult> {
        // サービス名に紐づく namespace パターンで検索
        // 例: "auth-server" → "system.auth.%" のような namespace にマッチ
        let pattern = format!("%.{}%", service_name.replace('-', "."));

        let rows = sqlx::query(
            r#"
            SELECT namespace, key, value_json
            FROM config_entries
            WHERE namespace LIKE $1
            ORDER BY namespace, key
            "#,
        )
        .bind(&pattern)
        .fetch_all(&self.pool)
        .await?;

        let entries: Vec<ServiceConfigEntry> = rows
            .into_iter()
            .map(row_to_service_config_entry)
            .collect::<Result<Vec<_>, _>>()?;

        if entries.is_empty() {
            // service_config_mappings テーブルから直接マッピングを検索
            let mapped_rows = sqlx::query(
                r#"
                SELECT ce.namespace, ce.key, ce.value_json
                FROM config_entries ce
                INNER JOIN service_config_mappings scm ON ce.id = scm.config_entry_id
                WHERE scm.service_name = $1
                ORDER BY ce.namespace, ce.key
                "#,
            )
            .bind(service_name)
            .fetch_all(&self.pool)
            .await?;

            let mapped_entries: Vec<ServiceConfigEntry> = mapped_rows
                .into_iter()
                .map(row_to_service_config_entry)
                .collect::<Result<Vec<_>, _>>()?;

            if mapped_entries.is_empty() {
                return Err(anyhow::anyhow!(
                    "service not found: {}",
                    service_name
                ));
            }

            return Ok(ServiceConfigResult {
                service_name: service_name.to_string(),
                entries: mapped_entries,
            });
        }

        Ok(ServiceConfigResult {
            service_name: service_name.to_string(),
            entries,
        })
    }

    async fn record_change_log(&self, log: &ConfigChangeLog) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO config_change_logs (id, config_entry_id, namespace, key, old_value_json, new_value_json, change_type, changed_by, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(log.id)
        .bind(log.config_entry_id)
        .bind(&log.namespace)
        .bind(&log.key)
        .bind(&log.old_value)
        .bind(&log.new_value)
        .bind(&log.change_type)
        .bind(&log.changed_by)
        .bind(log.changed_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn list_change_logs(
        &self,
        namespace: &str,
        key: &str,
    ) -> anyhow::Result<Vec<ConfigChangeLog>> {
        let rows = sqlx::query(
            r#"
            SELECT id, config_entry_id, namespace, key, old_value_json, new_value_json,
                   change_type, changed_by, created_at
            FROM config_change_logs
            WHERE namespace = $1 AND key = $2
            ORDER BY created_at DESC
            "#,
        )
        .bind(namespace)
        .bind(key)
        .fetch_all(&self.pool)
        .await?;

        let logs = rows
            .into_iter()
            .map(|row| {
                Ok(ConfigChangeLog {
                    id: row.try_get("id")?,
                    config_entry_id: row.try_get("config_entry_id")?,
                    namespace: row.try_get("namespace")?,
                    key: row.try_get("key")?,
                    old_value: row.try_get("old_value_json")?,
                    new_value: row.try_get("new_value_json")?,
                    old_version: 0,
                    new_version: 0,
                    change_type: row.try_get("change_type")?,
                    changed_by: row.try_get("changed_by")?,
                    changed_at: row.try_get("created_at")?,
                })
            })
            .collect::<Result<Vec<_>, sqlx::Error>>()?;

        Ok(logs)
    }

    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<ConfigEntry>> {
        let row = sqlx::query(
            r#"
            SELECT id, namespace, key, value_json, version, description,
                   created_by, updated_by, created_at, updated_at
            FROM config_entries
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(row_to_config_entry(row)?)),
            None => Ok(None),
        }
    }
}
