// 本ファイルは k1s0-sdk の Pii 動詞統一 facade。
use crate::client::Client;
use crate::proto::k1s0::tier1::pii::v1::{
    ClassifyRequest, MaskRequest, PiiFinding, PseudonymizeRequest,
    pii_service_client::PiiServiceClient,
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

    /// pseudonymize は FR-T1-PII-002（決定論的仮名化）の facade。
    /// 同一 salt + 同一 field_type + 同一 value で同一の URL-safe base64 仮名値を返す。
    /// salt / value / field_type いずれかが空文字なら server 側で InvalidArgument。
    pub async fn pseudonymize(
        &mut self,
        field_type: &str,
        value: &str,
        salt: &str,
    ) -> Result<String, Status> {
        let resp = self
            .raw
            .pseudonymize(PseudonymizeRequest {
                field_type: field_type.to_string(),
                value: value.to_string(),
                salt: salt.to_string(),
                context: Some(self.client.tenant_context()),
            })
            .await?
            .into_inner();
        Ok(resp.pseudonym)
    }
}
