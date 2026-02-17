use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::entity::audit_log::{AuditLog, AuditLogSearchParams};
use crate::domain::repository::AuditLogRepository;

/// AuditLogPostgresRepository は AuditLogRepository の PostgreSQL 実装。
pub struct AuditLogPostgresRepository {
    pool: PgPool,
}

impl AuditLogPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuditLogRepository for AuditLogPostgresRepository {
    async fn create(&self, log: &AuditLog) -> anyhow::Result<()> {
        let metadata = serde_json::to_value(&log.metadata)?;

        sqlx::query(
            r#"
            INSERT INTO audit_logs (id, event_type, user_id, ip_address, user_agent, resource, action, result, metadata, recorded_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(log.id)
        .bind(&log.event_type)
        .bind(&log.user_id)
        .bind(&log.ip_address)
        .bind(&log.user_agent)
        .bind(&log.resource)
        .bind(&log.action)
        .bind(&log.result)
        .bind(metadata)
        .bind(log.recorded_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn search(
        &self,
        params: &AuditLogSearchParams,
    ) -> anyhow::Result<(Vec<AuditLog>, i64)> {
        let offset = (params.page - 1) * params.page_size;

        // 動的にクエリを組み立てる
        let mut conditions = Vec::new();
        let mut bind_index = 1u32;

        if params.user_id.is_some() {
            conditions.push(format!("user_id = ${}", bind_index));
            bind_index += 1;
        }
        if params.event_type.is_some() {
            conditions.push(format!("event_type = ${}", bind_index));
            bind_index += 1;
        }
        if params.result.is_some() {
            conditions.push(format!("result = ${}", bind_index));
            bind_index += 1;
        }
        if params.from.is_some() {
            conditions.push(format!("recorded_at >= ${}", bind_index));
            bind_index += 1;
        }
        if params.to.is_some() {
            conditions.push(format!("recorded_at <= ${}", bind_index));
            bind_index += 1;
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let count_query = format!(
            "SELECT COUNT(*) as count FROM audit_logs {}",
            where_clause
        );
        let data_query = format!(
            "SELECT id, event_type, user_id, ip_address, user_agent, resource, action, result, metadata, recorded_at FROM audit_logs {} ORDER BY recorded_at DESC LIMIT ${} OFFSET ${}",
            where_clause, bind_index, bind_index + 1
        );

        // count クエリ
        let mut count_q = sqlx::query_scalar::<_, i64>(&count_query);
        if let Some(ref v) = params.user_id {
            count_q = count_q.bind(v);
        }
        if let Some(ref v) = params.event_type {
            count_q = count_q.bind(v);
        }
        if let Some(ref v) = params.result {
            count_q = count_q.bind(v);
        }
        if let Some(ref v) = params.from {
            count_q = count_q.bind(v);
        }
        if let Some(ref v) = params.to {
            count_q = count_q.bind(v);
        }

        let total_count = count_q.fetch_one(&self.pool).await?;

        // データクエリ
        let mut data_q = sqlx::query_as::<_, AuditLogRow>(&data_query);
        if let Some(ref v) = params.user_id {
            data_q = data_q.bind(v);
        }
        if let Some(ref v) = params.event_type {
            data_q = data_q.bind(v);
        }
        if let Some(ref v) = params.result {
            data_q = data_q.bind(v);
        }
        if let Some(ref v) = params.from {
            data_q = data_q.bind(v);
        }
        if let Some(ref v) = params.to {
            data_q = data_q.bind(v);
        }
        data_q = data_q.bind(params.page_size as i64);
        data_q = data_q.bind(offset as i64);

        let rows: Vec<AuditLogRow> = data_q.fetch_all(&self.pool).await?;
        let logs: Vec<AuditLog> = rows.into_iter().map(|r| r.into()).collect();

        Ok((logs, total_count))
    }
}

/// AuditLogRow は DB から取得した行を表す中間構造体。
#[derive(Debug, sqlx::FromRow)]
struct AuditLogRow {
    id: uuid::Uuid,
    event_type: String,
    user_id: String,
    ip_address: String,
    user_agent: String,
    resource: String,
    action: String,
    result: String,
    metadata: serde_json::Value,
    recorded_at: chrono::DateTime<chrono::Utc>,
}

impl From<AuditLogRow> for AuditLog {
    fn from(row: AuditLogRow) -> Self {
        let metadata: std::collections::HashMap<String, String> =
            serde_json::from_value(row.metadata).unwrap_or_default();

        AuditLog {
            id: row.id,
            event_type: row.event_type,
            user_id: row.user_id,
            ip_address: row.ip_address,
            user_agent: row.user_agent,
            resource: row.resource,
            action: row.action,
            result: row.result,
            metadata,
            recorded_at: row.recorded_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_log_row_to_audit_log() {
        let row = AuditLogRow {
            id: uuid::Uuid::new_v4(),
            event_type: "LOGIN_SUCCESS".to_string(),
            user_id: "user-1".to_string(),
            ip_address: "127.0.0.1".to_string(),
            user_agent: "test".to_string(),
            resource: "/api/v1/auth".to_string(),
            action: "POST".to_string(),
            result: "SUCCESS".to_string(),
            metadata: serde_json::json!({"client_id": "react-spa"}),
            recorded_at: chrono::Utc::now(),
        };

        let log: AuditLog = row.into();
        assert_eq!(log.event_type, "LOGIN_SUCCESS");
        assert_eq!(log.metadata.get("client_id").unwrap(), "react-spa");
    }

    #[test]
    fn test_audit_log_row_empty_metadata() {
        let row = AuditLogRow {
            id: uuid::Uuid::new_v4(),
            event_type: "TOKEN_VALIDATE".to_string(),
            user_id: "user-1".to_string(),
            ip_address: "10.0.0.1".to_string(),
            user_agent: "".to_string(),
            resource: "/api/v1/auth/token/validate".to_string(),
            action: "POST".to_string(),
            result: "SUCCESS".to_string(),
            metadata: serde_json::json!({}),
            recorded_at: chrono::Utc::now(),
        };

        let log: AuditLog = row.into();
        assert!(log.metadata.is_empty());
    }
}
