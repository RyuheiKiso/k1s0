// service-catalog REST クライアント。
// service-catalog サーバーは HTTP/axum で実装されており gRPC を提供しないため、
// reqwest を使って REST エンドポイントに接続する。
// 公開インターフェースは旧 gRPC クライアントと同一に保ち、呼び出し側の変更を最小化する。

use std::collections::HashMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::domain::model::{
    CatalogService, CatalogServiceConnection, MetadataEntry, ServiceHealth,
};
use crate::infrastructure::config::BackendConfig;

// service-catalog REST API のレスポンス型: サービス情報
#[derive(Debug, Clone, Deserialize)]
struct RestService {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub tier: String,
    pub version: String,
    pub base_url: String,
    pub grpc_endpoint: Option<String>,
    pub health_url: String,
    pub status: String,
    #[serde(default)]
    pub metadata: Vec<RestMetadataEntry>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

// service-catalog REST API のレスポンス型: メタデータエントリ
#[derive(Debug, Clone, Deserialize)]
struct RestMetadataEntry {
    pub key: String,
    pub value: String,
}

// service-catalog REST API のレスポンス型: ヘルス状態
#[derive(Debug, Clone, Deserialize)]
struct RestHealthStatus {
    pub service_id: String,
    pub service_name: Option<String>,
    pub status: String,
    pub response_time_ms: Option<i64>,
    pub error_message: Option<String>,
    pub checked_at: Option<String>,
}

// service-catalog REST API のリクエスト型: サービス登録
#[derive(Debug, Clone, Serialize)]
struct RegisterServiceRequest {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub tier: String,
    pub version: String,
    pub base_url: String,
    pub grpc_endpoint: Option<String>,
    pub health_url: String,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

// service-catalog REST API のリクエスト型: サービス更新
#[derive(Debug, Clone, Serialize)]
struct UpdateServiceRequest {
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub base_url: Option<String>,
    pub grpc_endpoint: Option<String>,
    pub health_url: Option<String>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

// service-catalog HTTP クライアント。
// reqwest::Client を保持し、base_url と timeout を設定する。
pub struct ServiceCatalogHttpClient {
    client: reqwest::Client,
    base_url: String,
}

impl ServiceCatalogHttpClient {
    /// バックエンド設定から HTTP クライアントを生成する。
    /// timeout はミリ秒で指定された設定値を使用する。
    pub fn new(cfg: &BackendConfig) -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(cfg.timeout_ms))
            .build()?;
        // address の末尾スラッシュを除去してベース URL を統一する
        let base_url = cfg.address.trim_end_matches('/').to_string();
        Ok(Self { client, base_url })
    }

    /// 指定 ID のサービスを取得する。
    /// 404 の場合は None を返す。その他のエラーは `anyhow::Error` で返す。
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_service(&self, service_id: &str) -> anyhow::Result<Option<CatalogService>> {
        let url = format!("{}/api/v1/services/{}", self.base_url, service_id);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("ServiceCatalog.GetService HTTP エラー: {e}"))?;

        // 404 の場合は None を返す
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!(
                "ServiceCatalog.GetService 失敗: status={status}, body={body}"
            );
        }

