use std::sync::Arc;

use crate::domain::entity::event::StoredEvent;
use crate::domain::repository::{EventRepository, EventStreamRepository};

#[derive(Debug, Clone)]
pub struct ReadEventBySequenceInput {
    pub stream_id: String,
    pub sequence: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum ReadEventBySequenceError {
    #[error("stream not found: {0}")]
    StreamNotFound(String),
    #[error("event not found: stream={stream_id}, sequence={sequence}")]
    EventNotFound { stream_id: String, sequence: u64 },
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct ReadEventBySequenceUseCase {
    stream_repo: Arc<dyn EventStreamRepository>,
    event_repo: Arc<dyn EventRepository>,
}

impl ReadEventBySequenceUseCase {
    pub fn new(
        stream_repo: Arc<dyn EventStreamRepository>,
        event_repo: Arc<dyn EventRepository>,
    ) -> Self {
        Self {
            stream_repo,
            event_repo,
        }
    }

    pub async fn execute(
        &self,
        input: &ReadEventBySequenceInput,
    ) -> Result<StoredEvent, ReadEventBySequenceError> {
        let stream = self
            .stream_repo
            .find_by_id(&input.stream_id)
            .await
            .map_err(|e| ReadEventBySequenceError::Internal(e.to_string()))?;

        if stream.is_none() {
            return Err(ReadEventBySequenceError::StreamNotFound(
                input.stream_id.clone(),
            ));
        }

        let event = self
            .event_repo
            .find_by_sequence(&input.stream_id, input.sequence)
            .await
            .map_err(|e| ReadEventBySequenceError::Internal(e.to_string()))?;

        event.ok_or_else(|| ReadEventBySequenceError::EventNotFound {
            stream_id: input.stream_id.clone(),
            sequence: input.sequence,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::event::{EventMetadata, EventStream, StoredEvent};
    use crate::domain::repository::event_repository::{
        MockEventRepository, MockEventStreamRepository,
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

    fn make_event() -> StoredEvent {
        StoredEvent::new(
            "order-001".to_string(),
            1,
            "OrderPlaced".to_string(),
            1,
            serde_json::json!({}),
            EventMetadata::new(None, None, None),
        )
    }

    #[tokio::test]
    async fn success() {
        let mut stream_repo = MockEventStreamRepository::new();
        let mut event_repo = MockEventRepository::new();

        stream_repo
            .expect_find_by_id()
            .returning(|_| Ok(Some(make_stream())));
        event_repo
            .expect_find_by_sequence()
            .returning(|_, _| Ok(Some(make_event())));

        let uc = ReadEventBySequenceUseCase::new(Arc::new(stream_repo), Arc::new(event_repo));
        let input = ReadEventBySequenceInput {
            stream_id: "order-001".to_string(),
            sequence: 1,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(event.event_type, "OrderPlaced");
    }

    #[tokio::test]
    async fn stream_not_found() {
        let mut stream_repo = MockEventStreamRepository::new();
        let event_repo = MockEventRepository::new();

        stream_repo
            .expect_find_by_id()
            .returning(|_| Ok(None));

        let uc = ReadEventBySequenceUseCase::new(Arc::new(stream_repo), Arc::new(event_repo));
        let input = ReadEventBySequenceInput {
            stream_id: "order-999".to_string(),
            sequence: 1,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ReadEventBySequenceError::StreamNotFound(_)
        ));
    }

    #[tokio::test]
    async fn event_not_found() {
        let mut stream_repo = MockEventStreamRepository::new();
        let mut event_repo = MockEventRepository::new();

        stream_repo
            .expect_find_by_id()
            .returning(|_| Ok(Some(make_stream())));
        event_repo
            .expect_find_by_sequence()
            .returning(|_, _| Ok(None));

        let uc = ReadEventBySequenceUseCase::new(Arc::new(stream_repo), Arc::new(event_repo));
        let input = ReadEventBySequenceInput {
            stream_id: "order-001".to_string(),
            sequence: 999,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ReadEventBySequenceError::EventNotFound { .. }
        ));
    }

    #[tokio::test]
    async fn internal_error() {
        let mut stream_repo = MockEventStreamRepository::new();
        let event_repo = MockEventRepository::new();

        stream_repo
            .expect_find_by_id()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = ReadEventBySequenceUseCase::new(Arc::new(stream_repo), Arc::new(event_repo));
        let input = ReadEventBySequenceInput {
            stream_id: "order-001".to_string(),
            sequence: 1,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ReadEventBySequenceError::Internal(msg) => assert!(msg.contains("db error")),
            e => panic!("unexpected error: {e}"),
        }
    }
}
