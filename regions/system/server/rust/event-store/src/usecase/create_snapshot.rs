use std::sync::Arc;

use crate::domain::entity::event::Snapshot;
use crate::domain::repository::{EventStreamRepository, SnapshotRepository};

#[derive(Debug, Clone)]
pub struct CreateSnapshotInput {
    pub stream_id: String,
    pub snapshot_version: i64,
    pub aggregate_type: String,
    pub state: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct CreateSnapshotOutput {
    pub id: String,
    pub stream_id: String,
    pub snapshot_version: i64,
    pub aggregate_type: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateSnapshotError {
    #[error("stream not found: {0}")]
    StreamNotFound(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct CreateSnapshotUseCase {
    stream_repo: Arc<dyn EventStreamRepository>,
    snapshot_repo: Arc<dyn SnapshotRepository>,
}

impl CreateSnapshotUseCase {
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
        input: &CreateSnapshotInput,
    ) -> Result<CreateSnapshotOutput, CreateSnapshotError> {
        let stream = self
            .stream_repo
            .find_by_id(&input.stream_id)
            .await
            .map_err(|e| CreateSnapshotError::Internal(e.to_string()))?;

        let stream =
            stream.ok_or_else(|| CreateSnapshotError::StreamNotFound(input.stream_id.clone()))?;

        if input.snapshot_version > stream.current_version {
            return Err(CreateSnapshotError::Validation(format!(
                "snapshot_version {} exceeds current stream version {}",
                input.snapshot_version, stream.current_version
            )));
        }

        let snap_id = format!("snap_{}", uuid::Uuid::new_v4().simple());
        let snapshot = Snapshot::new(
            snap_id.clone(),
            input.stream_id.clone(),
            input.snapshot_version,
            input.aggregate_type.clone(),
            input.state.clone(),
        );

        self.snapshot_repo
            .create(&snapshot)
            .await
            .map_err(|e| CreateSnapshotError::Internal(e.to_string()))?;

        Ok(CreateSnapshotOutput {
            id: snap_id,
            stream_id: input.stream_id.clone(),
            snapshot_version: input.snapshot_version,
            aggregate_type: input.aggregate_type.clone(),
            created_at: snapshot.created_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::event::EventStream;
    use crate::domain::repository::event_repository::MockEventStreamRepository;
    use crate::domain::repository::event_repository::MockSnapshotRepository;

    fn make_stream() -> EventStream {
        EventStream {
            id: "order-001".to_string(),
            aggregate_type: "Order".to_string(),
            current_version: 5,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    fn make_input() -> CreateSnapshotInput {
        CreateSnapshotInput {
            stream_id: "order-001".to_string(),
            snapshot_version: 3,
            aggregate_type: "Order".to_string(),
            state: serde_json::json!({"status": "shipped"}),
        }
    }

    #[tokio::test]
    async fn success() {
        let mut stream_repo = MockEventStreamRepository::new();
        let mut snapshot_repo = MockSnapshotRepository::new();

        stream_repo
            .expect_find_by_id()
            .returning(|_| Ok(Some(make_stream())));
        snapshot_repo.expect_create().returning(|_| Ok(()));

        let uc = CreateSnapshotUseCase::new(Arc::new(stream_repo), Arc::new(snapshot_repo));
        let result = uc.execute(&make_input()).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.stream_id, "order-001");
        assert_eq!(output.snapshot_version, 3);
        assert!(output.id.starts_with("snap_"));
    }

    #[tokio::test]
    async fn stream_not_found() {
        let mut stream_repo = MockEventStreamRepository::new();
        let snapshot_repo = MockSnapshotRepository::new();

        stream_repo
            .expect_find_by_id()
            .returning(|_| Ok(None));

        let uc = CreateSnapshotUseCase::new(Arc::new(stream_repo), Arc::new(snapshot_repo));
        let result = uc.execute(&make_input()).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CreateSnapshotError::StreamNotFound(_)
        ));
    }

    #[tokio::test]
    async fn version_exceeds_current() {
        let mut stream_repo = MockEventStreamRepository::new();
        let snapshot_repo = MockSnapshotRepository::new();

        stream_repo
            .expect_find_by_id()
            .returning(|_| Ok(Some(make_stream())));

        let uc = CreateSnapshotUseCase::new(Arc::new(stream_repo), Arc::new(snapshot_repo));
        let mut input = make_input();
        input.snapshot_version = 10; // exceeds current_version=5
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CreateSnapshotError::Validation(_)
        ));
    }

    #[tokio::test]
    async fn internal_error() {
        let mut stream_repo = MockEventStreamRepository::new();
        let snapshot_repo = MockSnapshotRepository::new();

        stream_repo
            .expect_find_by_id()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = CreateSnapshotUseCase::new(Arc::new(stream_repo), Arc::new(snapshot_repo));
        let result = uc.execute(&make_input()).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CreateSnapshotError::Internal(msg) => assert!(msg.contains("db error")),
            e => panic!("unexpected error: {e}"),
        }
    }
}
