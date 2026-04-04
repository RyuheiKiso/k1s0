use std::time::Duration;

use serde_json::Value;

use super::config::OpaConfig;

/// OPA (Open Policy Agent) HTTP API client.
pub struct OpaClient {
    client: reqwest::Client,
    base_url: String,
}

impl OpaClient {
    /// H-001 監査対応: package_path のアローリスト検証
    /// 英数字・アンダースコア・ドットのみ許可し、パストラバーサルを防止する
    fn validate_package_path(path: &str) -> anyhow::Result<()> {
        if path.is_empty() {
            return Err(anyhow::anyhow!("package_path must not be empty"));
        }
        // 先頭文字は英字またはアンダースコアのみ（数字・ドット先頭を禁止）
        let mut chars = path.chars();
        let first = chars.next().unwrap();
        if !first.is_ascii_alphabetic() && first != '_' {
            return Err(anyhow::anyhow!(
                "package_path must start with a letter or underscore, got: '{}'",
                first
            ));
        }
        // 残りの文字は英数字・アンダースコア・ドットのみ
        for c in chars {
            if !c.is_ascii_alphanumeric() && c != '_' && c != '.' {
                return Err(anyhow::anyhow!(
                    "package_path contains invalid character '{}'. Only [a-zA-Z0-9_.] are allowed.",
                    c
                ));
            }
        }
        Ok(())
    }

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
        // H-001 監査対応: パスインジェクション/SSRF 防止のためアローリスト検証を実施する
        // 英数字・アンダースコア・ドットのみ許可。ドット区切りのパッケージパス形式のみ有効
        Self::validate_package_path(package_path)?;

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
#[allow(clippy::unwrap_used)]
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
            .evaluate(
                "k1s0.system.tenant",
                &serde_json::json!({"action": "delete"}),
            )
            .await
            .unwrap();

        assert!(!result);
    }

    #[tokio::test]
    async fn evaluate_no_result_field_denies() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/data/k1s0/system/tenant"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
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
