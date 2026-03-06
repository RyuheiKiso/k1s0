use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::entity::event_record::EventRecord;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait EventRecordRepository: Send + Sync {
    async fn create(&self, record: &EventRecord) -> anyhow::Result<()>;
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<EventRecord>>;
    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        domain: Option<&str>,
        event_type: Option<&str>,
        source: Option<&str>,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        status: Option<&str>,
    ) -> anyhow::Result<(Vec<EventRecord>, u64)>;
    async fn find_by_correlation_id(
        &self,
        correlation_id: &str,
    ) -> anyhow::Result<Vec<EventRecord>>;
}
