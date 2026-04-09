use std::sync::Arc;

use chrono::Utc;

use crate::domain::entity::app::App;
use crate::domain::repository::AppRepository;

/// アプリ新規作成ユースケースの入力データ
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CreateAppInput {
    /// アプリ名（必須・空白不可）
    pub name: String,
    /// アプリの説明
    pub description: Option<String>,
    /// カテゴリ（必須・空白不可）
    pub category: String,
    /// アイコン URL
    pub icon_url: Option<String>,
}

/// アプリ新規作成ユースケースで発生するエラー
#[derive(Debug, thiserror::Error)]
pub enum CreateAppError {
    /// 入力値が不正
    #[error("validation error: {0}")]
    ValidationError(String),
    /// 内部エラー（DB 障害等）
    #[error("internal error: {0}")]
    Internal(String),
}

/// アプリを新規登録するユースケース
pub struct CreateAppUseCase {
    repo: Arc<dyn AppRepository>,
}

impl CreateAppUseCase {
    pub fn new(repo: Arc<dyn AppRepository>) -> Self {
        Self { repo }
    }

    // CRIT-004 監査対応: RLS テナント分離のため tenant_id を受け取りリポジトリに渡す。
    pub async fn execute(
        &self,
        tenant_id: &str,
        input: CreateAppInput,
    ) -> Result<App, CreateAppError> {
        if input.name.trim().is_empty() {
            return Err(CreateAppError::ValidationError(
                "name must not be empty".to_string(),
            ));
        }
        if input.category.trim().is_empty() {
            return Err(CreateAppError::ValidationError(
                "category must not be empty".to_string(),
            ));
        }

        let now = Utc::now();
        let app = App {
            id: uuid::Uuid::new_v4().to_string(),
            name: input.name,
            description: input.description,
            category: input.category,
            icon_url: input.icon_url,
            created_at: now,
            updated_at: now,
        };

        self.repo
            .create(tenant_id, &app)
            .await
            .map_err(|e| CreateAppError::Internal(e.to_string()))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::app_repository::MockAppRepository;

    #[tokio::test]
    async fn test_create_app_success() {
        let mut mock = MockAppRepository::new();
        mock.expect_create().returning(|_, app| Ok(app.clone()));

        let uc = CreateAppUseCase::new(Arc::new(mock));
        let input = CreateAppInput {
            name: "Test App".to_string(),
            description: Some("A test app".to_string()),
            category: "tools".to_string(),
            icon_url: None,
        };
        let result = uc.execute("tenant-1", input).await.unwrap();
        assert_eq!(result.name, "Test App");
        assert_eq!(result.category, "tools");
    }

    #[tokio::test]
    async fn test_create_app_empty_name() {
        let mock = MockAppRepository::new();
        let uc = CreateAppUseCase::new(Arc::new(mock));
        let input = CreateAppInput {
            name: "".to_string(),
            description: None,
            category: "tools".to_string(),
            icon_url: None,
        };
        let result = uc.execute("tenant-1", input).await;
        assert!(matches!(result, Err(CreateAppError::ValidationError(_))));
    }

    #[tokio::test]
    async fn test_create_app_empty_category() {
        let mock = MockAppRepository::new();
        let uc = CreateAppUseCase::new(Arc::new(mock));
        let input = CreateAppInput {
            name: "Test".to_string(),
            description: None,
            category: " ".to_string(),
            icon_url: None,
        };
        let result = uc.execute("tenant-1", input).await;
        assert!(matches!(result, Err(CreateAppError::ValidationError(_))));
    }
}
