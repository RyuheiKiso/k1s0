use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::entity::event_record::EventRecord;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait EventRecordRepository: Send + Sync {
    async fn create(&self, record: &EventRecord) -> anyhow::Result<()>;
    #[allow(dead_code)]
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<EventRecord>>;
    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        domain: Option<String>,
        event_type: Option<String>,
        source: Option<String>,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        status: Option<String>,
    ) -> anyhow::Result<(Vec<EventRecord>, u64)>;
    async fn find_by_correlation_id(
        &self,
        correlation_id: String,
    ) -> anyhow::Result<Vec<EventRecord>>;
}
