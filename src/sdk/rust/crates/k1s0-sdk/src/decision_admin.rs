// 本ファイルは k1s0-sdk の DecisionAdmin 動詞統一 facade。
use crate::client::Client;
use crate::proto::k1s0::tier1::decision::v1::{
    decision_admin_service_client::DecisionAdminServiceClient, GetRuleRequest, ListVersionsRequest,
    RegisterRuleRequest, RuleVersionMeta,
};
use tonic::{transport::Channel, Status};

pub struct DecisionAdminFacade {
    client: Client,
    raw: DecisionAdminServiceClient<Channel>,
}

impl DecisionAdminFacade {
    pub(crate) fn new(client: Client) -> Self {
        let raw = DecisionAdminServiceClient::new(client.channel());
        Self { client, raw }
    }

    /// register_rule は JDM 文書の登録。返り値は (rule_version, effective_at_ms)。
    pub async fn register_rule(
        &mut self,
        rule_id: &str,
        jdm_document: Vec<u8>,
        sigstore_signature: Vec<u8>,
        commit_hash: &str,
    ) -> Result<(String, i64), Status> {
        let resp = self.raw.register_rule(RegisterRuleRequest {
            rule_id: rule_id.to_string(),
            jdm_document,
            sigstore_signature,
            commit_hash: commit_hash.to_string(),
            context: Some(self.client.tenant_context()),
        }).await?.into_inner();
        Ok((resp.rule_version, resp.effective_at_ms))
    }

    /// list_versions はバージョン一覧。
    pub async fn list_versions(&mut self, rule_id: &str) -> Result<Vec<RuleVersionMeta>, Status> {
        let resp = self.raw.list_versions(ListVersionsRequest {
            rule_id: rule_id.to_string(),
            context: Some(self.client.tenant_context()),
        }).await?.into_inner();
        Ok(resp.versions)
    }

    /// get_rule は特定バージョンの取得。返り値は (jdm_document, meta)。
    pub async fn get_rule(
        &mut self,
        rule_id: &str,
        rule_version: &str,
    ) -> Result<(Vec<u8>, Option<RuleVersionMeta>), Status> {
        let resp = self.raw.get_rule(GetRuleRequest {
            rule_id: rule_id.to_string(),
            rule_version: rule_version.to_string(),
            context: Some(self.client.tenant_context()),
        }).await?.into_inner();
        Ok((resp.jdm_document, resp.meta))
    }
}
