// gRPC Tonicサービス実装
// proto生成コードのトレイトを実装し、ビジネスロジックに委譲する

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::ai_agent::v1::{
    ai_agent_service_server::AiAgentService, CancelExecutionRequest as ProtoCancelExecutionRequest,
    CancelExecutionResponse as ProtoCancelExecutionResponse, ExecuteRequest as ProtoExecuteRequest,
    ExecuteResponse as ProtoExecuteResponse, ExecuteStreamRequest as ProtoExecuteStreamRequest,
    ExecuteStreamResponse as ProtoExecuteStreamResponse, ExecutionStep as ProtoExecutionStep,
    ReviewStepRequest as ProtoReviewStepRequest, ReviewStepResponse as ProtoReviewStepResponse,
};

use super::agent_grpc::{AiAgentGrpcService, GrpcError};

/// GrpcErrorからtonic::Statusへの変換
impl From<GrpcError> for Status {
    fn from(e: GrpcError) -> Self {
        match e {
            GrpcError::NotFound(msg) => Status::not_found(msg),
            GrpcError::InvalidArgument(msg) => Status::invalid_argument(msg),
            GrpcError::Internal(msg) => Status::internal(msg),
        }
    }
}

/// AiAgentServiceTonic はtonic生成のトレイトを実装するラッパー
pub struct AiAgentServiceTonic {
    inner: Arc<AiAgentGrpcService>,
}

impl AiAgentServiceTonic {
    /// 新しいAiAgentServiceTonicを生成する
    pub fn new(inner: Arc<AiAgentGrpcService>) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl AiAgentService for AiAgentServiceTonic {
    /// エージェントを実行し、結果を返す
    async fn execute(
        &self,
        request: Request<ProtoExecuteRequest>,
    ) -> Result<Response<ProtoExecuteResponse>, Status> {
        let inner = request.into_inner();
        let resp = self
            .inner
            .execute(
                inner.agent_id,
                inner.input,
                inner.session_id,
                inner.tenant_id,
            )
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoExecuteResponse {
            execution_id: resp.execution_id,
            status: resp.status,
            output: resp.output,
            steps: resp
                .steps
                .into_iter()
                .map(|s| ProtoExecutionStep {
                    index: s.index,
                    step_type: s.step_type,
                    input: s.input,
                    output: s.output,
                    tool_name: s.tool_name,
                    status: s.status,
                })
                .collect(),
        }))
    }

    /// ストリーミング形式でエージェントを実行する（未実装、将来拡張用）
    type ExecuteStreamStream =
        tokio_stream::wrappers::ReceiverStream<Result<ProtoExecuteStreamResponse, Status>>;

    async fn execute_stream(
        &self,
        request: Request<ProtoExecuteStreamRequest>,
    ) -> Result<Response<Self::ExecuteStreamStream>, Status> {
        let inner = request.into_inner();

        // ストリーミング実行: 通常の実行結果をストリーミングイベントとして返す
        let (tx, rx) = tokio::sync::mpsc::channel(32);

        let grpc_svc = self.inner.clone();
        tokio::spawn(async move {
            match grpc_svc
                .execute(
                    inner.agent_id,
                    inner.input,
                    inner.session_id,
                    inner.tenant_id,
                )
                .await
            {
                Ok(resp) => {
                    // 各ステップをイベントとして送信する
                    for (i, step) in resp.steps.iter().enumerate() {
                        let event = ProtoExecuteStreamResponse {
                            execution_id: resp.execution_id.clone(),
                            event_type: step.step_type.clone(),
                            data: serde_json::json!({
                                "input": step.input,
                                "output": step.output,
                                "tool_name": step.tool_name,
                            })
                            .to_string(),
                            step_index: i as i32,
                        };
                        if tx.send(Ok(event)).await.is_err() {
                            break;
                        }
                    }
                    // 最終出力イベントを送信する
                    let _ = tx
                        .send(Ok(ProtoExecuteStreamResponse {
                            execution_id: resp.execution_id,
                            event_type: "output".to_string(),
                            data: resp.output,
                            step_index: -1,
                        }))
                        .await;
                }
                Err(e) => {
                    let _ = tx.send(Err(Status::from(e))).await;
                }
            }
        });

        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(
            rx,
        )))
    }

    /// 実行中のエージェントをキャンセルする
    async fn cancel_execution(
        &self,
        _request: Request<ProtoCancelExecutionRequest>,
    ) -> Result<Response<ProtoCancelExecutionResponse>, Status> {
        // キャンセル機能は将来実装予定
        Ok(Response::new(ProtoCancelExecutionResponse {
            success: true,
        }))
    }

    /// ステップをレビュー（承認/却下）する
    async fn review_step(
        &self,
        request: Request<ProtoReviewStepRequest>,
    ) -> Result<Response<ProtoReviewStepResponse>, Status> {
        let inner = request.into_inner();
        let resp = self
            .inner
            .review_step(
                inner.execution_id,
                inner.step_index,
                inner.approved,
                inner.feedback,
            )
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoReviewStepResponse {
            execution_id: resp.execution_id,
            resumed: resp.resumed,
        }))
    }
}
