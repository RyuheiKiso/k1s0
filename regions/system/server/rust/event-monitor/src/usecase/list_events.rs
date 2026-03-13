use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::domain::entity::event_record::EventRecord;
use crate::domain::repository::EventRecordRepository;

#[derive(Debug, Clone)]
pub struct ListEventsInput {
    pub page: u32,
    pub page_size: u32,
    pub domain: Option<String>,
    pub event_type: Option<String>,
    pub source: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub status: Option<String>,
}

#[derive(Debug)]
pub struct ListEventsOutput {
    pub events: Vec<EventRecord>,
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub has_next: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum ListEventsError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct ListEventsUseCase {
    repo: Arc<dyn EventRecordRepository>,
}

impl ListEventsUseCase {
    pub fn new(repo: Arc<dyn EventRecordRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, input: &ListEventsInput) -> Result<ListEventsOutput, ListEventsError> {
        let (events, total_count) = self
            .repo
            .find_all_paginated(
                input.page,
                input.page_size,
                input.domain.clone(),
                input.event_type.clone(),
                input.source.clone(),
                input.from,
                input.to,
                input.status.clone(),
            )
            .await
            .map_err(|e| ListEventsError::Internal(e.to_string()))?;

        let has_next = (input.page as u64 * input.page_size as u64) < total_count;

        Ok(ListEventsOutput {
            events,
            total_count,
            page: input.page,
            page_size: input.page_size,
            has_next,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::event_record_repository::MockEventRecordRepository;
    use chrono::Utc;

    fn make_event(corr_id: &str) -> EventRecord {
        EventRecord::new(
            corr_id.to_string(),
            "OrderCreated".to_string(),
            "order-service".to_string(),
            "service.order".to_string(),
            "trace-123".to_string(),
            Utc::now(),
        )
    }

    #[tokio::test]
    async fn paginated() {
        let mut mock = MockEventRecordRepository::new();
        mock.expect_find_all_paginated()
            .returning(|_, _, _, _, _, _, _, _| {
                Ok((vec![make_event("corr-1"), make_event("corr-2")], 5))
            });

        let uc = ListEventsUseCase::new(Arc::new(mock));
        let input = ListEventsInput {
            page: 1,
            page_size: 2,
            domain: None,
            event_type: None,
            source: None,
            from: None,
            to: None,
            status: None,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.events.len(), 2);
        assert_eq!(output.total_count, 5);
        assert!(output.has_next);
    }

    #[tokio::test]
    async fn empty() {
        let mut mock = MockEventRecordRepository::new();
        mock.expect_find_all_paginated()
            .returning(|_, _, _, _, _, _, _, _| Ok((vec![], 0)));

        let uc = ListEventsUseCase::new(Arc::new(mock));
        let input = ListEventsInput {
            page: 1,
            page_size: 10,
            domain: None,
            event_type: None,
            source: None,
            from: None,
            to: None,
            status: None,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.events.is_empty());
        assert!(!output.has_next);
    }
}
