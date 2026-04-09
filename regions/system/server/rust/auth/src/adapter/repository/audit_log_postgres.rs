use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::entity::audit_log::{AuditLog, AuditLogSearchParams};
use crate::domain::repository::AuditLogRepository;

/// `AuditLogPostgresRepository` は `AuditLogRepository` の `PostgreSQL` 実装。
pub struct AuditLogPostgresRepository {
    pool: PgPool,
    metrics: Option<Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl AuditLogPostgresRepository {
    #[allow(dead_code)]
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            metrics: None,
        }
    }

    #[must_use]
    pub fn with_metrics(pool: PgPool, metrics: Arc<k1s0_telemetry::metrics::Metrics>) -> Self {
        Self {
            pool,
            metrics: Some(metrics),
        }
    }
}

#[async_trait]
impl AuditLogRepository for AuditLogPostgresRepository {
    async fn create(&self, log: &AuditLog) -> anyhow::Result<()> {
        let start = std::time::Instant::now();

        let result = sqlx::query(
            r"
            INSERT INTO auth.audit_logs (
                id, event_type, user_id, ip_address, user_agent,
                resource, resource_id, action, result,
                detail, trace_id, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ",
        )
        .bind(log.id)
        .bind(&log.event_type)
        .bind(&log.user_id)
        .bind(&log.ip_address)
        .bind(&log.user_agent)
        .bind(&log.resource)
        .bind(&log.resource_id)
        .bind(&log.action)
        .bind(&log.result)
        .bind(&log.detail)
        .bind(&log.trace_id)
        .bind(log.created_at)
        .execute(&self.pool)
        .await;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("create", "audit_logs", start.elapsed().as_secs_f64());
        }

        result?;
        Ok(())
    }

    async fn search(&self, params: &AuditLogSearchParams) -> anyhow::Result<(Vec<AuditLog>, i64)> {
        let offset = (params.page - 1) * params.page_size;

        // MED-05 監査対応: bind_index 手動インクリメントを廃止し sqlx::QueryBuilder を使用する。
        // QueryBuilder はパラメータ番号（$N）を自動管理するため、
        // 条件追加・削除時の番号ずれによるバグを防止できる。
        // COUNT クエリ: QueryBuilder で WHERE 句を動的に組み立てる
        let mut count_qb =
            sqlx::QueryBuilder::new("SELECT COUNT(*) FROM auth.audit_logs WHERE 1=1");

        if let Some(ref v) = params.user_id {
            count_qb.push(" AND user_id = ").push_bind(v);
        }
        if let Some(ref v) = params.event_type {
            count_qb.push(" AND event_type = ").push_bind(v);
        }
        if let Some(ref v) = params.result {
            count_qb.push(" AND result = ").push_bind(v);
        }
        if let Some(ref v) = params.from {
            count_qb.push(" AND created_at >= ").push_bind(v);
        }
        if let Some(ref v) = params.to {
            count_qb.push(" AND created_at <= ").push_bind(v);
        }

        let start = std::time::Instant::now();
        let total_count: i64 = count_qb.build_query_scalar().fetch_one(&self.pool).await?;
        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("search_count", "audit_logs", start.elapsed().as_secs_f64());
        }

        // DATA クエリ: COUNT クエリと同一条件で QueryBuilder を使用する
        let mut data_qb = sqlx::QueryBuilder::new(
            "SELECT id, event_type, user_id, ip_address, user_agent, resource, resource_id, action, result, detail, trace_id, created_at FROM auth.audit_logs WHERE 1=1",
        );

        if let Some(ref v) = params.user_id {
            data_qb.push(" AND user_id = ").push_bind(v);
        }
        if let Some(ref v) = params.event_type {
            data_qb.push(" AND event_type = ").push_bind(v);
        }
        if let Some(ref v) = params.result {
            data_qb.push(" AND result = ").push_bind(v);
        }
        if let Some(ref v) = params.from {
            data_qb.push(" AND created_at >= ").push_bind(v);
        }
        if let Some(ref v) = params.to {
            data_qb.push(" AND created_at <= ").push_bind(v);
        }

        // ページネーション用の ORDER BY / LIMIT / OFFSET を追加する
        data_qb
            .push(" ORDER BY created_at DESC LIMIT ")
            .push_bind(i64::from(params.page_size))
            .push(" OFFSET ")
            .push_bind(i64::from(offset));

