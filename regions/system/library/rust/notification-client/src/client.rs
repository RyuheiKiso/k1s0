use async_trait::async_trait;

use crate::error::NotificationClientError;
use crate::request::{NotificationRequest, NotificationResponse};

#[async_trait]
#[cfg_attr(any(feature = "mock", test), mockall::automock)]
pub trait NotificationClient: Send + Sync {
    async fn send(
        &self,
        request: NotificationRequest,
    ) -> Result<NotificationResponse, NotificationClientError>;
    async fn send_batch(
        &self,
        requests: Vec<NotificationRequest>,
    ) -> Result<Vec<NotificationResponse>, NotificationClientError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::NotificationChannel;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_mock_send() {
        let mut mock = MockNotificationClient::new();
        let expected_id = Uuid::new_v4();
        let expected_response = NotificationResponse {
            id: expected_id,
            status: "sent".to_string(),
            message_id: Some("msg-123".to_string()),
        };
        let resp_clone = expected_response.clone();

        mock.expect_send()
            .times(1)
            .returning(move |_req| {
                Box::pin({
                    let resp = NotificationResponse {
                        id: resp_clone.id,
                        status: resp_clone.status.clone(),
                        message_id: resp_clone.message_id.clone(),
                    };
                    async move { Ok(resp) }
                })
            });

        let request = NotificationRequest::new(
            NotificationChannel::Email,
            "user@example.com",
            "Hello!",
        )
        .with_subject("Test Subject");

        let result = mock.send(request).await.unwrap();
        assert_eq!(result.id, expected_id);
        assert_eq!(result.status, "sent");
        assert_eq!(result.message_id, Some("msg-123".to_string()));
    }
}
