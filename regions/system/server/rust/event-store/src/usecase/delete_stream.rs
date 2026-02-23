use std::sync::Arc;

use crate::domain::repository::{EventRepository, EventStreamRepository, SnapshotRepository};

#[derive(Debug, Clone)]
pub struct DeleteStreamInput {
    pub stream_id: String,
}

#[derive(Debug, Clone)]
pub struct DeleteStreamOutput {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteStreamError {
    #[error("stream not found: {0}")]
    StreamNotFound(String),
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct DeleteStreamUseCase {
    stream_repo: Arc<dyn EventStreamRepository>,
    event_repo: Arc<dyn EventRepository>,
    snapshot_repo: Arc<dyn SnapshotRepository>,
}

impl DeleteStreamUseCase {
    pub fn new(
        stream_repo: Arc<dyn EventStreamRepository>,
        event_repo: Arc<dyn EventRepository>,
        snapshot_repo: Arc<dyn SnapshotRepository>,
    ) -> Self {
        Self {
            stream_repo,
            event_repo,
            snapshot_repo,
        }
    }

    pub async fn execute(
        &self,
        input: &DeleteStreamInput,
    ) -> Result<DeleteStreamOutput, DeleteStreamError> {
        let stream = self
            .stream_repo
            .find_by_id(&input.stream_id)
            .await
            .map_err(|e| DeleteStreamError::Internal(e.to_string()))?;

        if stream.is_none() {
            return Err(DeleteStreamError::StreamNotFound(
                input.stream_id.clone(),
            ));
        }

        // Delete snapshots first, then events, then the stream
        self.snapshot_repo
            .delete_by_stream(&input.stream_id)
            .await
            .map_err(|e| DeleteStreamError::Internal(e.to_string()))?;

        self.event_repo
            .delete_by_stream(&input.stream_id)
            .await
            .map_err(|e| DeleteStreamError::Internal(e.to_string()))?;

        self.stream_repo
            .delete(&input.stream_id)
            .await
            .map_err(|e| DeleteStreamError::Internal(e.to_string()))?;

        Ok(DeleteStreamOutput {
            success: true,
            message: format!(
                "stream {} and all related data deleted",
                input.stream_id
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::event::EventStream;
    use crate::domain::repository::event_repository::{
        MockEventRepository, MockEventStreamRepository, MockSnapshotRepository,
    };

    fn make_stream() -> EventStream {
        EventStream {
            id: "order-001".to_string(),
            aggregate_type: "Order".to_string(),
            current_version: 3,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn success() {
        let mut stream_repo = MockEventStreamRepository::new();
        let mut event_repo = MockEventRepository::new();
        let mut snapshot_repo = MockSnapshotRepository::new();

        stream_repo
            .expect_find_by_id()
            .returning(|_| Ok(Some(make_stream())));
        snapshot_repo
            .expect_delete_by_stream()
            .returning(|_| Ok(2));
        event_repo
            .expect_delete_by_stream()
            .returning(|_| Ok(5));
        stream_repo.expect_delete().returning(|_| Ok(true));

        let uc = DeleteStreamUseCase::new(
            Arc::new(stream_repo),
            Arc::new(event_repo),
            Arc::new(snapshot_repo),
        );
        let input = DeleteStreamInput {
            stream_id: "order-001".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.success);
        assert!(output.message.contains("order-001"));
    }

    #[tokio::test]
    async fn stream_not_found() {
        let mut stream_repo = MockEventStreamRepository::new();
        let event_repo = MockEventRepository::new();
        let snapshot_repo = MockSnapshotRepository::new();

        stream_repo
            .expect_find_by_id()
            .returning(|_| Ok(None));

        let uc = DeleteStreamUseCase::new(
            Arc::new(stream_repo),
            Arc::new(event_repo),
            Arc::new(snapshot_repo),
        );
        let input = DeleteStreamInput {
            stream_id: "order-999".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DeleteStreamError::StreamNotFound(_)
        ));
    }

    #[tokio::test]
    async fn internal_error() {
        let mut stream_repo = MockEventStreamRepository::new();
        let event_repo = MockEventRepository::new();
        let snapshot_repo = MockSnapshotRepository::new();

        stream_repo
            .expect_find_by_id()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = DeleteStreamUseCase::new(
            Arc::new(stream_repo),
            Arc::new(event_repo),
            Arc::new(snapshot_repo),
        );
        let input = DeleteStreamInput {
            stream_id: "order-001".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            DeleteStreamError::Internal(msg) => assert!(msg.contains("db error")),
            e => panic!("unexpected error: {e}"),
        }
    }
}
