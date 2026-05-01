// 本ファイルは k1s0-sdk の State 動詞統一 facade。
// `client.state().save(...)` 形式で StateService への呼出を提供する。

use crate::client::Client;
use crate::proto::k1s0::tier1::state::v1::{
    BulkGetRequest, DeleteRequest, GetRequest, SetRequest, TransactOp, TransactRequest,
    transact_op::Op as TransactOpInner, state_service_client::StateServiceClient,
};
use std::collections::HashMap;
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
    /// idempotency_key が空でなければ tier1 が 24h dedup する（共通規約 §「冪等性と再試行」）。
    pub async fn save(
        &mut self,
        store: &str,
        key: &str,
        data: Vec<u8>,
        expected_etag: &str,
        ttl_sec: i32,
        idempotency_key: &str,
    ) -> Result<String, Status> {
        // proto Request を構築する。
        let req = SetRequest {
            store: store.to_string(),
            key: key.to_string(),
            data,
            expected_etag: expected_etag.to_string(),
            ttl_sec,
            idempotency_key: idempotency_key.to_string(),
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

    /// BulkGet は複数キーの一括取得（FR-T1-STATE-003）。
    /// 1 回の呼出で最大 100 キー（tier1 側で強制、超過は ResourceExhausted）。
    /// 返却 map のエントリ値は (data, etag, found) の tuple。found=false は未存在。
    pub async fn bulk_get(
        &mut self,
        store: &str,
        keys: &[String],
    ) -> Result<HashMap<String, (Vec<u8>, String, bool)>, Status> {
        let req = BulkGetRequest {
            store: store.to_string(),
            keys: keys.to_vec(),
            context: Some(self.client.tenant_context()),
        };
        let resp = self.raw.bulk_get(req).await?.into_inner();
        let mut out = HashMap::with_capacity(resp.results.len());
        for (k, r) in resp.results.into_iter() {
            // proto GetResponse の not_found を反転して found に揃える。
            out.insert(k, (r.data, r.etag, !r.not_found));
        }
        Ok(out)
    }

    /// TransactOpInput は Transact の 1 操作（Set / Delete のいずれか）。
    /// SDK 利用者は TransactOpInput を作って Vec で渡す。
    pub fn op_set(
        key: &str,
        data: Vec<u8>,
        expected_etag: &str,
        ttl_sec: i32,
    ) -> TransactOpInput {
        TransactOpInput::Set {
            key: key.to_string(),
            data,
            expected_etag: expected_etag.to_string(),
            ttl_sec,
        }
    }

    /// TransactOpInput::Delete を生成するヘルパ。
    pub fn op_delete(key: &str, expected_etag: &str) -> TransactOpInput {
        TransactOpInput::Delete {
            key: key.to_string(),
            expected_etag: expected_etag.to_string(),
        }
    }

    /// Transact はトランザクション境界付き複数操作（FR-T1-STATE-005）。
    /// 全操作が成功するか全て失敗するの 2 値。最大 10 操作 / トランザクション。
    pub async fn transact(
        &mut self,
        store: &str,
        ops: Vec<TransactOpInput>,
    ) -> Result<bool, Status> {
        // SDK の TransactOpInput を proto TransactOp に変換する。
        let mut pops: Vec<TransactOp> = Vec::with_capacity(ops.len());
        for o in ops.into_iter() {
            match o {
                TransactOpInput::Set { key, data, expected_etag, ttl_sec } => {
                    pops.push(TransactOp {
                        op: Some(TransactOpInner::Set(SetRequest {
                            store: store.to_string(),
                            key,
                            data,
                            expected_etag,
                            ttl_sec,
                            idempotency_key: String::new(),
                            context: None,
                        })),
                    });
                }
                TransactOpInput::Delete { key, expected_etag } => {
                    pops.push(TransactOp {
                        op: Some(TransactOpInner::Delete(DeleteRequest {
                            store: store.to_string(),
                            key,
                            expected_etag,
                            context: None,
                        })),
                    });
                }
            }
        }
        let req = TransactRequest {
            store: store.to_string(),
            operations: pops,
            context: Some(self.client.tenant_context()),
        };
        let resp = self.raw.transact(req).await?.into_inner();
        Ok(resp.committed)
    }
}

/// TransactOpInput は Transact 内の 1 操作の種別と必要パラメータ。
#[derive(Debug, Clone)]
pub enum TransactOpInput {
    /// Set 操作（Key/Data/期待 ETag/TTL）。
    Set {
        key: String,
        data: Vec<u8>,
        expected_etag: String,
        ttl_sec: i32,
    },
    /// Delete 操作（Key/期待 ETag）。
    Delete {
        key: String,
        expected_etag: String,
    },
}
