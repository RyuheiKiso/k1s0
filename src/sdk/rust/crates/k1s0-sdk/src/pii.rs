// 本ファイルは k1s0-sdk の Pii 動詞統一 facade。
use crate::client::Client;
use crate::proto::k1s0::tier1::pii::v1::{
    ClassifyRequest, MaskRequest, PiiFinding, pii_service_client::PiiServiceClient,
};
use tonic::{Status, transport::Channel};

/// PiiFacade は PiiService の動詞統一 facade。
pub struct PiiFacade {
    client: Client,
    raw: PiiServiceClient<Channel>,
}

impl PiiFacade {
    pub(crate) fn new(client: Client) -> Self {
        let raw = PiiServiceClient::new(client.channel());
        Self { client, raw }
    }

    /// classify は PII 種別の検出。返り値は (findings, contains_pii)。
    pub async fn classify(&mut self, text: &str) -> Result<(Vec<PiiFinding>, bool), Status> {
        let resp = self
            .raw
            .classify(ClassifyRequest {
                text: text.to_string(),
                context: Some(self.client.tenant_context()),
            })
            .await?
            .into_inner();
        Ok((resp.findings, resp.contains_pii))
    }

    /// mask はマスキング。返り値は (masked_text, findings)。
    pub async fn mask(&mut self, text: &str) -> Result<(String, Vec<PiiFinding>), Status> {
        let resp = self
            .raw
            .mask(MaskRequest {
                text: text.to_string(),
                context: Some(self.client.tenant_context()),
            })
            .await?
            .into_inner();
        Ok((resp.masked_text, resp.findings))
    }
}
