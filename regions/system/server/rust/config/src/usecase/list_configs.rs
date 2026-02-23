use std::sync::Arc;

use serde::Deserialize;

use crate::domain::entity::config_entry::ConfigListResult;
use crate::domain::repository::ConfigRepository;

/// ListConfigsError は設定値一覧取得に関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum ListConfigsError {
    #[error("validation error: {0}")]
    Validation(String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// ListConfigsParams は設定値一覧取得のクエリパラメータを表す。
#[derive(Debug, Clone, Deserialize)]
pub struct ListConfigsParams {
    #[serde(default = "default_page")]
    pub page: i32,
    #[serde(default = "default_page_size")]
    pub page_size: i32,
    #[serde(default)]
    pub search: Option<String>,
}

fn default_page() -> i32 {
    1
}

fn default_page_size() -> i32 {
    20
}

/// ListConfigsUseCase は設定値一覧取得ユースケース。
pub struct ListConfigsUseCase {
    config_repo: Arc<dyn ConfigRepository>,
}

impl ListConfigsUseCase {
    pub fn new(config_repo: Arc<dyn ConfigRepository>) -> Self {
        Self { config_repo }
    }

    /// namespace 内の設定値一覧を取得する。
    pub async fn execute(
        &self,
        namespace: &str,
        params: &ListConfigsParams,
    ) -> Result<ConfigListResult, ListConfigsError> {
        // バリデーション
        if params.page < 1 {
            return Err(ListConfigsError::Validation(
                "page must be >= 1".to_string(),
            ));
        }
        if params.page_size < 1 || params.page_size > 100 {
            return Err(ListConfigsError::Validation(
                "page_size must be between 1 and 100".to_string(),
            ));
        }

        self.config_repo
            .list_by_namespace(
                namespace,
                params.page,
                params.page_size,
                params.search.clone(),
            )
            .await
            .map_err(|e| ListConfigsError::Internal(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::config_entry::{ConfigEntry, Pagination};
    use crate::domain::repository::config_repository::MockConfigRepository;
    use chrono::Utc;
    use uuid::Uuid;

    fn make_test_entry(key: &str) -> ConfigEntry {
        ConfigEntry {
            id: Uuid::new_v4(),
            namespace: "system.auth.database".to_string(),
            key: key.to_string(),
            value_json: serde_json::json!(25),
            version: 1,
            description: None,
            created_by: "admin@example.com".to_string(),
            updated_by: "admin@example.com".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_list_configs_success() {
        let mut mock = MockConfigRepository::new();
        mock.expect_list_by_namespace()
            .withf(|ns, page, page_size, _| {
                ns == "system.auth.database" && *page == 1 && *page_size == 20
            })
            .returning(|_, page, page_size, _| {
                Ok(ConfigListResult {
                    entries: vec![
                        make_test_entry("max_connections"),
                        make_test_entry("ssl_mode"),
                    ],
                    pagination: Pagination {
                        total_count: 2,
                        page,
                        page_size,
                        has_next: false,
                    },
                })
            });

        let uc = ListConfigsUseCase::new(Arc::new(mock));
        let params = ListConfigsParams {
            page: 1,
            page_size: 20,
            search: None,
        };
        let result = uc.execute("system.auth.database", &params).await;
        assert!(result.is_ok());

        let list = result.unwrap();
        assert_eq!(list.entries.len(), 2);
        assert_eq!(list.pagination.total_count, 2);
        assert!(!list.pagination.has_next);
    }

    #[tokio::test]
    async fn test_list_configs_with_search() {
        let mut mock = MockConfigRepository::new();
        mock.expect_list_by_namespace()
            .withf(|_, _, _, search| search.as_deref() == Some("max"))
            .returning(|_, page, page_size, _| {
                Ok(ConfigListResult {
                    entries: vec![make_test_entry("max_connections")],
                    pagination: Pagination {
                        total_count: 1,
                        page,
                        page_size,
                        has_next: false,
                    },
                })
            });

        let uc = ListConfigsUseCase::new(Arc::new(mock));
        let params = ListConfigsParams {
            page: 1,
            page_size: 20,
            search: Some("max".to_string()),
        };
        let result = uc.execute("system.auth.database", &params).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().entries.len(), 1);
    }

    #[tokio::test]
    async fn test_list_configs_invalid_page() {
        let mock = MockConfigRepository::new();
        let uc = ListConfigsUseCase::new(Arc::new(mock));
        let params = ListConfigsParams {
            page: 0,
            page_size: 20,
            search: None,
        };

        let result = uc.execute("system.auth.database", &params).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ListConfigsError::Validation(msg) => assert!(msg.contains("page must be >= 1")),
            e => unreachable!("unexpected error in test: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_list_configs_invalid_page_size_too_large() {
        let mock = MockConfigRepository::new();
        let uc = ListConfigsUseCase::new(Arc::new(mock));
        let params = ListConfigsParams {
            page: 1,
            page_size: 101,
            search: None,
        };

        let result = uc.execute("system.auth.database", &params).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ListConfigsError::Validation(msg) => {
                assert!(msg.contains("page_size must be between 1 and 100"))
            }
            e => unreachable!("unexpected error in test: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_list_configs_invalid_page_size_zero() {
        let mock = MockConfigRepository::new();
        let uc = ListConfigsUseCase::new(Arc::new(mock));
        let params = ListConfigsParams {
            page: 1,
            page_size: 0,
            search: None,
        };

        let result = uc.execute("system.auth.database", &params).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ListConfigsError::Validation(msg) => {
                assert!(msg.contains("page_size must be between 1 and 100"))
            }
            e => unreachable!("unexpected error in test: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_list_configs_internal_error() {
        let mut mock = MockConfigRepository::new();
        mock.expect_list_by_namespace()
            .returning(|_, _, _, _| Err(anyhow::anyhow!("connection refused")));

        let uc = ListConfigsUseCase::new(Arc::new(mock));
        let params = ListConfigsParams {
            page: 1,
            page_size: 20,
            search: None,
        };

        let result = uc.execute("system.auth.database", &params).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ListConfigsError::Internal(msg) => assert!(msg.contains("connection refused")),
            e => unreachable!("unexpected error in test: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_list_configs_empty_result() {
        let mut mock = MockConfigRepository::new();
        mock.expect_list_by_namespace()
            .returning(|_, page, page_size, _| {
                Ok(ConfigListResult {
                    entries: vec![],
                    pagination: Pagination {
                        total_count: 0,
                        page,
                        page_size,
                        has_next: false,
                    },
                })
            });

        let uc = ListConfigsUseCase::new(Arc::new(mock));
        let params = ListConfigsParams {
            page: 1,
            page_size: 20,
            search: None,
        };

        let result = uc.execute("empty.namespace", &params).await.unwrap();
        assert!(result.entries.is_empty());
        assert_eq!(result.pagination.total_count, 0);
    }
}
