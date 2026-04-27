// 本ファイルは k1s0-sdk の Decision 動詞統一 facade（評価部のみ）。
use crate::client::Client;
use crate::proto::k1s0::tier1::decision::v1::{
    decision_service_client::DecisionServiceClient, BatchEvaluateRequest, EvaluateRequest,
};
use tonic::{transport::Channel, Status};

/// DecisionFacade は DecisionService（評価）の動詞統一 facade。
pub struct DecisionFacade {
    client: Client,
    raw: DecisionServiceClient<Channel>,
}

impl DecisionFacade {
    pub(crate) fn new(client: Client) -> Self {
        let raw = DecisionServiceClient::new(client.channel());
        Self { client, raw }
    }

    /// evaluate はルール評価（同期）。返り値は (output_json, trace_json, elapsed_us)。
    pub async fn evaluate(
        &mut self,
        rule_id: &str,
        rule_version: &str,
        input_json: Vec<u8>,
        include_trace: bool,
    ) -> Result<(Vec<u8>, Vec<u8>, i64), Status> {
        let resp = self.raw.evaluate(EvaluateRequest {
            rule_id: rule_id.to_string(),
            rule_version: rule_version.to_string(),
            input_json,
            include_trace,
            context: Some(self.client.tenant_context()),
        }).await?.into_inner();
        Ok((resp.output_json, resp.trace_json, resp.elapsed_us))
    }

    /// batch_evaluate はバッチ評価。
    pub async fn batch_evaluate(
        &mut self,
        rule_id: &str,
        rule_version: &str,
        inputs: Vec<Vec<u8>>,
    ) -> Result<Vec<Vec<u8>>, Status> {
        let resp = self.raw.batch_evaluate(BatchEvaluateRequest {
            rule_id: rule_id.to_string(),
            rule_version: rule_version.to_string(),
            inputs_json: inputs,
            context: Some(self.client.tenant_context()),
        }).await?.into_inner();
        Ok(resp.outputs_json)
    }
}
