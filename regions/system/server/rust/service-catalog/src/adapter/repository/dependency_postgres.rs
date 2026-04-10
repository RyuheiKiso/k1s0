use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::dependency::{Dependency, DependencyType};
use crate::domain::repository::DependencyRepository;

#[derive(Debug, Clone, sqlx::FromRow)]
struct DependencyRow {
    source_service_id: Uuid,
    target_service_id: Uuid,
    dependency_type: String,
    description: Option<String>,
}

impl From<DependencyRow> for Dependency {
    fn from(row: DependencyRow) -> Self {
        let dependency_type = row
            .dependency_type
            .parse::<DependencyType>()
            .unwrap_or(DependencyType::Runtime);
        Dependency {
            source_service_id: row.source_service_id,
            target_service_id: row.target_service_id,
            dependency_type,
            description: row.description,
        }
    }
}

/// `DependencyPostgresRepository` は `PostgreSQL` ベースの依存関係リポジトリ。
pub struct DependencyPostgresRepository {
    pool: PgPool,
    metrics: Option<Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl DependencyPostgresRepository {
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
impl DependencyRepository for DependencyPostgresRepository {
    async fn list_by_service(&self, service_id: Uuid) -> anyhow::Result<Vec<Dependency>> {
        let start = std::time::Instant::now();

        let rows = sqlx::query_as::<_, DependencyRow>(
            "SELECT source_service_id, target_service_id, dependency_type, description \
             FROM service_catalog.dependencies WHERE source_service_id = $1",
        )
        .bind(service_id)
        .fetch_all(&self.pool)
        .await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration(
                "list_by_service",
                "dependencies",
                start.elapsed().as_secs_f64(),
            );
        }

        Ok(rows.into_iter().map(std::convert::Into::into).collect())
    }

    async fn set_dependencies(
        &self,
        service_id: Uuid,
        deps: Vec<Dependency>,
    ) -> anyhow::Result<()> {
        let start = std::time::Instant::now();
        let mut tx = self.pool.begin().await?;

        // Delete existing dependencies for this service
        sqlx::query("DELETE FROM service_catalog.dependencies WHERE source_service_id = $1")
            .bind(service_id)
            .execute(&mut *tx)
            .await?;

        // Insert new dependencies
        for dep in &deps {
            sqlx::query(
                "INSERT INTO service_catalog.dependencies \
                 (source_service_id, target_service_id, dependency_type, description) \
                 VALUES ($1, $2, $3, $4)",
            )
            .bind(dep.source_service_id)
            .bind(dep.target_service_id)
            .bind(dep.dependency_type.to_string())
            .bind(&dep.description)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration(
                "set_dependencies",
                "dependencies",
                start.elapsed().as_secs_f64(),
            );
        }

        Ok(())
    }

    async fn get_all_dependencies(&self) -> anyhow::Result<Vec<Dependency>> {
        let start = std::time::Instant::now();

        let rows = sqlx::query_as::<_, DependencyRow>(
            "SELECT source_service_id, target_service_id, dependency_type, description \
             FROM service_catalog.dependencies",
        )
        .fetch_all(&self.pool)
        .await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("get_all", "dependencies", start.elapsed().as_secs_f64());
        }

        Ok(rows.into_iter().map(std::convert::Into::into).collect())
    }
}
