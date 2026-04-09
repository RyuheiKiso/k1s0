use std::sync::Arc;

use async_trait::async_trait;
// CRIT-4 監査対応: LIKE/ILIKE メタキャラクター（%_\）をエスケープするユーティリティを使用する
use k1s0_server_common::escape_like_pattern;
use sqlx::PgPool;

use crate::domain::entity::app::App;
use crate::domain::repository::AppRepository;

/// `AppRow` は `app_registry.apps` テーブルの行を表す中間構造体。
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

/// `AppPostgresRepository` は `PostgreSQL` ベースのアプリリポジトリ。
pub struct AppPostgresRepository {
    pool: PgPool,
    metrics: Option<Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl AppPostgresRepository {
    // テスト環境でのみ使用するコンストラクタ。本番では with_metrics を使用する（M-01対応）
    #[cfg(test)]
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
impl AppRepository for AppPostgresRepository {
    // CRIT-004 監査対応: RLS テナント分離のため set_config をトランザクション内で設定する。
    // defense-in-depth として WHERE 句にも tenant_id 条件を追加する。
    async fn list(
        &self,
        tenant_id: &str,
        category: Option<String>,
        search: Option<String>,
    ) -> anyhow::Result<Vec<App>> {
        let start = std::time::Instant::now();
        let category = category.as_deref();
        let search = search.as_deref();

        // CRIT-4 監査対応: ILIKE 検索前に %_\ をエスケープして意図しない全件マッチを防止する
        let escaped_search = search.map(escape_like_pattern);
        let search = escaped_search.as_deref();

        // トランザクション内で RLS 用セッション変数を設定する
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        let rows = if let (Some(cat), Some(q)) = (category, search) {
            sqlx::query_as::<_, AppRow>(
                r"
                SELECT id, name, description, category, icon_url, created_at, updated_at
                FROM app_registry.apps
                WHERE tenant_id = $1 AND category = $2 AND (name ILIKE '%' || $3 || '%' ESCAPE '\' OR description ILIKE '%' || $3 || '%' ESCAPE '\')
                ORDER BY name ASC
                ",
            )
            .bind(tenant_id)
            .bind(cat)
            .bind(q)
            .fetch_all(&mut *tx)
            .await?
        } else if let Some(cat) = category {
            sqlx::query_as::<_, AppRow>(
                r"
                SELECT id, name, description, category, icon_url, created_at, updated_at
                FROM app_registry.apps
                WHERE tenant_id = $1 AND category = $2
                ORDER BY name ASC
                ",
            )
            .bind(tenant_id)
            .bind(cat)
            .fetch_all(&mut *tx)
            .await?
        } else if let Some(q) = search {
            sqlx::query_as::<_, AppRow>(
                r"
                SELECT id, name, description, category, icon_url, created_at, updated_at
                FROM app_registry.apps
                WHERE tenant_id = $1 AND (name ILIKE '%' || $2 || '%' ESCAPE '\' OR description ILIKE '%' || $2 || '%' ESCAPE '\')
                ORDER BY name ASC
                ",
            )
            .bind(tenant_id)
            .bind(q)
            .fetch_all(&mut *tx)
            .await?
        } else {
            sqlx::query_as::<_, AppRow>(
                r"
                SELECT id, name, description, category, icon_url, created_at, updated_at
                FROM app_registry.apps
                WHERE tenant_id = $1
                ORDER BY name ASC
                ",
            )
            .bind(tenant_id)
            .fetch_all(&mut *tx)
            .await?
        };

        tx.commit().await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("list", "apps", start.elapsed().as_secs_f64());
        }

        Ok(rows.into_iter().map(std::convert::Into::into).collect())
    }

    // テナントスコープで set_config を設定した後にアプリ ID で検索する。
    async fn find_by_id(&self, tenant_id: &str, id: &str) -> anyhow::Result<Option<App>> {
        let start = std::time::Instant::now();

        // トランザクション内で RLS 用セッション変数を設定する
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;
        // defense-in-depth として WHERE 句にも tenant_id を追加する
        let row = sqlx::query_as::<_, AppRow>(
            r"
            SELECT id, name, description, category, icon_url, created_at, updated_at
            FROM app_registry.apps
            WHERE tenant_id = $1 AND id = $2
            ",
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;
        tx.commit().await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("find_by_id", "apps", start.elapsed().as_secs_f64());
        }

        Ok(row.map(std::convert::Into::into))
    }

    // テナントスコープで set_config を設定した後にアプリを新規登録する。
    async fn create(&self, tenant_id: &str, app: &App) -> anyhow::Result<App> {
        let start = std::time::Instant::now();

        // トランザクション内で RLS 用セッション変数を設定する
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;
        // tenant_id カラムにも挿入して defense-in-depth を実現する
        sqlx::query(
            "INSERT INTO app_registry.apps (tenant_id, id, name, description, category, icon_url, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(tenant_id)
        .bind(&app.id)
        .bind(&app.name)
        .bind(&app.description)
        .bind(&app.category)
        .bind(&app.icon_url)
        .bind(app.created_at)
        .bind(app.updated_at)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("create", "apps", start.elapsed().as_secs_f64());
        }

        Ok(app.clone())
    }

    // テナントスコープで set_config を設定した後にアプリ情報を更新する。
    async fn update(&self, tenant_id: &str, app: &App) -> anyhow::Result<App> {
        let start = std::time::Instant::now();

        // トランザクション内で RLS 用セッション変数を設定する
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;
        // WHERE 句に tenant_id を追加して defense-in-depth を実現する
        sqlx::query(
            "UPDATE app_registry.apps SET name = $3, description = $4, category = $5, \
             icon_url = $6, updated_at = $7 WHERE tenant_id = $1 AND id = $2",
        )
        .bind(tenant_id)
        .bind(&app.id)
        .bind(&app.name)
        .bind(&app.description)
        .bind(&app.category)
        .bind(&app.icon_url)
        .bind(app.updated_at)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("update", "apps", start.elapsed().as_secs_f64());
        }

        Ok(app.clone())
    }

    // テナントスコープで set_config を設定した後にアプリを削除する。
    async fn delete(&self, tenant_id: &str, id: &str) -> anyhow::Result<bool> {
        let start = std::time::Instant::now();

        // トランザクション内で RLS 用セッション変数を設定する
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;
        // WHERE 句に tenant_id を追加して defense-in-depth を実現する
        let result = sqlx::query("DELETE FROM app_registry.apps WHERE tenant_id = $1 AND id = $2")
            .bind(tenant_id)
            .bind(id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("delete", "apps", start.elapsed().as_secs_f64());
        }

        Ok(result.rows_affected() > 0)
    }
}
