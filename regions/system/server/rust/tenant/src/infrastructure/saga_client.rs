use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;

/// SagaClient はテナントプロビジョニング Saga を開始するトレイト。
#[async_trait]
pub trait SagaClient: Send + Sync {
    /// テナント作成後のプロビジョニング Saga を開始する。
    async fn start_provisioning_saga(&self, tenant_id: &str, tenant_name: &str) -> Result<()>;

    /// テナント削除時のデプロビジョニング Saga を開始する。
    async fn start_deprovisioning_saga(&self, tenant_id: &str, tenant_name: &str) -> Result<()>;
}

/// NoopSagaClient は Saga 連携なしのデフォルト実装。
pub struct NoopSagaClient;

#[async_trait]
impl SagaClient for NoopSagaClient {
    async fn start_provisioning_saga(&self, _tenant_id: &str, _tenant_name: &str) -> Result<()> {
        tracing::debug!("saga client not configured, skipping provisioning saga");
        Ok(())
    }

    async fn start_deprovisioning_saga(&self, _tenant_id: &str, _tenant_name: &str) -> Result<()> {
        tracing::debug!("saga client not configured, skipping deprovisioning saga");
        Ok(())
    }
}

/// HttpSagaClient は saga-server に HTTP で Saga 開始を依頼する実装。
pub struct HttpSagaClient {
    client: reqwest::Client,
    base_url: String,
}

impl HttpSagaClient {
    /// 新しい HttpSagaClient を生成する。
    /// デフォルトタイムアウト30秒でHTTPクライアントを構築する。
    /// TLS バックエンドの初期化に失敗した場合は Err を返す。
    pub fn new(base_url: &str) -> anyhow::Result<Self> {
        // reqwest の Client 構築: TLS バックエンドが利用不可の場合はエラーとして伝播する
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| anyhow::anyhow!("HTTP クライアントの構築に失敗: {}", e))?;
        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
        })
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
                "payload": {
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

    async fn start_deprovisioning_saga(&self, tenant_id: &str, tenant_name: &str) -> Result<()> {
        let url = format!("{}/api/v1/sagas", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "workflow_name": "tenant-deprovisioning",
                "payload": {
                    "tenant_id": tenant_id,
                    "tenant_name": tenant_name
                }
            }))
            .send()
            .await?;
        if resp.status().is_success() {
            tracing::info!(tenant_id = %tenant_id, "deprovisioning saga started");
            Ok(())
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            tracing::error!(status = %status, body = %body, "failed to start deprovisioning saga");
            anyhow::bail!("saga server returned {}: {}", status, body)
        }
    }
}
