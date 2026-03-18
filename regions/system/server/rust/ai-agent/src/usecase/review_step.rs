// ステップレビューユースケース
// 実行中のステップを承認または拒否して、実行を再開する

use std::sync::Arc;

use chrono::Utc;
use tracing::info;

use crate::domain::entity::ExecutionStatus;
use crate::domain::repository::ExecutionRepository;

/// ReviewStepUseCase は実行ステップのレビュー（承認/拒否）を担当する
pub struct ReviewStepUseCase {
    /// 実行リポジトリ
    execution_repo: Arc<dyn ExecutionRepository>,
}

/// ステップレビューリクエスト
#[allow(dead_code)]
pub struct ReviewStepRequest {
    pub execution_id: String,
    pub step_index: i32,
    pub approved: bool,
    pub feedback: String,
}

/// ステップレビューレスポンス
pub struct ReviewStepResponse {
    pub execution_id: String,
    pub resumed: bool,
}

impl ReviewStepUseCase {
    /// 新しいReviewStepUseCaseを生成する
    pub fn new(execution_repo: Arc<dyn ExecutionRepository>) -> Self {
        Self { execution_repo }
    }

    /// 指定されたステップをレビューし、承認された場合は実行を再開する
    pub async fn execute(&self, req: ReviewStepRequest) -> anyhow::Result<ReviewStepResponse> {
        // 実行を取得する
        let mut execution = self
            .execution_repo
            .find_by_id(&req.execution_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Execution not found: {}", req.execution_id))?;

        info!(
            execution_id = %req.execution_id,
            step_index = req.step_index,
            approved = req.approved,
            "reviewing execution step"
        );

        // ステップのステータスを更新する
        if let Some(step) = execution
            .steps
            .iter_mut()
            .find(|s| s.index == req.step_index)
        {
            step.status = if req.approved {
                "approved".to_string()
            } else {
                "rejected".to_string()
            };
        }

        // 承認された場合は実行を再開可能にする
        let resumed = if req.approved {
            execution.status = ExecutionStatus::Running;
            true
        } else {
            execution.status = ExecutionStatus::Cancelled;
            false
        };

        execution.updated_at = Utc::now();
        self.execution_repo.save(&execution).await?;

        Ok(ReviewStepResponse {
            execution_id: req.execution_id,
            resumed,
        })
    }
}
