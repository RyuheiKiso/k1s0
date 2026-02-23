use std::sync::Arc;

use crate::domain::entity::event::{EventData, EventMetadata, EventStream, StoredEvent};
use crate::domain::repository::{EventRepository, EventStreamRepository};

#[derive(Debug, Clone)]
pub struct AppendEventsInput {
    pub stream_id: String,
    pub events: Vec<EventData>,
    pub expected_version: i64,
}

#[derive(Debug, Clone)]
pub struct AppendEventsOutput {
    pub stream_id: String,
    pub events: Vec<StoredEvent>,
    pub current_version: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum AppendEventsError {
    #[error("stream not found: {0}")]
    StreamNotFound(String),
    #[error("version conflict for stream {stream_id}: expected {expected}, actual {actual}")]
    VersionConflict {
        stream_id: String,
        expected: i64,
        actual: i64,
    },
    #[error("stream already exists: {0}")]
    StreamAlreadyExists(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct AppendEventsUseCase {
    stream_repo: Arc<dyn EventStreamRepository>,
    event_repo: Arc<dyn EventRepository>,
}

impl AppendEventsUseCase {
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
        input: &AppendEventsInput,
    ) -> Result<AppendEventsOutput, AppendEventsError> {
        if input.events.is_empty() {
            return Err(AppendEventsError::Validation(
                "events must not be empty".to_string(),
            ));
        }

        let stream = self
            .stream_repo
            .find_by_id(&input.stream_id)
            .await
            .map_err(|e| AppendEventsError::Internal(e.to_string()))?;

        // expected_version == -1 means new stream
        if input.expected_version == -1 {
            if stream.is_some() {
                return Err(AppendEventsError::StreamAlreadyExists(
                    input.stream_id.clone(),
                ));
            }
            let new_stream = EventStream::new(input.stream_id.clone(), String::new());
            self.stream_repo
                .create(&new_stream)
                .await
                .map_err(|e| AppendEventsError::Internal(e.to_string()))?;
        } else {
            match &stream {
                None => {
                    return Err(AppendEventsError::StreamNotFound(
                        input.stream_id.clone(),
                    ));
                }
                Some(s) => {
                    if s.current_version != input.expected_version {
                        return Err(AppendEventsError::VersionConflict {
                            stream_id: input.stream_id.clone(),
                            expected: input.expected_version,
                            actual: s.current_version,
                        });
                    }
                }
            }
        }

        let base_version = if input.expected_version == -1 {
            0
        } else {
            input.expected_version
        };

        let stored_events: Vec<StoredEvent> = input
            .events
            .iter()
            .enumerate()
            .map(|(i, data)| {
                StoredEvent::new(
                    input.stream_id.clone(),
                    0, // sequence assigned by storage
                    data.event_type.clone(),
                    base_version + (i as i64) + 1,
                    data.payload.clone(),
                    EventMetadata::new(
                        data.metadata.actor_id.clone(),
                        data.metadata.correlation_id.clone(),
                        data.metadata.causation_id.clone(),
                    ),
                )
            })
            .collect();

        let new_version = base_version + input.events.len() as i64;

        let persisted = self
            .event_repo
            .append(&input.stream_id, stored_events)
            .await
            .map_err(|e| AppendEventsError::Internal(e.to_string()))?;

        self.stream_repo
            .update_version(&input.stream_id, new_version)
            .await
            .map_err(|e| AppendEventsError::Internal(e.to_string()))?;

        Ok(AppendEventsOutput {
            stream_id: input.stream_id.clone(),
            events: persisted,
            current_version: new_version,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::event_repository::{
        MockEventRepository, MockEventStreamRepository,
    };

    fn make_input(stream_id: &str, expected_version: i64) -> AppendEventsInput {
        AppendEventsInput {
            stream_id: stream_id.to_string(),
            events: vec![EventData {
                event_type: "OrderPlaced".to_string(),
                payload: serde_json::json!({"order_id": "o-1"}),
                metadata: EventMetadata::new(Some("user-001".to_string()), None, None),
            }],
            expected_version,
        }
    }

    #[tokio::test]
    async fn success_new_stream() {
        let mut stream_repo = MockEventStreamRepository::new();
        let mut event_repo = MockEventRepository::new();

        stream_repo
            .expect_find_by_id()
            .returning(|_| Ok(None));
        stream_repo.expect_create().returning(|_| Ok(()));
        stream_repo
            .expect_update_version()
            .returning(|_, _| Ok(()));

        event_repo.expect_append().returning(|stream_id, events| {
            Ok(events
                .into_iter()
                .enumerate()
                .map(|(i, mut e)| {
                    e.sequence = (i as u64) + 1;
                    e.stream_id = stream_id.to_string();
                    e
                })
                .collect())
        });

        let uc = AppendEventsUseCase::new(Arc::new(stream_repo), Arc::new(event_repo));
        let input = make_input("order-001", -1);
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.stream_id, "order-001");
        assert_eq!(output.current_version, 1);
        assert_eq!(output.events.len(), 1);
    }

    #[tokio::test]
    async fn success_existing_stream() {
        let mut stream_repo = MockEventStreamRepository::new();
        let mut event_repo = MockEventRepository::new();

        stream_repo.expect_find_by_id().returning(|_| {
            Ok(Some(EventStream {
                id: "order-001".to_string(),
                aggregate_type: "Order".to_string(),
                current_version: 2,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }))
        });
        stream_repo
            .expect_update_version()
            .returning(|_, _| Ok(()));

        event_repo.expect_append().returning(|stream_id, events| {
            Ok(events
                .into_iter()
                .enumerate()
                .map(|(i, mut e)| {
                    e.sequence = (i as u64) + 10;
                    e.stream_id = stream_id.to_string();
                    e
                })
                .collect())
        });

        let uc = AppendEventsUseCase::new(Arc::new(stream_repo), Arc::new(event_repo));
        let input = make_input("order-001", 2);
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.current_version, 3);
    }

    #[tokio::test]
    async fn version_conflict() {
        let mut stream_repo = MockEventStreamRepository::new();
        let event_repo = MockEventRepository::new();

        stream_repo.expect_find_by_id().returning(|_| {
            Ok(Some(EventStream {
                id: "order-001".to_string(),
                aggregate_type: "Order".to_string(),
                current_version: 5,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }))
        });

        let uc = AppendEventsUseCase::new(Arc::new(stream_repo), Arc::new(event_repo));
        let input = make_input("order-001", 2);
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AppendEventsError::VersionConflict {
                expected, actual, ..
            } => {
                assert_eq!(expected, 2);
                assert_eq!(actual, 5);
            }
            e => panic!("unexpected error: {e}"),
        }
    }

    #[tokio::test]
    async fn stream_not_found() {
        let mut stream_repo = MockEventStreamRepository::new();
        let event_repo = MockEventRepository::new();

        stream_repo
            .expect_find_by_id()
            .returning(|_| Ok(None));

        let uc = AppendEventsUseCase::new(Arc::new(stream_repo), Arc::new(event_repo));
        let input = make_input("order-999", 0);
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AppendEventsError::StreamNotFound(_)
        ));
    }

