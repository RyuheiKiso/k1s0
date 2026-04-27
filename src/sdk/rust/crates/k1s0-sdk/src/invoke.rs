// 本ファイルは k1s0-sdk の ServiceInvoke 動詞統一 facade（unary + server streaming）。
use crate::client::Client;
use crate::proto::k1s0::tier1::serviceinvoke::v1::{
    InvokeChunk, InvokeRequest, invoke_service_client::InvokeServiceClient,
};
use tonic::{Status, Streaming, transport::Channel};

/// InvokeFacade は InvokeService の動詞統一 facade。
pub struct InvokeFacade {
    client: Client,
    raw: InvokeServiceClient<Channel>,
}

impl InvokeFacade {
    pub(crate) fn new(client: Client) -> Self {
        let raw = InvokeServiceClient::new(client.channel());
        Self { client, raw }
    }

    /// call は任意サービスの任意メソッドを呼び出す（unary）。返り値は (data, content_type, status)。
    pub async fn call(
        &mut self,
        app_id: &str,
        method: &str,
        data: Vec<u8>,
        content_type: &str,
        timeout_ms: i32,
    ) -> Result<(Vec<u8>, String, i32), Status> {
        let resp = self
            .raw
            .invoke(InvokeRequest {
                app_id: app_id.to_string(),
                method: method.to_string(),
                data,
                content_type: content_type.to_string(),
                context: Some(self.client.tenant_context()),
                timeout_ms,
            })
            .await?
            .into_inner();
        Ok((resp.data, resp.content_type, resp.status))
    }

    /// stream はサーバストリーミング呼出。`tonic::Streaming<InvokeChunk>` を返し、
    /// 利用者は `while let Some(chunk) = stream.message().await? { ... }` で消費する。
    pub async fn stream(
        &mut self,
        app_id: &str,
        method: &str,
        data: Vec<u8>,
        content_type: &str,
        timeout_ms: i32,
    ) -> Result<Streaming<InvokeChunk>, Status> {
        let resp = self
            .raw
            .invoke_stream(InvokeRequest {
                app_id: app_id.to_string(),
                method: method.to_string(),
                data,
                content_type: content_type.to_string(),
                context: Some(self.client.tenant_context()),
                timeout_ms,
            })
            .await?;
        Ok(resp.into_inner())
    }
}
