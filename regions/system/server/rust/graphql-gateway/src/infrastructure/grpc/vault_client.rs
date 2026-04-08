use std::collections::HashMap;
use std::time::Duration;

use chrono::{DateTime, Utc};
use tonic::transport::Channel;
use tracing::instrument;

use crate::domain::model::{SecretMetadata, VaultAuditLogEntry};
use crate::infrastructure::config::BackendConfig;

// HIGH-001 監査対応: tonic::include_proto!で展開される生成コードのClippy警告を抑制する
#[allow(
    dead_code,
    clippy::default_trait_access,
    clippy::trivially_copy_pass_by_ref,
    clippy::too_many_lines,
    clippy::doc_markdown,
    clippy::must_use_candidate
)]
pub mod proto {
    pub mod k1s0 {
        pub mod system {
            pub mod common {
                pub mod v1 {
                    tonic::include_proto!("k1s0.system.common.v1");
                }
            }
            pub mod vault {
                pub mod v1 {
                    tonic::include_proto!("k1s0.system.vault.v1");
                }
            }
        }
    }
}

use proto::k1s0::system::vault::v1::vault_service_client::VaultServiceClient;

pub struct VaultGrpcClient {
    client: VaultServiceClient<Channel>,
    /// バックエンドサービスのアドレス。gRPC Health Check Protocol のためのチャネル生成に使用する。
    address: String,
    /// `タイムアウト設定（ミリ秒）。health_check` のチャネル生成にも適用する。
    timeout_ms: u64,
}

impl VaultGrpcClient {
    /// バックエンド設定からクライアントを生成する。
    /// `connect_lazy()` により起動時の接続確立を不要とし、実際のRPC呼び出し時に接続する。
    pub fn new(cfg: &BackendConfig) -> anyhow::Result<Self> {
        let channel = Channel::from_shared(cfg.address.clone())?
            .timeout(Duration::from_millis(cfg.timeout_ms))
            .connect_lazy();
        Ok(Self {
            client: VaultServiceClient::new(channel),
            address: cfg.address.clone(),
            timeout_ms: cfg.timeout_ms,
        })
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_secret_metadata(&self, path: &str) -> anyhow::Result<Option<SecretMetadata>> {
        let request =
            tonic::Request::new(proto::k1s0::system::vault::v1::GetSecretMetadataRequest {
                path: path.to_owned(),
            });

        match self.client.clone().get_secret_metadata(request).await {
            Ok(resp) => {
                let r = resp.into_inner();
                Ok(Some(SecretMetadata {
                    path: r.path,
                    current_version: r.current_version,
                    version_count: r.version_count,
                    created_at: timestamp_to_rfc3339(r.created_at),
                    updated_at: timestamp_to_rfc3339(r.updated_at),
                }))
            }
            Err(status) if status.code() == tonic::Code::NotFound => Ok(None),
            Err(e) => Err(anyhow::anyhow!(
                "VaultService.GetSecretMetadata failed: {e}"
            )),
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_secrets(&self, prefix: Option<&str>) -> anyhow::Result<Vec<String>> {
        let request = tonic::Request::new(proto::k1s0::system::vault::v1::ListSecretsRequest {
            prefix: prefix.unwrap_or_default().to_owned(),
            // pagination は None（全件取得）
            pagination: None,
        });

        let resp = self
            .client
            .clone()
            .list_secrets(request)
            .await
            .map_err(|e| anyhow::anyhow!("VaultService.ListSecrets failed: {e}"))?
            .into_inner();

        Ok(resp.keys)
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_audit_logs(
        &self,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> anyhow::Result<Vec<VaultAuditLogEntry>> {
        // offset/limit は Pagination サブメッセージに移行済み（Option<i32> 型のため as i32 キャスト不要）
        let request = tonic::Request::new(proto::k1s0::system::vault::v1::ListAuditLogsRequest {
            pagination: Some(proto::k1s0::system::common::v1::Pagination {
                page: offset.unwrap_or(0) + 1,
                page_size: limit.unwrap_or(50),
            }),
            // keyset ページネーション用カーソル（空文字列 = 先頭ページ）
            after_cursor: String::new(),
        });

        let resp = self
            .client
            .clone()
            .list_audit_logs(request)
            .await
            .map_err(|e| anyhow::anyhow!("VaultService.ListAuditLogs failed: {e}"))?
            .into_inner();

        let entries = resp
            .logs
            .into_iter()
            .map(|e| VaultAuditLogEntry {
                id: e.id,
                key_path: e.key_path,
                action: e.action,
                actor_id: e.actor_id,
                ip_address: e.ip_address,
                success: e.success,
                error_msg: e.error_msg.filter(|s| !s.is_empty()),
                created_at: timestamp_to_rfc3339(e.created_at),
            })
            .collect();

        Ok(entries)
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn set_secret(
        &self,
        path: &str,
        data: HashMap<String, String>,
    ) -> anyhow::Result<(String, i64, String)> {
        let request = tonic::Request::new(proto::k1s0::system::vault::v1::SetSecretRequest {
            path: path.to_owned(),
            data,
        });

        let resp = self
            .client
            .clone()
            .set_secret(request)
            .await
            .map_err(|e| anyhow::anyhow!("VaultService.SetSecret failed: {e}"))?
            .into_inner();

        Ok((
            resp.path,
            resp.version,
            timestamp_to_rfc3339(resp.created_at),
        ))
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn rotate_secret(
        &self,
        path: &str,
        data: HashMap<String, String>,
    ) -> anyhow::Result<(String, i64, bool)> {
        let request = tonic::Request::new(proto::k1s0::system::vault::v1::RotateSecretRequest {
            path: path.to_owned(),
            data,
        });

        let resp = self
            .client
            .clone()
            .rotate_secret(request)
            .await
            .map_err(|e| anyhow::anyhow!("VaultService.RotateSecret failed: {e}"))?
            .into_inner();

        Ok((resp.path, resp.new_version, resp.rotated))
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn delete_secret(&self, path: &str, versions: Vec<i64>) -> anyhow::Result<bool> {
        let request = tonic::Request::new(proto::k1s0::system::vault::v1::DeleteSecretRequest {
            path: path.to_owned(),
            versions,
        });

        let resp = self
            .client
            .clone()
            .delete_secret(request)
            .await
            .map_err(|e| anyhow::anyhow!("VaultService.DeleteSecret failed: {e}"))?
            .into_inner();

        Ok(resp.success)
    }

    /// gRPC Health Check Protocol を使ってサービスの疎通確認を行う。
    /// Bearer token なしで接続できるため readyz ヘルスチェックに適している。
    /// tonic-health サービスが登録されているサーバーに対して Check RPC を送信する。
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn health_check(&self) -> anyhow::Result<()> {
        let channel = Channel::from_shared(self.address.clone())?
            .timeout(Duration::from_millis(self.timeout_ms))
            .connect_lazy();
        let mut health_client = tonic_health::pb::health_client::HealthClient::new(channel);
        let request = tonic::Request::new(tonic_health::pb::HealthCheckRequest {
            service: "k1s0.system.vault.v1.VaultService".to_string(),
        });
        health_client
            .check(request)
            .await
            .map_err(|e| anyhow::anyhow!("vault gRPC Health Check 失敗: {e}"))?;
        Ok(())
    }
}

fn timestamp_to_rfc3339(ts: Option<proto::k1s0::system::common::v1::Timestamp>) -> String {
    ts.and_then(|ts| DateTime::<Utc>::from_timestamp(ts.seconds, ts.nanos as u32))
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_default()
}
