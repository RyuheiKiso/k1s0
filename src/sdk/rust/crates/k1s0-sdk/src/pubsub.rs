// 本ファイルは k1s0-sdk の PubSub 動詞統一 facade。
// `client.pubsub().publish(...)` 形式で PubSubService への呼出を提供する。

use crate::client::Client;
use crate::proto::k1s0::tier1::pubsub::v1::{
    pub_sub_service_client::PubSubServiceClient, PublishRequest,
};
use std::collections::HashMap;
use tonic::{transport::Channel, Status};

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
}
