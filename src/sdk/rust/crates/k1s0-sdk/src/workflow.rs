// 本ファイルは k1s0-sdk の Workflow 動詞統一 facade。
use crate::client::Client;
use crate::proto::k1s0::tier1::workflow::v1::{
    CancelRequest, GetStatusRequest, GetStatusResponse, QueryRequest, SignalRequest, StartRequest,
    TerminateRequest, WorkflowBackend, workflow_service_client::WorkflowServiceClient,
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

    /// start はワークフロー開始。backend hint は BACKEND_AUTO（tier1 が振り分け）。
    /// 返り値は (workflow_id, run_id)。
    /// 短期 / 長期で意図的に振り分けたい時は run_short / run_long を使う。
    pub async fn start(
        &mut self,
        workflow_type: &str,
        workflow_id: &str,
        input: Vec<u8>,
        idempotent: bool,
    ) -> Result<(String, String), Status> {
        self.start_with_backend(
            workflow_type,
            workflow_id,
            input,
            idempotent,
            WorkflowBackend::BackendAuto,
        )
        .await
    }

    /// run_short は短期ワークフロー（≤7 日、BACKEND_DAPR）として開始する（FR-T1-WORKFLOW-001）。
    /// 短期ワークフローは Dapr Workflow building block で実行され、Pod 再起動でも履歴が保持される。
    pub async fn run_short(
        &mut self,
        workflow_type: &str,
        workflow_id: &str,
        input: Vec<u8>,
        idempotent: bool,
    ) -> Result<(String, String), Status> {
        self.start_with_backend(
            workflow_type,
            workflow_id,
            input,
            idempotent,
            WorkflowBackend::BackendDapr,
        )
        .await
    }

    /// run_long は長期ワークフロー（上限なし、BACKEND_TEMPORAL）として開始する（FR-T1-WORKFLOW-002）。
    /// Continue-as-New / cron / 高度な signal 機能が必要な場合に使う。
    pub async fn run_long(
        &mut self,
        workflow_type: &str,
        workflow_id: &str,
        input: Vec<u8>,
        idempotent: bool,
    ) -> Result<(String, String), Status> {
        self.start_with_backend(
            workflow_type,
            workflow_id,
            input,
            idempotent,
            WorkflowBackend::BackendTemporal,
        )
        .await
    }

    /// start_with_backend は start / run_short / run_long の共通実装。
    async fn start_with_backend(
        &mut self,
        workflow_type: &str,
        workflow_id: &str,
        input: Vec<u8>,
        idempotent: bool,
        backend: WorkflowBackend,
    ) -> Result<(String, String), Status> {
        let resp = self
            .raw
            .start(StartRequest {
                workflow_type: workflow_type.to_string(),
                workflow_id: workflow_id.to_string(),
                input,
                idempotent,
                // 共通規約 §「冪等性と再試行」: idempotent=true の場合は workflow_id を
                // dedup key として転用する（同 workflow_id 再実行は新 run を作らない）。
                // start_with_backend は内部 helper のため、外部公開 API（run_short / run_long）
                // が `idempotent=true` を渡せばこの値が使われる。
                idempotency_key: if idempotent { workflow_id.to_string() } else { String::new() },
                context: Some(self.client.tenant_context()),
                backend: backend as i32,
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
