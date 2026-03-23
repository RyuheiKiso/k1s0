// アクティビティ却下ユースケース（Submitted → Rejected）。
use crate::domain::entity::activity::{Activity, ActivityStatus};
use crate::domain::repository::activity_repository::ActivityRepository;
use std::sync::Arc;
use uuid::Uuid;

pub struct RejectActivityUseCase {
    repo: Arc<dyn ActivityRepository>,
}

impl RejectActivityUseCase {
    pub fn new(repo: Arc<dyn ActivityRepository>) -> Self {
        Self { repo }
    }

    // アクティビティ却下の全処理をトレースするためにスパンを自動生成する
    #[tracing::instrument(skip(self))]
    pub async fn execute(&self, tenant_id: &str, id: Uuid, rejector_id: &str, _reason: &str) -> anyhow::Result<Activity> {
        let activity = self
            .repo
            .find_by_id(tenant_id, id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Activity '{}' not found", id))?;
        activity.transition_to(ActivityStatus::Rejected)?;
        self.repo.update_status(tenant_id, id, "rejected", Some(rejector_id.to_string())).await
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

    // 正常系: Submitted 状態のアクティビティが Rejected に遷移することを確認する
    #[tokio::test]
    async fn test_reject_success_submitted_to_rejected() {
        let mut mock = MockActivityRepository::new();
        let activity = sample_activity_with_status(ActivityStatus::Submitted);
        let activity_id = activity.id;
        let activity_clone = activity.clone();
        let mut rejected = activity.clone();
        rejected.status = ActivityStatus::Rejected;
        let rejected_clone = rejected.clone();

        mock.expect_find_by_id()
            .times(1)
            .returning(move |_, _| Ok(Some(activity_clone.clone())));
        mock.expect_update_status()
            .times(1)
            .returning(move |_, _, _, _| Ok(rejected_clone.clone()));

        let uc = RejectActivityUseCase::new(Arc::new(mock));
        let result = uc.execute("tenant1", activity_id, "rejector1", "policy violation").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, ActivityStatus::Rejected);
    }

    // 異常系: Active 状態から Rejected への不正な遷移がエラーになることを確認する
    #[tokio::test]
    async fn test_reject_invalid_transition_from_active() {
        let mut mock = MockActivityRepository::new();
        let activity = sample_activity_with_status(ActivityStatus::Active);
        let activity_id = activity.id;
        let activity_clone = activity.clone();

        mock.expect_find_by_id()
            .times(1)
            .returning(move |_, _| Ok(Some(activity_clone.clone())));
        // update_status は呼ばれないはず（遷移バリデーションでエラーになる）

        let uc = RejectActivityUseCase::new(Arc::new(mock));
        let result = uc.execute("tenant1", activity_id, "rejector1", "reason").await;
        assert!(result.is_err());
    }

    // 異常系: Approved 状態から Rejected への不正な遷移がエラーになることを確認する
    #[tokio::test]
    async fn test_reject_invalid_transition_from_approved() {
        let mut mock = MockActivityRepository::new();
        let activity = sample_activity_with_status(ActivityStatus::Approved);
        let activity_id = activity.id;
        let activity_clone = activity.clone();

        mock.expect_find_by_id()
            .times(1)
            .returning(move |_, _| Ok(Some(activity_clone.clone())));

        let uc = RejectActivityUseCase::new(Arc::new(mock));
        let result = uc.execute("tenant1", activity_id, "rejector1", "reason").await;
        assert!(result.is_err());
    }

    // 異常系: 対象アクティビティが存在しない場合（NotFound）にエラーが返ることを確認する
    #[tokio::test]
    async fn test_reject_not_found() {
        let mut mock = MockActivityRepository::new();

        mock.expect_find_by_id()
            .times(1)
            .returning(|_, _| Ok(None));

        let uc = RejectActivityUseCase::new(Arc::new(mock));
        let result = uc.execute("tenant1", Uuid::new_v4(), "rejector1", "reason").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}
