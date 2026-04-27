// 本ファイルは k1s0-sdk の Binding 動詞統一 facade。
use crate::client::Client;
use crate::proto::k1s0::tier1::binding::v1::{
    InvokeBindingRequest, binding_service_client::BindingServiceClient,
};
use std::collections::HashMap;
use tonic::{Status, transport::Channel};

/// BindingFacade は BindingService の動詞統一 facade。
pub struct BindingFacade {
    client: Client,
    raw: BindingServiceClient<Channel>,
}

impl BindingFacade {
    pub(crate) fn new(client: Client) -> Self {
        let raw = BindingServiceClient::new(client.channel());
        Self { client, raw }
    }

    /// invoke は出力バインディング呼出。返り値は (data, metadata)。
    pub async fn invoke(
        &mut self,
        name: &str,
        operation: &str,
        data: Vec<u8>,
        metadata: HashMap<String, String>,
    ) -> Result<(Vec<u8>, HashMap<String, String>), Status> {
        let resp = self
            .raw
            .invoke(InvokeBindingRequest {
                name: name.to_string(),
                operation: operation.to_string(),
                data,
                metadata,
                context: Some(self.client.tenant_context()),
            })
            .await?
            .into_inner();
        Ok((resp.data, resp.metadata))
    }
}
