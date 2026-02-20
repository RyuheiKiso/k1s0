use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// AuditLog は監査ログエントリを表すドメインエンティティ。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditLog {
    pub id: Uuid,
    pub event_type: String,
    pub user_id: String,
    pub ip_address: String,
    pub user_agent: String,
    pub resource: String,
    pub resource_id: Option<String>,
    pub action: String,
    pub result: String,
    #[serde(default)]
    pub detail: Option<serde_json::Value>,
    pub trace_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// CreateAuditLogRequest は監査ログ記録リクエストを表す。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateAuditLogRequest {
    pub event_type: String,
    pub user_id: String,
    pub ip_address: String,
    #[serde(default)]
    pub user_agent: String,
    pub resource: String,
    pub resource_id: Option<String>,
    pub action: String,
    pub result: String,
    #[serde(default)]
    pub detail: Option<serde_json::Value>,
    pub trace_id: Option<String>,
}

/// AuditLogSearchParams は監査ログ検索パラメータを表す。
#[derive(Debug, Clone, Default)]
pub struct AuditLogSearchParams {
    pub user_id: Option<String>,
    pub event_type: Option<String>,
    pub result: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub page: i32,
    pub page_size: i32,
}

/// AuditLogSearchResult は監査ログ検索結果を表す。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogSearchResult {
    pub logs: Vec<AuditLog>,
    pub pagination: super::user::Pagination,
}

/// CreateAuditLogResponse は監査ログ作成レスポンスを表す。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAuditLogResponse {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl AuditLog {
    /// 新しい AuditLog エンティティを作成する。
    pub fn new(req: CreateAuditLogRequest) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type: req.event_type,
            user_id: req.user_id,
            ip_address: req.ip_address,
            user_agent: req.user_agent,
            resource: req.resource,
            resource_id: req.resource_id,
            action: req.action,
            result: req.result,
            detail: req.detail,
            trace_id: req.trace_id,
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_log_new() {
        let req = CreateAuditLogRequest {
            event_type: "LOGIN_SUCCESS".to_string(),
            user_id: "user-uuid-1234".to_string(),
            ip_address: "192.168.1.100".to_string(),
            user_agent: "Mozilla/5.0".to_string(),
            resource: "/api/v1/auth/token".to_string(),
            resource_id: None,
            action: "POST".to_string(),
            result: "SUCCESS".to_string(),
            detail: Some(serde_json::json!({"client_id": "react-spa"})),
            trace_id: Some("4bf92f3577b34da6a3ce929d0e0e4736".to_string()),
        };

        let log = AuditLog::new(req);

        assert_eq!(log.event_type, "LOGIN_SUCCESS");
        assert_eq!(log.user_id, "user-uuid-1234");
        assert_eq!(log.ip_address, "192.168.1.100");
        assert_eq!(log.result, "SUCCESS");
        assert_eq!(
            log.detail.as_ref().unwrap()["client_id"],
            "react-spa"
        );
        assert_eq!(
            log.trace_id.as_deref(),
            Some("4bf92f3577b34da6a3ce929d0e0e4736")
        );
        assert!(!log.id.is_nil());
    }

    #[test]
    fn test_audit_log_new_minimal() {
        let req = CreateAuditLogRequest {
            event_type: "TOKEN_VALIDATE".to_string(),
            user_id: "user-uuid-5678".to_string(),
            ip_address: "10.0.0.1".to_string(),
            user_agent: String::new(),
            resource: "/api/v1/auth/token/validate".to_string(),
            resource_id: None,
            action: "POST".to_string(),
            result: "SUCCESS".to_string(),
            detail: None,
            trace_id: None,
        };

        let log = AuditLog::new(req);
        assert!(log.detail.is_none());
        assert!(log.trace_id.is_none());
        assert!(log.resource_id.is_none());
    }

    #[test]
    fn test_audit_log_serialization_roundtrip() {
        let log = AuditLog {
            id: Uuid::new_v4(),
            event_type: "TOKEN_VALIDATE".to_string(),
            user_id: "user-uuid-5678".to_string(),
            ip_address: "10.0.0.1".to_string(),
            user_agent: "k1s0-client/1.0".to_string(),
            resource: "/api/v1/auth/token/validate".to_string(),
            resource_id: Some("token-001".to_string()),
            action: "POST".to_string(),
            result: "SUCCESS".to_string(),
            detail: Some(serde_json::json!({"grant_type": "authorization_code"})),
            trace_id: Some("abc123".to_string()),
            created_at: Utc::now(),
        };

        let json = serde_json::to_string(&log).unwrap();
        let deserialized: AuditLog = serde_json::from_str(&json).unwrap();
        assert_eq!(log, deserialized);
    }

    #[test]
    fn test_audit_log_result_failure() {
        let req = CreateAuditLogRequest {
            event_type: "LOGIN_FAILURE".to_string(),
            user_id: "user-uuid-1234".to_string(),
            ip_address: "192.168.1.100".to_string(),
            user_agent: String::new(),
            resource: "/api/v1/auth/token".to_string(),
            resource_id: None,
            action: "POST".to_string(),
            result: "FAILURE".to_string(),
            detail: None,
            trace_id: None,
        };

        let log = AuditLog::new(req);
        assert_eq!(log.result, "FAILURE");
    }

    #[test]
    fn test_search_params_default() {
        let params = AuditLogSearchParams::default();
        assert!(params.user_id.is_none());
        assert!(params.event_type.is_none());
        assert!(params.result.is_none());
        assert!(params.from.is_none());
        assert!(params.to.is_none());
        assert_eq!(params.page, 0);
        assert_eq!(params.page_size, 0);
    }
}
