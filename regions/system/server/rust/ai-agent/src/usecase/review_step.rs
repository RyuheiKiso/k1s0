// ステップレビューユースケース
// 実行中のステップを承認または拒否して、実行を再開する

use std::sync::Arc;

use chrono::Utc;
use tracing::info;

use crate::domain::entity::ExecutionStatus;
use crate::domain::repository::ExecutionRepository;

/// `ReviewStepUseCase` は実行ステップのレビュー（承認/拒否）を担当する
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
    /// `新しいReviewStepUseCaseを生成する`
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::{Execution, ExecutionStatus, ExecutionStep};
    use crate::domain::repository::execution_repository::MockExecutionRepository;
    use chrono::Utc;
    use uuid::Uuid;

    // テスト用の実行エンティティ（レビュー待ちステップ付き）を生成するヘルパー
    fn sample_execution_with_step(execution_id: &str, step_index: i32) -> Execution {
        let now = Utc::now();
        Execution {
            id: execution_id.to_string(),
            agent_id: "agent-001".to_string(),
            session_id: "session-001".to_string(),
            tenant_id: "tenant-001".to_string(),
            input: "テスト入力".to_string(),
            output: None,
            status: ExecutionStatus::Running,
            steps: vec![ExecutionStep {
                index: step_index,
                step_type: "tool_call".to_string(),
                input: "ツール入力".to_string(),
                output: "待機中".to_string(),
                tool_name: Some("search".to_string()),
                status: "pending_review".to_string(),
            }],
            created_at: now,
            updated_at: now,
        }
    }

    // テスト用のレビューリクエストを生成するヘルパー
    fn review_request(execution_id: &str, approved: bool) -> ReviewStepRequest {
        ReviewStepRequest {
            execution_id: execution_id.to_string(),
            step_index: 0,
            approved,
            feedback: "確認済み".to_string(),
        }
    }

    // 承認ケース: approved=trueのとき実行がRunningに戻り、resumed=trueが返る
    #[tokio::test]
    async fn test_review_step_approved() {
        let execution_id = Uuid::new_v4().to_string();
        let execution = sample_execution_with_step(&execution_id, 0);

        let mut mock_repo = MockExecutionRepository::new();
        let exec_clone = execution.clone();
        mock_repo
            .expect_find_by_id()
            .times(1)
            .returning(move |_| Ok(Some(exec_clone.clone())));
        mock_repo.expect_save().times(1).returning(|_| Ok(()));

        let uc = ReviewStepUseCase::new(Arc::new(mock_repo));
        let result = uc.execute(review_request(&execution_id, true)).await;

        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.execution_id, execution_id);
        // 承認された場合はresume=trueが返る
        assert!(resp.resumed);
    }

    // 拒否ケース: approved=falseのとき実行がCancelledになり、resumed=falseが返る
    #[tokio::test]
    async fn test_review_step_rejected() {
        let execution_id = Uuid::new_v4().to_string();
        let execution = sample_execution_with_step(&execution_id, 0);

        let mut mock_repo = MockExecutionRepository::new();
        let exec_clone = execution.clone();
        mock_repo
            .expect_find_by_id()
            .times(1)
            .returning(move |_| Ok(Some(exec_clone.clone())));
        // 拒否時も実行状態の更新のためsaveが1回呼ばれる
        mock_repo.expect_save().times(1).returning(|_| Ok(()));

        let uc = ReviewStepUseCase::new(Arc::new(mock_repo));
        let result = uc.execute(review_request(&execution_id, false)).await;

        assert!(result.is_ok());
        let resp = result.unwrap();
        // 拒否された場合はresume=falseが返る
        assert!(!resp.resumed);
    }

    // 異常系: 存在しない実行IDを指定した場合にエラーが返る
    #[tokio::test]
    async fn test_review_step_not_found() {
        let mut mock_repo = MockExecutionRepository::new();
        mock_repo
            .expect_find_by_id()
            .times(1)
            .returning(|_| Ok(None));

        let uc = ReviewStepUseCase::new(Arc::new(mock_repo));
        let result = uc
            .execute(ReviewStepRequest {
                execution_id: "no-such-execution".to_string(),
                step_index: 0,
                approved: true,
                feedback: "".to_string(),
            })
            .await;

        assert!(result.is_err());
        // err().unwrap() を使うことでDebugトレイト不要でエラーを取得する
        assert!(result
            .err()
            .unwrap()
            .to_string()
            .contains("Execution not found"));
    }

    // ステップ状態更新: 承認/拒否後にステップのstatusが正しく変更される
    #[tokio::test]
    async fn test_review_step_updates_step_status() {
        let execution_id = Uuid::new_v4().to_string();
        let execution = sample_execution_with_step(&execution_id, 0);

        let mut mock_repo = MockExecutionRepository::new();
        let exec_clone = execution.clone();
        mock_repo
            .expect_find_by_id()
            .times(1)
            .returning(move |_| Ok(Some(exec_clone.clone())));

        // saveの引数に渡される実行エンティティのステップ状態を検証する
        mock_repo.expect_save().times(1).returning(move |saved| {
            let step = saved.steps.iter().find(|s| s.index == 0).unwrap();
            assert_eq!(step.status, "approved");
            Ok(())
        });

        let uc = ReviewStepUseCase::new(Arc::new(mock_repo));
        let _ = uc.execute(review_request(&execution_id, true)).await;
    }
}
