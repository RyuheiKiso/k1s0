use std::sync::Arc;

use crate::domain::entity::event::{EventStream, PaginationInfo};
use crate::domain::repository::EventStreamRepository;

#[derive(Debug, Clone)]
pub struct ListStreamsInput {
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone)]
pub struct ListStreamsOutput {
    pub streams: Vec<EventStream>,
    pub pagination: PaginationInfo,
}

#[derive(Debug, thiserror::Error)]
pub enum ListStreamsError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct ListStreamsUseCase {
    stream_repo: Arc<dyn EventStreamRepository>,
}

impl ListStreamsUseCase {
    pub fn new(stream_repo: Arc<dyn EventStreamRepository>) -> Self {
        Self { stream_repo }
    }

    pub async fn execute(
        &self,
        input: &ListStreamsInput,
    ) -> Result<ListStreamsOutput, ListStreamsError> {
        let page = input.page.max(1);
        let page_size = input.page_size.max(1).min(200);

        let (streams, total_count) = self
            .stream_repo
            .list_all(page, page_size)
            .await
            .map_err(|e| ListStreamsError::Internal(e.to_string()))?;

        let has_next = (page as u64) * (page_size as u64) < total_count;

        Ok(ListStreamsOutput {
            streams,
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
    use crate::domain::entity::event::EventStream;
    use crate::domain::repository::event_repository::MockEventStreamRepository;

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
        stream_repo
            .expect_list_all()
            .returning(|_, _| Ok((vec![make_stream()], 1)));

        let uc = ListStreamsUseCase::new(Arc::new(stream_repo));
        let input = ListStreamsInput {
            page: 1,
            page_size: 50,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.streams.len(), 1);
        assert_eq!(output.pagination.total_count, 1);
        assert!(!output.pagination.has_next);
    }

    #[tokio::test]
    async fn pagination_has_next() {
        let mut stream_repo = MockEventStreamRepository::new();
        stream_repo
            .expect_list_all()
            .returning(|_, _| Ok((vec![make_stream()], 3)));

        let uc = ListStreamsUseCase::new(Arc::new(stream_repo));
        let input = ListStreamsInput {
            page: 1,
            page_size: 1,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        assert!(result.unwrap().pagination.has_next);
    }

    #[tokio::test]
    async fn internal_error() {
        let mut stream_repo = MockEventStreamRepository::new();
        stream_repo
            .expect_list_all()
            .returning(|_, _| Err(anyhow::anyhow!("db error")));

        let uc = ListStreamsUseCase::new(Arc::new(stream_repo));
        let input = ListStreamsInput {
            page: 1,
            page_size: 50,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ListStreamsError::Internal(msg) => assert!(msg.contains("db error")),
        }
    }
}
