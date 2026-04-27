// 本ファイルは k1s0-sdk の ServiceInvoke 動詞統一 facade。
// InvokeStream は本リリース時点 では raw 経由（client.raw_state() と同パターンで raw アクセス）。
use crate::client::Client;
use crate::proto::k1s0::tier1::serviceinvoke::v1::{
    invoke_service_client::InvokeServiceClient, InvokeRequest,
};
use tonic::{transport::Channel, Status};

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
        let resp = self.raw.invoke(InvokeRequest {
            app_id: app_id.to_string(),
            method: method.to_string(),
            data,
            content_type: content_type.to_string(),
            context: Some(self.client.tenant_context()),
            timeout_ms,
        }).await?.into_inner();
        Ok((resp.data, resp.content_type, resp.status))
    }
}