        let start = std::time::Instant::now();
        let rows: Vec<AuditLogRow> = data_qb
            .build_query_as::<AuditLogRow>()
            .fetch_all(&self.pool)
            .await?;
        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("search", "audit_logs", start.elapsed().as_secs_f64());
        }
        let logs: Vec<AuditLog> = rows.into_iter().map(std::convert::Into::into).collect();

        Ok((logs, total_count))
    }
}

/// `AuditLogRow` は DB から取得した行を表す中間構造体。
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AuditLogRow {
    pub id: uuid::Uuid,
    pub event_type: String,
    pub user_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub resource: Option<String>,
    pub resource_id: Option<String>,
    pub action: String,
    pub result: String,
    pub detail: Option<serde_json::Value>,
    pub trace_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<AuditLogRow> for AuditLog {
    fn from(row: AuditLogRow) -> Self {
        AuditLog {
            id: row.id,
            event_type: row.event_type,
            user_id: row.user_id.unwrap_or_default(),
            ip_address: row.ip_address.unwrap_or_default(),
            user_agent: row.user_agent.unwrap_or_default(),
            resource: row.resource.unwrap_or_default(),
            resource_id: row.resource_id,
            action: row.action,
            result: row.result,
            detail: row.detail,
            trace_id: row.trace_id,
            created_at: row.created_at,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::audit_log_repository::MockAuditLogRepository;

    #[test]
    fn test_audit_log_row_to_audit_log() {
        let row = AuditLogRow {
            id: uuid::Uuid::new_v4(),
            event_type: "LOGIN_SUCCESS".to_string(),
            user_id: Some("user-1".to_string()),
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("test".to_string()),
            resource: Some("/api/v1/auth".to_string()),
            resource_id: None,
            action: "POST".to_string(),
            result: "SUCCESS".to_string(),
            detail: Some(serde_json::json!({"client_id": "react-spa"})),
            trace_id: Some("trace-001".to_string()),
            created_at: chrono::Utc::now(),
        };

        let log: AuditLog = row.into();
        assert_eq!(log.event_type, "LOGIN_SUCCESS");
        assert_eq!(log.detail.as_ref().unwrap()["client_id"], "react-spa");
        assert_eq!(log.trace_id.as_deref(), Some("trace-001"));
    }

    #[test]
    fn test_audit_log_row_null_detail() {
        let row = AuditLogRow {
            id: uuid::Uuid::new_v4(),
            event_type: "TOKEN_VALIDATE".to_string(),
            user_id: Some("user-1".to_string()),
            ip_address: Some("10.0.0.1".to_string()),
            user_agent: Some("".to_string()),
            resource: Some("/api/v1/auth/token/validate".to_string()),
            resource_id: None,
            action: "POST".to_string(),
            result: "SUCCESS".to_string(),
            detail: None,
            trace_id: None,
            created_at: chrono::Utc::now(),
        };

        let log: AuditLog = row.into();
        assert!(log.detail.is_none());
        assert!(log.trace_id.is_none());
    }

    #[tokio::test]
    async fn test_mock_create_success() {
        let mut mock = MockAuditLogRepository::new();
        mock.expect_create().returning(|_| Ok(()));

        let log = AuditLog {
            id: uuid::Uuid::new_v4(),
            event_type: "LOGIN_SUCCESS".to_string(),
            user_id: "user-1".to_string(),
            ip_address: "127.0.0.1".to_string(),
            user_agent: "Mozilla/5.0".to_string(),
            resource: "/api/v1/auth/token".to_string(),
            resource_id: None,
            action: "POST".to_string(),
            result: "SUCCESS".to_string(),
            detail: Some(serde_json::json!({"client_id": "react-spa"})),
            trace_id: None,
            created_at: chrono::Utc::now(),
        };

        let result = mock.create(&log).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_search_by_user_id() {
        let mut mock = MockAuditLogRepository::new();
        let expected_log = AuditLog {
            id: uuid::Uuid::new_v4(),
            event_type: "LOGIN_SUCCESS".to_string(),
            user_id: "user-1".to_string(),
            ip_address: "127.0.0.1".to_string(),
            user_agent: "test".to_string(),
            resource: "/api/v1/auth/token".to_string(),
            resource_id: None,
            action: "POST".to_string(),
            result: "SUCCESS".to_string(),
            detail: None,
            trace_id: None,
            created_at: chrono::Utc::now(),
        };
        let log_clone = expected_log.clone();

        mock.expect_search()
            .withf(|p| p.user_id == Some("user-1".to_string()))
            .returning(move |_| Ok((vec![log_clone.clone()], 1)));

        let params = AuditLogSearchParams {
            user_id: Some("user-1".to_string()),
            ..Default::default()
        };
        let (logs, total): (Vec<AuditLog>, i64) = mock.search(&params).await.unwrap();
        assert_eq!(total, 1);
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].user_id, "user-1");
    }

    #[tokio::test]
    async fn test_mock_search_by_event_type() {
        let mut mock = MockAuditLogRepository::new();

        mock.expect_search()
            .withf(|p| p.event_type == Some("TOKEN_VALIDATE".to_string()))
            .returning(|_| {
                let log = AuditLog {
                    id: uuid::Uuid::new_v4(),
                    event_type: "TOKEN_VALIDATE".to_string(),
                    user_id: "user-1".to_string(),
                    ip_address: "10.0.0.1".to_string(),
                    user_agent: "".to_string(),
                    resource: "/api/v1/auth/token/validate".to_string(),
                    resource_id: None,
                    action: "POST".to_string(),
                    result: "SUCCESS".to_string(),
                    detail: None,
                    trace_id: None,
                    created_at: chrono::Utc::now(),
                };
                Ok((vec![log], 1))
            });

        let params = AuditLogSearchParams {
            event_type: Some("TOKEN_VALIDATE".to_string()),
            ..Default::default()
        };
        let (logs, total): (Vec<AuditLog>, i64) = mock.search(&params).await.unwrap();
        assert_eq!(total, 1);
        assert_eq!(logs[0].event_type, "TOKEN_VALIDATE");
    }

    #[tokio::test]
    async fn test_mock_search_by_date_range() {
        let mut mock = MockAuditLogRepository::new();

        mock.expect_search()
            .withf(|p| p.from.is_some() && p.to.is_some())
            .returning(|_| {
                let log = AuditLog {
                    id: uuid::Uuid::new_v4(),
                    event_type: "LOGIN_SUCCESS".to_string(),
                    user_id: "user-1".to_string(),
                    ip_address: "127.0.0.1".to_string(),
                    user_agent: "test".to_string(),
                    resource: "/api/v1/auth/token".to_string(),
                    resource_id: None,
                    action: "POST".to_string(),
                    result: "SUCCESS".to_string(),
                    detail: None,
                    trace_id: None,
                    created_at: chrono::Utc::now(),
                };
                Ok((vec![log], 1))
            });

        let from = chrono::Utc::now() - chrono::Duration::days(30);
        let to = chrono::Utc::now();
        let params = AuditLogSearchParams {
            from: Some(from),
            to: Some(to),
            page: 1,
            page_size: 20,
            ..Default::default()
        };
        let (logs, total): (Vec<AuditLog>, i64) = mock.search(&params).await.unwrap();
        assert_eq!(total, 1);
        assert_eq!(logs.len(), 1);
    }

    #[tokio::test]
    async fn test_mock_search_pagination() {
        let mut mock = MockAuditLogRepository::new();

        mock.expect_search()
            .withf(|p| p.page == 2 && p.page_size == 10)
            .returning(|_| Ok((vec![], 25)));

        let params = AuditLogSearchParams {
            page: 2,
            page_size: 10,
            ..Default::default()
        };
        let (logs, total): (Vec<AuditLog>, i64) = mock.search(&params).await.unwrap();
        assert_eq!(total, 25);
        assert!(logs.is_empty());
    }

    #[tokio::test]
    async fn test_mock_search_no_results() {
        let mut mock = MockAuditLogRepository::new();

        mock.expect_search()
            .withf(|p| p.user_id == Some("nonexistent-user".to_string()))
            .returning(|_| Ok((vec![], 0)));

        let params = AuditLogSearchParams {
            user_id: Some("nonexistent-user".to_string()),
            page: 1,
            page_size: 20,
            ..Default::default()
        };
        let (logs, total): (Vec<AuditLog>, i64) = mock.search(&params).await.unwrap();
        assert_eq!(total, 0);
        assert!(logs.is_empty());
    }
}
