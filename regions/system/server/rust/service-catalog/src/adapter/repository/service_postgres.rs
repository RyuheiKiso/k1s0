use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

// HIGH-003 監査対応: ILIKE 検索前に %_\ をエスケープして意図しない全件マッチを防止する
use k1s0_server_common::escape_like_pattern;
use crate::domain::entity::service::{Service, ServiceLifecycle, ServiceTier};
use crate::domain::repository::service_repository::{ServiceListFilters, ServiceRepository};

/// `ServiceRow` は `service_catalog.services` テーブルの行を表す中間構造体。
#[derive(Debug, Clone, sqlx::FromRow)]
struct ServiceRow {
    id: Uuid,
    name: String,
    description: Option<String>,
    team_id: Uuid,
    tier: String,
    lifecycle: String,
    repository_url: Option<String>,
    api_endpoint: Option<String>,
    healthcheck_url: Option<String>,
    tags: serde_json::Value,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<ServiceRow> for Service {
    fn from(row: ServiceRow) -> Self {
        let tier = row
            .tier
            .parse::<ServiceTier>()
            .unwrap_or(ServiceTier::Standard);
        let lifecycle = row
            .lifecycle
            .parse::<ServiceLifecycle>()
            .unwrap_or(ServiceLifecycle::Development);
        let tags: Vec<String> = serde_json::from_value(row.tags).unwrap_or_default();

        Service {
            id: row.id,
            name: row.name,
            description: row.description,
            team_id: row.team_id,
            tier,
            lifecycle,
            repository_url: row.repository_url,
            api_endpoint: row.api_endpoint,
            healthcheck_url: row.healthcheck_url,
            tags,
            metadata: row.metadata,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// `ServicePostgresRepository` は `PostgreSQL` ベースのサービスリポジトリ。
pub struct ServicePostgresRepository {
    pool: PgPool,
    metrics: Option<Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl ServicePostgresRepository {
    #[allow(dead_code)]
    #[must_use] 
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            metrics: None,
        }
    }

    #[must_use] 
    pub fn with_metrics(pool: PgPool, metrics: Arc<k1s0_telemetry::metrics::Metrics>) -> Self {
        Self {
            pool,
            metrics: Some(metrics),
        }
    }
}

#[async_trait]
impl ServiceRepository for ServicePostgresRepository {
    // CRIT-004 監査対応: RLS テナント分離のため set_config をトランザクション内で設定する。
    // defense-in-depth として WHERE 句にも tenant_id 条件を追加する。
    async fn list(&self, tenant_id: &str, filters: ServiceListFilters) -> anyhow::Result<Vec<Service>> {
        let start = std::time::Instant::now();

        // トランザクション内で RLS 用セッション変数を設定する
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        // defense-in-depth として WHERE 句にも tenant_id を追加する
        let mut query = String::from(
            "SELECT id, name, description, team_id, tier, lifecycle, repository_url, \
             api_endpoint, healthcheck_url, tags, metadata, created_at, updated_at \
             FROM service_catalog.services WHERE tenant_id = $1",
        );
        let mut param_idx = 2u32;

        if filters.team_id.is_some() {
            query.push_str(&format!(" AND team_id = ${param_idx}"));
            param_idx += 1;
        }
        if filters.tier.is_some() {
            query.push_str(&format!(" AND tier = ${param_idx}"));
            param_idx += 1;
        }
        if filters.lifecycle.is_some() {
            query.push_str(&format!(" AND lifecycle = ${param_idx}"));
            param_idx += 1;
        }
        if filters.tag.is_some() {
            query.push_str(&format!(" AND tags @> ${param_idx}::jsonb"));
            // param_idx += 1; // 最後のパラメータ
        }

        query.push_str(" ORDER BY name ASC");

        let mut q = sqlx::query_as::<_, ServiceRow>(&query);
        q = q.bind(tenant_id);

        if let Some(ref team_id) = filters.team_id {
            q = q.bind(team_id);
        }
        if let Some(ref tier) = filters.tier {
            q = q.bind(tier.to_string());
        }
        if let Some(ref lifecycle) = filters.lifecycle {
            q = q.bind(lifecycle.to_string());
        }
        if let Some(ref tag) = filters.tag {
            let tag_json = serde_json::json!([tag]);
            q = q.bind(tag_json);
        }

        let rows = q.fetch_all(&mut *tx).await?;
        tx.commit().await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("list", "services", start.elapsed().as_secs_f64());
        }

        Ok(rows.into_iter().map(std::convert::Into::into).collect())
    }

    // テナントスコープで set_config を設定した後にサービス ID で検索する。
    async fn find_by_id(&self, tenant_id: &str, id: Uuid) -> anyhow::Result<Option<Service>> {
        let start = std::time::Instant::now();

        // トランザクション内で RLS 用セッション変数を設定する
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;
        // defense-in-depth として WHERE 句にも tenant_id を追加する
        let row = sqlx::query_as::<_, ServiceRow>(
            "SELECT id, name, description, team_id, tier, lifecycle, repository_url, \
             api_endpoint, healthcheck_url, tags, metadata, created_at, updated_at \
             FROM service_catalog.services WHERE tenant_id = $1 AND id = $2",
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;
        tx.commit().await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("find_by_id", "services", start.elapsed().as_secs_f64());
        }

        Ok(row.map(std::convert::Into::into))
    }

    // テナントスコープで set_config を設定した後に新規サービスを登録する。
    async fn create(&self, tenant_id: &str, service: &Service) -> anyhow::Result<Service> {
        let start = std::time::Instant::now();
        let tags_json = serde_json::to_value(&service.tags)?;

        // トランザクション内で RLS 用セッション変数を設定する
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;
        // tenant_id カラムにも挿入して defense-in-depth を実現する
        let row = sqlx::query_as::<_, ServiceRow>(
            "INSERT INTO service_catalog.services \
             (tenant_id, id, name, description, team_id, tier, lifecycle, repository_url, \
              api_endpoint, healthcheck_url, tags, metadata, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14) \
             RETURNING id, name, description, team_id, tier, lifecycle, repository_url, \
             api_endpoint, healthcheck_url, tags, metadata, created_at, updated_at",
        )
        .bind(tenant_id)
        .bind(service.id)
        .bind(&service.name)
        .bind(&service.description)
        .bind(service.team_id)
        .bind(service.tier.to_string())
        .bind(service.lifecycle.to_string())
        .bind(&service.repository_url)
        .bind(&service.api_endpoint)
        .bind(&service.healthcheck_url)
        .bind(&tags_json)
        .bind(&service.metadata)
        .bind(service.created_at)
        .bind(service.updated_at)
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("create", "services", start.elapsed().as_secs_f64());
        }

        Ok(row.into())
    }

    // テナントスコープで set_config を設定した後にサービスを更新する。
    async fn update(&self, tenant_id: &str, service: &Service) -> anyhow::Result<Service> {
        let start = std::time::Instant::now();
        let tags_json = serde_json::to_value(&service.tags)?;

        // トランザクション内で RLS 用セッション変数を設定する
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;
        // WHERE 句に tenant_id を追加して defense-in-depth を実現する
        let row = sqlx::query_as::<_, ServiceRow>(
            "UPDATE service_catalog.services SET \
             name = $3, description = $4, tier = $5, lifecycle = $6, \
             repository_url = $7, api_endpoint = $8, healthcheck_url = $9, \
             tags = $10, metadata = $11, updated_at = $12 \
             WHERE tenant_id = $1 AND id = $2 \
             RETURNING id, name, description, team_id, tier, lifecycle, repository_url, \
             api_endpoint, healthcheck_url, tags, metadata, created_at, updated_at",
        )
        .bind(tenant_id)
        .bind(service.id)
        .bind(&service.name)
        .bind(&service.description)
        .bind(service.tier.to_string())
        .bind(service.lifecycle.to_string())
        .bind(&service.repository_url)
        .bind(&service.api_endpoint)
        .bind(&service.healthcheck_url)
        .bind(&tags_json)
        .bind(&service.metadata)
        .bind(service.updated_at)
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("update", "services", start.elapsed().as_secs_f64());
        }

        Ok(row.into())
    }

