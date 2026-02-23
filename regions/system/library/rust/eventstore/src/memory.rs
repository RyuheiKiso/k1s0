use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::envelope::EventEnvelope;
use crate::error::EventStoreError;
use crate::snapshot::Snapshot;
use crate::store::EventStore;
use crate::stream::StreamId;

pub struct InMemoryEventStore {
    streams: Arc<RwLock<HashMap<String, Vec<EventEnvelope>>>>,
}

impl InMemoryEventStore {
    pub fn new() -> Self {
        Self {
            streams: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryEventStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventStore for InMemoryEventStore {
    async fn append(
        &self,
        stream_id: &StreamId,
        events: Vec<EventEnvelope>,
        expected_version: Option<u64>,
    ) -> Result<u64, EventStoreError> {
        let mut streams = self.streams.write().await;
        let key = stream_id.to_string();
        let stream = streams.entry(key).or_default();

        let current_version = stream.last().map(|e| e.version).unwrap_or(0);

        if let Some(expected) = expected_version {
            if expected != current_version {
                return Err(EventStoreError::VersionConflict {
                    expected,
                    actual: current_version,
                });
            }
        }

        let mut version = current_version;
        for mut event in events {
            version += 1;
            event.version = version;
            event.stream_id = stream_id.to_string();
            stream.push(event);
        }

        Ok(version)
    }

    async fn load(&self, stream_id: &StreamId) -> Result<Vec<EventEnvelope>, EventStoreError> {
        let streams = self.streams.read().await;
        let key = stream_id.to_string();
        Ok(streams.get(&key).cloned().unwrap_or_default())
    }

    async fn load_from(
        &self,
        stream_id: &StreamId,
        from_version: u64,
    ) -> Result<Vec<EventEnvelope>, EventStoreError> {
        let streams = self.streams.read().await;
        let key = stream_id.to_string();
        Ok(streams
            .get(&key)
            .map(|events| {
                events
                    .iter()
                    .filter(|e| e.version >= from_version)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default())
    }

    async fn exists(&self, stream_id: &StreamId) -> Result<bool, EventStoreError> {
        let streams = self.streams.read().await;
        Ok(streams.contains_key(&stream_id.to_string()))
    }

    async fn current_version(&self, stream_id: &StreamId) -> Result<u64, EventStoreError> {
        let streams = self.streams.read().await;
        let key = stream_id.to_string();
        Ok(streams
            .get(&key)
            .and_then(|events| events.last())
            .map(|e| e.version)
            .unwrap_or(0))
    }
}

pub struct InMemorySnapshotStore {
    snapshots: Arc<RwLock<HashMap<String, Snapshot>>>,
}

impl InMemorySnapshotStore {
    pub fn new() -> Self {
        Self {
            snapshots: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemorySnapshotStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl crate::snapshot::SnapshotStore for InMemorySnapshotStore {
    async fn save_snapshot(&self, snapshot: Snapshot) -> Result<(), EventStoreError> {
        let mut snapshots = self.snapshots.write().await;
        snapshots.insert(snapshot.stream_id.clone(), snapshot);
        Ok(())
    }

    async fn load_snapshot(
        &self,
        stream_id: &StreamId,
    ) -> Result<Option<Snapshot>, EventStoreError> {
        let snapshots = self.snapshots.read().await;
        Ok(snapshots.get(&stream_id.to_string()).cloned())
    }
}
