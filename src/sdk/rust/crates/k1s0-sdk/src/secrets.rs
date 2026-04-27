// 本ファイルは k1s0-sdk の Secrets 動詞統一 facade。
// `client.secrets().get(...)` 形式で SecretsService への呼出を提供する。

use crate::client::Client;
use crate::proto::k1s0::tier1::secrets::v1::{
    secrets_service_client::SecretsServiceClient, GetSecretRequest, RotateSecretRequest,
};
use std::collections::HashMap;
use tonic::{transport::Channel, Status};

/// SecretsFacade は SecretsService の動詞統一 facade。
pub struct SecretsFacade {
    client: Client,
    raw: SecretsServiceClient<Channel>,
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
