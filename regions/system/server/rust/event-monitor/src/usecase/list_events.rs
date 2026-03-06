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
                input.domain.as_deref(),
                input.event_type.as_deref(),
                input.source.as_deref(),
                input.from,
                input.to,
                input.status.as_deref(),
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
