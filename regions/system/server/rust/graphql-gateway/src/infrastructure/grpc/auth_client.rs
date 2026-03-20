use std::time::Duration;

use chrono::{DateTime, Utc};
use tonic::transport::Channel;
use tracing::instrument;

use crate::domain::model::{AuditLog, PermissionCheck, Role, User};
use crate::infrastructure::config::BackendConfig;

#[allow(dead_code)]
pub mod proto {
    pub mod k1s0 {
        pub mod system {
            pub mod common {
                pub mod v1 {
                    tonic::include_proto!("k1s0.system.common.v1");
                }
            }
            pub mod auth {
                pub mod v1 {
                    tonic::include_proto!("k1s0.system.auth.v1");
                }
            }
        }
    }
}

use proto::k1s0::system::auth::v1::audit_service_client::AuditServiceClient;
use proto::k1s0::system::auth::v1::auth_service_client::AuthServiceClient;

pub struct AuthGrpcClient {
    auth_client: AuthServiceClient<Channel>,
    audit_client: AuditServiceClient<Channel>,
}

impl AuthGrpcClient {
    pub async fn connect(cfg: &BackendConfig) -> anyhow::Result<Self> {
        let channel = Channel::from_shared(cfg.address.clone())?
            .timeout(Duration::from_millis(cfg.timeout_ms))
            .connect()
            .await?;
        Ok(Self {
            auth_client: AuthServiceClient::new(channel.clone()),
            audit_client: AuditServiceClient::new(channel),
        })
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_user(&self, user_id: &str) -> anyhow::Result<Option<User>> {
        let request = tonic::Request::new(proto::k1s0::system::auth::v1::GetUserRequest {
            user_id: user_id.to_owned(),
        });

        match self.auth_client.clone().get_user(request).await {
            Ok(resp) => {
                let u = match resp.into_inner().user {
                    Some(u) => u,
                    None => return Ok(None),
                };
                Ok(Some(user_from_proto(u)))
            }
            Err(status) if status.code() == tonic::Code::NotFound => Ok(None),
            Err(e) => Err(anyhow::anyhow!("AuthService.GetUser failed: {}", e)),
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_users(
        &self,
        page_size: Option<i32>,
        page: Option<i32>,
        search: Option<&str>,
        enabled: Option<bool>,
    ) -> anyhow::Result<Vec<User>> {
        let request = tonic::Request::new(proto::k1s0::system::auth::v1::ListUsersRequest {
            pagination: Some(proto::k1s0::system::common::v1::Pagination {
                page: page.unwrap_or(1),
                page_size: page_size.unwrap_or(50),
            }),
            search: search.unwrap_or_default().to_owned(),
            enabled,
        });

        let resp = self
            .auth_client
            .clone()
            .list_users(request)
            .await
            .map_err(|e| anyhow::anyhow!("AuthService.ListUsers failed: {}", e))?
            .into_inner();

        Ok(resp.users.into_iter().map(user_from_proto).collect())
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_user_roles(&self, user_id: &str) -> anyhow::Result<Vec<Role>> {
        let request = tonic::Request::new(proto::k1s0::system::auth::v1::GetUserRolesRequest {
            user_id: user_id.to_owned(),
        });

        let resp = self
            .auth_client
            .clone()
            .get_user_roles(request)
            .await
            .map_err(|e| anyhow::anyhow!("AuthService.GetUserRoles failed: {}", e))?
            .into_inner();

        Ok(resp.realm_roles.into_iter().map(role_from_proto).collect())
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn check_permission(
        &self,
        user_id: Option<&str>,
        permission: &str,
        resource: &str,
        roles: &[String],
    ) -> anyhow::Result<PermissionCheck> {
        let request = tonic::Request::new(proto::k1s0::system::auth::v1::CheckPermissionRequest {
            user_id: user_id.map(|s| s.to_owned()),
            permission: permission.to_owned(),
            resource: resource.to_owned(),
            roles: roles.to_vec(),
        });

        let resp = self
            .auth_client
            .clone()
            .check_permission(request)
            .await
            .map_err(|e| anyhow::anyhow!("AuthService.CheckPermission failed: {}", e))?
            .into_inner();

        Ok(PermissionCheck {
            allowed: resp.allowed,
            reason: resp.reason,
        })
    }

    #[allow(clippy::too_many_arguments)]
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn record_audit_log(
        &self,
        event_type: &str,
        user_id: &str,
        ip_address: &str,
        user_agent: &str,
        resource: &str,
        action: &str,
        result: &str,
        resource_id: &str,
        trace_id: &str,
    ) -> anyhow::Result<(String, String)> {
        let request = tonic::Request::new(proto::k1s0::system::auth::v1::RecordAuditLogRequest {
            event_type: event_type.to_owned(),
            user_id: user_id.to_owned(),
            ip_address: ip_address.to_owned(),
            user_agent: user_agent.to_owned(),
            resource: resource.to_owned(),
            action: action.to_owned(),
            result: result.to_owned(),
            resource_id: resource_id.to_owned(),
            trace_id: trace_id.to_owned(),
            detail: None,
            // 後方互換フィールド（0 = UNSPECIFIED）
            event_type_enum: 0,
            result_enum: 0,
        });

        let resp = self
            .audit_client
            .clone()
            .record_audit_log(request)
            .await
            .map_err(|e| anyhow::anyhow!("AuditService.RecordAuditLog failed: {}", e))?
            .into_inner();

        Ok((resp.id, timestamp_to_rfc3339(resp.created_at)))
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn search_audit_logs(
        &self,
        page_size: Option<i32>,
        page: Option<i32>,
        user_id: Option<&str>,
        event_type: Option<&str>,
        result: Option<&str>,
    ) -> anyhow::Result<(Vec<AuditLog>, i64, bool)> {
        let request = tonic::Request::new(proto::k1s0::system::auth::v1::SearchAuditLogsRequest {
            pagination: Some(proto::k1s0::system::common::v1::Pagination {
                page: page.unwrap_or(1),
                page_size: page_size.unwrap_or(50),
            }),
            user_id: user_id.unwrap_or_default().to_owned(),
            event_type: event_type.unwrap_or_default().to_owned(),
            result: result.unwrap_or_default().to_owned(),
            from: None,
            to: None,
            // 後方互換フィールド（0 = UNSPECIFIED はフィルタなし）
            event_type_enum: 0,
            result_enum: 0,
        });

        let resp = self
            .audit_client
            .clone()
            .search_audit_logs(request)
            .await
            .map_err(|e| anyhow::anyhow!("AuditService.SearchAuditLogs failed: {}", e))?
            .into_inner();

        let logs = resp.logs.into_iter().map(audit_log_from_proto).collect();

        let (total_count, has_next) = resp
            .pagination
            .map(|p| (p.total_count, p.has_next))
            .unwrap_or((0, false));

        Ok((logs, total_count, has_next))
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn health_check(&self) -> anyhow::Result<()> {
        self.list_users(Some(1), Some(1), None, None).await?;
        Ok(())
    }
}

fn user_from_proto(u: proto::k1s0::system::auth::v1::User) -> User {
    User {
        id: u.id,
        username: u.username,
        email: u.email,
        first_name: u.first_name,
        last_name: u.last_name,
        enabled: u.enabled,
        email_verified: u.email_verified,
        created_at: timestamp_to_rfc3339(u.created_at),
    }
}

fn role_from_proto(r: proto::k1s0::system::auth::v1::Role) -> Role {
    Role {
        id: r.id,
        name: r.name,
        description: r.description,
    }
}

fn audit_log_from_proto(l: proto::k1s0::system::auth::v1::AuditLog) -> AuditLog {
    AuditLog {
        id: l.id,
        event_type: l.event_type,
        user_id: l.user_id,
        ip_address: l.ip_address,
        user_agent: l.user_agent,
        resource: l.resource,
        action: l.action,
        result: l.result,
        resource_id: l.resource_id,
        trace_id: l.trace_id,
        created_at: timestamp_to_rfc3339(l.created_at),
    }
}

fn timestamp_to_rfc3339(ts: Option<proto::k1s0::system::common::v1::Timestamp>) -> String {
    ts.and_then(|ts| DateTime::<Utc>::from_timestamp(ts.seconds, ts.nanos as u32))
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_default()
}
