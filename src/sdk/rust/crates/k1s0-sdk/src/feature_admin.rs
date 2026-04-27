// 本ファイルは k1s0-sdk の FeatureAdmin 動詞統一 facade。
use crate::client::Client;
use crate::proto::k1s0::tier1::feature::v1::{
    FlagDefinition, FlagKind, FlagState, GetFlagRequest, ListFlagsRequest, RegisterFlagRequest,
    feature_admin_service_client::FeatureAdminServiceClient,
};
use tonic::{Status, transport::Channel};

pub struct FeatureAdminFacade {
    client: Client,
    raw: FeatureAdminServiceClient<Channel>,
}

impl FeatureAdminFacade {
    pub(crate) fn new(client: Client) -> Self {
        let raw = FeatureAdminServiceClient::new(client.channel());
        Self { client, raw }
    }

    /// register_flag は Flag 定義の登録（permission 種別は approval_id 必須）。
    pub async fn register_flag(
        &mut self,
        flag: FlagDefinition,
        change_reason: &str,
        approval_id: &str,
    ) -> Result<i64, Status> {
        let resp = self
            .raw
            .register_flag(RegisterFlagRequest {
                flag: Some(flag),
                change_reason: change_reason.to_string(),
                approval_id: approval_id.to_string(),
                context: Some(self.client.tenant_context()),
            })
            .await?
            .into_inner();
        Ok(resp.version)
    }

    /// get_flag は Flag 定義の取得。version=None で最新。
    pub async fn get_flag(
        &mut self,
        flag_key: &str,
        version: Option<i64>,
    ) -> Result<(Option<FlagDefinition>, i64), Status> {
        let resp = self
            .raw
            .get_flag(GetFlagRequest {
                flag_key: flag_key.to_string(),
                version,
                context: Some(self.client.tenant_context()),
            })
            .await?
            .into_inner();
        Ok((resp.flag, resp.version))
    }

    /// list_flags は Flag 定義の一覧。kind / state は None で全件。
    pub async fn list_flags(
        &mut self,
        kind: Option<FlagKind>,
        state: Option<FlagState>,
    ) -> Result<Vec<FlagDefinition>, Status> {
        let resp = self
            .raw
            .list_flags(ListFlagsRequest {
                kind: kind.map(|k| k as i32),
                state: state.map(|s| s as i32),
                context: Some(self.client.tenant_context()),
            })
            .await?
            .into_inner();
        Ok(resp.flags)
    }
}
