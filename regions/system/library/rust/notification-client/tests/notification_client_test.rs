#![allow(clippy::unwrap_used)]
use k1s0_notification_client::{
    NotificationChannel, NotificationClient, NotificationClientError, NotificationRequest,
    NotificationResponse,
};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn ok_response() -> NotificationResponse {
    NotificationResponse {
        id: Uuid::new_v4(),
        status: "sent".to_string(),
        message_id: Some("msg-001".to_string()),
    }
}

fn make_email_request() -> NotificationRequest {
    NotificationRequest::new(
        NotificationChannel::Email,
        "user@example.com",
        "Hello from tests",
    )
    .with_subject("Test Subject")
}

// ===========================================================================
// NotificationRequest construction
// ===========================================================================

// NotificationRequest::new がチャンネル・受信者・本文を正しく設定することを確認する。
#[test]
fn request_new_sets_fields_correctly() {
    let req = NotificationRequest::new(NotificationChannel::Email, "alice@example.com", "Welcome!");
    assert_eq!(req.channel, NotificationChannel::Email);
    assert_eq!(req.recipient, "alice@example.com");
    assert_eq!(req.body, "Welcome!");
    assert!(req.subject.is_none());
    assert!(req.metadata.is_none());
    // id should be a valid UUID
    assert!(!req.id.is_nil());
}

// with_subject でサブジェクトが設定されることを確認する。
#[test]
fn request_with_subject_sets_subject() {
    let req = NotificationRequest::new(NotificationChannel::Sms, "+1234567890", "Code: 1234")
        .with_subject("Verification");
    assert_eq!(req.subject, Some("Verification".to_string()));
}

// 各リクエストが一意の ID を持つことを確認する。
#[test]
fn each_request_gets_unique_id() {
    let r1 = NotificationRequest::new(NotificationChannel::Email, "a@b.com", "body1");
    let r2 = NotificationRequest::new(NotificationChannel::Email, "a@b.com", "body2");
    assert_ne!(r1.id, r2.id);
}

// ===========================================================================
// NotificationChannel variants
// ===========================================================================

// 全ての NotificationChannel バリアントが JSON シリアライズ・デシリアライズで正しく往復することを確認する。
#[test]
fn channel_enum_all_variants() {
    let channels = vec![
        NotificationChannel::Email,
        NotificationChannel::Sms,
        NotificationChannel::Push,
        NotificationChannel::Slack,
        NotificationChannel::Webhook,
    ];
    // Ensure all variants are distinct via serde roundtrip
    for ch in &channels {
        let json = serde_json::to_string(ch).unwrap();
        let deserialized: NotificationChannel = serde_json::from_str(&json).unwrap();
        assert_eq!(&deserialized, ch);
    }
}

// NotificationChannel が JSON で往復シリアライズされることを確認する。
#[test]
fn channel_serde_roundtrip() {
    let ch = NotificationChannel::Slack;
    let json = serde_json::to_string(&ch).unwrap();
    let back: NotificationChannel = serde_json::from_str(&json).unwrap();
    assert_eq!(back, NotificationChannel::Slack);
}

// ===========================================================================
// NotificationRequest serde
// ===========================================================================

// NotificationRequest が JSON で往復シリアライズされることを確認する。
#[test]
fn request_serde_roundtrip() {
    let req = make_email_request();
    let json = serde_json::to_string(&req).unwrap();
    let deserialized: NotificationRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, req.id);
    assert_eq!(deserialized.channel, req.channel);
    assert_eq!(deserialized.recipient, req.recipient);
    assert_eq!(deserialized.body, req.body);
    assert_eq!(deserialized.subject, req.subject);
}

// NotificationResponse が JSON で往復シリアライズされることを確認する。
#[test]
fn response_serde_roundtrip() {
    let resp = ok_response();
    let json = serde_json::to_string(&resp).unwrap();
    let deserialized: NotificationResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, resp.id);
    assert_eq!(deserialized.status, resp.status);
    assert_eq!(deserialized.message_id, resp.message_id);
}

// ===========================================================================
// Stub implementation for testing the trait contract
// ===========================================================================

/// A simple in-memory stub that records sent notifications.
struct StubNotificationClient {
    /// If set, send() returns this error.
    fail_with: Option<NotificationClientError>,
    /// Tracks how many times send was called.
    send_count: std::sync::atomic::AtomicU32,
}

