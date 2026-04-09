/// C-02 監査対応: `GrpcRateLimitClient` → `HttpRateLimitClient` にリネーム
/// API パスをサーバー実装（POST /api/v1/ratelimit/check 等）に合わせる
#[cfg(feature = "grpc")]
mod inner {
    use async_trait::async_trait;
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};

    use crate::client::RateLimitClient;
    use crate::error::RateLimitError;
    use crate::types::{RateLimitPolicy, RateLimitResult, RateLimitStatus};

    /// HTTP REST API を使用する ratelimit-server クライアント
    /// C-02/L-16 監査対応: `GrpcRateLimitClient` から `HttpRateLimitClient` にリネーム
    pub struct HttpRateLimitClient {
        http: reqwest::Client,
        base_url: String,
    }

    impl HttpRateLimitClient {
        /// デフォルトタイムアウト30秒でHTTPクライアントを構築して接続する
        // HIGH-001 監査対応: reqwest::Client::builder().build() は同期処理のため async を除去する
        pub fn new(server_url: impl Into<String>) -> Result<Self, RateLimitError> {
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .map_err(|e| RateLimitError::ConnectionError(e.to_string()))?;
            Self::with_http_client(server_url, client)
        }

        pub fn with_http_client(
            server_url: impl Into<String>,
            http_client: reqwest::Client,
        ) -> Result<Self, RateLimitError> {
            let mut base = server_url.into();
            if !base.starts_with("http://") && !base.starts_with("https://") {
                base = format!("http://{base}");
            }
            let base = base.trim_end_matches('/').to_string();
            Ok(Self {
                http: http_client,
                base_url: base,
            })
        }
    }

    /// C-02 監査対応: サーバー API に合わせたリクエスト構造体（scope + identifier + window）
    #[derive(Serialize)]
    struct CheckRequest {
        scope: String,
        identifier: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        window: Option<String>,
    }

    /// サーバーの `CheckRateLimitResponse` に合わせたレスポンス構造体
    #[derive(Deserialize)]
    struct CheckResponse {
        allowed: bool,
        remaining: i64,
        reset_at: String,
    }

    /// サーバーの `UsageResponse` に合わせたレスポンス構造体
    #[derive(Deserialize)]
    struct UsageResponse {
        #[serde(default)]
        key: String,
        #[serde(default)]
        limit: u32,
        #[serde(default)]
        window_secs: u64,
        #[serde(default)]
        algorithm: String,
    }

    fn parse_reset_at(s: &str) -> DateTime<Utc> {
        s.parse::<DateTime<Utc>>().unwrap_or_else(|_| Utc::now())
    }

    fn map_reqwest_err(e: reqwest::Error) -> RateLimitError {
        if e.is_timeout() {
            RateLimitError::Timeout
        } else {
            RateLimitError::ServerError(e.to_string())
        }
    }

    async fn map_error_response(resp: reqwest::Response, op: &str) -> RateLimitError {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        let msg = if body.trim().is_empty() {
            format!("status {}", status.as_u16())
        } else {
            body.trim().to_string()
        };
        match status.as_u16() {
            404 => RateLimitError::KeyNotFound {
                key: format!("{op}: {msg}"),
            },
            429 => RateLimitError::LimitExceeded {
                retry_after_secs: 0,
            },
            _ => RateLimitError::ServerError(format!(
                "{} failed (status {}): {}",
                op,
                status.as_u16(),
                msg
            )),
        }
    }

    #[async_trait]
    impl RateLimitClient for HttpRateLimitClient {
        /// C-02 監査対応: POST /api/v1/ratelimit/check（key をパスではなくボディに含める）
        /// key は "scope:identifier" 形式で受け取り、scope と identifier に分割する
        async fn check(&self, key: &str, cost: u32) -> Result<RateLimitStatus, RateLimitError> {
            let url = format!("{}/api/v1/ratelimit/check", self.base_url);
            let (scope, identifier) = split_key(key);
            let window = if cost > 1 {
                Some(format!("{cost}s"))
            } else {
                None
            };
            let resp = self
                .http
                .post(&url)
                .json(&CheckRequest {
                    scope,
                    identifier,
                    window,
                })
                .send()
                .await
                .map_err(map_reqwest_err)?;

            if !resp.status().is_success() {
                return Err(map_error_response(resp, "check").await);
            }

            let result: CheckResponse = resp
                .json()
                .await
                .map_err(|e| RateLimitError::ServerError(format!("check: decode response: {e}")))?;

            Ok(RateLimitStatus {
                allowed: result.allowed,
                // HIGH-001 監査対応: i64→u32 の安全なキャスト（負値と過大値を0に変換）
                remaining: u32::try_from(result.remaining.max(0)).unwrap_or(0),
                reset_at: parse_reset_at(&result.reset_at),
                retry_after_secs: if result.allowed { None } else { Some(0) },
            })
        }

        /// C-02 監査対応: consume は check と同じエンドポイントを使用する
        /// サーバー側に consume エンドポイントはないため、check で代用する
        async fn consume(&self, key: &str, cost: u32) -> Result<RateLimitResult, RateLimitError> {
            let status = self.check(key, cost).await?;
            Ok(RateLimitResult {
                remaining: status.remaining,
                reset_at: status.reset_at,
            })
        }

        /// C-02 監査対応: GET /api/v1/ratelimit/usage
        async fn get_limit(&self, key: &str) -> Result<RateLimitPolicy, RateLimitError> {
            let url = format!("{}/api/v1/ratelimit/usage", self.base_url);
            let resp = self.http.get(&url).send().await.map_err(map_reqwest_err)?;

            if !resp.status().is_success() {
                return Err(map_error_response(resp, "get_limit").await);
            }

            let result: UsageResponse = resp.json().await.map_err(|e| {
                RateLimitError::ServerError(format!("get_limit: decode response: {e}"))
            })?;

            Ok(RateLimitPolicy {
                key: if result.key.is_empty() {
                    key.to_string()
                } else {
                    result.key
                },
                limit: result.limit,
                window_secs: result.window_secs,
                algorithm: result.algorithm,
            })
        }
    }

    /// key を "scope:identifier" 形式から (scope, identifier) に分割する
    /// ":" が含まれない場合は scope="default"、identifier=key とする
    fn split_key(key: &str) -> (String, String) {
        if let Some((scope, identifier)) = key.split_once(':') {
            (scope.to_string(), identifier.to_string())
        } else {
            ("default".to_string(), key.to_string())
        }
    }
}

#[cfg(feature = "grpc")]
pub use inner::HttpRateLimitClient;

/// 後方互換性のための型エイリアス（L-16 監査対応: 旧名称からの移行期間用）
#[cfg(feature = "grpc")]
#[deprecated(note = "GrpcRateLimitClient は HttpRateLimitClient にリネームされました")]
pub type GrpcRateLimitClient = inner::HttpRateLimitClient;
