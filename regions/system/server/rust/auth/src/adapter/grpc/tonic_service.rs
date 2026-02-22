//! tonic gRPC サービス実装。
//!
//! proto 生成コード (`src/proto/`) の AuthService / AuditService トレイトを実装する。
//! 各メソッドで proto 型 ↔ 手動型の変換を行い、既存の AuthGrpcService / AuditGrpcService に委譲する。

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::auth::v1::{
    audit_service_server::AuditService, auth_service_server::AuthService,
    AuditLog as ProtoAuditLog, CheckPermissionRequest as ProtoCheckPermissionRequest,
    CheckPermissionResponse as ProtoCheckPermissionResponse, ClientRoles as ProtoClientRoles,
    GetUserRequest as ProtoGetUserRequest, GetUserResponse as ProtoGetUserResponse,
    GetUserRolesRequest as ProtoGetUserRolesRequest,
    GetUserRolesResponse as ProtoGetUserRolesResponse, ListUsersRequest as ProtoListUsersRequest,
    ListUsersResponse as ProtoListUsersResponse, RealmAccess as ProtoRealmAccess,
    RecordAuditLogRequest as ProtoRecordAuditLogRequest,
    RecordAuditLogResponse as ProtoRecordAuditLogResponse, Role as ProtoRole,
    RoleList as ProtoRoleList, SearchAuditLogsRequest as ProtoSearchAuditLogsRequest,
    SearchAuditLogsResponse as ProtoSearchAuditLogsResponse, StringList as ProtoStringList,
    TokenClaims as ProtoTokenClaims, User as ProtoUser,
    ValidateTokenRequest as ProtoValidateTokenRequest,
    ValidateTokenResponse as ProtoValidateTokenResponse,
};
use crate::proto::k1s0::system::common::v1::{
    PaginationResult as ProtoPaginationResult, Timestamp as ProtoTimestamp,
};

use super::audit_grpc::{AuditGrpcService, RecordAuditLogGrpcRequest, SearchAuditLogsGrpcRequest};
use super::auth_grpc::{
    AuthGrpcService, CheckPermissionRequest, GetUserRequest, GetUserRolesRequest, GrpcError,
    ListUsersRequest, PbPagination, ValidateTokenRequest,
};

// --- GrpcError -> tonic::Status 変換 ---

impl From<GrpcError> for Status {
    fn from(e: GrpcError) -> Self {
        match e {
            GrpcError::NotFound(msg) => Status::not_found(msg),
            GrpcError::Unauthenticated(msg) => Status::unauthenticated(msg),
            GrpcError::InvalidArgument(msg) => Status::invalid_argument(msg),
            GrpcError::Internal(msg) => Status::internal(msg),
        }
    }
}

// --- 変換ヘルパー ---

fn pb_timestamp_to_proto(ts: &super::auth_grpc::PbTimestamp) -> ProtoTimestamp {
    ProtoTimestamp {
        seconds: ts.seconds,
        nanos: ts.nanos,
    }
}

/// prost_types::Struct を serde_json::Value に変換するヘルパー。
fn prost_struct_to_json(s: prost_types::Struct) -> serde_json::Value {
    let obj: serde_json::Map<String, serde_json::Value> = s
        .fields
        .into_iter()
        .map(|(k, v)| (k, prost_value_to_json(v)))
        .collect();
    serde_json::Value::Object(obj)
}

fn prost_value_to_json(v: prost_types::Value) -> serde_json::Value {
    match v.kind {
        None => serde_json::Value::Null,
        Some(prost_types::value::Kind::NullValue(_)) => serde_json::Value::Null,
        Some(prost_types::value::Kind::BoolValue(b)) => serde_json::Value::Bool(b),
        Some(prost_types::value::Kind::NumberValue(n)) => serde_json::Value::Number(
            serde_json::Number::from_f64(n).unwrap_or(serde_json::Number::from(0)),
        ),
        Some(prost_types::value::Kind::StringValue(s)) => serde_json::Value::String(s),
        Some(prost_types::value::Kind::StructValue(s)) => prost_struct_to_json(s),
        Some(prost_types::value::Kind::ListValue(l)) => {
            serde_json::Value::Array(l.values.into_iter().map(prost_value_to_json).collect())
        }
    }
}

// --- AuthService tonic ラッパー ---

/// AuthServiceTonic は tonic の AuthService として AuthGrpcService をラップする。
pub struct AuthServiceTonic {
    inner: Arc<AuthGrpcService>,
}

