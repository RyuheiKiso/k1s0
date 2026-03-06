use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;

use crate::domain::entity::flow_kpi::FlowKpi;

pub struct KpiCache {
    inner: Cache<String, Arc<FlowKpi>>,
}

impl KpiCache {
    pub fn new(max_capacity: u64, ttl_secs: u64) -> Self {
        let inner = Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(Duration::from_secs(ttl_secs))
            .build();
        Self { inner }
    }

    pub async fn get(&self, key: &str) -> Option<Arc<FlowKpi>> {
        self.inner.get(key).await
    }

    pub async fn insert(&self, key: String, kpi: Arc<FlowKpi>) {
        self.inner.insert(key, kpi).await;
    }

    pub async fn invalidate(&self, key: &str) {
        self.inner.invalidate(key).await;
    }

    pub async fn invalidate_all(&self) {
        self.inner.invalidate_all();
        self.inner.run_pending_tasks().await;
    }
}
