use async_trait::async_trait;

use crate::domain::entity::event::{EventStream, Snapshot, StoredEvent};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait EventStreamRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<EventStream>>;
    async fn list_all(&self, page: u32, page_size: u32) -> anyhow::Result<(Vec<EventStream>, u64)>;
    async fn create(&self, stream: &EventStream) -> anyhow::Result<()>;
    async fn update_version(&self, id: &str, new_version: i64) -> anyhow::Result<()>;
    async fn delete(&self, id: &str) -> anyhow::Result<bool>;
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait EventRepository: Send + Sync {
    async fn append(
        &self,
        stream_id: &str,
        events: Vec<StoredEvent>,
    ) -> anyhow::Result<Vec<StoredEvent>>;

    async fn find_by_stream(
        &self,
        stream_id: &str,
        from_version: i64,
        to_version: Option<i64>,
        event_type: Option<String>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<StoredEvent>, u64)>;

    async fn find_all(
        &self,
        event_type: Option<String>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<StoredEvent>, u64)>;

    async fn find_by_sequence(
        &self,
        stream_id: &str,
        sequence: u64,
    ) -> anyhow::Result<Option<StoredEvent>>;

    async fn delete_by_stream(&self, stream_id: &str) -> anyhow::Result<u64>;
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SnapshotRepository: Send + Sync {
    async fn create(&self, snapshot: &Snapshot) -> anyhow::Result<()>;
    async fn find_latest(&self, stream_id: &str) -> anyhow::Result<Option<Snapshot>>;
    async fn delete_by_stream(&self, stream_id: &str) -> anyhow::Result<u64>;
}
