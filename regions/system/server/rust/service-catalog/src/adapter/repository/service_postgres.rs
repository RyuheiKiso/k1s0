use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::service::{Service, ServiceLifecycle, ServiceTier};
use crate::domain::repository::service_repository::{ServiceListFilters, ServiceRepository};

/// ServiceRow は service_catalog.services テーブルの行を表す中間構造体。
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
        let tier = row.tier.parse::<ServiceTier>().unwrap_or(ServiceTier::Standard);
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

/// ServicePostgresRepository は PostgreSQL ベースのサービスリポジトリ。
pub struct ServicePostgresRepository {
    pool: PgPool,
    metrics: Option<Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl ServicePostgresRepository {
    #[allow(dead_code)]
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            metrics: None,
        }
    }

    pub fn with_metrics(pool: PgPool, metrics: Arc<k1s0_telemetry::metrics::Metrics>) -> Self {
        Self {
            pool,
            metrics: Some(metrics),
        }
    }
}

#[async_trait]
impl ServiceRepository for ServicePostgresRepository {
    async fn list(&self, filters: ServiceListFilters) -> anyhow::Result<Vec<Service>> {
        let start = std::time::Instant::now();

        // Build dynamic query based on filters
        let mut query = String::from(
            "SELECT id, name, description, team_id, tier, lifecycle, repository_url, \
             api_endpoint, healthcheck_url, tags, metadata, created_at, updated_at \
             FROM service_catalog.services WHERE 1=1",
        );
        let mut param_idx = 1u32;

        if filters.team_id.is_some() {
            query.push_str(&format!(" AND team_id = ${}", param_idx));
            param_idx += 1;
        }
        if filters.tier.is_some() {
            query.push_str(&format!(" AND tier = ${}", param_idx));
            param_idx += 1;
        }
        if filters.lifecycle.is_some() {
            query.push_str(&format!(" AND lifecycle = ${}", param_idx));
            param_idx += 1;
        }
        if filters.tag.is_some() {
            query.push_str(&format!(" AND tags @> ${}::jsonb", param_idx));
            // param_idx += 1; // last param
        }

        query.push_str(" ORDER BY name ASC");

        let mut q = sqlx::query_as::<_, ServiceRow>(&query);

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

        let rows = q.fetch_all(&self.pool).await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("list", "services", start.elapsed().as_secs_f64());
        }

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<Service>> {
        let start = std::time::Instant::now();
        let row = sqlx::query_as::<_, ServiceRow>(
            "SELECT id, name, description, team_id, tier, lifecycle, repository_url, \
             api_endpoint, healthcheck_url, tags, metadata, created_at, updated_at \
             FROM service_catalog.services WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("find_by_id", "services", start.elapsed().as_secs_f64());
        }

        Ok(row.map(|r| r.into()))
    }

    async fn create(&self, service: &Service) -> anyhow::Result<Service> {
        let start = std::time::Instant::now();
        let tags_json = serde_json::to_value(&service.tags)?;

        let row = sqlx::query_as::<_, ServiceRow>(
            "INSERT INTO service_catalog.services \
             (id, name, description, team_id, tier, lifecycle, repository_url, \
              api_endpoint, healthcheck_url, tags, metadata, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) \
             RETURNING id, name, description, team_id, tier, lifecycle, repository_url, \
             api_endpoint, healthcheck_url, tags, metadata, created_at, updated_at",
        )
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
        .fetch_one(&self.pool)
        .await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("create", "services", start.elapsed().as_secs_f64());
        }

        Ok(row.into())
    }

    async fn update(&self, service: &Service) -> anyhow::Result<Service> {
        let start = std::time::Instant::now();
        let tags_json = serde_json::to_value(&service.tags)?;

        let row = sqlx::query_as::<_, ServiceRow>(
            "UPDATE service_catalog.services SET \
             name = $2, description = $3, tier = $4, lifecycle = $5, \
             repository_url = $6, api_endpoint = $7, healthcheck_url = $8, \
             tags = $9, metadata = $10, updated_at = $11 \
             WHERE id = $1 \
             RETURNING id, name, description, team_id, tier, lifecycle, repository_url, \
             api_endpoint, healthcheck_url, tags, metadata, created_at, updated_at",
        )
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
        .fetch_one(&self.pool)
        .await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("update", "services", start.elapsed().as_secs_f64());
        }

        Ok(row.into())
    }

    async fn delete(&self, id: Uuid) -> anyhow::Result<()> {
        let start = std::time::Instant::now();

        sqlx::query("DELETE FROM service_catalog.services WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("delete", "services", start.elapsed().as_secs_f64());
        }

        Ok(())
    }

    async fn search(
        &self,
        query: Option<String>,
        tags: Option<Vec<String>>,
        tier: Option<ServiceTier>,
    ) -> anyhow::Result<Vec<Service>> {
        let start = std::time::Instant::now();

        let mut sql = String::from(
            "SELECT id, name, description, team_id, tier, lifecycle, repository_url, \
             api_endpoint, healthcheck_url, tags, metadata, created_at, updated_at \
             FROM service_catalog.services WHERE 1=1",
        );
        let mut param_idx = 1u32;

        if query.is_some() {
            sql.push_str(&format!(
                " AND (name ILIKE '%' || ${idx} || '%' OR description ILIKE '%' || ${idx} || '%')",
                idx = param_idx
            ));
            param_idx += 1;
        }
        if tags.is_some() {
            sql.push_str(&format!(" AND tags @> ${}::jsonb", param_idx));
            param_idx += 1;
        }
        if tier.is_some() {
            sql.push_str(&format!(" AND tier = ${}", param_idx));
            // param_idx += 1;
        }

        sql.push_str(" ORDER BY name ASC");

        let mut q = sqlx::query_as::<_, ServiceRow>(&sql);

        if let Some(ref query_str) = query {
            q = q.bind(query_str);
        }
        if let Some(ref tag_list) = tags {
            let tags_json = serde_json::to_value(tag_list)?;
            q = q.bind(tags_json);
        }
        if let Some(ref t) = tier {
            q = q.bind(t.to_string());
        }

        let rows = q.fetch_all(&self.pool).await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("search", "services", start.elapsed().as_secs_f64());
        }

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
}
