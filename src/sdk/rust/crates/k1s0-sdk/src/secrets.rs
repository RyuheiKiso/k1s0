// 本ファイルは k1s0-sdk の Secrets 動詞統一 facade。
// `client.secrets().get(...)` 形式で SecretsService への呼出を提供する。

use crate::client::Client;
use crate::proto::k1s0::tier1::secrets::v1::{
    BulkGetSecretRequest, DecryptRequest, EncryptRequest, GetDynamicSecretRequest,
    GetSecretRequest, RotateKeyRequest, RotateSecretRequest,
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

    /// BulkGet はテナント配下の全シークレットを一括取得する（FR-T1-SECRETS-001）。
    /// 戻り値は シークレット名 → (values, version) の HashMap。
    pub async fn bulk_get(
        &mut self,
    ) -> Result<HashMap<String, (HashMap<String, String>, i32)>, Status> {
        let req = BulkGetSecretRequest {
            context: Some(self.client.tenant_context()),
        };
        let resp = self.raw.bulk_get(req).await?.into_inner();
        let mut out = HashMap::with_capacity(resp.results.len());
        for (name, sec) in resp.results.into_iter() {
            out.insert(name, (sec.values, sec.version));
        }
        Ok(out)
    }

    /// Encrypt は Transit Engine 経由の暗号化（FR-T1-SECRETS-003）。
    /// key_name は tier1 が <tenant_id>.<key_name> で自動 prefix する。
    /// aad は GCM 追加認証データ（同じ aad を Decrypt 時にも渡す必要あり）。
    /// 戻り値は (ciphertext, key_version)。
    pub async fn encrypt(
        &mut self,
        key_name: &str,
        plaintext: Vec<u8>,
        aad: Vec<u8>,
    ) -> Result<(Vec<u8>, i32), Status> {
        let req = EncryptRequest {
            context: Some(self.client.tenant_context()),
            key_name: key_name.to_string(),
            plaintext,
            aad,
        };
        let resp = self.raw.encrypt(req).await?.into_inner();
        Ok((resp.ciphertext, resp.key_version))
    }

    /// Decrypt は Transit Engine 経由の復号（FR-T1-SECRETS-003）。
    /// key_name / aad は Encrypt 時と同じ値を渡すこと。
    /// 戻り値は (plaintext, key_version)。
    pub async fn decrypt(
        &mut self,
        key_name: &str,
        ciphertext: Vec<u8>,
        aad: Vec<u8>,
    ) -> Result<(Vec<u8>, i32), Status> {
        let req = DecryptRequest {
            context: Some(self.client.tenant_context()),
            key_name: key_name.to_string(),
            ciphertext,
            aad,
        };
        let resp = self.raw.decrypt(req).await?.into_inner();
        Ok((resp.plaintext, resp.key_version))
    }

    /// RotateKey は Transit Engine の鍵をローテーションする（FR-T1-SECRETS-003）。
    /// 既存版は保持され、その鍵で暗号化された ciphertext は引き続き Decrypt 可能。
    /// 戻り値は (new_version, previous_version, rotated_at_ms)。
    pub async fn rotate_key(
        &mut self,
        key_name: &str,
    ) -> Result<(i32, i32, i64), Status> {
        let req = RotateKeyRequest {
            context: Some(self.client.tenant_context()),
            key_name: key_name.to_string(),
        };
        let resp = self.raw.rotate_key(req).await?.into_inner();
        Ok((resp.new_version, resp.previous_version, resp.rotated_at_ms))
    }
}
