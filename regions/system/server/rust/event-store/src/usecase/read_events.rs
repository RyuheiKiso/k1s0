use std::sync::Arc;

use crate::domain::entity::event::{PaginationInfo, StoredEvent};
use crate::domain::repository::{EventRepository, EventStreamRepository};

#[derive(Debug, Clone)]
pub struct ReadEventsInput {
    pub stream_id: String,
    pub from_version: i64,
    pub to_version: Option<i64>,
    pub event_type: Option<String>,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone)]
pub struct ReadEventsOutput {
    pub stream_id: String,
    pub events: Vec<StoredEvent>,
    pub current_version: i64,
    pub pagination: PaginationInfo,
}

#[derive(Debug, thiserror::Error)]
pub enum ReadEventsError {
    #[error("stream not found: {0}")]
    StreamNotFound(String),
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct ReadEventsUseCase {
    stream_repo: Arc<dyn EventStreamRepository>,
    event_repo: Arc<dyn EventRepository>,
}

impl ReadEventsUseCase {
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
        input: &ReadEventsInput,
    ) -> Result<ReadEventsOutput, ReadEventsError> {
        let stream = self
            .stream_repo
            .find_by_id(&input.stream_id)
            .await
            .map_err(|e| ReadEventsError::Internal(e.to_string()))?;

        let stream = stream.ok_or_else(|| ReadEventsError::StreamNotFound(input.stream_id.clone()))?;

        let page_size = input.page_size.min(200).max(1);
        let page = input.page.max(1);

        let (events, total_count) = self
            .event_repo
            .find_by_stream(
                &input.stream_id,
                input.from_version,
                input.to_version,
                input.event_type.clone(),
                page,
                page_size,
            )
            .await
            .map_err(|e| ReadEventsError::Internal(e.to_string()))?;

        let has_next = (page as u64) * (page_size as u64) < total_count;

        Ok(ReadEventsOutput {
            stream_id: input.stream_id.clone(),
            events,
            current_version: stream.current_version,
            pagination: PaginationInfo {
                total_count,
                page,
                page_size,
                has_next,
            },
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

    fn make_event(version: i64) -> StoredEvent {
        StoredEvent::new(
            "order-001".to_string(),
            version as u64,
            "OrderPlaced".to_string(),
            version,
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
            .expect_find_by_stream()
            .returning(|_, _, _, _, _, _| Ok((vec![make_event(1), make_event(2)], 2)));

        let uc = ReadEventsUseCase::new(Arc::new(stream_repo), Arc::new(event_repo));
        let input = ReadEventsInput {
            stream_id: "order-001".to_string(),
            from_version: 1,
            to_version: None,
            event_type: None,
            page: 1,
            page_size: 50,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.events.len(), 2);
        assert_eq!(output.current_version, 3);
        assert_eq!(output.pagination.total_count, 2);
        assert!(!output.pagination.has_next);
    }

    #[tokio::test]
    async fn stream_not_found() {
        let mut stream_repo = MockEventStreamRepository::new();
        let event_repo = MockEventRepository::new();

        stream_repo
            .expect_find_by_id()
            .returning(|_| Ok(None));

        let uc = ReadEventsUseCase::new(Arc::new(stream_repo), Arc::new(event_repo));
        let input = ReadEventsInput {
            stream_id: "order-999".to_string(),
            from_version: 1,
            to_version: None,
            event_type: None,
            page: 1,
            page_size: 50,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ReadEventsError::StreamNotFound(_)
        ));
    }

    #[tokio::test]
    async fn internal_error() {
        let mut stream_repo = MockEventStreamRepository::new();
        let event_repo = MockEventRepository::new();

        stream_repo
            .expect_find_by_id()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = ReadEventsUseCase::new(Arc::new(stream_repo), Arc::new(event_repo));
        let input = ReadEventsInput {
            stream_id: "order-001".to_string(),
            from_version: 1,
            to_version: None,
            event_type: None,
            page: 1,
            page_size: 50,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ReadEventsError::Internal(msg) => assert!(msg.contains("db error")),
            e => panic!("unexpected error: {e}"),
        }
    }

    #[tokio::test]
    async fn pagination_has_next() {
        let mut stream_repo = MockEventStreamRepository::new();
        let mut event_repo = MockEventRepository::new();

        stream_repo
            .expect_find_by_id()
            .returning(|_| Ok(Some(make_stream())));

        event_repo
            .expect_find_by_stream()
            .returning(|_, _, _, _, _, _| Ok((vec![make_event(1)], 3)));

        let uc = ReadEventsUseCase::new(Arc::new(stream_repo), Arc::new(event_repo));
        let input = ReadEventsInput {
            stream_id: "order-001".to_string(),
            from_version: 1,
            to_version: None,
            event_type: None,
            page: 1,
            page_size: 1,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.pagination.has_next);
    }
}
