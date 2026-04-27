// 本ファイルは k1s0-sdk の Workflow 動詞統一 facade。
use crate::client::Client;
use crate::proto::k1s0::tier1::workflow::v1::{
    CancelRequest, GetStatusRequest, GetStatusResponse, QueryRequest, SignalRequest, StartRequest,
    TerminateRequest, workflow_service_client::WorkflowServiceClient,
};
use tonic::{Status, transport::Channel};

/// WorkflowFacade は WorkflowService の動詞統一 facade。
pub struct WorkflowFacade {
    client: Client,
    raw: WorkflowServiceClient<Channel>,
}

impl WorkflowFacade {
    pub(crate) fn new(client: Client) -> Self {
        let raw = WorkflowServiceClient::new(client.channel());
        Self { client, raw }
    }

    /// start はワークフロー開始。返り値は (workflow_id, run_id)。
    pub async fn start(
        &mut self,
        workflow_type: &str,
        workflow_id: &str,
        input: Vec<u8>,
        idempotent: bool,
    ) -> Result<(String, String), Status> {
        let resp = self
            .raw
            .start(StartRequest {
                workflow_type: workflow_type.to_string(),
                workflow_id: workflow_id.to_string(),
                input,
                idempotent,
                context: Some(self.client.tenant_context()),
            })
            .await?
            .into_inner();
        Ok((resp.workflow_id, resp.run_id))
    }

    /// signal はシグナル送信。
    pub async fn signal(
        &mut self,
        workflow_id: &str,
        signal_name: &str,
        payload: Vec<u8>,
    ) -> Result<(), Status> {
        self.raw
            .signal(SignalRequest {
                workflow_id: workflow_id.to_string(),
                signal_name: signal_name.to_string(),
                payload,
                context: Some(self.client.tenant_context()),
            })
            .await?;
        Ok(())
    }

    /// query はワークフロー状態のクエリ（副作用なし）。
    pub async fn query(
        &mut self,
        workflow_id: &str,
        query_name: &str,
        payload: Vec<u8>,
    ) -> Result<Vec<u8>, Status> {
        let resp = self
            .raw
            .query(QueryRequest {
                workflow_id: workflow_id.to_string(),
                query_name: query_name.to_string(),
                payload,
                context: Some(self.client.tenant_context()),
            })
            .await?
            .into_inner();
        Ok(resp.result)
    }

    /// cancel は正常終了の依頼。
    pub async fn cancel(&mut self, workflow_id: &str, reason: &str) -> Result<(), Status> {
        self.raw
            .cancel(CancelRequest {
                workflow_id: workflow_id.to_string(),
                reason: reason.to_string(),
                context: Some(self.client.tenant_context()),
            })
            .await?;
        Ok(())
    }

    /// terminate は強制終了。
    pub async fn terminate(&mut self, workflow_id: &str, reason: &str) -> Result<(), Status> {
        self.raw
            .terminate(TerminateRequest {
                workflow_id: workflow_id.to_string(),
                reason: reason.to_string(),
                context: Some(self.client.tenant_context()),
            })
            .await?;
        Ok(())
    }

    /// get_status は現在状態を取得する（生 GetStatusResponse を返す）。
    pub async fn get_status(&mut self, workflow_id: &str) -> Result<GetStatusResponse, Status> {
        Ok(self
            .raw
            .get_status(GetStatusRequest {
                workflow_id: workflow_id.to_string(),
                context: Some(self.client.tenant_context()),
            })
            .await?
            .into_inner())
    }
}
