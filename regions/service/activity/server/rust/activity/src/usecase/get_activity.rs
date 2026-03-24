// アクティビティ取得ユースケース。
use crate::domain::entity::activity::Activity;
use crate::domain::repository::activity_repository::ActivityRepository;
use std::sync::Arc;
use uuid::Uuid;

pub struct GetActivityUseCase {
    repo: Arc<dyn ActivityRepository>,
}

impl GetActivityUseCase {
    pub fn new(repo: Arc<dyn ActivityRepository>) -> Self {
        Self { repo }
    }

    // アクティビティ取得の全処理をトレースするためにスパンを自動生成する
    #[tracing::instrument(skip(self))]
    pub async fn execute(&self, tenant_id: &str, id: Uuid) -> anyhow::Result<Option<Activity>> {
        // ActivityError を anyhow::Error に変換して戻り値の型を合わせる
        self.repo.find_by_id(tenant_id, id).await.map_err(anyhow::Error::from)
    }
}

#[cfg(test)]
// テストコード内の .unwrap() 呼び出しを許容する（テスト失敗時にパニックで意図を明示するため）
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::activity::{ActivityStatus, ActivityType};
    use crate::domain::repository::activity_repository::MockActivityRepository;
    use chrono::Utc;
    use uuid::Uuid;

    // テスト用のサンプルアクティビティを生成するヘルパー関数
    fn sample_activity() -> Activity {
        Activity {
            id: Uuid::new_v4(),
            task_id: Uuid::new_v4(),
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

    // 正常系: 指定した ID のアクティビティが取得できることを確認する
    #[tokio::test]
    async fn test_get_activity_success() {
        let mut mock = MockActivityRepository::new();
        let activity = sample_activity();
        let activity_id = activity.id;
        let activity_clone = activity.clone();

        mock.expect_find_by_id()
            .times(1)
            .returning(move |_, _| Ok(Some(activity_clone.clone())));

        let uc = GetActivityUseCase::new(Arc::new(mock));
        let result = uc.execute("tenant1", activity_id).await;
        assert!(result.is_ok());
        let found = result.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, activity_id);
    }

    // 異常系: 指定した ID のアクティビティが存在しない場合（NotFound）に None が返ることを確認する
    #[tokio::test]
    async fn test_get_activity_not_found() {
        let mut mock = MockActivityRepository::new();

        mock.expect_find_by_id()
            .times(1)
            .returning(|_, _| Ok(None));

        let uc = GetActivityUseCase::new(Arc::new(mock));
        let result = uc.execute("tenant1", Uuid::new_v4()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // 異常系: リポジトリがエラーを返した場合にエラーが伝播することを確認する
    #[tokio::test]
    async fn test_get_activity_repository_error() {
        let mut mock = MockActivityRepository::new();

        mock.expect_find_by_id()
            .times(1)
            .returning(|_, _| Err(anyhow::anyhow!("database error")));

        let uc = GetActivityUseCase::new(Arc::new(mock));
        let result = uc.execute("tenant1", Uuid::new_v4()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("database error"));
    }
}
