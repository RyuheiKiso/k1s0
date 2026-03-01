use async_trait::async_trait;
use rand::Rng;
use std::time::Duration;
use tracing::{info, warn};
use uuid::Uuid;

use crate::error::WebhookError;
use crate::payload::WebhookPayload;
use crate::signature::generate_signature;

/// 署名ヘッダー名。
pub const SIGNATURE_HEADER: &str = "X-K1s0-Signature";

/// べき等性キーのヘッダー名。
pub const IDEMPOTENCY_KEY_HEADER: &str = "Idempotency-Key";

/// リトライ設定。
#[derive(Debug, Clone)]
pub struct WebhookConfig {
    pub max_retries: u32,
    pub initial_backoff_ms: u64,
    pub max_backoff_ms: u64,
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff_ms: 100,
            max_backoff_ms: 10000,
        }
    }
}

#[async_trait]
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait WebhookClient: Send + Sync {
    async fn send(&self, url: &str, payload: &WebhookPayload) -> Result<u16, WebhookError>;
    async fn send_with_signature(
        &self,
        url: &str,
        payload: &WebhookPayload,
        secret: &str,
    ) -> Result<u16, WebhookError>;
}

/// リトライ対象のステータスコードかどうかを判定する。
fn is_retryable_status(status_code: u16) -> bool {
    status_code == 429 || status_code >= 500
}

/// 指数バックオフ + ジッターの待機時間を計算する。
fn calculate_backoff(attempt: u32, initial_backoff_ms: u64, max_backoff_ms: u64) -> Duration {
    let mut backoff = initial_backoff_ms;
    for _ in 0..attempt {
        backoff = backoff.saturating_mul(2);
        if backoff >= max_backoff_ms {
            backoff = max_backoff_ms;
            break;
        }
    }
    // ジッター: 0 ~ backoff/2 のランダム値を加算
    let jitter_max = std::cmp::max(backoff / 2, 1);
    let jitter = rand::thread_rng().gen_range(0..jitter_max);
    Duration::from_millis(backoff + jitter)
}

/// HTTPベースのWebhookクライアント（リトライ + べき等性対応）。
pub struct HttpWebhookClient {
    config: WebhookConfig,
    http_client: reqwest::Client,
    /// テスト用: スリープ関数をオーバーライドするためのフック。
    /// None の場合は tokio::time::sleep を使用する。
    #[cfg(test)]
    sleep_fn: Option<Box<dyn Fn(Duration) + Send + Sync>>,
}

impl HttpWebhookClient {
    /// デフォルト設定で新しい HttpWebhookClient を生成する。
    pub fn new() -> Self {
        Self {
            config: WebhookConfig::default(),
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("HTTP client の作成に失敗"),
            #[cfg(test)]
            sleep_fn: None,
        }
    }

    /// リトライ設定付きの HttpWebhookClient を生成する。
    pub fn with_config(config: WebhookConfig) -> Self {
        Self {
            config,
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("HTTP client の作成に失敗"),
            #[cfg(test)]
            sleep_fn: None,
        }
    }

    #[cfg(test)]
    fn with_config_and_no_sleep(config: WebhookConfig) -> Self {
        Self {
            config,
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("HTTP client の作成に失敗"),
            sleep_fn: Some(Box::new(|_| {})),
        }
    }

    async fn do_sleep(&self, duration: Duration) {
        #[cfg(test)]
        {
            if let Some(ref f) = self.sleep_fn {
                f(duration);
                return;
            }
        }
        tokio::time::sleep(duration).await;
    }

    /// リトライロジックを含む送信処理の共通実装。
    async fn send_with_retry(
        &self,
        url: &str,
        body: &[u8],
        signature: Option<String>,
    ) -> Result<u16, WebhookError> {
        let idempotency_key = Uuid::new_v4().to_string();
        let max_attempts = self.config.max_retries + 1;
        let mut last_status_code: u16 = 0;

        for attempt in 0..max_attempts {
            if attempt > 0 {
                let delay = calculate_backoff(
                    attempt - 1,
                    self.config.initial_backoff_ms,
                    self.config.max_backoff_ms,
                );
                warn!(
                    attempt = attempt + 1,
                    max_attempts = max_attempts,
                    delay_ms = delay.as_millis() as u64,
                    url = url,
                    "リトライ実行"
                );
                self.do_sleep(delay).await;
            } else {
                info!(
                    url = url,
                    idempotency_key = %idempotency_key,
                    "Webhook送信開始"
                );
            }

            let mut req = self
                .http_client
                .post(url)
                .header("Content-Type", "application/json")
                .header(IDEMPOTENCY_KEY_HEADER, &idempotency_key)
                .body(body.to_vec());

            if let Some(ref sig) = signature {
                req = req.header(SIGNATURE_HEADER, sig.as_str());
            }

            let resp = match req.send().await {
                Ok(resp) => resp,
                Err(e) => {
                    warn!(
                        attempt = attempt + 1,
                        max_attempts = max_attempts,
                        error = %e,
                        "送信エラー"
                    );
                    if attempt == max_attempts - 1 {
                        return Err(WebhookError::RequestFailed(e.to_string()));
                    }
                    continue;
                }
            };

            last_status_code = resp.status().as_u16();

            if !is_retryable_status(last_status_code) {
                info!(
                    status = last_status_code,
                    attempt = attempt + 1,
                    max_attempts = max_attempts,
                    "送信完了"
                );
                return Ok(last_status_code);
            }

            warn!(
                status = last_status_code,
                attempt = attempt + 1,
                max_attempts = max_attempts,
                "リトライ対象ステータス"
            );
        }

        Err(WebhookError::MaxRetriesExceeded {
            attempts: max_attempts,
            last_status_code,
        })
    }
}

