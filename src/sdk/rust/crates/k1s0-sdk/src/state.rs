// 本ファイルは k1s0-sdk の State 動詞統一 facade。
// `client.state().save(...)` 形式で StateService への呼出を提供する。

use crate::client::Client;
use crate::proto::k1s0::tier1::state::v1::{
    DeleteRequest, GetRequest, SetRequest, state_service_client::StateServiceClient,
};
use tonic::{Status, transport::Channel};

/// StateFacade は StateService の動詞統一 facade。
pub struct StateFacade {
    /// 内部に Client（Channel + Config 共有）を保持する。
    client: Client,
    /// 都度生成を避けるため StateServiceClient<Channel> を保持する。
    raw: StateServiceClient<Channel>,
}

impl StateFacade {
    /// Client から StateFacade を生成する（Client::state() から呼ばれる）。
    pub(crate) fn new(client: Client) -> Self {
        // raw client を構築する。
        let raw = StateServiceClient::new(client.channel());
        // 構造体を返却する。
        Self { client, raw }
    }

    /// Get はキー単位の取得。未存在時は Ok(None) を返す。
    pub async fn get(
        &mut self,
        store: &str,
        key: &str,
    ) -> Result<Option<(Vec<u8>, String)>, Status> {
        // proto Request を構築する。
        let req = GetRequest {
            store: store.to_string(),
            key: key.to_string(),
            context: Some(self.client.tenant_context()),
        };
        // 生成 stub 経由で RPC を呼び出す。
        let resp = self.raw.get(req).await?.into_inner();
        // 未存在時は Ok(None)。
        if resp.not_found {
            return Ok(None);
        }
        // (data, etag) を Ok(Some) で返却する。
        Ok(Some((resp.data, resp.etag)))
    }

    /// Save はキー単位の保存。新 ETag を返す。
    /// expected_etag が空の場合は無条件、ttl_sec=0 は永続。
    pub async fn save(
        &mut self,
        store: &str,
        key: &str,
        data: Vec<u8>,
        expected_etag: &str,
        ttl_sec: i32,
    ) -> Result<String, Status> {
        // proto Request を構築する。
        let req = SetRequest {
            store: store.to_string(),
            key: key.to_string(),
            data,
            expected_etag: expected_etag.to_string(),
            ttl_sec,
            context: Some(self.client.tenant_context()),
        };
        // RPC 呼出。
        let resp = self.raw.set(req).await?.into_inner();
        // 新 ETag を返却する。
        Ok(resp.new_etag)
    }

    /// Delete はキー単位の削除。expected_etag が空なら無条件。
    pub async fn delete(
        &mut self,
        store: &str,
        key: &str,
        expected_etag: &str,
    ) -> Result<bool, Status> {
        // proto Request を構築する。
        let req = DeleteRequest {
            store: store.to_string(),
            key: key.to_string(),
            expected_etag: expected_etag.to_string(),
            context: Some(self.client.tenant_context()),
        };
        // RPC 呼出。
        let resp = self.raw.delete(req).await?.into_inner();
        // deleted フラグを返却する。
        Ok(resp.deleted)
    }
}
