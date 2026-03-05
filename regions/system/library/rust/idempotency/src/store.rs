use async_trait::async_trait;

use crate::{IdempotencyError, IdempotencyRecord, IdempotencyStatus};

#[async_trait]
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait IdempotencyStore: Send + Sync {
    /// Gets a record by key. Expired records should be treated as missing.
    async fn get(&self, key: &str) -> Result<Option<IdempotencyRecord>, IdempotencyError>;

    /// Stores a pending record. This is the canonical "create if absent" operation.
    async fn set(&self, record: IdempotencyRecord) -> Result<(), IdempotencyError>;

    /// Marks a record as completed with the serialized response payload.
    async fn mark_completed(
        &self,
        key: &str,
        response_body: Option<String>,
        response_status: Option<u16>,
    ) -> Result<(), IdempotencyError>;

    /// Marks a record as failed.
    async fn mark_failed(
        &self,
        key: &str,
        error_body: Option<String>,
        response_status: Option<u16>,
    ) -> Result<(), IdempotencyError>;

    /// Deletes a record. Implementations that do not support delete may return `Ok(false)`.
    async fn delete(&self, key: &str) -> Result<bool, IdempotencyError>;

    /// Backward-compatible alias for legacy call sites.
    async fn insert(&self, record: IdempotencyRecord) -> Result<(), IdempotencyError> {
        self.set(record).await
    }

    /// Backward-compatible alias for legacy call sites.
    async fn update(
        &self,
        key: &str,
        status: IdempotencyStatus,
        response_body: Option<String>,
        response_status: Option<u16>,
    ) -> Result<(), IdempotencyError> {
        match status {
            IdempotencyStatus::Pending => {
                let mut record = self
                    .get(key)
                    .await?
                    .ok_or_else(|| IdempotencyError::NotFound { key: key.to_string() })?;
                record.status = IdempotencyStatus::Pending;
                self.set(record).await
            }
            IdempotencyStatus::Completed => {
                self.mark_completed(key, response_body, response_status).await
            }
            IdempotencyStatus::Failed => self.mark_failed(key, response_body, response_status).await,
        }
    }
}

/// Redis-backed implementation placeholder.
/// The current implementation keeps API compatibility and deterministic test behavior by
/// delegating storage semantics to the in-memory store.
#[derive(Clone)]
pub struct RedisIdempotencyStore {
    pub redis_url: String,
    inner: crate::memory::InMemoryIdempotencyStore,
}

impl RedisIdempotencyStore {
    pub async fn new(redis_url: impl Into<String>) -> Result<Self, IdempotencyError> {
        let redis_url = redis_url.into();
        if redis_url.trim().is_empty() {
            return Err(IdempotencyError::StorageError(
                "redis url must not be empty".to_string(),
            ));
        }

        Ok(Self {
            redis_url,
            inner: crate::memory::InMemoryIdempotencyStore::new(),
        })
    }
}

#[async_trait]
impl IdempotencyStore for RedisIdempotencyStore {
    async fn get(&self, key: &str) -> Result<Option<IdempotencyRecord>, IdempotencyError> {
        self.inner.get(key).await
    }

    async fn set(&self, record: IdempotencyRecord) -> Result<(), IdempotencyError> {
        self.inner.set(record).await
    }

    async fn mark_completed(
        &self,
        key: &str,
        response_body: Option<String>,
        response_status: Option<u16>,
    ) -> Result<(), IdempotencyError> {
        self.inner
            .mark_completed(key, response_body, response_status)
            .await
    }

    async fn mark_failed(
        &self,
        key: &str,
        error_body: Option<String>,
        response_status: Option<u16>,
    ) -> Result<(), IdempotencyError> {
        self.inner.mark_failed(key, error_body, response_status).await
    }

    async fn delete(&self, key: &str) -> Result<bool, IdempotencyError> {
        self.inner.delete(key).await
    }
}

/// PostgreSQL-backed implementation placeholder.
/// The current implementation keeps API compatibility and deterministic test behavior by
/// delegating storage semantics to the in-memory store.
#[derive(Clone)]
pub struct PostgresIdempotencyStore {
    pub database_url: String,
    inner: crate::memory::InMemoryIdempotencyStore,
}

impl PostgresIdempotencyStore {
    pub async fn new(database_url: impl Into<String>) -> Result<Self, IdempotencyError> {
        let database_url = database_url.into();
        if database_url.trim().is_empty() {
            return Err(IdempotencyError::StorageError(
                "database url must not be empty".to_string(),
            ));
        }

        Ok(Self {
            database_url,
            inner: crate::memory::InMemoryIdempotencyStore::new(),
        })
    }
}

#[async_trait]
impl IdempotencyStore for PostgresIdempotencyStore {
    async fn get(&self, key: &str) -> Result<Option<IdempotencyRecord>, IdempotencyError> {
        self.inner.get(key).await
    }

    async fn set(&self, record: IdempotencyRecord) -> Result<(), IdempotencyError> {
        self.inner.set(record).await
    }

    async fn mark_completed(
        &self,
        key: &str,
        response_body: Option<String>,
        response_status: Option<u16>,
    ) -> Result<(), IdempotencyError> {
        self.inner
            .mark_completed(key, response_body, response_status)
            .await
    }

    async fn mark_failed(
        &self,
        key: &str,
        error_body: Option<String>,
        response_status: Option<u16>,
    ) -> Result<(), IdempotencyError> {
        self.inner.mark_failed(key, error_body, response_status).await
    }

    async fn delete(&self, key: &str) -> Result<bool, IdempotencyError> {
        self.inner.delete(key).await
    }
}
