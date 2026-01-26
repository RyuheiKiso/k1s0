//! PostgreSQLリポジトリ実装

use sqlx::{PgPool, Row};
use std::time::SystemTime;

use crate::domain::{ConfigError, Setting, SettingList, SettingQuery, SettingRepository, SettingValueType};

/// PostgreSQLリポジトリ
pub struct PostgresRepository {
    pool: PgPool,
}

impl PostgresRepository {
    /// 新しいリポジトリを作成
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl SettingRepository for PostgresRepository {
    async fn get(
        &self,
        service_name: &str,
        key: &str,
        env: Option<&str>,
    ) -> Result<Option<Setting>, ConfigError> {
        let query = match env {
            Some(e) => {
                sqlx::query(
                    r#"
                    SELECT setting_id, service_name, environment, setting_key, value_type,
                           setting_value, description, created_at, updated_at
                    FROM fw_m_setting
                    WHERE (service_name = $1 OR service_name IS NULL)
                      AND setting_key = $2
                      AND (environment = $3 OR environment IS NULL)
                      AND (valid_from <= CURRENT_TIMESTAMP)
                      AND (valid_to IS NULL OR valid_to > CURRENT_TIMESTAMP)
                    ORDER BY
                      CASE WHEN service_name = $1 THEN 0 ELSE 1 END,
                      CASE WHEN environment = $3 THEN 0 ELSE 1 END
                    LIMIT 1
                    "#,
                )
                .bind(service_name)
                .bind(key)
                .bind(e)
            }
            None => {
                sqlx::query(
                    r#"
                    SELECT setting_id, service_name, environment, setting_key, value_type,
                           setting_value, description, created_at, updated_at
                    FROM fw_m_setting
                    WHERE (service_name = $1 OR service_name IS NULL)
                      AND setting_key = $2
                      AND environment IS NULL
                      AND (valid_from <= CURRENT_TIMESTAMP)
                      AND (valid_to IS NULL OR valid_to > CURRENT_TIMESTAMP)
                    ORDER BY
                      CASE WHEN service_name = $1 THEN 0 ELSE 1 END
                    LIMIT 1
                    "#,
                )
                .bind(service_name)
                .bind(key)
            }
        };

        let row = query
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| ConfigError::storage(e.to_string()))?;

        Ok(row.map(|r| row_to_setting(&r)))
    }

    async fn list(&self, query: &SettingQuery) -> Result<SettingList, ConfigError> {
        let page_size = query.page_size.unwrap_or(100).min(1000) as i64;
        let offset = query
            .page_token
            .as_ref()
            .and_then(|t| t.strip_prefix("offset:"))
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);

        let (sql, binds) = build_list_query(query, page_size, offset);

        let rows = sqlx::query(&sql)
            .bind(binds.service_name.as_deref())
            .bind(binds.env.as_deref())
            .bind(binds.key_prefix.as_deref().map(|p| format!("{}%", p)))
            .bind(page_size + 1)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ConfigError::storage(e.to_string()))?;

        let has_next = rows.len() as i64 > page_size;
        let settings: Vec<Setting> = rows
            .into_iter()
            .take(page_size as usize)
            .map(|r| row_to_setting(&r))
            .collect();

        let mut list = SettingList::new(settings);
        if has_next {
            list = list.with_next_page_token(format!("offset:{}", offset + page_size));
        }

        Ok(list)
    }

    async fn save(&self, setting: &Setting) -> Result<(), ConfigError> {
        let env_value = if setting.env.is_empty() || setting.env == "default" {
            None
        } else {
            Some(setting.env.as_str())
        };

        let service_value = if setting.service_name.is_empty() {
            None
        } else {
            Some(setting.service_name.as_str())
        };

        sqlx::query(
            r#"
            INSERT INTO fw_m_setting (setting_key, setting_value, value_type, description, service_name, environment, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (setting_key, service_name, environment)
            DO UPDATE SET
                setting_value = EXCLUDED.setting_value,
                value_type = EXCLUDED.value_type,
                description = EXCLUDED.description,
                updated_at = EXCLUDED.updated_at,
                version = fw_m_setting.version + 1
            "#,
        )
        .bind(&setting.key)
        .bind(&setting.value)
        .bind(setting.value_type.as_str())
        .bind(&setting.description)
        .bind(service_value)
        .bind(env_value)
        .bind(chrono::DateTime::<chrono::Utc>::from(setting.created_at))
        .bind(chrono::DateTime::<chrono::Utc>::from(setting.updated_at))
        .execute(&self.pool)
        .await
        .map_err(|e| ConfigError::storage(e.to_string()))?;

        Ok(())
    }

    async fn delete(
        &self,
        service_name: &str,
        key: &str,
        env: &str,
    ) -> Result<bool, ConfigError> {
        let env_value = if env.is_empty() || env == "default" {
            None
        } else {
            Some(env)
        };

        let service_value = if service_name.is_empty() {
            None
        } else {
            Some(service_name)
        };

        let result = sqlx::query(
            r#"
            DELETE FROM fw_m_setting
            WHERE setting_key = $1
              AND (service_name = $2 OR (service_name IS NULL AND $2 IS NULL))
              AND (environment = $3 OR (environment IS NULL AND $3 IS NULL))
            "#,
        )
        .bind(key)
        .bind(service_value)
        .bind(env_value)
        .execute(&self.pool)
        .await
        .map_err(|e| ConfigError::storage(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }
}

fn row_to_setting(row: &sqlx::postgres::PgRow) -> Setting {
    let value_type_str: String = row.get("value_type");
    let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
    let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");

    Setting {
        id: row.get("setting_id"),
        service_name: row.get::<Option<String>, _>("service_name").unwrap_or_default(),
        env: row.get::<Option<String>, _>("environment").unwrap_or_else(|| "default".to_string()),
        key: row.get("setting_key"),
        value_type: SettingValueType::from_str(&value_type_str).unwrap_or_default(),
        value: row.get("setting_value"),
        description: row.get("description"),
        is_active: true,
        created_at: SystemTime::from(created_at),
        updated_at: SystemTime::from(updated_at),
    }
}

struct QueryBinds {
    service_name: Option<String>,
    env: Option<String>,
    key_prefix: Option<String>,
}

fn build_list_query(query: &SettingQuery, _page_size: i64, _offset: i64) -> (String, QueryBinds) {
    let mut conditions = vec![
        "(valid_from <= CURRENT_TIMESTAMP)",
        "(valid_to IS NULL OR valid_to > CURRENT_TIMESTAMP)",
    ];

    if query.service_name.is_some() {
        conditions.push("(service_name = $1 OR service_name IS NULL)");
    }
    if query.env.is_some() {
        conditions.push("(environment = $2 OR environment IS NULL)");
    }
    if query.key_prefix.is_some() {
        conditions.push("setting_key LIKE $3");
    }

    let sql = format!(
        r#"
        SELECT setting_id, service_name, environment, setting_key, value_type,
               setting_value, description, created_at, updated_at
        FROM fw_m_setting
        WHERE {}
        ORDER BY service_name NULLS LAST, environment NULLS LAST, setting_key
        LIMIT $4 OFFSET $5
        "#,
        conditions.join(" AND ")
    );

    let binds = QueryBinds {
        service_name: query.service_name.clone(),
        env: query.env.clone(),
        key_prefix: query.key_prefix.clone(),
    };

    (sql, binds)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_postgres_repository_new() {
        // Pool creation requires a database connection, so we just test the struct
        // Full integration tests would require a test database
    }
}
