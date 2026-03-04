use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::domain::entity::config_change_log::ConfigChangeLog;
use crate::domain::repository::ConfigChangeLogRepository;

pub struct ConfigChangeLogPostgresRepository {
    pool: Arc<PgPool>,
    metrics: Option<Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl ConfigChangeLogPostgresRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self {
            pool,
            metrics: None,
        }
    }

    pub fn with_metrics(pool: Arc<PgPool>, metrics: Arc<k1s0_telemetry::metrics::Metrics>) -> Self {
        Self {
            pool,
            metrics: Some(metrics),
        }
    }
}

#[async_trait]
impl ConfigChangeLogRepository for ConfigChangeLogPostgresRepository {
    async fn record_change_log(&self, log: &ConfigChangeLog) -> anyhow::Result<()> {
        let start = std::time::Instant::now();
        sqlx::query(
            r#"
            INSERT INTO config_change_logs (
                id, config_entry_id, namespace, key, old_value_json, new_value_json,
                old_version, new_version, change_type, changed_by, trace_id, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(log.id)
        .bind(log.config_entry_id)
        .bind(&log.namespace)
        .bind(&log.key)
        .bind(&log.old_value)
        .bind(&log.new_value)
        .bind(log.old_version)
        .bind(log.new_version)
        .bind(&log.change_type)
        .bind(&log.changed_by)
        .bind(&log.trace_id)
        .bind(log.changed_at)
        .execute(self.pool.as_ref())
        .await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration(
                "record_change_log",
                "config_change_logs",
                start.elapsed().as_secs_f64(),
            );
        }

        Ok(())
    }

    async fn list_change_logs(
        &self,
        namespace: &str,
        key: &str,
    ) -> anyhow::Result<Vec<ConfigChangeLog>> {
        let start = std::time::Instant::now();
        let rows = sqlx::query(
            r#"
            SELECT id, config_entry_id, namespace, key, old_value_json, new_value_json,
                   old_version, new_version, change_type, changed_by, trace_id, created_at
            FROM config_change_logs
            WHERE namespace = $1 AND key = $2
            ORDER BY created_at DESC
            "#,
        )
        .bind(namespace)
        .bind(key)
        .fetch_all(self.pool.as_ref())
        .await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration(
                "list_change_logs",
                "config_change_logs",
                start.elapsed().as_secs_f64(),
            );
        }

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
                    old_version: row.try_get("old_version")?,
                    new_version: row.try_get("new_version")?,
                    change_type: row.try_get("change_type")?,
                    changed_by: row.try_get("changed_by")?,
                    trace_id: row.try_get("trace_id")?,
                    changed_at: row.try_get("created_at")?,
                })
            })
            .collect::<Result<Vec<_>, sqlx::Error>>()?;

        Ok(logs)
    }
}
