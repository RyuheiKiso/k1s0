use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{EventStoreError, StreamId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub stream_id: String,
    pub version: u64,
    pub state: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[async_trait]
pub trait SnapshotStore: Send + Sync {
    async fn save_snapshot(&self, snapshot: Snapshot) -> Result<(), EventStoreError>;
    async fn load_snapshot(
        &self,
        stream_id: &StreamId,
    ) -> Result<Option<Snapshot>, EventStoreError>;
}
