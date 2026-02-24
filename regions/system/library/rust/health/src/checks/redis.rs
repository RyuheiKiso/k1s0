use async_trait::async_trait;
use deadpool_redis::{Pool, redis::cmd};
use crate::checker::HealthCheck;
use crate::error::HealthError;

pub struct RedisHealthCheck {
    name: String,
    pool: Pool,
}

impl RedisHealthCheck {
    pub fn new(name: impl Into<String>, pool: Pool) -> Self {
        Self {
            name: name.into(),
            pool,
        }
    }
}

#[async_trait]
impl HealthCheck for RedisHealthCheck {
    fn name(&self) -> &str {
        &self.name
    }

    async fn check(&self) -> Result<(), HealthError> {
        let mut conn = self.pool
            .get()
            .await
            .map_err(|e| HealthError::CheckFailed(format!("Redis pool error: {}", e)))?;

        cmd("PING")
            .query_async::<String>(&mut conn)
            .await
            .map(|_| ())
            .map_err(|e| HealthError::CheckFailed(format!("Redis PING failed: {}", e)))
    }
}
