use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::entity::audit_log::{AuditLog, CreateAuditLogRequest};
use crate::usecase::record_audit_log::{RecordAuditLogError, RecordAuditLogUseCase};
use crate::usecase::search_audit_logs::{SearchAuditLogsQueryParams, SearchAuditLogsUseCase};

use super::auth_grpc::{GrpcError, PbPagination, PbPaginationResult, PbTimestamp};

// --- gRPC Request/Response Types ---

#[derive(Debug, Clone)]
pub struct RecordAuditLogGrpcRequest {
    pub event_type: String,
    pub user_id: String,
    pub ip_address: String,
    pub user_agent: String,
    pub resource: String,
    pub action: String,
    pub result: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct RecordAuditLogGrpcResponse {
    pub id: String,
    pub recorded_at: Option<PbTimestamp>,
}

#[derive(Debug, Clone)]
pub struct SearchAuditLogsGrpcRequest {
    pub pagination: Option<PbPagination>,
    pub user_id: String,
    pub event_type: String,
    pub from: Option<PbTimestamp>,
    pub to: Option<PbTimestamp>,
    pub result: String,
}

#[derive(Debug, Clone)]
pub struct SearchAuditLogsGrpcResponse {
    pub logs: Vec<PbAuditLog>,
    pub pagination: Option<PbPaginationResult>,
}

#[derive(Debug, Clone)]
pub struct PbAuditLog {
    pub id: String,
    pub event_type: String,
    pub user_id: String,
    pub ip_address: String,
    pub user_agent: String,
    pub resource: String,
    pub action: String,
    pub result: String,
    pub metadata: HashMap<String, String>,
    pub recorded_at: Option<PbTimestamp>,
}

// --- AuditGrpcService ---

pub struct AuditGrpcService {
    record_audit_log_uc: Arc<RecordAuditLogUseCase>,
    search_audit_logs_uc: Arc<SearchAuditLogsUseCase>,
}

impl AuditGrpcService {
    pub fn new(
        record_audit_log_uc: Arc<RecordAuditLogUseCase>,
        search_audit_logs_uc: Arc<SearchAuditLogsUseCase>,
    ) -> Self {
        Self {
            record_audit_log_uc,
            search_audit_logs_uc,
        }
    }

