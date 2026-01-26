//! gRPCサービス実装

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use tonic::{Request, Response, Status};

use crate::application::AuthService;
use crate::domain::{PermissionRepository, RoleRepository, TokenRepository, UserRepository};

// Generated code from proto
pub mod auth_v1 {
    tonic::include_proto!("k1s0.auth.v1");
}

use auth_v1::auth_service_server::AuthService as AuthServiceTrait;
use auth_v1::{
    AuthenticateRequest, AuthenticateResponse, CheckPermissionRequest, CheckPermissionResponse,
    GetUserRequest, GetUserResponse, ListUserRolesRequest, ListUserRolesResponse,
    RefreshTokenRequest, RefreshTokenResponse, Role as ProtoRole, User as ProtoUser,
};

/// gRPCサービス実装
pub struct GrpcAuthService<U, R, P, T>
where
    U: UserRepository,
    R: RoleRepository,
    P: PermissionRepository,
    T: TokenRepository,
{
    service: Arc<AuthService<U, R, P, T>>,
}

impl<U, R, P, T> GrpcAuthService<U, R, P, T>
where
    U: UserRepository,
    R: RoleRepository,
    P: PermissionRepository,
    T: TokenRepository,
{
    /// 新しいgRPCサービスを作成
    pub fn new(service: Arc<AuthService<U, R, P, T>>) -> Self {
        Self { service }
    }
}

fn system_time_to_rfc3339(time: SystemTime) -> String {
    let duration = time.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();
    let datetime = chrono::DateTime::from_timestamp(secs as i64, 0)
        .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap());
    datetime.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn option_system_time_to_rfc3339(time: Option<SystemTime>) -> String {
    time.map(system_time_to_rfc3339).unwrap_or_default()
}

fn user_to_proto(user: crate::domain::User) -> ProtoUser {
    ProtoUser {
        user_id: user.user_id,
        login_id: user.login_id,
        email: user.email,
        display_name: user.display_name,
        status: user.status.to_i32(),
        last_login_at: option_system_time_to_rfc3339(user.last_login_at),
        created_at: system_time_to_rfc3339(user.created_at),
        updated_at: system_time_to_rfc3339(user.updated_at),
    }
}

fn role_to_proto(role: crate::domain::Role) -> ProtoRole {
    ProtoRole {
        role_id: role.role_id,
        role_name: role.role_name,
        description: role.description,
    }
}

#[tonic::async_trait]
impl<U, R, P, T> AuthServiceTrait for GrpcAuthService<U, R, P, T>
where
    U: UserRepository,
    R: RoleRepository,
    P: PermissionRepository,
    T: TokenRepository,
{
    async fn authenticate(
        &self,
        request: Request<AuthenticateRequest>,
    ) -> Result<Response<AuthenticateResponse>, Status> {
        let req = request.into_inner();

        let token = self
            .service
            .authenticate(&req.login_id, &req.password)
            .await
            .map_err(|e| Status::new(e.to_grpc_code().into(), e.to_string()))?;

        Ok(Response::new(AuthenticateResponse {
            access_token: token.access_token,
            refresh_token: token.refresh_token,
            expires_in: token.expires_in,
            token_type: token.token_type,
        }))
    }

    async fn refresh_token(
        &self,
        request: Request<RefreshTokenRequest>,
    ) -> Result<Response<RefreshTokenResponse>, Status> {
        let req = request.into_inner();

        let token = self
            .service
            .refresh_token(&req.refresh_token)
            .await
            .map_err(|e| Status::new(e.to_grpc_code().into(), e.to_string()))?;

        Ok(Response::new(RefreshTokenResponse {
            access_token: token.access_token,
            refresh_token: token.refresh_token,
            expires_in: token.expires_in,
        }))
    }

    async fn check_permission(
        &self,
        request: Request<CheckPermissionRequest>,
    ) -> Result<Response<CheckPermissionResponse>, Status> {
        let req = request.into_inner();

        let service_name = if req.service_name.is_empty() {
            None
        } else {
            Some(req.service_name.as_str())
        };

        let allowed = self
            .service
            .check_permission(req.user_id, &req.permission_key, service_name)
            .await
            .map_err(|e| Status::new(e.to_grpc_code().into(), e.to_string()))?;

        Ok(Response::new(CheckPermissionResponse { allowed }))
    }

    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<GetUserResponse>, Status> {
        let req = request.into_inner();

        let user = self
            .service
            .get_user(req.user_id)
            .await
            .map_err(|e| Status::new(e.to_grpc_code().into(), e.to_string()))?;

        Ok(Response::new(GetUserResponse {
            user: Some(user_to_proto(user)),
        }))
    }

    async fn list_user_roles(
        &self,
        request: Request<ListUserRolesRequest>,
    ) -> Result<Response<ListUserRolesResponse>, Status> {
        let req = request.into_inner();

        let roles = self
            .service
            .list_user_roles(req.user_id)
            .await
            .map_err(|e| Status::new(e.to_grpc_code().into(), e.to_string()))?;

        Ok(Response::new(ListUserRolesResponse {
            roles: roles.into_iter().map(role_to_proto).collect(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_time_to_rfc3339() {
        let time = SystemTime::UNIX_EPOCH;
        let result = system_time_to_rfc3339(time);
        assert_eq!(result, "1970-01-01T00:00:00Z");
    }

    #[test]
    fn test_option_system_time_to_rfc3339_none() {
        let result = option_system_time_to_rfc3339(None);
        assert_eq!(result, "");
    }
}
