use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::entity::app::App;
use crate::domain::repository::AppRepository;

/// AppRow は app_registry.apps テーブルの行を表す中間構造体。
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AppRow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub icon_url: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<AppRow> for App {
    fn from(row: AppRow) -> Self {
        App {
            id: row.id,
            name: row.name,
            description: row.description,
            category: row.category,
            icon_url: row.icon_url,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// AppPostgresRepository は PostgreSQL ベースのアプリリポジトリ。
pub struct AppPostgresRepository {
    pool: PgPool,
    metrics: Option<Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl AppPostgresRepository {
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
impl AppRepository for AppPostgresRepository {
    async fn list(&self, category: Option<String>, search: Option<String>) -> anyhow::Result<Vec<App>> {
        let start = std::time::Instant::now();
        let category = category.as_deref();
        let search = search.as_deref();

        let rows = if let (Some(cat), Some(q)) = (category, search) {
            sqlx::query_as::<_, AppRow>(
                r#"
                SELECT id, name, description, category, icon_url, created_at, updated_at
                FROM app_registry.apps
                WHERE category = $1 AND (name ILIKE '%' || $2 || '%' OR description ILIKE '%' || $2 || '%')
                ORDER BY name ASC
                "#,
            )
            .bind(cat)
            .bind(q)
            .fetch_all(&self.pool)
            .await?
        } else if let Some(cat) = category {
            sqlx::query_as::<_, AppRow>(
                r#"
                SELECT id, name, description, category, icon_url, created_at, updated_at
                FROM app_registry.apps
                WHERE category = $1
                ORDER BY name ASC
                "#,
            )
            .bind(cat)
            .fetch_all(&self.pool)
            .await?
        } else if let Some(q) = search {
            sqlx::query_as::<_, AppRow>(
                r#"
                SELECT id, name, description, category, icon_url, created_at, updated_at
                FROM app_registry.apps
                WHERE name ILIKE '%' || $1 || '%' OR description ILIKE '%' || $1 || '%'
                ORDER BY name ASC
                "#,
            )
            .bind(q)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, AppRow>(
                r#"
                SELECT id, name, description, category, icon_url, created_at, updated_at
                FROM app_registry.apps
                ORDER BY name ASC
                "#,
            )
            .fetch_all(&self.pool)
            .await?
        };

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("list", "apps", start.elapsed().as_secs_f64());
        }

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<App>> {
        let start = std::time::Instant::now();
        let row = sqlx::query_as::<_, AppRow>(
            r#"
            SELECT id, name, description, category, icon_url, created_at, updated_at
            FROM app_registry.apps
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("find_by_id", "apps", start.elapsed().as_secs_f64());
        }

        Ok(row.map(|r| r.into()))
    }

    async fn create(&self, app: &App) -> anyhow::Result<App> {
        let start = std::time::Instant::now();

        sqlx::query(
            "INSERT INTO app_registry.apps (id, name, description, category, icon_url, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(&app.id)
        .bind(&app.name)
        .bind(&app.description)
        .bind(&app.category)
        .bind(&app.icon_url)
        .bind(app.created_at)
        .bind(app.updated_at)
        .execute(&self.pool)
        .await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("create", "apps", start.elapsed().as_secs_f64());
        }

        Ok(app.clone())
    }

    async fn update(&self, app: &App) -> anyhow::Result<App> {
        let start = std::time::Instant::now();

        sqlx::query(
            "UPDATE app_registry.apps SET name = $2, description = $3, category = $4, \
             icon_url = $5, updated_at = $6 WHERE id = $1",
        )
        .bind(&app.id)
        .bind(&app.name)
        .bind(&app.description)
        .bind(&app.category)
        .bind(&app.icon_url)
        .bind(app.updated_at)
        .execute(&self.pool)
        .await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("update", "apps", start.elapsed().as_secs_f64());
        }

        Ok(app.clone())
    }

    async fn delete(&self, id: &str) -> anyhow::Result<bool> {
        let start = std::time::Instant::now();

        let result = sqlx::query("DELETE FROM app_registry.apps WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("delete", "apps", start.elapsed().as_secs_f64());
        }

        Ok(result.rows_affected() > 0)
    }
}
