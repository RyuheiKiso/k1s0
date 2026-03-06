use std::collections::HashMap;

use chrono::Utc;

use crate::domain::entity::event::{EventStream, Snapshot, StoredEvent};
use crate::domain::repository::{EventRepository, EventStreamRepository, SnapshotRepository};

pub struct InMemoryEventStreamRepository {
    streams: tokio::sync::RwLock<HashMap<String, EventStream>>,
}

impl InMemoryEventStreamRepository {
    pub fn new() -> Self {
        Self {
            streams: tokio::sync::RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl EventStreamRepository for InMemoryEventStreamRepository {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<EventStream>> {
        let streams = self.streams.read().await;
        Ok(streams.get(id).cloned())
    }

    async fn list_all(&self, page: u32, page_size: u32) -> anyhow::Result<(Vec<EventStream>, u64)> {
        let streams = self.streams.read().await;
        let all: Vec<EventStream> = streams.values().cloned().collect();
        let total = all.len() as u64;
        let page = page.max(1);
        let page_size = page_size.max(1).min(200);
        let offset = ((page - 1) * page_size) as usize;
        let paged: Vec<EventStream> = all
            .into_iter()
            .skip(offset)
            .take(page_size as usize)
            .collect();
        Ok((paged, total))
    }

    async fn create(&self, stream: &EventStream) -> anyhow::Result<()> {
        let mut streams = self.streams.write().await;
        streams.insert(stream.id.clone(), stream.clone());
        Ok(())
    }

    async fn update_version(&self, id: &str, new_version: i64) -> anyhow::Result<()> {
        let mut streams = self.streams.write().await;
        if let Some(stream) = streams.get_mut(id) {
            stream.current_version = new_version;
            stream.updated_at = Utc::now();
        }
        Ok(())
    }

    async fn delete(&self, id: &str) -> anyhow::Result<bool> {
        let mut streams = self.streams.write().await;
        Ok(streams.remove(id).is_some())
    }
}

pub struct InMemoryEventRepository {
    events: tokio::sync::RwLock<Vec<StoredEvent>>,
    sequence_counter: tokio::sync::RwLock<u64>,
}

impl InMemoryEventRepository {
    pub fn new() -> Self {
        Self {
            events: tokio::sync::RwLock::new(Vec::new()),
            sequence_counter: tokio::sync::RwLock::new(0),
        }
    }
}

#[async_trait::async_trait]
impl EventRepository for InMemoryEventRepository {
    async fn append(
        &self,
        _stream_id: &str,
        events: Vec<StoredEvent>,
    ) -> anyhow::Result<Vec<StoredEvent>> {
        let mut all_events = self.events.write().await;
        let mut counter = self.sequence_counter.write().await;
        let mut result = Vec::new();
        for mut event in events {
            *counter += 1;
            event.sequence = *counter;
            result.push(event.clone());
            all_events.push(event);
        }
        Ok(result)
    }

    async fn find_by_stream(
        &self,
        stream_id: &str,
        from_version: i64,
        to_version: Option<i64>,
        event_type: Option<String>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<StoredEvent>, u64)> {
        let all_events = self.events.read().await;
        let filtered: Vec<_> = all_events
            .iter()
            .filter(|e| {
                e.stream_id == stream_id
                    && e.version >= from_version
                    && to_version.is_none_or(|tv| e.version <= tv)
                    && event_type.as_ref().is_none_or(|et| e.event_type == *et)
            })
            .cloned()
            .collect();
        let total = filtered.len() as u64;
        let offset = ((page - 1) * page_size) as usize;
        let paged: Vec<_> = filtered
            .into_iter()
            .skip(offset)
            .take(page_size as usize)
            .collect();
        Ok((paged, total))
    }

    async fn find_all(
        &self,
        event_type: Option<String>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<StoredEvent>, u64)> {
        let all_events = self.events.read().await;
        let filtered: Vec<_> = all_events
            .iter()
            .filter(|e| event_type.as_ref().is_none_or(|et| e.event_type == *et))
            .cloned()
            .collect();
        let total = filtered.len() as u64;
        let page = page.max(1);
        let page_size = page_size.max(1).min(200);
        let offset = ((page - 1) * page_size) as usize;
        let paged: Vec<_> = filtered
            .into_iter()
            .skip(offset)
            .take(page_size as usize)
            .collect();
        Ok((paged, total))
    }

    async fn find_by_sequence(
        &self,
        stream_id: &str,
        sequence: u64,
    ) -> anyhow::Result<Option<StoredEvent>> {
        let all_events = self.events.read().await;
        Ok(all_events
            .iter()
            .find(|e| e.stream_id == stream_id && e.sequence == sequence)
            .cloned())
    }

    async fn delete_by_stream(&self, stream_id: &str) -> anyhow::Result<u64> {
        let mut all_events = self.events.write().await;
        let before = all_events.len();
        all_events.retain(|e| e.stream_id != stream_id);
        Ok((before - all_events.len()) as u64)
    }
}

pub struct InMemorySnapshotRepository {
    snapshots: tokio::sync::RwLock<Vec<Snapshot>>,
}

impl InMemorySnapshotRepository {
    pub fn new() -> Self {
        Self {
            snapshots: tokio::sync::RwLock::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl SnapshotRepository for InMemorySnapshotRepository {
    async fn create(&self, snapshot: &Snapshot) -> anyhow::Result<()> {
        let mut snapshots = self.snapshots.write().await;
        snapshots.push(snapshot.clone());
        Ok(())
    }

    async fn find_latest(&self, stream_id: &str) -> anyhow::Result<Option<Snapshot>> {
        let snapshots = self.snapshots.read().await;
        Ok(snapshots
            .iter()
            .filter(|s| s.stream_id == stream_id)
            .max_by_key(|s| s.snapshot_version)
            .cloned())
    }

    async fn delete_by_stream(&self, stream_id: &str) -> anyhow::Result<u64> {
        let mut snapshots = self.snapshots.write().await;
        let before = snapshots.len();
        snapshots.retain(|s| s.stream_id != stream_id);
        Ok((before - snapshots.len()) as u64)
    }
}
