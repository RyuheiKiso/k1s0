use async_trait::async_trait;
use sqlx::PgPool;
use crate::checker::HealthCheck;
use crate::error::HealthError;

pub struct PostgresHealthCheck {
    name: String,
    pool: PgPool,
}

impl PostgresHealthCheck {
    pub fn new(name: impl Into<String>, pool: PgPool) -> Self {
        Self {
            name: name.into(),
            pool,
        }
    }
}

#[async_trait]
impl HealthCheck for PostgresHealthCheck {
    fn name(&self) -> &str {
        &self.name
    }

    async fn check(&self) -> Result<(), HealthError> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|e| HealthError::CheckFailed(format!("PostgreSQL check failed: {}", e)))
    }
}