impl Default for HttpWebhookClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WebhookClient for HttpWebhookClient {
    async fn send(&self, url: &str, payload: &WebhookPayload) -> Result<u16, WebhookError> {
        let body = serde_json::to_vec(payload)?;
        self.send_with_retry(url, &body, None).await
    }

    async fn send_with_signature(
        &self,
        url: &str,
        payload: &WebhookPayload,
        secret: &str,
    ) -> Result<u16, WebhookError> {
        let body = serde_json::to_vec(payload)?;
        let signature = generate_signature(secret, &body);
        self.send_with_retry(url, &body, Some(signature)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use wiremock::matchers::{header_exists, method};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_payload() -> WebhookPayload {
        WebhookPayload {
            event_type: "test.event".to_string(),
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            data: json!({"key": "value"}),
        }
    }

    #[tokio::test]
    async fn test_send_success() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(header_exists(IDEMPOTENCY_KEY_HEADER))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let client = HttpWebhookClient::with_config_and_no_sleep(WebhookConfig::default());
        let result = client.send(&mock_server.uri(), &test_payload()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 200);
    }

    #[tokio::test]
    async fn test_send_with_signature_uses_k1s0_header() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(header_exists(SIGNATURE_HEADER))
            .and(header_exists(IDEMPOTENCY_KEY_HEADER))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let client = HttpWebhookClient::with_config_and_no_sleep(WebhookConfig::default());
        let result = client
            .send_with_signature(&mock_server.uri(), &test_payload(), "test-secret")
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 200);
    }

    #[tokio::test]
    async fn test_retry_on_500() {
        let mock_server = MockServer::start().await;
        let call_count = Arc::new(AtomicU32::new(0));
        let call_count_clone = call_count.clone();

        Mock::given(method("POST"))
            .respond_with(move |_: &wiremock::Request| {
                let count = call_count_clone.fetch_add(1, Ordering::SeqCst) + 1;
                if count <= 2 {
                    ResponseTemplate::new(500)
                } else {
                    ResponseTemplate::new(200)
                }
            })
            .mount(&mock_server)
            .await;

        let config = WebhookConfig {
            max_retries: 3,
            initial_backoff_ms: 10,
            max_backoff_ms: 100,
        };
        let client = HttpWebhookClient::with_config_and_no_sleep(config);
        let result = client.send(&mock_server.uri(), &test_payload()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 200);
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_on_429() {
        let mock_server = MockServer::start().await;
        let call_count = Arc::new(AtomicU32::new(0));
        let call_count_clone = call_count.clone();

        Mock::given(method("POST"))
            .respond_with(move |_: &wiremock::Request| {
                let count = call_count_clone.fetch_add(1, Ordering::SeqCst) + 1;
                if count <= 1 {
                    ResponseTemplate::new(429)
                } else {
                    ResponseTemplate::new(200)
                }
            })
            .mount(&mock_server)
            .await;

        let config = WebhookConfig {
            max_retries: 2,
            initial_backoff_ms: 10,
            max_backoff_ms: 100,
        };
        let client = HttpWebhookClient::with_config_and_no_sleep(config);
        let result = client.send(&mock_server.uri(), &test_payload()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 200);
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_no_retry_on_4xx() {
        let mock_server = MockServer::start().await;
        let call_count = Arc::new(AtomicU32::new(0));
        let call_count_clone = call_count.clone();

        Mock::given(method("POST"))
            .respond_with(move |_: &wiremock::Request| {
                call_count_clone.fetch_add(1, Ordering::SeqCst);
                ResponseTemplate::new(400)
            })
            .mount(&mock_server)
            .await;

        let config = WebhookConfig {
            max_retries: 3,
            initial_backoff_ms: 10,
            max_backoff_ms: 100,
        };
        let client = HttpWebhookClient::with_config_and_no_sleep(config);
        let result = client.send(&mock_server.uri(), &test_payload()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 400);
        assert_eq!(call_count.load(Ordering::SeqCst), 1); // リトライなし
    }

    #[tokio::test]
    async fn test_max_retries_exceeded() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let config = WebhookConfig {
            max_retries: 2,
            initial_backoff_ms: 10,
            max_backoff_ms: 100,
        };
        let client = HttpWebhookClient::with_config_and_no_sleep(config);
        let result = client.send(&mock_server.uri(), &test_payload()).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            WebhookError::MaxRetriesExceeded {
                attempts,
                last_status_code,
            } => {
                assert_eq!(attempts, 3);
                assert_eq!(last_status_code, 500);
            }
            e => panic!("予期しないエラー型: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_idempotency_key_is_uuid_format() {
        let mock_server = MockServer::start().await;
        let received_key = Arc::new(std::sync::Mutex::new(String::new()));
        let received_key_clone = received_key.clone();

        Mock::given(method("POST"))
            .respond_with(move |req: &wiremock::Request| {
                if let Some(val) = req.headers.get(IDEMPOTENCY_KEY_HEADER) {
                    *received_key_clone.lock().unwrap() = val.to_str().unwrap().to_string();
                }
                ResponseTemplate::new(200)
            })
            .mount(&mock_server)
            .await;

        let client = HttpWebhookClient::with_config_and_no_sleep(WebhookConfig::default());
        let result = client.send(&mock_server.uri(), &test_payload()).await;
        assert!(result.is_ok());

        let key = received_key.lock().unwrap().clone();
        assert_eq!(key.len(), 36);
        // UUID v4 フォーマット: 8-4-4-4-12
        let parts: Vec<&str> = key.split('-').collect();
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[0].len(), 8);
        assert_eq!(parts[1].len(), 4);
        assert_eq!(parts[2].len(), 4);
        assert_eq!(parts[3].len(), 4);
        assert_eq!(parts[4].len(), 12);
    }

    #[tokio::test]
    async fn test_idempotency_key_same_across_retries() {
        let mock_server = MockServer::start().await;
        let keys = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
        let keys_clone = keys.clone();
        let call_count = Arc::new(AtomicU32::new(0));
        let call_count_clone = call_count.clone();

        Mock::given(method("POST"))
            .respond_with(move |req: &wiremock::Request| {
                if let Some(val) = req.headers.get(IDEMPOTENCY_KEY_HEADER) {
                    keys_clone.lock().unwrap().push(val.to_str().unwrap().to_string());
                }
                let count = call_count_clone.fetch_add(1, Ordering::SeqCst) + 1;
                if count <= 2 {
                    ResponseTemplate::new(500)
                } else {
                    ResponseTemplate::new(200)
                }
            })
            .mount(&mock_server)
            .await;

        let config = WebhookConfig {
            max_retries: 3,
            initial_backoff_ms: 10,
            max_backoff_ms: 100,
        };
        let client = HttpWebhookClient::with_config_and_no_sleep(config);
        let result = client.send(&mock_server.uri(), &test_payload()).await;
        assert!(result.is_ok());

        let collected_keys = keys.lock().unwrap().clone();
        assert_eq!(collected_keys.len(), 3);
        // 全リトライで同一のIdempotency-Key
        assert_eq!(collected_keys[0], collected_keys[1]);
        assert_eq!(collected_keys[1], collected_keys[2]);
    }

    #[test]
    fn test_default_config() {
        let config = WebhookConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_backoff_ms, 100);
        assert_eq!(config.max_backoff_ms, 10000);
    }

    #[test]
    fn test_is_retryable_status() {
        assert!(is_retryable_status(429));
        assert!(is_retryable_status(500));
        assert!(is_retryable_status(502));
        assert!(is_retryable_status(503));
        assert!(!is_retryable_status(200));
        assert!(!is_retryable_status(400));
        assert!(!is_retryable_status(401));
        assert!(!is_retryable_status(404));
    }

    #[test]
    fn test_calculate_backoff() {
        // attempt 0: initialBackoff * 2^0 = 100ms + ジッター
        let d = calculate_backoff(0, 100, 10000);
        assert!(d.as_millis() >= 100 && d.as_millis() < 200);

        // attempt 1: initialBackoff * 2^1 = 200ms + ジッター
        let d = calculate_backoff(1, 100, 10000);
        assert!(d.as_millis() >= 200 && d.as_millis() < 400);

        // max_backoff を超えない
        let d = calculate_backoff(20, 100, 500);
        assert!(d.as_millis() <= 750); // 500 + 250(jitter max)
    }
}
