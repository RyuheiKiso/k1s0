// gRPCサービスのビジネスロジック実装
// ユースケースを呼び出し、gRPC用のデータ型で結果を返す

use std::sync::Arc;

use crate::domain::entity::{Execution, ExecutionStep as DomainStep};
use crate::usecase::{ExecuteAgentUseCase, ReviewStepUseCase};

/// gRPCエラー型
#[derive(Debug)]
pub enum GrpcError {
    NotFound(String),
    InvalidArgument(String),
    Internal(String),
}

/// 実行レスポンスデータ
pub struct ExecutionData {
    pub execution_id: String,
    pub status: String,
    pub output: String,
    pub steps: Vec<StepData>,
}

/// 実行ステップデータ
pub struct StepData {
    pub index: i32,
    pub step_type: String,
    pub input: String,
    pub output: String,
    pub tool_name: String,
    pub status: String,
}

/// レビューレスポンスデータ
pub struct ReviewData {
    pub execution_id: String,
    pub resumed: bool,
}

/// `AiAgentGrpcService` `はgRPCサービスのビジネスロジックを実装する`
pub struct AiAgentGrpcService {
    /// エージェント実行ユースケース
    execute_agent_uc: Arc<ExecuteAgentUseCase>,
    /// ステップレビューユースケース
    review_step_uc: Arc<ReviewStepUseCase>,
}

impl AiAgentGrpcService {
    /// `新しいAiAgentGrpcServiceを生成する`
    #[must_use] 
    pub fn new(
        execute_agent_uc: Arc<ExecuteAgentUseCase>,
        review_step_uc: Arc<ReviewStepUseCase>,
    ) -> Self {
        Self {
            execute_agent_uc,
            review_step_uc,
        }
    }

    /// エージェントを実行する
    pub async fn execute(
        &self,
        agent_id: String,
        input: String,
        session_id: String,
        tenant_id: String,
    ) -> Result<ExecutionData, GrpcError> {
        let req = crate::usecase::execute_agent::ExecuteAgentRequest {
            agent_id,
            input,
            session_id,
            tenant_id,
        };

        let resp = self
            .execute_agent_uc
            .execute(req)
            .await
            .map_err(|e| GrpcError::Internal(e.to_string()))?;

        Ok(execution_to_data(resp.execution))
    }

    /// ステップをレビューする
    pub async fn review_step(
        &self,
        execution_id: String,
        step_index: i32,
        approved: bool,
        feedback: String,
    ) -> Result<ReviewData, GrpcError> {
        let req = crate::usecase::review_step::ReviewStepRequest {
            execution_id,
            step_index,
            approved,
            feedback,
        };

        let resp = self
            .review_step_uc
            .execute(req)
            .await
            .map_err(|e| GrpcError::Internal(e.to_string()))?;

        Ok(ReviewData {
            execution_id: resp.execution_id,
            resumed: resp.resumed,
        })
    }
}

/// Executionエンティティをgrpc用データ型に変換する
fn execution_to_data(exec: Execution) -> ExecutionData {
    ExecutionData {
        execution_id: exec.id,
        status: exec.status.to_string(),
        output: exec.output.unwrap_or_default(),
        steps: exec.steps.into_iter().map(step_to_data).collect(),
    }
}

/// `ExecutionStepをgRPC用データ型に変換する`
fn step_to_data(step: DomainStep) -> StepData {
    StepData {
        index: step.index,
        step_type: step.step_type,
        input: step.input,
        output: step.output,
        tool_name: step.tool_name.unwrap_or_default(),
        status: step.status,
    }
}
