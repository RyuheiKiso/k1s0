// 本ファイルは k1s0-sdk の PubSub 動詞統一 facade。
// `client.pubsub().publish(...)` 形式で PubSubService への呼出を提供する。

use crate::client::Client;
use crate::proto::k1s0::tier1::pubsub::v1::{
    BulkPublishRequest, Event, PublishRequest, SubscribeRequest,
    pub_sub_service_client::PubSubServiceClient,
};
use std::collections::HashMap;
use tonic::{Status, Streaming, transport::Channel};

/// BulkPublishEntryInput は bulk_publish の 1 件分の入力。
#[derive(Debug, Clone)]
pub struct BulkPublishEntryInput {
    /// データ本文。
    pub data: Vec<u8>,
    /// Content-Type（application/json / application/protobuf 等）。
    pub content_type: String,
    /// 冪等性キー（24h 重複抑止）。
    pub idempotency_key: String,
    /// メタデータ（partition_key 等）。
    pub metadata: HashMap<String, String>,
}

/// PubSubFacade は PubSubService の動詞統一 facade。
pub struct PubSubFacade {
    client: Client,
    raw: PubSubServiceClient<Channel>,
}

impl PubSubFacade {
    pub(crate) fn new(client: Client) -> Self {
        let raw = PubSubServiceClient::new(client.channel());
        Self { client, raw }
    }

    /// Publish は単発 Publish。Kafka offset を返す。
    /// idempotency_key が空文字でなければ 24h 重複抑止。
    pub async fn publish(
        &mut self,
        topic: &str,
        data: Vec<u8>,
        content_type: &str,
        idempotency_key: &str,
        metadata: HashMap<String, String>,
    ) -> Result<i64, Status> {
        // proto Request を構築する。
        let req = PublishRequest {
            topic: topic.to_string(),
            data,
            content_type: content_type.to_string(),
            idempotency_key: idempotency_key.to_string(),
            metadata,
            context: Some(self.client.tenant_context()),
        };
        // RPC 呼出。
        let resp = self.raw.publish(req).await?.into_inner();
        // offset を返却する。
        Ok(resp.offset)
    }

    /// bulk_publish は複数エントリの一括 Publish（FR-T1-PUBSUB-001）。
    /// 各エントリの結果を個別に返す（部分成功あり、全体エラーにはしない）。
    /// 戻り値は (entry_index, offset, error_code) の Vec。error_code が空でなければ失敗。
    pub async fn bulk_publish(
        &mut self,
        topic: &str,
        entries: Vec<BulkPublishEntryInput>,
    ) -> Result<Vec<(i32, i64, String)>, Status> {
        // 親 Client の TenantContext を 1 回構築して各 entry に共有する。
        let tctx = self.client.tenant_context();
        // SDK の入力 → proto PublishRequest に詰め替える。
        let pe: Vec<PublishRequest> = entries
            .into_iter()
            .map(|e| PublishRequest {
                topic: topic.to_string(),
                data: e.data,
                content_type: e.content_type,
                idempotency_key: e.idempotency_key,
                metadata: e.metadata,
                context: Some(tctx.clone()),
            })
            .collect();
        let req = BulkPublishRequest {
            topic: topic.to_string(),
            entries: pe,
        };
        let resp = self.raw.bulk_publish(req).await?.into_inner();
        let out: Vec<(i32, i64, String)> = resp
            .results
            .into_iter()
            .map(|r| (r.entry_index, r.offset, r.error_code))
            .collect();
        Ok(out)
    }

    /// subscribe はトピックの購読。`tonic::Streaming<Event>` を返し、利用者は
    /// `while let Some(ev) = stream.message().await? { ... }` で消費する。
    pub async fn subscribe(
        &mut self,
        topic: &str,
        consumer_group: &str,
    ) -> Result<Streaming<Event>, Status> {
        let resp = self
            .raw
            .subscribe(SubscribeRequest {
                topic: topic.to_string(),
                consumer_group: consumer_group.to_string(),
                context: Some(self.client.tenant_context()),
            })
            .await?;
        Ok(resp.into_inner())
    }
}