impl AuthServiceTonic {
    pub fn new(inner: Arc<AuthGrpcService>) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl AuthService for AuthServiceTonic {
    async fn validate_token(
        &self,
        request: Request<ProtoValidateTokenRequest>,
    ) -> Result<Response<ProtoValidateTokenResponse>, Status> {
        let req = ValidateTokenRequest {
            token: request.into_inner().token,
        };
        let resp = self
            .inner
            .validate_token(req)
            .await
            .map_err(Into::<Status>::into)?;

        let proto_claims = resp.claims.map(|c| ProtoTokenClaims {
            sub: c.sub,
            iss: c.iss,
            aud: c.aud,
            exp: c.exp,
            iat: c.iat,
            jti: c.jti,
            preferred_username: c.preferred_username,
            email: c.email,
            realm_access: c
                .realm_access
                .map(|ra| ProtoRealmAccess { roles: ra.roles }),
            resource_access: c
                .resource_access
                .into_iter()
                .map(|(k, v)| (k, ProtoClientRoles { roles: v.roles }))
                .collect(),
            tier_access: c.tier_access,
        });

        Ok(Response::new(ProtoValidateTokenResponse {
            valid: resp.valid,
            claims: proto_claims,
            error_message: resp.error_message,
        }))
    }

    async fn get_user(
        &self,
        request: Request<ProtoGetUserRequest>,
    ) -> Result<Response<ProtoGetUserResponse>, Status> {
        let req = GetUserRequest {
            user_id: request.into_inner().user_id,
        };
        let resp = self
            .inner
            .get_user(req)
            .await
            .map_err(Into::<Status>::into)?;

        let proto_user = resp.user.map(|u| ProtoUser {
            id: u.id,
            username: u.username,
            email: u.email,
            first_name: u.first_name,
            last_name: u.last_name,
            enabled: u.enabled,
            email_verified: u.email_verified,
            created_at: u.created_at.map(|ts| pb_timestamp_to_proto(&ts)),
            attributes: u
                .attributes
                .into_iter()
                .map(|(k, v)| (k, ProtoStringList { values: v.values }))
                .collect(),
        });

        Ok(Response::new(ProtoGetUserResponse { user: proto_user }))
    }

    async fn list_users(
        &self,
        request: Request<ProtoListUsersRequest>,
    ) -> Result<Response<ProtoListUsersResponse>, Status> {
        let inner = request.into_inner();
        let req = ListUsersRequest {
            pagination: inner.pagination.map(|p| PbPagination {
                page: p.page,
                page_size: p.page_size,
            }),
            search: inner.search,
            enabled: inner.enabled,
        };
        let resp = self
            .inner
            .list_users(req)
            .await
            .map_err(Into::<Status>::into)?;

        let proto_users = resp
            .users
            .into_iter()
            .map(|u| ProtoUser {
                id: u.id,
                username: u.username,
                email: u.email,
                first_name: u.first_name,
                last_name: u.last_name,
                enabled: u.enabled,
                email_verified: u.email_verified,
                created_at: u.created_at.map(|ts| pb_timestamp_to_proto(&ts)),
                attributes: u
                    .attributes
                    .into_iter()
                    .map(|(k, v)| (k, ProtoStringList { values: v.values }))
                    .collect(),
            })
            .collect();

        let proto_pagination = resp.pagination.map(|p| ProtoPaginationResult {
            total_count: p.total_count as i32,
            page: p.page,
            page_size: p.page_size,
            has_next: p.has_next,
        });

        Ok(Response::new(ProtoListUsersResponse {
            users: proto_users,
            pagination: proto_pagination,
        }))
    }

    async fn get_user_roles(
        &self,
        request: Request<ProtoGetUserRolesRequest>,
    ) -> Result<Response<ProtoGetUserRolesResponse>, Status> {
        let req = GetUserRolesRequest {
            user_id: request.into_inner().user_id,
        };
        let resp = self
            .inner
            .get_user_roles(req)
            .await
            .map_err(Into::<Status>::into)?;

        let proto_realm_roles = resp
            .realm_roles
            .into_iter()
            .map(|r| ProtoRole {
                id: r.id,
                name: r.name,
                description: r.description,
            })
            .collect();

        let proto_client_roles = resp
            .client_roles
            .into_iter()
            .map(|(k, v)| {
                let roles = v
                    .roles
                    .into_iter()
                    .map(|r| ProtoRole {
                        id: r.id,
                        name: r.name,
                        description: r.description,
                    })
                    .collect();
                (k, ProtoRoleList { roles })
            })
            .collect();

        Ok(Response::new(ProtoGetUserRolesResponse {
            user_id: resp.user_id,
            realm_roles: proto_realm_roles,
            client_roles: proto_client_roles,
        }))
    }

    async fn check_permission(
        &self,
        request: Request<ProtoCheckPermissionRequest>,
    ) -> Result<Response<ProtoCheckPermissionResponse>, Status> {
        let inner = request.into_inner();
        let req = CheckPermissionRequest {
            user_id: inner.user_id,
            permission: inner.permission,
            resource: inner.resource,
            roles: inner.roles,
        };
        let resp = self
            .inner
            .check_permission(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoCheckPermissionResponse {
            allowed: resp.allowed,
            reason: resp.reason,
        }))
    }
}

