// アクティビティ提出ユースケース（Active → Submitted）。
use crate::domain::entity::activity::{Activity, ActivityStatus};
use crate::domain::repository::activity_repository::ActivityRepository;
use std::sync::Arc;
use uuid::Uuid;

pub struct SubmitActivityUseCase {
    repo: Arc<dyn ActivityRepository>,
}

impl SubmitActivityUseCase {
    pub fn new(repo: Arc<dyn ActivityRepository>) -> Self {
        Self { repo }
    }

    // アクティビティ提出の全処理をトレースするためにスパンを自動生成する
    #[tracing::instrument(skip(self))]
    pub async fn execute(&self, id: Uuid, actor_id: &str) -> anyhow::Result<Activity> {
        let activity = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Activity '{}' not found", id))?;
        activity.transition_to(ActivityStatus::Submitted)?;
        self.repo.update_status(id, "submitted", Some(actor_id.to_string())).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::activity::{ActivityStatus, ActivityType};
    use crate::domain::repository::activity_repository::MockActivityRepository;
    use chrono::Utc;
    use uuid::Uuid;

    // テスト用のサンプルアクティビティを生成するヘルパー関数（ステータス指定可能）
    fn sample_activity_with_status(status: ActivityStatus) -> Activity {
        Activity {
            id: Uuid::new_v4(),
            task_id: Uuid::new_v4(),
            actor_id: "user1".to_string(),
            activity_type: ActivityType::Comment,
            content: Some("sample comment".to_string()),
            duration_minutes: None,
            status,
            idempotency_key: None,
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    // 正常系: Active 状態のアクティビティが Submitted に遷移することを確認する
    #[tokio::test]
    async fn test_submit_success_active_to_submitted() {
        let mut mock = MockActivityRepository::new();
        let activity = sample_activity_with_status(ActivityStatus::Active);
        let activity_id = activity.id;
        let activity_clone = activity.clone();
        let mut submitted = activity.clone();
        submitted.status = ActivityStatus::Submitted;
        let submitted_clone = submitted.clone();

        mock.expect_find_by_id()
            .times(1)
            .returning(move |_| Ok(Some(activity_clone.clone())));
        mock.expect_update_status()
            .times(1)
            .returning(move |_, _, _| Ok(submitted_clone.clone()));

        let uc = SubmitActivityUseCase::new(Arc::new(mock));
        let result = uc.execute(activity_id, "user1").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, ActivityStatus::Submitted);
    }

    // 異常系: Submitted 状態から Submitted への不正な遷移がエラーになることを確認する
    #[tokio::test]
    async fn test_submit_invalid_transition_from_submitted() {
        let mut mock = MockActivityRepository::new();
        let activity = sample_activity_with_status(ActivityStatus::Submitted);
        let activity_id = activity.id;
        let activity_clone = activity.clone();

        mock.expect_find_by_id()
            .times(1)
            .returning(move |_| Ok(Some(activity_clone.clone())));
        // update_status は呼ばれないはず（遷移バリデーションでエラーになる）

        let uc = SubmitActivityUseCase::new(Arc::new(mock));
        let result = uc.execute(activity_id, "user1").await;
        assert!(result.is_err());
    }

    // 異常系: Approved 状態から Submitted への不正な遷移がエラーになることを確認する
    #[tokio::test]
    async fn test_submit_invalid_transition_from_approved() {
        let mut mock = MockActivityRepository::new();
        let activity = sample_activity_with_status(ActivityStatus::Approved);
        let activity_id = activity.id;
        let activity_clone = activity.clone();

        mock.expect_find_by_id()
            .times(1)
            .returning(move |_| Ok(Some(activity_clone.clone())));

        let uc = SubmitActivityUseCase::new(Arc::new(mock));
        let result = uc.execute(activity_id, "user1").await;
        assert!(result.is_err());
    }

    // 異常系: 対象アクティビティが存在しない場合（NotFound）にエラーが返ることを確認する
    #[tokio::test]
    async fn test_submit_not_found() {
        let mut mock = MockActivityRepository::new();

        mock.expect_find_by_id()
            .times(1)
            .returning(|_| Ok(None));

        let uc = SubmitActivityUseCase::new(Arc::new(mock));
        let result = uc.execute(Uuid::new_v4(), "user1").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}