impl StubNotificationClient {
    fn new() -> Self {
        Self {
            fail_with: None,
            send_count: std::sync::atomic::AtomicU32::new(0),
        }
    }

    fn failing(err: NotificationClientError) -> Self {
        Self {
            fail_with: Some(err),
            send_count: std::sync::atomic::AtomicU32::new(0),
        }
    }

    fn call_count(&self) -> u32 {
        self.send_count.load(std::sync::atomic::Ordering::SeqCst)
    }
}

#[async_trait::async_trait]
impl NotificationClient for StubNotificationClient {
    async fn send(
        &self,
        request: NotificationRequest,
    ) -> Result<NotificationResponse, NotificationClientError> {
        self.send_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if let Some(ref err_template) = self.fail_with {
            return Err(match err_template {
                NotificationClientError::SendError(msg) => {
                    NotificationClientError::SendError(msg.clone())
                }
                NotificationClientError::BatchError(msg) => {
                    NotificationClientError::BatchError(msg.clone())
                }
                NotificationClientError::InvalidChannel(msg) => {
                    NotificationClientError::InvalidChannel(msg.clone())
                }
                NotificationClientError::Internal(msg) => {
                    NotificationClientError::Internal(msg.clone())
                }
            });
        }
        Ok(NotificationResponse {
            id: request.id,
            status: "sent".to_string(),
            message_id: Some(format!("msg-{}", request.id)),
        })
    }

    async fn send_batch(
        &self,
        requests: Vec<NotificationRequest>,
    ) -> Result<Vec<NotificationResponse>, NotificationClientError> {
        let mut responses = Vec::new();
        for req in requests {
            responses.push(self.send(req).await?);
        }
        Ok(responses)
    }
}

// ===========================================================================
// Stub: send (success per channel)
// ===========================================================================

// スタブクライアントで Email チャンネルへの送信が成功することを確認する。
#[tokio::test]
async fn stub_send_email_success() {
    let client = StubNotificationClient::new();
    let req = make_email_request();
    let result = client.send(req).await.unwrap();
    assert_eq!(result.status, "sent");
    assert!(result.message_id.is_some());
    assert_eq!(client.call_count(), 1);
}

// スタブクライアントで SMS チャンネルへの送信が成功することを確認する。
#[tokio::test]
async fn stub_send_sms_success() {
    let client = StubNotificationClient::new();
    let req =
        NotificationRequest::new(NotificationChannel::Sms, "+819012345678", "Your code: 9999");
    let result = client.send(req).await.unwrap();
    assert_eq!(result.status, "sent");
}

// スタブクライアントで Push チャンネルへの送信が成功することを確認する。
#[tokio::test]
async fn stub_send_push_success() {
    let client = StubNotificationClient::new();
    let req = NotificationRequest::new(
        NotificationChannel::Push,
        "device-token-abc",
        "New message!",
    );
    let result = client.send(req).await.unwrap();
    assert_eq!(result.status, "sent");
}

// スタブクライアントで Slack チャンネルへの送信が成功することを確認する。
#[tokio::test]
async fn stub_send_slack_success() {
    let client = StubNotificationClient::new();
    let req = NotificationRequest::new(NotificationChannel::Slack, "#alerts", "Server is down!");
    let result = client.send(req).await.unwrap();
    assert_eq!(result.status, "sent");
}

// スタブクライアントで Webhook チャンネルへの送信が成功することを確認する。
#[tokio::test]
async fn stub_send_webhook_channel_success() {
    let client = StubNotificationClient::new();
    let req = NotificationRequest::new(
        NotificationChannel::Webhook,
        "https://hooks.example.com/callback",
        r#"{"event":"user.created"}"#,
    );
    let result = client.send(req).await.unwrap();
    assert_eq!(result.status, "sent");
}

// ===========================================================================
// Stub: send preserves request id in response
// ===========================================================================

// レスポンスの ID がリクエストの ID と一致することを確認する。
#[tokio::test]
async fn stub_send_response_id_matches_request_id() {
    let client = StubNotificationClient::new();
    let req = make_email_request();
    let req_id = req.id;
    let result = client.send(req).await.unwrap();
    assert_eq!(result.id, req_id);
}