// --- AuditService tonic ラッパー ---

/// AuditServiceTonic は tonic の AuditService として AuditGrpcService をラップする。
pub struct AuditServiceTonic {
    inner: Arc<AuditGrpcService>,
}

impl AuditServiceTonic {
    pub fn new(inner: Arc<AuditGrpcService>) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl AuditService for AuditServiceTonic {
    async fn record_audit_log(
        &self,
        request: Request<ProtoRecordAuditLogRequest>,
    ) -> Result<Response<ProtoRecordAuditLogResponse>, Status> {
        let inner = request.into_inner();
        let detail = inner.detail.map(prost_struct_to_json);
        let req = RecordAuditLogGrpcRequest {
            event_type: inner.event_type,
            user_id: inner.user_id,
            ip_address: inner.ip_address,
            user_agent: inner.user_agent,
            resource: inner.resource,
            resource_id: inner.resource_id,
            action: inner.action,
            result: inner.result,
            detail,
            trace_id: inner.trace_id,
        };
        let resp = self
            .inner
            .record_audit_log(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoRecordAuditLogResponse {
            id: resp.id,
            created_at: resp.created_at.map(|ts| pb_timestamp_to_proto(&ts)),
        }))
    }

    async fn search_audit_logs(
        &self,
        request: Request<ProtoSearchAuditLogsRequest>,
    ) -> Result<Response<ProtoSearchAuditLogsResponse>, Status> {
        let inner = request.into_inner();
        let req = SearchAuditLogsGrpcRequest {
            pagination: inner.pagination.map(|p| PbPagination {
                page: p.page,
                page_size: p.page_size,
            }),
            user_id: inner.user_id,
            event_type: inner.event_type,
            from: inner.from.map(|t| super::auth_grpc::PbTimestamp {
                seconds: t.seconds,
                nanos: t.nanos,
            }),
            to: inner.to.map(|t| super::auth_grpc::PbTimestamp {
                seconds: t.seconds,
                nanos: t.nanos,
            }),
            result: inner.result,
        };
        let resp = self
            .inner
            .search_audit_logs(req)
            .await
            .map_err(Into::<Status>::into)?;

        let proto_logs = resp
            .logs
            .into_iter()
            .map(|l| ProtoAuditLog {
                id: l.id,
                event_type: l.event_type,
                user_id: l.user_id,
                ip_address: l.ip_address,
                user_agent: l.user_agent,
                resource: l.resource,
                action: l.action,
                result: l.result,
                detail: None, // serde_json::Value → prost_types::Struct 変換は省略
                created_at: l.created_at.map(|ts| pb_timestamp_to_proto(&ts)),
                resource_id: l.resource_id,
                trace_id: l.trace_id,
            })
            .collect();

        let proto_pagination = resp.pagination.map(|p| ProtoPaginationResult {
            total_count: p.total_count as i32,
            page: p.page,
            page_size: p.page_size,
            has_next: p.has_next,
        });

        Ok(Response::new(ProtoSearchAuditLogsResponse {
            logs: proto_logs,
            pagination: proto_pagination,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::claims::{Claims, RealmAccess};
    use crate::domain::repository::user_repository::MockUserRepository;
    use crate::infrastructure::MockTokenVerifier;
    use crate::usecase::get_user::GetUserUseCase;
    use crate::usecase::get_user_roles::GetUserRolesUseCase;
    use crate::usecase::list_users::ListUsersUseCase;
    use crate::usecase::validate_token::ValidateTokenUseCase;
    use std::collections::HashMap;

    #[test]
    fn test_grpc_error_not_found_to_status() {
        let err = GrpcError::NotFound("user not found: abc".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::NotFound);
        assert!(status.message().contains("user not found: abc"));
    }

    #[test]
    fn test_grpc_error_unauthenticated_to_status() {
        let err = GrpcError::Unauthenticated("invalid token".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::Unauthenticated);
        assert!(status.message().contains("invalid token"));
    }

    #[test]
    fn test_grpc_error_invalid_argument_to_status() {
        let err = GrpcError::InvalidArgument("namespace is required".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::InvalidArgument);
        assert!(status.message().contains("namespace is required"));
    }

    #[test]
    fn test_grpc_error_internal_to_status() {
        let err = GrpcError::Internal("database connection failed".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::Internal);
        assert!(status.message().contains("database connection failed"));
    }

    #[tokio::test]
    async fn test_auth_service_tonic_validate_token() {
        let mut mock_verifier = MockTokenVerifier::new();
        mock_verifier.expect_verify_token().returning(|_| {
            Ok(Claims {
                sub: "tonic-test-user".to_string(),
                iss: "test-issuer".to_string(),
                aud: "test-audience".to_string(),
                exp: chrono::Utc::now().timestamp() + 3600,
                iat: chrono::Utc::now().timestamp(),
                jti: "test-jti".to_string(),
                typ: "Bearer".to_string(),
                azp: "test-client".to_string(),
                scope: "openid".to_string(),
                preferred_username: "tonic.user".to_string(),
                email: "tonic@test.com".to_string(),
                realm_access: RealmAccess {
                    roles: vec!["user".to_string()],
                },
                resource_access: HashMap::new(),
                tier_access: vec!["system".to_string()],
            })
        });

        let validate_uc = Arc::new(ValidateTokenUseCase::new(
            Arc::new(mock_verifier),
            "test-issuer".to_string(),
            "test-audience".to_string(),
        ));
        let user_repo = Arc::new(MockUserRepository::new());
        let get_user_uc = Arc::new(GetUserUseCase::new(user_repo.clone()));
        let get_user_roles_uc = Arc::new(GetUserRolesUseCase::new(user_repo.clone()));
        let list_users_uc = Arc::new(ListUsersUseCase::new(user_repo));

        let auth_svc = Arc::new(AuthGrpcService::new(
            validate_uc,
            get_user_uc,
            get_user_roles_uc,
            list_users_uc,
        ));
        let tonic_svc = AuthServiceTonic::new(auth_svc);

        let req = Request::new(ProtoValidateTokenRequest {
            token: "test-token".to_string(),
        });
        let resp = tonic_svc.validate_token(req).await.unwrap();
        let inner = resp.into_inner();

        assert!(inner.valid);
        let claims = inner.claims.unwrap();
        assert_eq!(claims.sub, "tonic-test-user");
        assert_eq!(claims.preferred_username, "tonic.user");
    }

    #[tokio::test]
    async fn test_auth_service_tonic_check_permission() {
        use crate::proto::k1s0::system::auth::v1::CheckPermissionRequest as ProtoReq;

        let mock_verifier = MockTokenVerifier::new();
        let validate_uc = Arc::new(ValidateTokenUseCase::new(
            Arc::new(mock_verifier),
            "test-issuer".to_string(),
            "test-audience".to_string(),
        ));
        let user_repo = Arc::new(MockUserRepository::new());
        let get_user_uc = Arc::new(GetUserUseCase::new(user_repo.clone()));
        let get_user_roles_uc = Arc::new(GetUserRolesUseCase::new(user_repo.clone()));
        let list_users_uc = Arc::new(ListUsersUseCase::new(user_repo));

        let auth_svc = Arc::new(AuthGrpcService::new(
            validate_uc,
            get_user_uc,
            get_user_roles_uc,
            list_users_uc,
        ));
        let tonic_svc = AuthServiceTonic::new(auth_svc);

        // sys_admin should be allowed
        let req = Request::new(ProtoReq {
            user_id: "user-1".to_string(),
            permission: "admin".to_string(),
            resource: "configs".to_string(),
            roles: vec!["sys_admin".to_string()],
        });
        let resp = tonic_svc.check_permission(req).await.unwrap();
        assert!(resp.into_inner().allowed);

        // regular user should be denied
        let req = Request::new(ProtoReq {
            user_id: "user-1".to_string(),
            permission: "admin".to_string(),
            resource: "configs".to_string(),
            roles: vec!["user".to_string()],
        });
        let resp = tonic_svc.check_permission(req).await.unwrap();
        let inner = resp.into_inner();
        assert!(!inner.allowed);
        assert!(inner.reason.contains("insufficient permissions"));
    }
}
