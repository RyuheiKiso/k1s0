use std::sync::Arc;

use crate::metrics::Metrics;

/// TracedPool wraps a sqlx::PgPool with automatic metrics recording.
pub struct TracedPool {
    inner: sqlx::PgPool,
    metrics: Arc<Metrics>,
}

impl TracedPool {
    /// Creates a new TracedPool wrapping the given pool with metrics.
    pub fn new(pool: sqlx::PgPool, metrics: Arc<Metrics>) -> Self {
        Self {
            inner: pool,
            metrics,
        }
    }

    /// Returns a reference to the underlying pool.
    pub fn inner(&self) -> &sqlx::PgPool {
        &self.inner
    }

    /// Returns a reference to the metrics instance.
    pub fn metrics(&self) -> &Metrics {
        &self.metrics
    }
}
