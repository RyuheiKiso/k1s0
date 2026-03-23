// アクティビティ一覧ユースケース。
use crate::domain::entity::activity::{Activity, ActivityFilter};
use crate::domain::repository::activity_repository::ActivityRepository;
use std::sync::Arc;

pub struct ListActivitiesUseCase {
    repo: Arc<dyn ActivityRepository>,
}

impl ListActivitiesUseCase {
    pub fn new(repo: Arc<dyn ActivityRepository>) -> Self {
        Self { repo }
    }

    // アクティビティ一覧取得の全処理をトレースするためにスパンを自動生成する
    #[tracing::instrument(skip(self))]
    pub async fn execute(&self, tenant_id: &str, filter: &ActivityFilter) -> anyhow::Result<(Vec<Activity>, i64)> {
        let items = self.repo.find_all(tenant_id, filter).await?;
        let count = self.repo.count(tenant_id, filter).await?;
        Ok((items, count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::activity::{ActivityStatus, ActivityType};
    use crate::domain::repository::activity_repository::MockActivityRepository;
    use chrono::Utc;
    use uuid::Uuid;

    // テスト用のサンプルアクティビティを生成するヘルパー関数
    fn sample_activity(task_id: Uuid) -> Activity {
        Activity {
            id: Uuid::new_v4(),
            task_id,
            actor_id: "user1".to_string(),
            activity_type: ActivityType::Comment,
            content: Some("sample comment".to_string()),
            duration_minutes: None,
            status: ActivityStatus::Active,
            idempotency_key: None,
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    // 正常系: タスク ID を指定してアクティビティ一覧が取得できることを確認する
    #[tokio::test]
    async fn test_list_activities_success() {
        let mut mock = MockActivityRepository::new();
        let task_id = Uuid::new_v4();
        let a1 = sample_activity(task_id);
        let a2 = sample_activity(task_id);
        let items = vec![a1, a2];
        let items_clone = items.clone();

        mock.expect_find_all()
            .times(1)
            .returning(move |_, _| Ok(items_clone.clone()));
        mock.expect_count()
            .times(1)
            .returning(|_, _| Ok(2));

        let uc = ListActivitiesUseCase::new(Arc::new(mock));
        let filter = ActivityFilter {
            task_id: Some(task_id),
            ..Default::default()
        };
        let result = uc.execute("tenant1", &filter).await;
        assert!(result.is_ok());
        let (activities, count) = result.unwrap();
        assert_eq!(activities.len(), 2);
        assert_eq!(count, 2);
    }

    // 正常系: 対象タスクにアクティビティが存在しない場合に空リストが返ることを確認する
    #[tokio::test]
    async fn test_list_activities_empty() {
        let mut mock = MockActivityRepository::new();

        mock.expect_find_all()
            .times(1)
            .returning(|_, _| Ok(vec![]));
        mock.expect_count()
            .times(1)
            .returning(|_, _| Ok(0));

        let uc = ListActivitiesUseCase::new(Arc::new(mock));
        let filter = ActivityFilter {
            task_id: Some(Uuid::new_v4()),
            ..Default::default()
        };
        let result = uc.execute("tenant1", &filter).await;
        assert!(result.is_ok());
        let (activities, count) = result.unwrap();
        assert!(activities.is_empty());
        assert_eq!(count, 0);
    }

    // 異常系: find_all でリポジトリがエラーを返した場合にエラーが伝播することを確認する
    #[tokio::test]
    async fn test_list_activities_find_all_error() {
        let mut mock = MockActivityRepository::new();

        mock.expect_find_all()
            .times(1)
            .returning(|_, _| Err(anyhow::anyhow!("database connection failed")));

        let uc = ListActivitiesUseCase::new(Arc::new(mock));
        let filter = ActivityFilter::default();
        let result = uc.execute("tenant1", &filter).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("database connection failed"));
    }

    // 異常系: count でリポジトリがエラーを返した場合にエラーが伝播することを確認する
    #[tokio::test]
    async fn test_list_activities_count_error() {
        let mut mock = MockActivityRepository::new();
        let task_id = Uuid::new_v4();
        let items = vec![sample_activity(task_id)];

        mock.expect_find_all()
            .times(1)
            .returning(move |_, _| Ok(items.clone()));
        mock.expect_count()
            .times(1)
            .returning(|_, _| Err(anyhow::anyhow!("count query failed")));

        let uc = ListActivitiesUseCase::new(Arc::new(mock));
        let filter = ActivityFilter {
            task_id: Some(task_id),
            ..Default::default()
        };
        let result = uc.execute("tenant1", &filter).await;
        assert!(result.is_err());
    }
}
