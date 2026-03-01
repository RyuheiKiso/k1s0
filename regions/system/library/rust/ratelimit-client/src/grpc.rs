#[cfg(feature = "grpc")]
mod inner {
    use async_trait::async_trait;
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};

    use crate::client::RateLimitClient;
    use crate::error::RateLimitError;
    use crate::types::{RateLimitPolicy, RateLimitResult, RateLimitStatus};

    pub struct GrpcRateLimitClient {
        http: reqwest::Client,
        base_url: String,
    }

    impl GrpcRateLimitClient {
        pub async fn new(server_url: impl Into<String>) -> Result<Self, RateLimitError> {
            let mut base = server_url.into();
            if !base.starts_with("http://") && !base.starts_with("https://") {
                base = format!("http://{}", base);
            }
            let base = base.trim_end_matches('/').to_string();
            Ok(Self {
                http: reqwest::Client::new(),
                base_url: base,
            })
        }
    }

    #[derive(Serialize)]
    struct CheckRequest {
        cost: u32,
    }

    #[derive(Deserialize)]
    struct CheckResponse {
        allowed: bool,
        remaining: u32,
        reset_at: String,
        retry_after_secs: Option<u64>,
    }

    #[derive(Serialize)]
    struct ConsumeRequest {
        cost: u32,
    }

    #[derive(Deserialize)]
    struct ConsumeResponse {
        remaining: u32,
        reset_at: String,
    }

    #[derive(Deserialize)]
    struct PolicyResponse {
        key: String,
        limit: u32,
        window_secs: u64,
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
                key: format!("{}: {}", op, msg),
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
    impl RateLimitClient for GrpcRateLimitClient {
        async fn check(&self, key: &str, cost: u32) -> Result<RateLimitStatus, RateLimitError> {
            let url = format!("{}/api/v1/ratelimit/{}/check", self.base_url, key);
            let resp = self
                .http
                .post(&url)
                .json(&CheckRequest { cost })
                .send()
                .await
                .map_err(map_reqwest_err)?;

            if !resp.status().is_success() {
                return Err(map_error_response(resp, "check").await);
            }

            let result: CheckResponse = resp.json().await.map_err(|e| {
                RateLimitError::ServerError(format!("check: decode response: {}", e))
            })?;

            Ok(RateLimitStatus {
                allowed: result.allowed,
                remaining: result.remaining,
                reset_at: parse_reset_at(&result.reset_at),
                retry_after_secs: result.retry_after_secs,
            })
        }

        async fn consume(&self, key: &str, cost: u32) -> Result<RateLimitResult, RateLimitError> {
            let url = format!("{}/api/v1/ratelimit/{}/consume", self.base_url, key);
            let resp = self
                .http
                .post(&url)
                .json(&ConsumeRequest { cost })
                .send()
                .await
                .map_err(map_reqwest_err)?;

            if !resp.status().is_success() {
                return Err(map_error_response(resp, "consume").await);
            }

            let result: ConsumeResponse = resp.json().await.map_err(|e| {
                RateLimitError::ServerError(format!("consume: decode response: {}", e))
            })?;

            Ok(RateLimitResult {
                remaining: result.remaining,
                reset_at: parse_reset_at(&result.reset_at),
            })
        }

        async fn get_limit(&self, key: &str) -> Result<RateLimitPolicy, RateLimitError> {
            let url = format!("{}/api/v1/ratelimit/{}/policy", self.base_url, key);
            let resp = self.http.get(&url).send().await.map_err(map_reqwest_err)?;

            if !resp.status().is_success() {
                return Err(map_error_response(resp, "get_limit").await);
            }

            let result: PolicyResponse = resp.json().await.map_err(|e| {
                RateLimitError::ServerError(format!("get_limit: decode response: {}", e))
            })?;

            Ok(RateLimitPolicy {
                key: result.key,
                limit: result.limit,
                window_secs: result.window_secs,
                algorithm: result.algorithm,
            })
        }
    }
}

#[cfg(feature = "grpc")]
pub use inner::GrpcRateLimitClient;
