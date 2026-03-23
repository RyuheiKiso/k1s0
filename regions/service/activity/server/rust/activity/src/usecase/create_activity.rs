// アクティビティ作成ユースケース。冪等性キーによる重複防止付き。
use crate::domain::entity::activity::{Activity, CreateActivity};
use crate::domain::repository::activity_repository::ActivityRepository;
use std::sync::Arc;

pub struct CreateActivityUseCase {
    repo: Arc<dyn ActivityRepository>,
}

impl CreateActivityUseCase {
    pub fn new(repo: Arc<dyn ActivityRepository>) -> Self {
        Self { repo }
    }

    // アクティビティ作成の全処理をトレースするためにスパンを自動生成する
    #[tracing::instrument(skip(self))]
    pub async fn execute(&self, input: &CreateActivity, actor_id: &str) -> anyhow::Result<Activity> {
        input.validate()?;

        // 冪等性チェック: 同じキーが既に存在する場合は既存を返す
        if let Some(ref key) = input.idempotency_key {
            if let Some(existing) = self.repo.find_by_idempotency_key(key).await? {
                return Ok(existing);
            }
        }

        self.repo.create(input, actor_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::activity::{ActivityStatus, ActivityType};
    use crate::domain::repository::activity_repository::MockActivityRepository;
    use chrono::Utc;
    use mockall::predicate::*;
    use uuid::Uuid;

    fn sample_activity() -> Activity {
        Activity {
            id: Uuid::new_v4(),
            task_id: Uuid::new_v4(),
            actor_id: "user1".to_string(),
            activity_type: ActivityType::Comment,
            content: Some("Done!".to_string()),
            duration_minutes: None,
            status: ActivityStatus::Active,
            idempotency_key: Some("key-001".to_string()),
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_create_returns_existing_on_duplicate_key() {
        let mut mock = MockActivityRepository::new();
        let existing = sample_activity();
        let existing_clone = existing.clone();

        mock.expect_find_by_idempotency_key()
            .with(eq("key-001"))
            .times(1)
            .returning(move |_| Ok(Some(existing_clone.clone())));

        let uc = CreateActivityUseCase::new(Arc::new(mock));
        let input = CreateActivity {
            task_id: Uuid::new_v4(),
            activity_type: ActivityType::Comment,
            content: Some("Hello".to_string()),
            duration_minutes: None,
            idempotency_key: Some("key-001".to_string()),
        };
        let result = uc.execute(&input, "user1").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().idempotency_key, Some("key-001".to_string()));
    }

    #[tokio::test]
    async fn test_create_new_activity() {
        let mut mock = MockActivityRepository::new();
        let activity = sample_activity();
        let activity_clone = activity.clone();

        mock.expect_find_by_idempotency_key()
            .times(0); // no key → skip check

        mock.expect_create()
            .times(1)
            .returning(move |_, _| Ok(activity_clone.clone()));

        let uc = CreateActivityUseCase::new(Arc::new(mock));
        let input = CreateActivity {
            task_id: Uuid::new_v4(),
            activity_type: ActivityType::Comment,
            content: Some("Hello".to_string()),
            duration_minutes: None,
            idempotency_key: None,
        };
        let result = uc.execute(&input, "user1").await;
        assert!(result.is_ok());
    }
}
