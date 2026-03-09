use std::sync::Arc;

use crate::domain::entity::event::{PaginationInfo, StoredEvent};
use crate::domain::repository::EventRepository;

#[derive(Debug, Clone)]
pub struct ListEventsInput {
    pub event_type: Option<String>,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone)]
pub struct ListEventsOutput {
    pub events: Vec<StoredEvent>,
    pub pagination: PaginationInfo,
}

#[derive(Debug, thiserror::Error)]
pub enum ListEventsError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct ListEventsUseCase {
    event_repo: Arc<dyn EventRepository>,
}

impl ListEventsUseCase {
    pub fn new(event_repo: Arc<dyn EventRepository>) -> Self {
        Self { event_repo }
    }

    pub async fn execute(
        &self,
        input: &ListEventsInput,
    ) -> Result<ListEventsOutput, ListEventsError> {
        let page = input.page.max(1);
        let page_size = input.page_size.max(1).min(200);

        let (events, total_count) = self
            .event_repo
            .find_all(input.event_type.clone(), page, page_size)
            .await
            .map_err(|e| ListEventsError::Internal(e.to_string()))?;

        let has_next = (page as u64) * (page_size as u64) < total_count;

        Ok(ListEventsOutput {
            events,
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
    use crate::domain::entity::event::{EventMetadata, StoredEvent};
    use crate::domain::repository::event_repository::MockEventRepository;

    fn make_event(seq: u64) -> StoredEvent {
        StoredEvent::new(
            "order-001".to_string(),
            seq,
            "OrderPlaced".to_string(),
            seq as i64,
            serde_json::json!({}),
            EventMetadata::new(None, None, None),
        )
    }

    #[tokio::test]
    async fn success() {
        let mut event_repo = MockEventRepository::new();
        event_repo
            .expect_find_all()
            .returning(|_, _, _| Ok((vec![make_event(1), make_event(2)], 2)));

        let uc = ListEventsUseCase::new(Arc::new(event_repo));
        let input = ListEventsInput {
            event_type: None,
            page: 1,
            page_size: 50,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.events.len(), 2);
        assert_eq!(output.pagination.total_count, 2);
        assert!(!output.pagination.has_next);
    }

    #[tokio::test]
    async fn with_event_type_filter() {
        let mut event_repo = MockEventRepository::new();
        event_repo
            .expect_find_all()
            .withf(|et, _, _| et.as_deref() == Some("OrderPlaced"))
            .returning(|_, _, _| Ok((vec![make_event(1)], 1)));

        let uc = ListEventsUseCase::new(Arc::new(event_repo));
        let input = ListEventsInput {
            event_type: Some("OrderPlaced".to_string()),
            page: 1,
            page_size: 50,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().events.len(), 1);
    }

    #[tokio::test]
    async fn pagination_has_next() {
        let mut event_repo = MockEventRepository::new();
        event_repo
            .expect_find_all()
            .returning(|_, _, _| Ok((vec![make_event(1)], 3)));

        let uc = ListEventsUseCase::new(Arc::new(event_repo));
        let input = ListEventsInput {
            event_type: None,
            page: 1,
            page_size: 1,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        assert!(result.unwrap().pagination.has_next);
    }

    #[tokio::test]
    async fn internal_error() {
        let mut event_repo = MockEventRepository::new();
        event_repo
            .expect_find_all()
            .returning(|_, _, _| Err(anyhow::anyhow!("db error")));

        let uc = ListEventsUseCase::new(Arc::new(event_repo));
        let input = ListEventsInput {
            event_type: None,
            page: 1,
            page_size: 50,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ListEventsError::Internal(msg) => assert!(msg.contains("db error")),
        }
    }
}
