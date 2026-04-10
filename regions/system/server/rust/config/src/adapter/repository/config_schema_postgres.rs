use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::domain::entity::config_schema::ConfigSchema;
use crate::domain::repository::config_schema_repository::ConfigSchemaRepository;

/// `ConfigSchemaPostgresRepository` は `ConfigSchemaRepository` の `PostgreSQL` 実装。
pub struct ConfigSchemaPostgresRepository {
    pool: PgPool,
    metrics: Option<Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl ConfigSchemaPostgresRepository {
    #[must_use]
    pub fn with_metrics(pool: PgPool, metrics: Arc<k1s0_telemetry::metrics::Metrics>) -> Self {
        Self {
            pool,
            metrics: Some(metrics),
        }
    }
}

/// `PostgreSQL` の行から `ConfigSchema` を構築するヘルパー。
/// CRITICAL-RUST-001 監査対応: `tenant_id` を SELECT カラムから取得してマッピングする。
fn row_to_config_schema(row: sqlx::postgres::PgRow) -> Result<ConfigSchema, sqlx::Error> {
    Ok(ConfigSchema {
        id: row.try_get("id")?,
        tenant_id: row.try_get("tenant_id")?,
        service_name: row.try_get("service_name")?,
        namespace_prefix: row.try_get("namespace_prefix")?,
        schema_json: row.try_get("schema_json")?,
        updated_by: row.try_get("updated_by")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[async_trait]
impl ConfigSchemaRepository for ConfigSchemaPostgresRepository {
    // CRITICAL-RUST-001 監査対応: SELECT 前に set_config でテナントIDをセッション変数に設定し、
    // FORCE ROW LEVEL SECURITY が有効な config_schemas テーブルで正しくフィルタリングされるようにする。
    async fn find_by_service_name(
        &self,
        service_name: &str,
        tenant_id: &str,
    ) -> anyhow::Result<Option<ConfigSchema>> {
        // CRITICAL-RUST-001 監査対応: RLS テナント分離のためセッション変数を設定する。
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&self.pool)
            .await?;

        let start = std::time::Instant::now();
        let row = sqlx::query(
            r"
            SELECT id, tenant_id, service_name, namespace_prefix, schema_json,
                   updated_by, created_at, updated_at
            FROM config.config_schemas
            WHERE service_name = $1
            ",
        )
        .bind(service_name)
        .fetch_optional(&self.pool)
        .await?;
        if let Some(ref m) = self.metrics {
            m.record_db_query_duration(
                "find_by_service_name",
                "config_schemas",
                start.elapsed().as_secs_f64(),
            );
        }

        match row {
            Some(row) => Ok(Some(row_to_config_schema(row)?)),
            None => Ok(None),
        }
    }

    // CRITICAL-RUST-001 監査対応: SELECT 前に set_config でテナントIDをセッション変数に設定する。
    async fn find_by_namespace(
        &self,
        namespace: &str,
        tenant_id: &str,
    ) -> anyhow::Result<Option<ConfigSchema>> {
        // CRITICAL-RUST-001 監査対応: RLS テナント分離のためセッション変数を設定する。
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&self.pool)
            .await?;

        let start = std::time::Instant::now();
        let row = sqlx::query(
            r"
            SELECT id, tenant_id, service_name, namespace_prefix, schema_json,
                   updated_by, created_at, updated_at
            FROM config.config_schemas
            WHERE $1 LIKE namespace_prefix || '%'
            ORDER BY LENGTH(namespace_prefix) DESC
            LIMIT 1
            ",
        )
        .bind(namespace)
        .fetch_optional(&self.pool)
        .await?;
        if let Some(ref m) = self.metrics {
            m.record_db_query_duration(
                "find_by_namespace",
                "config_schemas",
                start.elapsed().as_secs_f64(),
            );
        }

        match row {
            Some(row) => Ok(Some(row_to_config_schema(row)?)),
            None => Ok(None),
        }
    }

    // CRITICAL-RUST-001 監査対応: SELECT 前に set_config でテナントIDをセッション変数に設定する。
    async fn list_all(&self, tenant_id: &str) -> anyhow::Result<Vec<ConfigSchema>> {
        // CRITICAL-RUST-001 監査対応: RLS テナント分離のためセッション変数を設定する。
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&self.pool)
            .await?;

        let start = std::time::Instant::now();
        let rows = sqlx::query(
            r"
            SELECT id, tenant_id, service_name, namespace_prefix, schema_json,
                   updated_by, created_at, updated_at
            FROM config.config_schemas
            ORDER BY service_name ASC
            ",
        )
        .fetch_all(&self.pool)
        .await?;
        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("list_all", "config_schemas", start.elapsed().as_secs_f64());
        }

        rows.into_iter()
            .map(|row| row_to_config_schema(row).map_err(|e| anyhow::anyhow!(e)))
            .collect()
    }

    // CRITICAL-RUST-001 監査対応: INSERT 前に set_config でテナントIDをセッション変数に設定し、
    // tenant_id カラムを INSERT に含めて NOT NULL 制約違反を防ぐ。
    async fn upsert(&self, schema: &ConfigSchema) -> anyhow::Result<ConfigSchema> {
        // CRITICAL-RUST-001 監査対応: RLS テナント分離のためセッション変数を設定する。
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(&schema.tenant_id)
            .execute(&self.pool)
            .await?;

        let start = std::time::Instant::now();
        let row = sqlx::query(
            r"
            INSERT INTO config.config_schemas (id, tenant_id, service_name, namespace_prefix, schema_json, updated_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (service_name) DO UPDATE
            SET namespace_prefix = EXCLUDED.namespace_prefix,
                schema_json = EXCLUDED.schema_json,
                updated_by = EXCLUDED.updated_by,
                updated_at = EXCLUDED.updated_at
            RETURNING id, tenant_id, service_name, namespace_prefix, schema_json, updated_by, created_at, updated_at
            ",
        )
        .bind(schema.id)
        .bind(&schema.tenant_id)
        .bind(&schema.service_name)
        .bind(&schema.namespace_prefix)
        .bind(&schema.schema_json)
        .bind(&schema.updated_by)
        .bind(schema.created_at)
        .bind(schema.updated_at)
        .fetch_one(&self.pool)
        .await?;
        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("upsert", "config_schemas", start.elapsed().as_secs_f64());
        }

        Ok(row_to_config_schema(row)?)
    }
}
