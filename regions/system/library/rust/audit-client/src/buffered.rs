use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::client::AuditClient;
use crate::error::AuditError;
use crate::event::AuditEvent;

pub struct BufferedAuditClient {
    buffer: Mutex<Vec<AuditEvent>>,
}

impl BufferedAuditClient {
    pub fn new() -> Self {
        Self {
            buffer: Mutex::new(Vec::new()),
        }
    }
}

impl Default for BufferedAuditClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuditClient for BufferedAuditClient {
    async fn record(&self, event: AuditEvent) -> Result<(), AuditError> {
        let mut buffer = self.buffer.lock().await;
        buffer.push(event);
        Ok(())
    }

    async fn flush(&self) -> Result<Vec<AuditEvent>, AuditError> {
        let mut buffer = self.buffer.lock().await;
        let events = buffer.drain(..).collect();
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_record_and_flush() {
        let client = BufferedAuditClient::new();
        let event = AuditEvent::new(
            "tenant-1",
            "user-1",
            "create",
            "document",
            "doc-123",
            json!({"key": "value"}),
        );
        let event_id = event.id;

        client.record(event).await.unwrap();
        let flushed = client.flush().await.unwrap();

        assert_eq!(flushed.len(), 1);
        assert_eq!(flushed[0].id, event_id);
        assert_eq!(flushed[0].tenant_id, "tenant-1");
        assert_eq!(flushed[0].action, "create");
    }

    #[tokio::test]
    async fn test_flush_empties_buffer() {
        let client = BufferedAuditClient::new();
        let event = AuditEvent::new(
            "tenant-1",
            "user-1",
            "create",
            "document",
            "doc-123",
            json!({}),
        );
        client.record(event).await.unwrap();
        let _ = client.flush().await.unwrap();
        let flushed = client.flush().await.unwrap();
        assert!(flushed.is_empty());
    }

    #[tokio::test]
    async fn test_multiple_events_buffering() {
        let client = BufferedAuditClient::new();
        for i in 0..5 {
            let event = AuditEvent::new(
                "tenant-1",
                "user-1",
                format!("action-{}", i),
                "resource",
                format!("res-{}", i),
                json!({}),
            );
            client.record(event).await.unwrap();
        }
        let flushed = client.flush().await.unwrap();
        assert_eq!(flushed.len(), 5);
        for (i, event) in flushed.iter().enumerate() {
            assert_eq!(event.action, format!("action-{}", i));
        }
    }
}