    /// 監査ログエントリを記録する。
    pub async fn record_audit_log(
        &self,
        req: RecordAuditLogGrpcRequest,
    ) -> Result<RecordAuditLogGrpcResponse, GrpcError> {
        if req.event_type.is_empty() {
            return Err(GrpcError::InvalidArgument(
                "event_type is required".to_string(),
            ));
        }

        let create_req = CreateAuditLogRequest {
            event_type: req.event_type,
            user_id: req.user_id,
            ip_address: req.ip_address,
            user_agent: req.user_agent,
            resource: req.resource,
            action: req.action,
            result: req.result,
            metadata: req.metadata,
        };

        match self.record_audit_log_uc.execute(create_req).await {
            Ok(response) => Ok(RecordAuditLogGrpcResponse {
                id: response.id.to_string(),
                recorded_at: Some(PbTimestamp {
                    seconds: response.recorded_at.timestamp(),
                    nanos: response.recorded_at.timestamp_subsec_nanos() as i32,
                }),
            }),
            Err(RecordAuditLogError::Validation(msg)) => {
                Err(GrpcError::InvalidArgument(msg))
            }
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    /// 監査ログを検索する。
    pub async fn search_audit_logs(
        &self,
        req: SearchAuditLogsGrpcRequest,
    ) -> Result<SearchAuditLogsGrpcResponse, GrpcError> {
        let page = req.pagination.as_ref().map(|p| p.page);
        let page_size = req.pagination.as_ref().map(|p| p.page_size);

        let from_str = req.from.as_ref().map(|t| {
            chrono::DateTime::from_timestamp(t.seconds, t.nanos as u32)
                .unwrap_or_default()
                .to_rfc3339()
        });

        let to_str = req.to.as_ref().map(|t| {
            chrono::DateTime::from_timestamp(t.seconds, t.nanos as u32)
                .unwrap_or_default()
                .to_rfc3339()
        });

        let query = SearchAuditLogsQueryParams {
            page,
            page_size,
            user_id: if req.user_id.is_empty() {
                None
            } else {
                Some(req.user_id)
            },
            event_type: if req.event_type.is_empty() {
                None
            } else {
                Some(req.event_type)
            },
            from: from_str,
            to: to_str,
            result: if req.result.is_empty() {
                None
            } else {
                Some(req.result)
            },
        };

        match self.search_audit_logs_uc.execute(&query).await {
            Ok(result) => {
                let pb_logs: Vec<PbAuditLog> = result
                    .logs
                    .iter()
                    .map(domain_audit_log_to_pb)
                    .collect();

                Ok(SearchAuditLogsGrpcResponse {
                    logs: pb_logs,
                    pagination: Some(PbPaginationResult {
                        total_count: result.pagination.total_count,
                        page: result.pagination.page,
                        page_size: result.pagination.page_size,
                        has_next: result.pagination.has_next,
                    }),
                })
            }
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }
}

fn domain_audit_log_to_pb(log: &AuditLog) -> PbAuditLog {
    PbAuditLog {
        id: log.id.to_string(),
        event_type: log.event_type.clone(),
        user_id: log.user_id.clone(),
        ip_address: log.ip_address.clone(),
        user_agent: log.user_agent.clone(),
        resource: log.resource.clone(),
        action: log.action.clone(),
        result: log.result.clone(),
        metadata: log.metadata.clone(),
        recorded_at: Some(PbTimestamp {
            seconds: log.recorded_at.timestamp(),
            nanos: log.recorded_at.timestamp_subsec_nanos() as i32,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::audit_log_repository::MockAuditLogRepository;
    use std::collections::HashMap;

    fn make_audit_service(mock_repo: MockAuditLogRepository) -> AuditGrpcService {
        let repo = Arc::new(mock_repo);
        let record_uc = Arc::new(RecordAuditLogUseCase::new(repo.clone()));
        let search_uc = Arc::new(SearchAuditLogsUseCase::new(repo));
        AuditGrpcService::new(record_uc, search_uc)
    }

    #[tokio::test]
    async fn test_record_audit_log_success() {
        let mut mock_repo = MockAuditLogRepository::new();
        mock_repo.expect_create().returning(|_| Ok(()));

        let svc = make_audit_service(mock_repo);

        let req = RecordAuditLogGrpcRequest {
            event_type: "LOGIN_SUCCESS".to_string(),
            user_id: "user-uuid-1234".to_string(),
            ip_address: "192.168.1.100".to_string(),
            user_agent: "Mozilla/5.0".to_string(),
            resource: "/api/v1/auth/token".to_string(),
            action: "POST".to_string(),
            result: "SUCCESS".to_string(),
            metadata: HashMap::from([("client_id".to_string(), "react-spa".to_string())]),
        };

        let resp = svc.record_audit_log(req).await.unwrap();
        assert!(!resp.id.is_empty());
        assert!(resp.recorded_at.is_some());
    }

    #[tokio::test]
    async fn test_record_audit_log_empty_event_type() {
        let mock_repo = MockAuditLogRepository::new();
        let svc = make_audit_service(mock_repo);

        let req = RecordAuditLogGrpcRequest {
            event_type: String::new(),
            user_id: "user-uuid-1234".to_string(),
            ip_address: String::new(),
            user_agent: String::new(),
            resource: String::new(),
            action: String::new(),
            result: "SUCCESS".to_string(),
            metadata: HashMap::new(),
        };

        let result = svc.record_audit_log(req).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            GrpcError::InvalidArgument(msg) => {
                assert!(msg.contains("event_type is required"));
            }
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_search_audit_logs_with_filters() {
        let mut mock_repo = MockAuditLogRepository::new();
        mock_repo
            .expect_search()
            .withf(|params| {
                params.user_id.as_deref() == Some("user-1")
                    && params.event_type.as_deref() == Some("LOGIN_SUCCESS")
            })
            .returning(|_| Ok((vec![], 0)));

        let svc = make_audit_service(mock_repo);

        let req = SearchAuditLogsGrpcRequest {
            pagination: Some(PbPagination {
                page: 1,
                page_size: 50,
            }),
            user_id: "user-1".to_string(),
            event_type: "LOGIN_SUCCESS".to_string(),
            from: None,
            to: None,
            result: String::new(),
        };

        let resp = svc.search_audit_logs(req).await.unwrap();
        assert!(resp.logs.is_empty());
        let pagination = resp.pagination.unwrap();
        assert_eq!(pagination.total_count, 0);
        assert_eq!(pagination.page, 1);
        assert_eq!(pagination.page_size, 50);
        assert!(!pagination.has_next);
    }

    #[tokio::test]
    async fn test_search_audit_logs_empty() {
        let mut mock_repo = MockAuditLogRepository::new();
        mock_repo
            .expect_search()
            .returning(|_| Ok((vec![], 0)));

        let svc = make_audit_service(mock_repo);

        let req = SearchAuditLogsGrpcRequest {
            pagination: Some(PbPagination {
                page: 1,
                page_size: 50,
            }),
            user_id: String::new(),
            event_type: String::new(),
            from: None,
            to: None,
            result: String::new(),
        };

        let resp = svc.search_audit_logs(req).await.unwrap();
        assert!(resp.logs.is_empty());
        let pagination = resp.pagination.unwrap();
        assert_eq!(pagination.total_count, 0);
        assert!(!pagination.has_next);
    }
}
