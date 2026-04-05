use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::policy::Policy;
use crate::domain::repository::PolicyRepository;

/// PostgreSQL 実装の PolicyRepository。
pub struct PolicyPostgresRepository {
    pool: Arc<PgPool>,
}

impl PolicyPostgresRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct PolicyRow {
    id: Uuid,
    name: String,
    description: String,
    rego_content: String,
    package_path: String,
    bundle_id: Option<Uuid>,
    enabled: bool,
    version: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    tenant_id: String,
}

impl From<PolicyRow> for Policy {
    fn from(r: PolicyRow) -> Self {
        Policy {
            id: r.id,
            name: r.name,
            description: r.description,
            rego_content: r.rego_content,
            package_path: r.package_path,
            bundle_id: r.bundle_id,
            version: r.version as u32,
            enabled: r.enabled,
            created_at: r.created_at,
            updated_at: r.updated_at,
            tenant_id: r.tenant_id,
        }
    }
}

#[async_trait]
impl PolicyRepository for PolicyPostgresRepository {
    async fn find_by_id(&self, id: &Uuid, tenant_id: &str) -> anyhow::Result<Option<Policy>> {
        // CRIT-005 対応: トランザクション内で tenant_id をセッション変数に設定してから SELECT する
        let mut tx = self.pool.begin().await?;

        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        let row: Option<PolicyRow> = sqlx::query_as(
            "SELECT id, name, description, rego_content, package_path, bundle_id, enabled, version, created_at, updated_at, tenant_id \
             FROM policy.policies WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(row.map(Into::into))
    }

    async fn find_all(&self, tenant_id: &str) -> anyhow::Result<Vec<Policy>> {
        // CRIT-005 対応: トランザクション内で tenant_id をセッション変数に設定してから SELECT する
        let mut tx = self.pool.begin().await?;

        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        let rows: Vec<PolicyRow> = sqlx::query_as(
            "SELECT id, name, description, rego_content, package_path, bundle_id, enabled, version, created_at, updated_at, tenant_id \
             FROM policy.policies ORDER BY created_at DESC",
        )
        .fetch_all(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        bundle_id: Option<Uuid>,
        enabled_only: bool,
        tenant_id: &str,
    ) -> anyhow::Result<(Vec<Policy>, u64)> {
        // CRIT-005 対応: トランザクション内で tenant_id をセッション変数に設定してから SELECT する
        let mut tx = self.pool.begin().await?;

        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        let offset = (page.saturating_sub(1) * page_size) as i64;
        let limit = page_size as i64;

        // 動的 WHERE 句を組み立てる。
        // セキュリティ注記（M-05 監査対応）: format!() で埋め込むのはハードコードされたカラム名定数のみ。
        // ユーザー入力（bundle_id 等）は全て sqlx のバインドパラメータ（$N）経由で渡すため
        // SQL インジェクションのリスクはない。
        let mut conditions = Vec::new();
        let mut bind_index = 1u32;

        if bundle_id.is_some() {
            conditions.push(format!("bundle_id = ${}", bind_index));
            bind_index += 1;
        }
        if enabled_only {
            conditions.push("enabled = true".to_string());
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let count_query = format!("SELECT COUNT(*) FROM policy.policies {}", where_clause);
        let data_query = format!(
            "SELECT id, name, description, rego_content, package_path, bundle_id, enabled, version, created_at, updated_at, tenant_id \
             FROM policy.policies {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            where_clause, bind_index, bind_index + 1
        );

        let mut count_q = sqlx::query_scalar::<_, i64>(&count_query);
        if let Some(ref v) = bundle_id {
            count_q = count_q.bind(v);
        }
        let total_count = count_q.fetch_one(&mut *tx).await?;

        let mut data_q = sqlx::query_as::<_, PolicyRow>(&data_query);
        if let Some(ref v) = bundle_id {
            data_q = data_q.bind(v);
        }
        data_q = data_q.bind(limit);
        data_q = data_q.bind(offset);

        let rows: Vec<PolicyRow> = data_q.fetch_all(&mut *tx).await?;

        tx.commit().await?;
        Ok((
            rows.into_iter().map(Into::into).collect(),
            total_count as u64,
        ))
    }

    async fn create(&self, policy: &Policy) -> anyhow::Result<()> {
        // CRIT-005 対応: トランザクション内で tenant_id をセッション変数に設定してから INSERT する
        let mut tx = self.pool.begin().await?;

        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(&policy.tenant_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query(
            "INSERT INTO policy.policies \
             (id, name, description, rego_content, package_path, bundle_id, enabled, version, created_at, updated_at, tenant_id) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
        )
        .bind(policy.id)
        .bind(&policy.name)
        .bind(&policy.description)
        .bind(&policy.rego_content)
        .bind(&policy.package_path)
        .bind(policy.bundle_id)
        .bind(policy.enabled)
        .bind(policy.version as i32)
        .bind(policy.created_at)
        .bind(policy.updated_at)
        .bind(&policy.tenant_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    async fn update(&self, policy: &Policy) -> anyhow::Result<()> {
        // CRIT-005 対応: トランザクション内で tenant_id をセッション変数に設定してから UPDATE する
        let mut tx = self.pool.begin().await?;

        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(&policy.tenant_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query(
            "UPDATE policy.policies \
             SET description = $2, rego_content = $3, package_path = $4, bundle_id = $5, enabled = $6, version = $7, updated_at = $8 \
             WHERE id = $1 AND tenant_id = $9",
        )
        .bind(policy.id)
        .bind(&policy.description)
        .bind(&policy.rego_content)
        .bind(&policy.package_path)
        .bind(policy.bundle_id)
        .bind(policy.enabled)
        .bind(policy.version as i32)
        .bind(policy.updated_at)
        .bind(&policy.tenant_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    async fn delete(&self, id: &Uuid, tenant_id: &str) -> anyhow::Result<bool> {
        // CRIT-005 対応: トランザクション内で tenant_id をセッション変数に設定してから DELETE する
        let mut tx = self.pool.begin().await?;

        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        let result = sqlx::query("DELETE FROM policy.policies WHERE id = $1 AND tenant_id = $2")
            .bind(id)
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(result.rows_affected() > 0)
    }

    async fn exists_by_name(&self, name: &str, tenant_id: &str) -> anyhow::Result<bool> {
        // CRIT-005 対応: トランザクション内で tenant_id をセッション変数に設定してから検索する
        let mut tx = self.pool.begin().await?;

        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        let row: (bool,) =
            sqlx::query_as("SELECT EXISTS(SELECT 1 FROM policy.policies WHERE name = $1 AND tenant_id = $2)")
                .bind(name)
                .bind(tenant_id)
                .fetch_one(&mut *tx)
                .await?;

        tx.commit().await?;
        Ok(row.0)
    }
}
