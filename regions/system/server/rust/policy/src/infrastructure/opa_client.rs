use std::time::Duration;

use serde_json::Value;

use super::config::OpaConfig;

/// OPA (Open Policy Agent) HTTP API client.
pub struct OpaClient {
    client: reqwest::Client,
    base_url: String,
}

impl OpaClient {
    pub fn new(config: &OpaConfig) -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms))
            .build()?;
        let base_url = config.url.trim_end_matches('/').to_string();
        Ok(Self { client, base_url })
    }

    /// Evaluate a policy against the given input.
    ///
    /// `package_path` uses dot-separated notation (e.g. "k1s0.system.tenant")
    /// which is converted to slash-separated for the OPA Data API.
    ///
    /// Returns `true` if `result.allow` is `true`, otherwise `false` (deny-by-default).
    pub async fn evaluate(&self, package_path: &str, input: &Value) -> anyhow::Result<bool> {
        let path = package_path.replace('.', "/");
        let url = format!("{}/v1/data/{}", self.base_url, path);

        let body = serde_json::json!({ "input": input });

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        let json: Value = response.json().await?;

        Ok(json
            .get("result")
            .and_then(|r| r.get("allow"))
            .and_then(|a| a.as_bool())
            .unwrap_or(false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn make_config(url: &str) -> OpaConfig {
        OpaConfig {
            url: url.to_string(),
            timeout_ms: 5000,
        }
    }

    #[tokio::test]
    async fn evaluate_allowed_true() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/data/k1s0/system/tenant"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"result": {"allow": true}})),
            )
            .mount(&server)
            .await;

        let client = OpaClient::new(&make_config(&server.uri())).unwrap();
        let result = client
            .evaluate("k1s0.system.tenant", &serde_json::json!({"action": "read"}))
            .await
            .unwrap();

        assert!(result);
    }

    #[tokio::test]
    async fn evaluate_allowed_false() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/data/k1s0/system/tenant"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"result": {"allow": false}})),
            )
            .mount(&server)
            .await;

        let client = OpaClient::new(&make_config(&server.uri())).unwrap();
        let result = client
            .evaluate("k1s0.system.tenant", &serde_json::json!({"action": "delete"}))
            .await
            .unwrap();

        assert!(!result);
    }

    #[tokio::test]
    async fn evaluate_no_result_field_denies() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/data/k1s0/system/tenant"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({})),
            )
            .mount(&server)
            .await;

        let client = OpaClient::new(&make_config(&server.uri())).unwrap();
        let result = client
            .evaluate("k1s0.system.tenant", &serde_json::json!({}))
            .await
            .unwrap();

        assert!(!result, "missing result field should deny by default");
    }
}