    // テナントスコープで set_config を設定した後にサービスを削除する。
    async fn delete(&self, tenant_id: &str, id: Uuid) -> anyhow::Result<()> {
        let start = std::time::Instant::now();

        // トランザクション内で RLS 用セッション変数を設定する
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;
        // WHERE 句に tenant_id を追加して defense-in-depth を実現する
        sqlx::query("DELETE FROM service_catalog.services WHERE tenant_id = $1 AND id = $2")
            .bind(tenant_id)
            .bind(id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("delete", "services", start.elapsed().as_secs_f64());
        }

        Ok(())
    }

    // テナントスコープで set_config を設定した後にサービスを検索する。
    async fn search(
        &self,
        tenant_id: &str,
        query: Option<String>,
        tags: Option<Vec<String>>,
        tier: Option<ServiceTier>,
    ) -> anyhow::Result<Vec<Service>> {
        let start = std::time::Instant::now();

        // トランザクション内で RLS 用セッション変数を設定する
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        // defense-in-depth として WHERE 句にも tenant_id を追加する
        let mut sql = String::from(
            "SELECT id, name, description, team_id, tier, lifecycle, repository_url, \
             api_endpoint, healthcheck_url, tags, metadata, created_at, updated_at \
             FROM service_catalog.services WHERE tenant_id = $1",
        );
        let mut param_idx = 2u32;

        if query.is_some() {
            // HIGH-003 監査対応: ILIKE のワイルドカード特殊文字をエスケープし、ESCAPE '\' を指定する
            sql.push_str(&format!(
                " AND (name ILIKE '%' || ${param_idx} || '%' ESCAPE '\\' OR description ILIKE '%' || ${param_idx} || '%' ESCAPE '\\')"
            ));
            param_idx += 1;
        }
        if tags.is_some() {
            sql.push_str(&format!(" AND tags @> ${param_idx}::jsonb"));
            param_idx += 1;
        }
        if tier.is_some() {
            sql.push_str(&format!(" AND tier = ${param_idx}"));
            // param_idx += 1;
        }

        sql.push_str(" ORDER BY name ASC");

        let mut q = sqlx::query_as::<_, ServiceRow>(&sql);
        q = q.bind(tenant_id);

        if let Some(ref query_str) = query {
            // HIGH-003 監査対応: バインド前に escape_like_pattern でエスケープ済みの値を渡す
            let escaped = escape_like_pattern(query_str);
            q = q.bind(escaped);
        }
        if let Some(ref tag_list) = tags {
            let tags_json = serde_json::to_value(tag_list)?;
            q = q.bind(tags_json);
        }
        if let Some(ref t) = tier {
            q = q.bind(t.to_string());
        }

        let rows = q.fetch_all(&mut *tx).await?;
        tx.commit().await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("search", "services", start.elapsed().as_secs_f64());
        }

        Ok(rows.into_iter().map(std::convert::Into::into).collect())
    }
}
