// 本ファイルは k1s0-sdk の Feature 動詞統一 facade（評価部のみ）。
use crate::client::Client;
use crate::proto::k1s0::tier1::feature::v1::{
    feature_service_client::FeatureServiceClient, EvaluateRequest, FlagMetadata,
};
use std::collections::HashMap;
use tonic::{transport::Channel, Status};

/// FeatureFacade は FeatureService の動詞統一 facade。
pub struct FeatureFacade {
    client: Client,
    raw: FeatureServiceClient<Channel>,
}

impl FeatureFacade {
    pub(crate) fn new(client: Client) -> Self {
        let raw = FeatureServiceClient::new(client.channel());
        Self { client, raw }
    }

    fn make_req(&self, flag_key: &str, evaluation_context: HashMap<String, String>) -> EvaluateRequest {
        EvaluateRequest {
            flag_key: flag_key.to_string(),
            evaluation_context,
            context: Some(self.client.tenant_context()),
        }
    }

    /// evaluate_boolean は boolean Flag 評価。返り値は (value, metadata)。
    pub async fn evaluate_boolean(
        &mut self,
        flag_key: &str,
        eval_ctx: HashMap<String, String>,
    ) -> Result<(bool, Option<FlagMetadata>), Status> {
        let resp = self.raw.evaluate_boolean(self.make_req(flag_key, eval_ctx)).await?.into_inner();
        Ok((resp.value, resp.metadata))
    }

    /// evaluate_string は string Flag 評価。
    pub async fn evaluate_string(
        &mut self,
        flag_key: &str,
        eval_ctx: HashMap<String, String>,
    ) -> Result<(String, Option<FlagMetadata>), Status> {
        let resp = self.raw.evaluate_string(self.make_req(flag_key, eval_ctx)).await?.into_inner();
        Ok((resp.value, resp.metadata))
    }

    /// evaluate_number は number Flag 評価。
    pub async fn evaluate_number(
        &mut self,
        flag_key: &str,
        eval_ctx: HashMap<String, String>,
    ) -> Result<(f64, Option<FlagMetadata>), Status> {
        let resp = self.raw.evaluate_number(self.make_req(flag_key, eval_ctx)).await?.into_inner();
        Ok((resp.value, resp.metadata))
    }

    /// evaluate_object は object Flag 評価（JSON シリアライズ済 bytes）。
    pub async fn evaluate_object(
        &mut self,
        flag_key: &str,
        eval_ctx: HashMap<String, String>,
    ) -> Result<(Vec<u8>, Option<FlagMetadata>), Status> {
        let resp = self.raw.evaluate_object(self.make_req(flag_key, eval_ctx)).await?.into_inner();
        Ok((resp.value_json, resp.metadata))
    }
}
