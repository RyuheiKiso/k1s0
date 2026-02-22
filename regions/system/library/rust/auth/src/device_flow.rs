//! Device Authorization Grant フロー（RFC 8628）のクライアント実装。

use serde::{Deserialize, Serialize};
use std::future::Future;
use std::time::Duration;

/// DeviceCodeResponse はデバイス認可リクエストのレスポンス。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    #[serde(default)]
    pub verification_uri_complete: Option<String>,
    pub expires_in: u64,
    pub interval: u64,
}

/// TokenResult はトークンエンドポイントのレスポンス。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResult {
    pub access_token: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_in: u64,
}

/// DeviceFlowError は Device Authorization Grant フローのエラー。
#[derive(thiserror::Error, Debug)]
pub enum DeviceFlowError {
    #[error("expired_token: device code has expired")]
    ExpiredToken,

    #[error("access_denied: user denied the authorization request")]
    AccessDenied,

    #[error("device flow error: {0}")]
    OAuthError(String),

    #[error("HTTP request failed: {0}")]
    HttpError(String),

    #[error("failed to parse response: {0}")]
    ParseError(String),

    #[error("polling cancelled")]
    Cancelled,
}

/// OAuth2 エラーレスポンス。
#[derive(Debug, Deserialize)]
struct OAuthErrorResponse {
    error: String,
    #[allow(dead_code)]
    error_description: Option<String>,
}

/// HTTP クライアントの抽象化（テスト用にモック可能）。
#[async_trait::async_trait]
pub trait DeviceFlowHttpClient: Send + Sync {
    async fn post_form(
        &self,
        url: &str,
        params: &[(&str, &str)],
    ) -> Result<(u16, String), DeviceFlowError>;
}

/// reqwest ベースのデフォルト HTTP クライアント。
pub struct DefaultDeviceFlowHttpClient;

#[async_trait::async_trait]
impl DeviceFlowHttpClient for DefaultDeviceFlowHttpClient {
    async fn post_form(
        &self,
        url: &str,
        params: &[(&str, &str)],
    ) -> Result<(u16, String), DeviceFlowError> {
        let client = reqwest::Client::new();
        let resp = client
            .post(url)
            .form(params)
            .send()
            .await
            .map_err(|e| DeviceFlowError::HttpError(e.to_string()))?;
        let status = resp.status().as_u16();
        let body = resp
            .text()
            .await
            .map_err(|e| DeviceFlowError::HttpError(e.to_string()))?;
        Ok((status, body))
    }
}

/// DeviceAuthClient は Device Authorization Grant フロー（RFC 8628）のクライアント。
pub struct DeviceAuthClient {
    device_endpoint: String,
    token_endpoint: String,
    http_client: Box<dyn DeviceFlowHttpClient>,
}

impl DeviceAuthClient {
    /// 新しい DeviceAuthClient を生成する。
    pub fn new(device_endpoint: &str, token_endpoint: &str) -> Self {
        Self {
            device_endpoint: device_endpoint.to_string(),
            token_endpoint: token_endpoint.to_string(),
            http_client: Box::new(DefaultDeviceFlowHttpClient),
        }
    }

    /// カスタム HTTP クライアントを使う DeviceAuthClient を生成する（テスト用）。
    pub fn with_http_client(
        device_endpoint: &str,
        token_endpoint: &str,
        http_client: Box<dyn DeviceFlowHttpClient>,
    ) -> Self {
        Self {
            device_endpoint: device_endpoint.to_string(),
            token_endpoint: token_endpoint.to_string(),
            http_client,
        }
    }

    /// デバイス認可リクエストを送信し、デバイスコード情報を返す。
    pub async fn request_device_code(
        &self,
        client_id: &str,
        scope: Option<&str>,
    ) -> Result<DeviceCodeResponse, DeviceFlowError> {
        let mut params: Vec<(&str, &str)> = vec![("client_id", client_id)];
        if let Some(s) = scope {
            params.push(("scope", s));
        }

        let (status, body) = self
            .http_client
            .post_form(&self.device_endpoint, &params)
            .await?;

        if status != 200 {
            return Err(DeviceFlowError::HttpError(format!(
                "device code request failed with status {}: {}",
                status, body
            )));
        }

        serde_json::from_str(&body).map_err(|e| DeviceFlowError::ParseError(e.to_string()))
    }

