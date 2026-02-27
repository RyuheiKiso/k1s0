use async_trait::async_trait;
use anyhow::Result;

/// SagaClient はテナントプロビジョニング Saga を開始するトレイト。
#[async_trait]
pub trait SagaClient: Send + Sync {
    /// テナント作成後のプロビジョニング Saga を開始する。
    async fn start_provisioning_saga(&self, tenant_id: &str, tenant_name: &str) -> Result<()>;
}

/// NoopSagaClient は Saga 連携なしのデフォルト実装。
pub struct NoopSagaClient;

#[async_trait]
impl SagaClient for NoopSagaClient {
    async fn start_provisioning_saga(&self, _tenant_id: &str, _tenant_name: &str) -> Result<()> {
        tracing::debug!("saga client not configured, skipping provisioning saga");
        Ok(())
    }
}

/// HttpSagaClient は saga-server に HTTP で Saga 開始を依頼する実装。
pub struct HttpSagaClient {
    client: reqwest::Client,
    base_url: String,
}

impl HttpSagaClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }
}

#[async_trait]
impl SagaClient for HttpSagaClient {
    async fn start_provisioning_saga(&self, tenant_id: &str, tenant_name: &str) -> Result<()> {
        let url = format!("{}/api/v1/sagas", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "workflow_name": "tenant-provisioning",
                "input": {
                    "tenant_id": tenant_id,
                    "tenant_name": tenant_name
                }
            }))
            .send()
            .await?;
        if resp.status().is_success() {
            tracing::info!(tenant_id = %tenant_id, "provisioning saga started");
            Ok(())
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            tracing::error!(status = %status, body = %body, "failed to start provisioning saga");
            anyhow::bail!("saga server returned {}: {}", status, body)
        }
    }
}