    #[tokio::test]
    async fn stream_already_exists() {
        let mut stream_repo = MockEventStreamRepository::new();
        let event_repo = MockEventRepository::new();

        stream_repo.expect_find_by_id().returning(|_| {
            Ok(Some(EventStream {
                id: "order-001".to_string(),
                aggregate_type: "Order".to_string(),
                current_version: 0,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }))
        });

        let uc = AppendEventsUseCase::new(Arc::new(stream_repo), Arc::new(event_repo));
        let input = make_input("order-001", -1);
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AppendEventsError::StreamAlreadyExists(_)
        ));
    }

    #[tokio::test]
    async fn validation_empty_events() {
        let stream_repo = MockEventStreamRepository::new();
        let event_repo = MockEventRepository::new();

        let uc = AppendEventsUseCase::new(Arc::new(stream_repo), Arc::new(event_repo));
        let input = AppendEventsInput {
            stream_id: "order-001".to_string(),
            events: vec![],
            expected_version: 0,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AppendEventsError::Validation(_)
        ));
    }

    #[tokio::test]
    async fn internal_error() {
        let mut stream_repo = MockEventStreamRepository::new();
        let event_repo = MockEventRepository::new();

        stream_repo
            .expect_find_by_id()
            .returning(|_| Err(anyhow::anyhow!("db connection failed")));

        let uc = AppendEventsUseCase::new(Arc::new(stream_repo), Arc::new(event_repo));
        let input = make_input("order-001", 0);
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AppendEventsError::Internal(msg) => {
                assert!(msg.contains("db connection failed"));
            }
            e => panic!("unexpected error: {e}"),
        }
    }
}
