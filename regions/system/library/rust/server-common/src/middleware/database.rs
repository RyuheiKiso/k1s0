use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_secs: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            max_connections: 10,
            min_connections: 1,
            connect_timeout_secs: 30,
        }
    }
}

pub struct DatabaseSetup {
    config: DatabaseConfig,
    run_migrations: bool,
}

impl DatabaseSetup {
    pub fn new(config: DatabaseConfig) -> Self {
        Self {
            config,
            run_migrations: true,
        }
    }

    pub fn without_migrations(mut self) -> Self {
        self.run_migrations = false;
        self
    }

    pub fn config(&self) -> &DatabaseConfig {
        &self.config
    }

    pub fn should_run_migrations(&self) -> bool {
        self.run_migrations
    }

    /// PgPool を作成する。`DATABASE_URL` 環境変数があればそちらを優先する。
    pub async fn connect(&self) -> Result<sqlx::PgPool, sqlx::Error> {
        let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| self.config.url.clone());
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(self.config.max_connections)
            .min_connections(self.config.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(self.config.connect_timeout_secs))
            .connect(&url)
            .await?;
        Ok(pool)
    }
}
