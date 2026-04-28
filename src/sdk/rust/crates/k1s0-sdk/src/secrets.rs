// 本ファイルは k1s0-sdk の Secrets 動詞統一 facade。
// `client.secrets().get(...)` 形式で SecretsService への呼出を提供する。

use crate::client::Client;
use crate::proto::k1s0::tier1::secrets::v1::{
    GetDynamicSecretRequest, GetSecretRequest, RotateSecretRequest,
    secrets_service_client::SecretsServiceClient,
};
use std::collections::HashMap;
use tonic::{Status, transport::Channel};

/// SecretsFacade は SecretsService の動詞統一 facade。
pub struct SecretsFacade {
    client: Client,
    raw: SecretsServiceClient<Channel>,
}

/// 動的 Secret 発行（FR-T1-SECRETS-002）の応答を SDK 利用者向けに整理した型。
#[derive(Debug, Clone)]
pub struct DynamicSecret {
    /// credential 一式（"username" / "password" など、engine 別の field）。
    pub values: HashMap<String, String>,
    /// OpenBao の lease ID（renewal / revoke 用）。
    pub lease_id: String,
    /// 実際に付与された TTL 秒（要求値から ceiling までクランプされる）。
    pub ttl_sec: i32,
    /// 発効時刻（Unix epoch ミリ秒）。
    pub issued_at_ms: i64,
}

impl SecretsFacade {
    pub(crate) fn new(client: Client) -> Self {
        let raw = SecretsServiceClient::new(client.channel());
        Self { client, raw }
    }

    /// Get はシークレット名で値（key=value マップ）と version を取得する。
    pub async fn get(&mut self, name: &str) -> Result<(HashMap<String, String>, i32), Status> {
        // proto Request を構築する。
        let req = GetSecretRequest {
            name: name.to_string(),
            context: Some(self.client.tenant_context()),
            version: None,
        };
        // RPC 呼出。
        let resp = self.raw.get(req).await?.into_inner();
        // (values, version) を返却する。
        Ok((resp.values, resp.version))
    }

    /// 動的 Secret 発行（FR-T1-SECRETS-002）の応答 SDK 型。
    /// engine 別の credential（OpenBao Database Engine の標準では username / password）
    /// と OpenBao lease ID を返す。
    pub async fn get_dynamic(
        &mut self,
        engine: &str,
        role: &str,
        ttl_sec: i32,
    ) -> Result<DynamicSecret, Status> {
        // proto Request を構築する。
        let req = GetDynamicSecretRequest {
            engine: engine.to_string(),
            role: role.to_string(),
            ttl_sec,
            context: Some(self.client.tenant_context()),
        };
        // RPC 呼出。
        let resp = self.raw.get_dynamic(req).await?.into_inner();
        // SDK 型に詰め替える。
        Ok(DynamicSecret {
            values: resp.values,
            lease_id: resp.lease_id,
            ttl_sec: resp.ttl_sec,
            issued_at_ms: resp.issued_at_ms,
        })
    }

    /// Rotate はシークレットのローテーション。新バージョンと旧バージョンを返す。
    pub async fn rotate(
        &mut self,
        name: &str,
        grace_period_sec: i32,
        idempotency_key: &str,
    ) -> Result<(i32, i32), Status> {
        // proto Request を構築する。
        let req = RotateSecretRequest {
            name: name.to_string(),
            context: Some(self.client.tenant_context()),
            grace_period_sec,
            policy: None,
            idempotency_key: idempotency_key.to_string(),
        };
        // RPC 呼出。
        let resp = self.raw.rotate(req).await?.into_inner();
        // (new_version, previous_version) を返却する。
        Ok((resp.new_version, resp.previous_version))
    }
}
