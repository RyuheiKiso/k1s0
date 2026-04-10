use std::sync::Arc;

use chrono::Utc;

use crate::domain::entity::app::App;
use crate::domain::repository::AppRepository;

/// アプリ更新ユースケースの入力データ
#[derive(Debug, Clone, serde::Deserialize)]
pub struct UpdateAppInput {
    /// 更新対象のアプリ ID
    pub id: String,
    /// 新しいアプリ名（必須・空白不可）
    pub name: String,
    /// 新しい説明
    pub description: Option<String>,
    /// 新しいカテゴリ
    pub category: String,
    /// 新しいアイコン URL
    pub icon_url: Option<String>,
}

/// アプリ更新ユースケースで発生するエラー
#[derive(Debug, thiserror::Error)]
pub enum UpdateAppError {
    /// 指定した ID のアプリが存在しない
    #[error("app not found: {0}")]
    NotFound(String),
    /// 入力値が不正
    #[error("validation error: {0}")]
    ValidationError(String),
    /// 内部エラー（DB 障害等）
    #[error("internal error: {0}")]
    Internal(String),
}

/// アプリ情報を更新するユースケース
pub struct UpdateAppUseCase {
    repo: Arc<dyn AppRepository>,
}

impl UpdateAppUseCase {
    pub fn new(repo: Arc<dyn AppRepository>) -> Self {
        Self { repo }
    }

    // CRIT-004 監査対応: RLS テナント分離のため tenant_id を受け取りリポジトリに渡す。
    pub async fn execute(
        &self,
        tenant_id: &str,
        input: UpdateAppInput,
    ) -> Result<App, UpdateAppError> {
        if input.name.trim().is_empty() {
            return Err(UpdateAppError::ValidationError(
                "name must not be empty".to_string(),
            ));
        }

        let existing = self
            .repo
            .find_by_id(tenant_id, &input.id)
            .await
            .map_err(|e| UpdateAppError::Internal(e.to_string()))?
            .ok_or_else(|| UpdateAppError::NotFound(input.id.clone()))?;

        let app = App {
            id: existing.id,
            name: input.name,
            description: input.description,
            category: input.category,
            icon_url: input.icon_url,
            created_at: existing.created_at,
            updated_at: Utc::now(),
        };

        self.repo
            .update(tenant_id, &app)
            .await
            .map_err(|e| UpdateAppError::Internal(e.to_string()))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::app_repository::MockAppRepository;

    fn make_app() -> App {
        App {
            id: "app-1".to_string(),
            name: "Old Name".to_string(),
            description: None,
            category: "tools".to_string(),
            icon_url: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_update_app_success() {
        let mut mock = MockAppRepository::new();
        mock.expect_find_by_id()
            .returning(|_, _| Ok(Some(make_app())));
        mock.expect_update().returning(|_, app| Ok(app.clone()));

        let uc = UpdateAppUseCase::new(Arc::new(mock));
        let input = UpdateAppInput {
            id: "app-1".to_string(),
            name: "New Name".to_string(),
            description: Some("Updated".to_string()),
            category: "utils".to_string(),
            icon_url: None,
        };
        let result = uc.execute("tenant-1", input).await.unwrap();
        assert_eq!(result.name, "New Name");
        assert_eq!(result.category, "utils");
    }

    #[tokio::test]
    async fn test_update_app_not_found() {
        let mut mock = MockAppRepository::new();
        mock.expect_find_by_id().returning(|_, _| Ok(None));

        let uc = UpdateAppUseCase::new(Arc::new(mock));
        let input = UpdateAppInput {
            id: "missing".to_string(),
            name: "Name".to_string(),
            description: None,
            category: "tools".to_string(),
            icon_url: None,
        };
        let result = uc.execute("tenant-1", input).await;
        assert!(matches!(result, Err(UpdateAppError::NotFound(_))));
    }
}
