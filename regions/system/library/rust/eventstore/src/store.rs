use async_trait::async_trait;

use crate::{EventEnvelope, EventStoreError, StreamId};

#[async_trait]
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait EventStore: Send + Sync {
    async fn append(
        &self,
        stream_id: &StreamId,
        events: Vec<EventEnvelope>,
        expected_version: Option<u64>,
    ) -> Result<u64, EventStoreError>;

    async fn load(&self, stream_id: &StreamId) -> Result<Vec<EventEnvelope>, EventStoreError>;

    async fn load_from(
        &self,
        stream_id: &StreamId,
        from_version: u64,
    ) -> Result<Vec<EventEnvelope>, EventStoreError>;

    async fn exists(&self, stream_id: &StreamId) -> Result<bool, EventStoreError>;

    async fn current_version(&self, stream_id: &StreamId) -> Result<u64, EventStoreError>;
}
