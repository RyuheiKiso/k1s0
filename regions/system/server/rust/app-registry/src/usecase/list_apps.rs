use std::sync::Arc;

use crate::domain::entity::app::App;
use crate::domain::repository::AppRepository;

/// ListAppsUseCase はアプリ一覧取得ユースケース。
pub struct ListAppsUseCase {
    app_repo: Arc<dyn AppRepository>,
}

impl ListAppsUseCase {
    pub fn new(app_repo: Arc<dyn AppRepository>) -> Self {
        Self { app_repo }
    }

    pub async fn execute(
        &self,
        category: Option<&str>,
        search: Option<&str>,
    ) -> anyhow::Result<Vec<App>> {
        self.app_repo.list(category, search).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::app_repository::MockAppRepository;

    #[tokio::test]
    async fn test_list_apps_success() {
        let mut mock = MockAppRepository::new();
        mock.expect_list().returning(|_, _| {
            Ok(vec![App {
                id: "cli".to_string(),
                name: "k1s0 CLI".to_string(),
                description: Some("Command-line interface".to_string()),
                category: "tools".to_string(),
                icon_url: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }])
        });

        let uc = ListAppsUseCase::new(Arc::new(mock));
        let result = uc.execute(None, None).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "cli");
    }

    #[tokio::test]
    async fn test_list_apps_empty() {
        let mut mock = MockAppRepository::new();
        mock.expect_list().returning(|_, _| Ok(vec![]));

        let uc = ListAppsUseCase::new(Arc::new(mock));
        let result = uc.execute(Some("nonexistent"), None).await.unwrap();
        assert!(result.is_empty());
    }
}