    /// device_code を使ってトークンエンドポイントをポーリングする。
    /// interval が 0 の場合はデフォルトの 5 秒を使用する。
    pub async fn poll_token(
        &self,
        client_id: &str,
        device_code: &str,
        interval: u64,
        cancel: impl Future<Output = ()> + Send,
    ) -> Result<TokenResult, DeviceFlowError> {
        let mut interval_secs = if interval == 0 { 5 } else { interval };
        tokio::pin!(cancel);

        loop {
            let params = [
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("device_code", device_code),
                ("client_id", client_id),
            ];

            let (status, body) = self
                .http_client
                .post_form(&self.token_endpoint, &params)
                .await?;

            if status == 200 {
                return serde_json::from_str(&body)
                    .map_err(|e| DeviceFlowError::ParseError(e.to_string()));
            }

            let err_resp: OAuthErrorResponse = serde_json::from_str(&body)
                .map_err(|e| DeviceFlowError::ParseError(e.to_string()))?;

            match err_resp.error.as_str() {
                "authorization_pending" => {}
                "slow_down" => {
                    interval_secs += 5;
                }
                "expired_token" => return Err(DeviceFlowError::ExpiredToken),
                "access_denied" => return Err(DeviceFlowError::AccessDenied),
                other => return Err(DeviceFlowError::OAuthError(other.to_string())),
            }

            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(interval_secs)) => {}
                _ = &mut cancel => {
                    return Err(DeviceFlowError::Cancelled);
                }
            }
        }
    }

    /// Device Authorization Grant フロー全体を実行する統合メソッド。
    /// on_user_code コールバックでユーザーにデバイスコード情報を通知する。
    pub async fn device_flow<F>(
        &self,
        client_id: &str,
        scope: Option<&str>,
        on_user_code: F,
        cancel: impl Future<Output = ()> + Send,
    ) -> Result<TokenResult, DeviceFlowError>
    where
        F: FnOnce(&DeviceCodeResponse),
    {
        let device_resp = self.request_device_code(client_id, scope).await?;
        on_user_code(&device_resp);
        self.poll_token(
            client_id,
            &device_resp.device_code,
            device_resp.interval,
            cancel,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    /// テスト用のモック HTTP クライアント。
    struct MockHttpClient {
        handler: Box<dyn Fn(&str, &[(&str, &str)]) -> (u16, String) + Send + Sync>,
    }

    #[async_trait::async_trait]
    impl DeviceFlowHttpClient for MockHttpClient {
        async fn post_form(
            &self,
            url: &str,
            params: &[(&str, &str)],
        ) -> Result<(u16, String), DeviceFlowError> {
            Ok((self.handler)(url, params))
        }
    }

    #[tokio::test]
    async fn test_request_device_code_success() {
        let client = DeviceAuthClient::with_http_client(
            "https://auth.example.com/device",
            "https://auth.example.com/token",
            Box::new(MockHttpClient {
                handler: Box::new(|url, params| {
                    assert!(url.contains("/device"));
                    let client_id = params.iter().find(|(k, _)| *k == "client_id").unwrap().1;
                    assert_eq!(client_id, "test-client");
                    let scope = params.iter().find(|(k, _)| *k == "scope").unwrap().1;
                    assert_eq!(scope, "openid profile");

                    (200, serde_json::json!({
                        "device_code": "device-code-123",
                        "user_code": "ABCD-EFGH",
                        "verification_uri": "https://auth.example.com/device",
                        "verification_uri_complete": "https://auth.example.com/device?user_code=ABCD-EFGH",
                        "expires_in": 600,
                        "interval": 5
                    }).to_string())
                }),
            }),
        );

        let resp = client
            .request_device_code("test-client", Some("openid profile"))
            .await
            .unwrap();
        assert_eq!(resp.device_code, "device-code-123");
        assert_eq!(resp.user_code, "ABCD-EFGH");
        assert_eq!(resp.verification_uri, "https://auth.example.com/device");
        assert_eq!(
            resp.verification_uri_complete.as_deref(),
            Some("https://auth.example.com/device?user_code=ABCD-EFGH")
        );
        assert_eq!(resp.expires_in, 600);
        assert_eq!(resp.interval, 5);
    }

    #[tokio::test]
    async fn test_poll_token_authorization_pending_then_success() {
        let call_count = Arc::new(AtomicU32::new(0));
        let cc = call_count.clone();

        let client = DeviceAuthClient::with_http_client(
            "https://auth.example.com/device",
            "https://auth.example.com/token",
            Box::new(MockHttpClient {
                handler: Box::new(move |_url, params| {
                    let gt = params.iter().find(|(k, _)| *k == "grant_type").unwrap().1;
                    assert_eq!(gt, "urn:ietf:params:oauth:grant-type:device_code");

                    let count = cc.fetch_add(1, Ordering::SeqCst) + 1;
                    if count <= 2 {
                        (
                            400,
                            serde_json::json!({"error": "authorization_pending"}).to_string(),
                        )
                    } else {
                        (
                            200,
                            serde_json::json!({
                                "access_token": "access-token-xyz",
                                "refresh_token": "refresh-token-xyz",
                                "token_type": "Bearer",
                                "expires_in": 900
                            })
                            .to_string(),
                        )
                    }
                }),
            }),
        );

        let never = std::future::pending::<()>();
        let result = client
            .poll_token("test-client", "device-code-123", 1, never)
            .await
            .unwrap();

        assert_eq!(result.access_token, "access-token-xyz");
        assert_eq!(result.refresh_token.as_deref(), Some("refresh-token-xyz"));
        assert_eq!(result.token_type, "Bearer");
        assert_eq!(result.expires_in, 900);
        assert!(call_count.load(Ordering::SeqCst) >= 3);
    }

    #[tokio::test]
    async fn test_poll_token_slow_down() {
        let call_count = Arc::new(AtomicU32::new(0));
        let cc = call_count.clone();

        let client = DeviceAuthClient::with_http_client(
            "https://auth.example.com/device",
            "https://auth.example.com/token",
            Box::new(MockHttpClient {
                handler: Box::new(move |_url, _params| {
                    let count = cc.fetch_add(1, Ordering::SeqCst) + 1;
                    if count == 1 {
                        (400, serde_json::json!({"error": "slow_down"}).to_string())
                    } else {
                        (
                            200,
                            serde_json::json!({
                                "access_token": "access-token",
                                "token_type": "Bearer",
                                "expires_in": 900
                            })
                            .to_string(),
                        )
                    }
                }),
            }),
        );

        let start = tokio::time::Instant::now();
        let never = std::future::pending::<()>();
        let result = client
            .poll_token("test-client", "device-code-123", 1, never)
            .await
            .unwrap();

        assert_eq!(result.access_token, "access-token");
        // slow_down 後は interval が 1+5=6 秒以上になるはず
        assert!(start.elapsed() >= Duration::from_secs(6));
    }

    #[tokio::test]
    async fn test_poll_token_expired_token() {
        let client = DeviceAuthClient::with_http_client(
            "https://auth.example.com/device",
            "https://auth.example.com/token",
            Box::new(MockHttpClient {
                handler: Box::new(|_url, _params| {
                    (
                        400,
                        serde_json::json!({"error": "expired_token"}).to_string(),
                    )
                }),
            }),
        );

        let never = std::future::pending::<()>();
        let result = client
            .poll_token("test-client", "device-code-123", 1, never)
            .await;
        assert!(matches!(result, Err(DeviceFlowError::ExpiredToken)));
    }

    #[tokio::test]
    async fn test_poll_token_access_denied() {
        let client = DeviceAuthClient::with_http_client(
            "https://auth.example.com/device",
            "https://auth.example.com/token",
            Box::new(MockHttpClient {
                handler: Box::new(|_url, _params| {
                    (
                        400,
                        serde_json::json!({"error": "access_denied"}).to_string(),
                    )
                }),
            }),
        );

        let never = std::future::pending::<()>();
        let result = client
            .poll_token("test-client", "device-code-123", 1, never)
            .await;
        assert!(matches!(result, Err(DeviceFlowError::AccessDenied)));
    }

    #[tokio::test]
    async fn test_device_flow_integration() {
        let token_call_count = Arc::new(AtomicU32::new(0));
        let tcc = token_call_count.clone();

        let client = DeviceAuthClient::with_http_client(
            "https://auth.example.com/device",
            "https://auth.example.com/token",
            Box::new(MockHttpClient {
                handler: Box::new(move |_url, params| {
                    let has_grant_type = params.iter().any(|(k, _)| *k == "grant_type");
                    if !has_grant_type {
                        // device code request
                        return (
                            200,
                            serde_json::json!({
                                "device_code": "device-code-flow",
                                "user_code": "WXYZ-1234",
                                "verification_uri": "https://auth.example.com/device",
                                "verification_uri_complete": "https://auth.example.com/device?user_code=WXYZ-1234",
                                "expires_in": 600,
                                "interval": 1
                            })
                            .to_string(),
                        );
                    }

                    let count = tcc.fetch_add(1, Ordering::SeqCst) + 1;
                    if count <= 1 {
                        (
                            400,
                            serde_json::json!({"error": "authorization_pending"}).to_string(),
                        )
                    } else {
                        (
                            200,
                            serde_json::json!({
                                "access_token": "flow-access-token",
                                "refresh_token": "flow-refresh-token",
                                "token_type": "Bearer",
                                "expires_in": 900
                            })
                            .to_string(),
                        )
                    }
                }),
            }),
        );

        let mut received_user_code = String::new();
        let mut received_verification_uri = String::new();

        let never = std::future::pending::<()>();
        let result = client
            .device_flow(
                "test-client",
                Some("openid"),
                |resp| {
                    received_user_code = resp.user_code.clone();
                    received_verification_uri = resp.verification_uri.clone();
                },
                never,
            )
            .await
            .unwrap();

        assert_eq!(result.access_token, "flow-access-token");
        assert_eq!(result.refresh_token.as_deref(), Some("flow-refresh-token"));
        assert_eq!(received_user_code, "WXYZ-1234");
        assert_eq!(received_verification_uri, "https://auth.example.com/device");
    }

    #[tokio::test]
    async fn test_poll_token_cancellation() {
        let client = DeviceAuthClient::with_http_client(
            "https://auth.example.com/device",
            "https://auth.example.com/token",
            Box::new(MockHttpClient {
                handler: Box::new(|_url, _params| {
                    (
                        400,
                        serde_json::json!({"error": "authorization_pending"}).to_string(),
                    )
                }),
            }),
        );

        let cancel = async {
            tokio::time::sleep(Duration::from_secs(2)).await;
        };
        let result = client
            .poll_token("test-client", "device-code-123", 5, cancel)
            .await;
        assert!(matches!(result, Err(DeviceFlowError::Cancelled)));
    }
}
