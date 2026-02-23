use std::sync::Arc;

use crate::domain::entity::event::Snapshot;
use crate::domain::repository::{EventStreamRepository, SnapshotRepository};

#[derive(Debug, Clone)]
pub struct GetLatestSnapshotInput {
    pub stream_id: String,
}

#[derive(Debug, thiserror::Error)]
pub enum GetLatestSnapshotError {
    #[error("stream not found: {0}")]
    StreamNotFound(String),
    #[error("snapshot not found for stream: {0}")]
    SnapshotNotFound(String),
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct GetLatestSnapshotUseCase {
    stream_repo: Arc<dyn EventStreamRepository>,
    snapshot_repo: Arc<dyn SnapshotRepository>,
}

impl GetLatestSnapshotUseCase {
    pub fn new(
        stream_repo: Arc<dyn EventStreamRepository>,
        snapshot_repo: Arc<dyn SnapshotRepository>,
    ) -> Self {
        Self {
            stream_repo,
            snapshot_repo,
        }
    }

    pub async fn execute(
        &self,
        input: &GetLatestSnapshotInput,
    ) -> Result<Snapshot, GetLatestSnapshotError> {
        let stream = self
            .stream_repo
            .find_by_id(&input.stream_id)
            .await
            .map_err(|e| GetLatestSnapshotError::Internal(e.to_string()))?;

        if stream.is_none() {
            return Err(GetLatestSnapshotError::StreamNotFound(
                input.stream_id.clone(),
            ));
        }

        let snapshot = self
            .snapshot_repo
            .find_latest(&input.stream_id)
            .await
            .map_err(|e| GetLatestSnapshotError::Internal(e.to_string()))?;

        snapshot.ok_or_else(|| GetLatestSnapshotError::SnapshotNotFound(input.stream_id.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::event::{EventStream, Snapshot};
    use crate::domain::repository::event_repository::{
        MockEventStreamRepository, MockSnapshotRepository,
    };

    fn make_stream() -> EventStream {
        EventStream {
            id: "order-001".to_string(),
            aggregate_type: "Order".to_string(),
            current_version: 5,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    fn make_snapshot() -> Snapshot {
        Snapshot::new(
            "snap_001".to_string(),
            "order-001".to_string(),
            3,
            "Order".to_string(),
            serde_json::json!({"status": "shipped"}),
        )
    }

    #[tokio::test]
    async fn success() {
        let mut stream_repo = MockEventStreamRepository::new();
        let mut snapshot_repo = MockSnapshotRepository::new();

        stream_repo
            .expect_find_by_id()
            .returning(|_| Ok(Some(make_stream())));
        snapshot_repo
            .expect_find_latest()
            .returning(|_| Ok(Some(make_snapshot())));

        let uc = GetLatestSnapshotUseCase::new(Arc::new(stream_repo), Arc::new(snapshot_repo));
        let input = GetLatestSnapshotInput {
            stream_id: "order-001".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let snap = result.unwrap();
        assert_eq!(snap.id, "snap_001");
        assert_eq!(snap.snapshot_version, 3);
    }

    #[tokio::test]
    async fn stream_not_found() {
        let mut stream_repo = MockEventStreamRepository::new();
        let snapshot_repo = MockSnapshotRepository::new();

        stream_repo
            .expect_find_by_id()
            .returning(|_| Ok(None));

        let uc = GetLatestSnapshotUseCase::new(Arc::new(stream_repo), Arc::new(snapshot_repo));
        let input = GetLatestSnapshotInput {
            stream_id: "order-999".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            GetLatestSnapshotError::StreamNotFound(_)
        ));
    }

    #[tokio::test]
    async fn snapshot_not_found() {
        let mut stream_repo = MockEventStreamRepository::new();
        let mut snapshot_repo = MockSnapshotRepository::new();

        stream_repo
            .expect_find_by_id()
            .returning(|_| Ok(Some(make_stream())));
        snapshot_repo
            .expect_find_latest()
            .returning(|_| Ok(None));

        let uc = GetLatestSnapshotUseCase::new(Arc::new(stream_repo), Arc::new(snapshot_repo));
        let input = GetLatestSnapshotInput {
            stream_id: "order-001".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            GetLatestSnapshotError::SnapshotNotFound(_)
        ));
    }

    #[tokio::test]
    async fn internal_error() {
        let mut stream_repo = MockEventStreamRepository::new();
        let snapshot_repo = MockSnapshotRepository::new();

        stream_repo
            .expect_find_by_id()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = GetLatestSnapshotUseCase::new(Arc::new(stream_repo), Arc::new(snapshot_repo));
        let input = GetLatestSnapshotInput {
            stream_id: "order-001".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            GetLatestSnapshotError::Internal(msg) => assert!(msg.contains("db error")),
            e => panic!("unexpected error: {e}"),
        }
    }
}
