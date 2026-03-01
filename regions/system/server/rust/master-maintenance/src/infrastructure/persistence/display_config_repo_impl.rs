use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use crate::domain::entity::display_config::DisplayConfig;
use crate::domain::repository::display_config_repository::DisplayConfigRepository;

pub struct DisplayConfigPostgresRepository {
    pool: PgPool,
}

impl DisplayConfigPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DisplayConfigRepository for DisplayConfigPostgresRepository {
    async fn find_by_table_id(&self, table_id: Uuid) -> anyhow::Result<Vec<DisplayConfig>> {
        let rows = sqlx::query_as::<_, DisplayConfigRow>(
            "SELECT * FROM master_maintenance.display_configs WHERE table_id = $1 ORDER BY created_at"
        )
        .bind(table_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<DisplayConfig>> {
        let row = sqlx::query_as::<_, DisplayConfigRow>(
            "SELECT * FROM master_maintenance.display_configs WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    async fn create(&self, config: &DisplayConfig) -> anyhow::Result<DisplayConfig> {
        let row = sqlx::query_as::<_, DisplayConfigRow>(
            r#"INSERT INTO master_maintenance.display_configs
               (id, table_id, config_type, config_json, is_default, created_by)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING *"#
        )
        .bind(config.id)
        .bind(config.table_id)
        .bind(&config.config_type)
        .bind(&config.config_json)
        .bind(config.is_default)
        .bind(&config.created_by)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn update(&self, id: Uuid, config: &DisplayConfig) -> anyhow::Result<DisplayConfig> {
        let row = sqlx::query_as::<_, DisplayConfigRow>(
            r#"UPDATE master_maintenance.display_configs SET
               config_type = $2,
               config_json = $3,
               is_default = $4,
               updated_at = now()
               WHERE id = $1 RETURNING *"#
        )
        .bind(id)
        .bind(&config.config_type)
        .bind(&config.config_json)
        .bind(config.is_default)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn delete(&self, id: Uuid) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM master_maintenance.display_configs WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct DisplayConfigRow {
    id: Uuid,
    table_id: Uuid,
    config_type: String,
    config_json: serde_json::Value,
    is_default: bool,
    created_by: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<DisplayConfigRow> for DisplayConfig {
    fn from(row: DisplayConfigRow) -> Self {
        Self {
            id: row.id,
            table_id: row.table_id,
            config_type: row.config_type,
            config_json: row.config_json,
            is_default: row.is_default,
            created_by: row.created_by,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
