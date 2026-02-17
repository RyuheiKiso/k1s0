use std::sync::Arc;

use crate::domain::entity::audit_log::{AuditLogSearchParams, AuditLogSearchResult};
use crate::domain::entity::user::Pagination;
use crate::domain::repository::AuditLogRepository;

/// SearchAuditLogsError は監査ログ検索に関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum SearchAuditLogsError {
    #[error("invalid page: {0}")]
    InvalidPage(String),

    #[error("invalid page_size: {0}")]
    InvalidPageSize(String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// SearchAuditLogsQueryParams は監査ログ検索のクエリパラメータを表す。
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct SearchAuditLogsQueryParams {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub user_id: Option<String>,
    pub event_type: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub result: Option<String>,
}

/// SearchAuditLogsUseCase は監査ログ検索ユースケース。
pub struct SearchAuditLogsUseCase {
    audit_repo: Arc<dyn AuditLogRepository>,
}

impl SearchAuditLogsUseCase {
    pub fn new(audit_repo: Arc<dyn AuditLogRepository>) -> Self {
        Self { audit_repo }
    }

    /// 監査ログを検索する。
    pub async fn execute(
        &self,
        query: &SearchAuditLogsQueryParams,
    ) -> Result<AuditLogSearchResult, SearchAuditLogsError> {
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(50);

        // バリデーション
        if page < 1 {
            return Err(SearchAuditLogsError::InvalidPage(
                "page must be >= 1".to_string(),
            ));
        }
        if !(1..=200).contains(&page_size) {
            return Err(SearchAuditLogsError::InvalidPageSize(
                "page_size must be between 1 and 200".to_string(),
            ));
        }

        // ISO 8601 の日時パース
        let from = query
            .from
            .as_ref()
            .map(|s| {
                chrono::DateTime::parse_from_rfc3339(s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .map_err(|_| {
                        SearchAuditLogsError::InvalidPage(format!(
                            "invalid from datetime: {}",
                            s
                        ))
                    })
            })
            .transpose()?;

        let to = query
            .to
            .as_ref()
            .map(|s| {
                chrono::DateTime::parse_from_rfc3339(s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .map_err(|_| {
                        SearchAuditLogsError::InvalidPage(format!(
                            "invalid to datetime: {}",
                            s
                        ))
                    })
            })
            .transpose()?;

        let params = AuditLogSearchParams {
            user_id: query.user_id.clone(),
            event_type: query.event_type.clone(),
            result: query.result.clone(),
            from,
            to,
            page,
            page_size,
        };

        let (logs, total_count) = self
            .audit_repo
            .search(&params)
            .await
            .map_err(|e| SearchAuditLogsError::Internal(e.to_string()))?;

        let has_next = (page as i64 * page_size as i64) < total_count;

        Ok(AuditLogSearchResult {
            logs,
            pagination: Pagination {
                total_count,
                page,
                page_size,
                has_next,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::audit_log::AuditLog;
    use crate::domain::repository::audit_log_repository::MockAuditLogRepository;
    use std::collections::HashMap;
    use uuid::Uuid;

    fn make_sample_logs() -> Vec<AuditLog> {
        vec![
            AuditLog {
                id: Uuid::new_v4(),
                event_type: "LOGIN_SUCCESS".to_string(),
                user_id: "user-1".to_string(),
                ip_address: "192.168.1.100".to_string(),
                user_agent: "Mozilla/5.0".to_string(),
                resource: "/api/v1/auth/token".to_string(),
                action: "POST".to_string(),
                result: "SUCCESS".to_string(),
                metadata: HashMap::from([("client_id".to_string(), "react-spa".to_string())]),
                recorded_at: chrono::Utc::now(),
            },
            AuditLog {
                id: Uuid::new_v4(),
                event_type: "LOGIN_FAILURE".to_string(),
                user_id: "user-2".to_string(),
                ip_address: "10.0.0.1".to_string(),
                user_agent: "curl/7.0".to_string(),
                resource: "/api/v1/auth/token".to_string(),
                action: "POST".to_string(),
                result: "FAILURE".to_string(),
                metadata: HashMap::new(),
                recorded_at: chrono::Utc::now(),
            },
        ]
    }

    #[tokio::test]
    async fn test_search_audit_logs_default_params() {
        let mut mock = MockAuditLogRepository::new();
        let logs = make_sample_logs();
        let log_count = logs.len() as i64;

        mock.expect_search()
            .returning(move |params| {
                assert_eq!(params.page, 1);
                assert_eq!(params.page_size, 50);
                Ok((logs.clone(), log_count))
            });

        let uc = SearchAuditLogsUseCase::new(Arc::new(mock));
        let result = uc
            .execute(&SearchAuditLogsQueryParams::default())
            .await;
        assert!(result.is_ok());

        let search_result = result.unwrap();
        assert_eq!(search_result.logs.len(), 2);
        assert_eq!(search_result.pagination.total_count, 2);
        assert_eq!(search_result.pagination.page, 1);
        assert_eq!(search_result.pagination.page_size, 50);
        assert!(!search_result.pagination.has_next);
    }

    #[tokio::test]
    async fn test_search_audit_logs_with_filters() {
        let mut mock = MockAuditLogRepository::new();
        mock.expect_search()
            .withf(|params| {
                params.user_id.as_deref() == Some("user-1")
                    && params.event_type.as_deref() == Some("LOGIN_SUCCESS")
                    && params.result.as_deref() == Some("SUCCESS")
            })
            .returning(|_| {
                Ok((vec![make_sample_logs()[0].clone()], 1))
            });

        let uc = SearchAuditLogsUseCase::new(Arc::new(mock));
        let query = SearchAuditLogsQueryParams {
            user_id: Some("user-1".to_string()),
            event_type: Some("LOGIN_SUCCESS".to_string()),
            result: Some("SUCCESS".to_string()),
            ..Default::default()
        };

        let result = uc.execute(&query).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().logs.len(), 1);
    }

    #[tokio::test]
    async fn test_search_audit_logs_pagination() {
        let mut mock = MockAuditLogRepository::new();
        mock.expect_search()
            .withf(|params| params.page == 2 && params.page_size == 10)
            .returning(|_| Ok((vec![], 25)));

        let uc = SearchAuditLogsUseCase::new(Arc::new(mock));
        let query = SearchAuditLogsQueryParams {
            page: Some(2),
            page_size: Some(10),
            ..Default::default()
        };

        let result = uc.execute(&query).await;
        assert!(result.is_ok());

        let search_result = result.unwrap();
        assert_eq!(search_result.pagination.total_count, 25);
        assert!(search_result.pagination.has_next); // 2*10=20 < 25
    }

    #[tokio::test]
    async fn test_search_audit_logs_has_next_false_on_last_page() {
        let mut mock = MockAuditLogRepository::new();
        mock.expect_search()
            .returning(|_| Ok((vec![], 20)));

        let uc = SearchAuditLogsUseCase::new(Arc::new(mock));
        let query = SearchAuditLogsQueryParams {
            page: Some(2),
            page_size: Some(10),
            ..Default::default()
        };

        let result = uc.execute(&query).await;
        let search_result = result.unwrap();
        assert!(!search_result.pagination.has_next); // 2*10=20 == 20
    }

    #[tokio::test]
    async fn test_search_audit_logs_invalid_page() {
        let mock = MockAuditLogRepository::new();
        let uc = SearchAuditLogsUseCase::new(Arc::new(mock));

        let query = SearchAuditLogsQueryParams {
            page: Some(0),
            ..Default::default()
        };

        let result = uc.execute(&query).await;
        assert!(matches!(
            result.unwrap_err(),
            SearchAuditLogsError::InvalidPage(_)
        ));
    }

    #[tokio::test]
    async fn test_search_audit_logs_invalid_page_size() {
        let mock = MockAuditLogRepository::new();
        let uc = SearchAuditLogsUseCase::new(Arc::new(mock));

        let query = SearchAuditLogsQueryParams {
            page_size: Some(201),
            ..Default::default()
        };

        let result = uc.execute(&query).await;
        assert!(matches!(
            result.unwrap_err(),
            SearchAuditLogsError::InvalidPageSize(_)
        ));
    }

    #[tokio::test]
    async fn test_search_audit_logs_with_date_range() {
        let mut mock = MockAuditLogRepository::new();
        mock.expect_search()
            .withf(|params| params.from.is_some() && params.to.is_some())
            .returning(|_| Ok((vec![], 0)));

        let uc = SearchAuditLogsUseCase::new(Arc::new(mock));
        let query = SearchAuditLogsQueryParams {
            from: Some("2026-02-01T00:00:00Z".to_string()),
            to: Some("2026-02-17T23:59:59Z".to_string()),
            ..Default::default()
        };

        let result = uc.execute(&query).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_search_audit_logs_invalid_date_format() {
        let mock = MockAuditLogRepository::new();
        let uc = SearchAuditLogsUseCase::new(Arc::new(mock));

        let query = SearchAuditLogsQueryParams {
            from: Some("not-a-date".to_string()),
            ..Default::default()
        };

        let result = uc.execute(&query).await;
        assert!(result.is_err());
    }
}