// ===========================================================================
// Stub: send (error cases)
// ===========================================================================

// send が SendError を返す場合にエラーが正しく伝播することを確認する。
#[tokio::test]
async fn stub_send_returns_send_error() {
    let client = StubNotificationClient::failing(NotificationClientError::SendError(
        "SMTP connection refused".to_string(),
    ));
    let req = make_email_request();
    let result = client.send(req).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        NotificationClientError::SendError(msg) => {
            assert!(msg.contains("SMTP connection refused"));
        }
        other => panic!("expected SendError, got: {:?}", other),
    }
}

// send が InvalidChannel エラーを返す場合にエラーが正しく伝播することを確認する。
#[tokio::test]
async fn stub_send_returns_invalid_channel_error() {
    let client = StubNotificationClient::failing(NotificationClientError::InvalidChannel(
        "unsupported channel".to_string(),
    ));
    let req = make_email_request();
    let result = client.send(req).await;
    match result.unwrap_err() {
        NotificationClientError::InvalidChannel(msg) => {
            assert!(msg.contains("unsupported"));
        }
        other => panic!("expected InvalidChannel, got: {:?}", other),
    }
}

// send が Internal エラーを返す場合にエラーが正しく伝播することを確認する。
#[tokio::test]
async fn stub_send_returns_internal_error() {
    let client = StubNotificationClient::failing(NotificationClientError::Internal(
        "database unavailable".to_string(),
    ));
    let req = make_email_request();
    let result = client.send(req).await;
    match result.unwrap_err() {
        NotificationClientError::Internal(msg) => {
            assert!(msg.contains("database unavailable"));
        }
        other => panic!("expected Internal, got: {:?}", other),
    }
}

// ===========================================================================
// Stub: send_batch
// ===========================================================================

// send_batch が複数のリクエストを全て成功させることを確認する。
#[tokio::test]
async fn stub_send_batch_success() {
    let client = StubNotificationClient::new();
    let requests = vec![
        NotificationRequest::new(NotificationChannel::Email, "a@example.com", "Hello A"),
        NotificationRequest::new(NotificationChannel::Email, "b@example.com", "Hello B"),
        NotificationRequest::new(NotificationChannel::Sms, "+1234567890", "Code: 1111"),
    ];
    let results = client.send_batch(requests).await.unwrap();
    assert_eq!(results.len(), 3);
    assert!(results.iter().all(|r| r.status == "sent"));
    assert_eq!(client.call_count(), 3);
}

// send_batch がエラー発生時にエラーを呼び出し元に伝播することを確認する。
#[tokio::test]
async fn stub_send_batch_error_propagates() {
    let client = StubNotificationClient::failing(NotificationClientError::SendError(
        "connection refused".to_string(),
    ));
    let requests = vec![NotificationRequest::new(
        NotificationChannel::Email,
        "a@example.com",
        "body",
    )];
    let result = client.send_batch(requests).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        NotificationClientError::SendError(msg) => {
            assert!(msg.contains("connection refused"));
        }
        other => panic!("expected SendError, got: {:?}", other),
    }
}

// 空のリクエストリストで send_batch を呼び出した場合に空のレスポンスが返ることを確認する。
#[tokio::test]
async fn stub_send_batch_empty_input() {
    let client = StubNotificationClient::new();
    let results = client.send_batch(vec![]).await.unwrap();
    assert!(results.is_empty());
    assert_eq!(client.call_count(), 0);
}

// ===========================================================================
// Error display
// ===========================================================================

// SendError の表示文字列にエラーメッセージが含まれることを確認する。
#[test]
fn error_display_send_error() {
    let err = NotificationClientError::SendError("timeout".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("timeout"));
}

// BatchError の表示文字列にエラーメッセージが含まれることを確認する。
#[test]
fn error_display_batch_error() {
    let err = NotificationClientError::BatchError("3 failed".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("3 failed"));
}

// InvalidChannel の表示文字列にエラーメッセージが含まれることを確認する。
#[test]
fn error_display_invalid_channel() {
    let err = NotificationClientError::InvalidChannel("fax".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("fax"));
}

// Internal の表示文字列にエラーメッセージが含まれることを確認する。
#[test]
fn error_display_internal() {
    let err = NotificationClientError::Internal("panic".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("panic"));
}