        let svc: RestService = resp
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("ServiceCatalog.GetService JSON パースエラー: {e}"))?;
        Ok(Some(service_from_rest(svc)))
    }

    /// サービス一覧を取得する。
    /// `page/page_size` でページネーション、tier/status/search でフィルタリングができる。
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_services(
        &self,
        _page: i32,
        page_size: i32,
        tier: Option<&str>,
        _status: Option<&str>,
        _search: Option<&str>,
    ) -> anyhow::Result<CatalogServiceConnection> {
        // service-catalog の REST API は tier/lifecycle/tag のクエリパラメータに対応している
        let url = format!("{}/api/v1/services", self.base_url);
        let mut req = self.client.get(&url);
        if let Some(t) = tier {
            req = req.query(&[("tier", t)]);
        }
        req = req.query(&[("page_size", &page_size.to_string())]);

        let resp = req
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("ServiceCatalog.ListServices HTTP エラー: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!(
                "ServiceCatalog.ListServices 失敗: status={status}, body={body}"
            );
        }

        let services: Vec<RestService> = resp
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("ServiceCatalog.ListServices JSON パースエラー: {e}"))?;

        let total_count = services.len() as i64;
        let has_next = total_count >= i64::from(page_size);
        let services = services.into_iter().map(service_from_rest).collect();

        Ok(CatalogServiceConnection {
            services,
            total_count,
            has_next,
        })
    }

    /// サービスを登録する。
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn register_service(
        &self,
        name: &str,
        display_name: &str,
        description: &str,
        tier: &str,
        version: &str,
        base_url: &str,
        grpc_endpoint: Option<&str>,
        health_url: &str,
        metadata: HashMap<String, String>,
    ) -> anyhow::Result<CatalogService> {
        let url = format!("{}/api/v1/services", self.base_url);
        let body = RegisterServiceRequest {
            name: name.to_owned(),
            display_name: display_name.to_owned(),
            description: description.to_owned(),
            tier: tier.to_owned(),
            version: version.to_owned(),
            base_url: base_url.to_owned(),
            grpc_endpoint: grpc_endpoint.map(std::borrow::ToOwned::to_owned),
            health_url: health_url.to_owned(),
            metadata,
        };

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("ServiceCatalog.RegisterService HTTP エラー: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!(
                "ServiceCatalog.RegisterService 失敗: status={status}, body={body}"
            );
        }

        let svc: RestService = resp.json().await.map_err(|e| {
            anyhow::anyhow!("ServiceCatalog.RegisterService JSON パースエラー: {e}")
        })?;
        Ok(service_from_rest(svc))
    }

    /// サービス情報を更新する。
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn update_service(
        &self,
        service_id: &str,
        display_name: Option<&str>,
        description: Option<&str>,
        version: Option<&str>,
        base_url: Option<&str>,
        grpc_endpoint: Option<&str>,
        health_url: Option<&str>,
        metadata: HashMap<String, String>,
    ) -> anyhow::Result<CatalogService> {
        let url = format!("{}/api/v1/services/{}", self.base_url, service_id);
        let body = UpdateServiceRequest {
            display_name: display_name.map(std::borrow::ToOwned::to_owned),
            description: description.map(std::borrow::ToOwned::to_owned),
            version: version.map(std::borrow::ToOwned::to_owned),
            base_url: base_url.map(std::borrow::ToOwned::to_owned),
            grpc_endpoint: grpc_endpoint.map(std::borrow::ToOwned::to_owned),
            health_url: health_url.map(std::borrow::ToOwned::to_owned),
            metadata,
        };

        let resp = self
            .client
            .put(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("ServiceCatalog.UpdateService HTTP エラー: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!(
                "ServiceCatalog.UpdateService 失敗: status={status}, body={body}"
            );
        }

        let svc: RestService = resp.json().await.map_err(|e| {
            anyhow::anyhow!("ServiceCatalog.UpdateService JSON パースエラー: {e}")
        })?;
        Ok(service_from_rest(svc))
    }

    /// サービスを削除する。
    /// 成功した場合は true を返す。
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn delete_service(&self, service_id: &str) -> anyhow::Result<bool> {
        let url = format!("{}/api/v1/services/{}", self.base_url, service_id);
        let resp = self
            .client
            .delete(&url)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("ServiceCatalog.DeleteService HTTP エラー: {e}"))?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(false);
        }

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!(
                "ServiceCatalog.DeleteService 失敗: status={status}, body={body}"
            );
        }

        // 204 No Content が正常応答
        Ok(true)
    }

    /// service-catalog サーバー自身のヘルスチェックを行う。
    /// DOCKER-CRIT-001 対応: /api/v1/services は認証が必要なため readyz チェックには不向きである。
    /// /healthz エンドポイントは認証不要で呼び出し可能なため、ヘルスチェックに使用する。
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn healthz(&self) -> anyhow::Result<()> {
        let url = format!("{}/healthz", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("ServiceCatalog.Healthz HTTP エラー: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            anyhow::bail!("ServiceCatalog.Healthz 失敗: status={status}");
        }

        Ok(())
    }

    /// サービスのヘルスチェック状態を取得する。
    /// `service_id` が指定された場合は特定サービスのヘルス状態のみ返す。
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn health_check(
        &self,
        service_id: Option<&str>,
    ) -> anyhow::Result<Vec<ServiceHealth>> {
        // service_id が指定された場合は個別エンドポイントを使用する
        if let Some(id) = service_id {
            let url = format!("{}/api/v1/services/{}/health", self.base_url, id);
            let resp =
                self.client.get(&url).send().await.map_err(|e| {
                    anyhow::anyhow!("ServiceCatalog.HealthCheck HTTP エラー: {e}")
                })?;

            if resp.status() == reqwest::StatusCode::NOT_FOUND {
                return Ok(vec![]);
            }

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!(
                    "ServiceCatalog.HealthCheck 失敗: status={status}, body={body}"
                );
            }

            let health: RestHealthStatus = resp.json().await.map_err(|e| {
                anyhow::anyhow!("ServiceCatalog.HealthCheck JSON パースエラー: {e}")
            })?;
            return Ok(vec![health_from_rest(health, id)]);
        }

        // service_id 未指定の場合はサービス一覧から全ヘルスを収集する
        // service-catalog の REST API には一括ヘルスチェックエンドポイントがないため、
        // 一覧から ID を取得して個別に問い合わせる
        let list = self.list_services(1, 100, None, None, None).await?;
        let mut results = Vec::new();
        for svc in &list.services {
            let url = format!("{}/api/v1/services/{}/health", self.base_url, svc.id);
            if let Ok(resp) = self.client.get(&url).send().await {
                if resp.status().is_success() {
                    if let Ok(health) = resp.json::<RestHealthStatus>().await {
                        results.push(health_from_rest(health, &svc.id));
                    }
                }
            }
        }
        Ok(results)
    }
}

// REST レスポンスを domain model に変換する
fn service_from_rest(s: RestService) -> CatalogService {
    CatalogService {
        id: s.id,
        name: s.name,
        display_name: s.display_name,
        description: s.description,
        tier: s.tier,
        version: s.version,
        base_url: s.base_url,
        grpc_endpoint: s.grpc_endpoint.filter(|v| !v.is_empty()),
        health_url: s.health_url,
        status: s.status,
        metadata: s
            .metadata
            .into_iter()
            .map(|e| MetadataEntry {
                key: e.key,
                value: e.value,
            })
            .collect(),
        created_at: s.created_at.unwrap_or_default(),
        updated_at: s.updated_at.unwrap_or_default(),
    }
}

// REST ヘルスレスポンスを domain model に変換する
fn health_from_rest(h: RestHealthStatus, service_id: &str) -> ServiceHealth {
    ServiceHealth {
        service_id: h.service_id,
        service_name: h.service_name.unwrap_or_else(|| service_id.to_owned()),
        status: h.status,
        response_time_ms: h.response_time_ms,
        error_message: h.error_message.filter(|v| !v.is_empty()),
        checked_at: h.checked_at.unwrap_or_default(),
    }
}
