use std::time::Duration;

use chrono::{DateTime, Utc};
use tonic::transport::Channel;
use tracing::instrument;

use crate::domain::model::auth::{
    audit_event_type_to_str, audit_result_to_str, AuditEventType, AuditResult,
};
use crate::domain::model::{AuditLog, PermissionCheck, Role, User};
use crate::infrastructure::config::BackendConfig;
use crate::infrastructure::grpc_retry::with_retry;

#[allow(dead_code)]
pub mod proto {
    // buf generate の compile_well_known_types オプションで生成された .rs ファイルは
    // google::protobuf::Struct を super チェーンで参照する（例: super*4::google::protobuf::Struct）。
    // prost_types クレートが提供する型を google::protobuf 名前空間として再エクスポートし、
    // include_proto! 展開後のパス解決を正常に機能させる。
    pub mod google {
        pub mod protobuf {
            pub use ::prost_types::*;
        }
    }
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
    /// バックエンドサービスのアドレス。gRPC Health Check Protocol のためのチャネル生成に使用する。
    address: String,
    /// タイムアウト設定（ミリ秒）。health_check のチャネル生成にも適用する。
    timeout_ms: u64,
}

impl AuthGrpcClient {
    /// バックエンド設定からクライアントを生成する。
    /// connect_lazy() により起動時の接続確立を不要とし、実際のRPC呼び出し時に接続する。
    pub fn new(cfg: &BackendConfig) -> anyhow::Result<Self> {
        let channel = Channel::from_shared(cfg.address.clone())?
            .timeout(Duration::from_millis(cfg.timeout_ms))
            .connect_lazy();
        Ok(Self {
            auth_client: AuthServiceClient::new(channel.clone()),
            audit_client: AuditServiceClient::new(channel),
            address: cfg.address.clone(),
            timeout_ms: cfg.timeout_ms,
        })
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_user(&self, user_id: &str) -> anyhow::Result<Option<User>> {
        // 一時的エラー（UNAVAILABLE/DEADLINE_EXCEEDED）は最大 3 回リトライする
        let resp = with_retry("AuthService.GetUser", 3, || {
            let mut client = self.auth_client.clone();
            let req = tonic::Request::new(proto::k1s0::system::auth::v1::GetUserRequest {
                user_id: user_id.to_owned(),
            });
            async move { client.get_user(req).await }
        })
        .await;

        match resp {
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
        event_type: AuditEventType,
        user_id: &str,
        ip_address: &str,
        user_agent: &str,
        resource: &str,
        action: &str,
        result: AuditResult,
        resource_id: &str,
        trace_id: &str,
    ) -> anyhow::Result<(String, String)> {
        // HIGH-014 監査対応: deprecated string フィールド削除済み。enum フィールドのみ設定する。
        let request = tonic::Request::new(proto::k1s0::system::auth::v1::RecordAuditLogRequest {
            event_type_enum: domain_event_type_to_proto_i32(event_type),
            user_id: user_id.to_owned(),
            ip_address: ip_address.to_owned(),
            user_agent: user_agent.to_owned(),
            resource: resource.to_owned(),
            action: action.to_owned(),
            result_enum: domain_result_to_proto_i32(result),
            resource_id: resource_id.to_owned(),
            trace_id: trace_id.to_owned(),
            detail: None,
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
        event_type: Option<AuditEventType>,
        result: Option<AuditResult>,
    ) -> anyhow::Result<(Vec<AuditLog>, i64, bool)> {
        // HIGH-014 監査対応: deprecated string フィールド削除済み。enum フィールドのみ設定する。
        let request = tonic::Request::new(proto::k1s0::system::auth::v1::SearchAuditLogsRequest {
            pagination: Some(proto::k1s0::system::common::v1::Pagination {
                page: page.unwrap_or(1),
                page_size: page_size.unwrap_or(50),
            }),
            user_id: user_id.unwrap_or_default().to_owned(),
            event_type_enum: event_type
                .map(domain_event_type_to_proto_i32)
                .unwrap_or(proto::k1s0::system::auth::v1::AuditEventType::Unspecified as i32),
            result_enum: result
                .map(domain_result_to_proto_i32)
                .unwrap_or(proto::k1s0::system::auth::v1::AuditResult::Unspecified as i32),
            from: None,
            to: None,
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

    /// gRPC Health Check Protocol を使ってサービスの疎通確認を行う。
    /// Bearer token なしで接続できるため readyz ヘルスチェックに適している。
    /// tonic-health サービスが登録されているサーバーに対して Check RPC を送信する。
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn health_check(&self) -> anyhow::Result<()> {
        let channel = Channel::from_shared(self.address.clone())?
            .timeout(Duration::from_millis(self.timeout_ms))
            .connect_lazy();
        let mut health_client = tonic_health::pb::health_client::HealthClient::new(channel);
        let request = tonic::Request::new(tonic_health::pb::HealthCheckRequest {
            service: "k1s0.system.auth.v1.AuthService".to_string(),
        });
        health_client
            .check(request)
            .await
            .map_err(|e| anyhow::anyhow!("auth gRPC Health Check 失敗: {}", e))?;
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

// HIGH-014 監査対応: deprecated string フィールド削除済み。enum フィールドから逆変換して後方互換 string を生成する。
fn audit_log_from_proto(l: proto::k1s0::system::auth::v1::AuditLog) -> AuditLog {
    // proto enum i32 → ドメイン enum
    let event_type_enum = proto_i32_to_domain_event_type(l.event_type_enum);
    let result_enum = proto_i32_to_domain_result(l.result_enum);
    // GraphQL deprecated string フィールドは enum から逆変換して後方互換クライアントに提供する
    let event_type_str = event_type_enum
        .map(audit_event_type_to_str)
        .unwrap_or("")
        .to_string();
    let result_str = result_enum
        .map(audit_result_to_str)
        .unwrap_or("")
        .to_string();
    AuditLog {
        id: l.id,
        event_type: event_type_str,
        user_id: l.user_id,
        ip_address: l.ip_address,
        user_agent: l.user_agent,
        resource: l.resource,
        action: l.action,
        result: result_str,
        resource_id: l.resource_id,
        trace_id: l.trace_id,
        created_at: timestamp_to_rfc3339(l.created_at),
        event_type_enum,
        result_enum,
    }
}

fn timestamp_to_rfc3339(ts: Option<proto::k1s0::system::common::v1::Timestamp>) -> String {
    ts.and_then(|ts| DateTime::<Utc>::from_timestamp(ts.seconds, ts.nanos as u32))
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_default()
}

/// ドメイン AuditEventType → proto AuditEventType の i32 値に変換する。
fn domain_event_type_to_proto_i32(e: AuditEventType) -> i32 {
    use proto::k1s0::system::auth::v1::AuditEventType as ProtoAET;
    let proto_enum = match e {
        AuditEventType::Login => ProtoAET::Login,
        AuditEventType::Logout => ProtoAET::Logout,
        AuditEventType::TokenRefresh => ProtoAET::TokenRefresh,
        AuditEventType::PermissionCheck => ProtoAET::PermissionCheck,
        AuditEventType::Create => ProtoAET::Create,
        AuditEventType::Update => ProtoAET::Update,
        AuditEventType::Delete => ProtoAET::Delete,
        AuditEventType::Read => ProtoAET::Read,
        AuditEventType::PermissionDenied => ProtoAET::PermissionDenied,
        AuditEventType::ApiKeyCreated => ProtoAET::ApiKeyCreated,
        AuditEventType::ApiKeyRevoked => ProtoAET::ApiKeyRevoked,
        AuditEventType::SecretAccessed => ProtoAET::SecretAccessed,
        AuditEventType::SecretRotated => ProtoAET::SecretRotated,
    };
    proto_enum as i32
}

/// ドメイン AuditResult → proto AuditResult の i32 値に変換する。
fn domain_result_to_proto_i32(r: AuditResult) -> i32 {
    use proto::k1s0::system::auth::v1::AuditResult as ProtoAR;
    let proto_enum = match r {
        AuditResult::Success => ProtoAR::Success,
        AuditResult::Failure => ProtoAR::Failure,
        AuditResult::Partial => ProtoAR::Partial,
    };
    proto_enum as i32
}

/// proto AuditEventType i32 → ドメイン AuditEventType に変換する。
fn proto_i32_to_domain_event_type(i: i32) -> Option<AuditEventType> {
    use proto::k1s0::system::auth::v1::AuditEventType as ProtoAET;
    match ProtoAET::try_from(i).unwrap_or(ProtoAET::Unspecified) {
        ProtoAET::Unspecified => None,
        ProtoAET::Login => Some(AuditEventType::Login),
        ProtoAET::Logout => Some(AuditEventType::Logout),
        ProtoAET::TokenRefresh => Some(AuditEventType::TokenRefresh),
        ProtoAET::PermissionCheck => Some(AuditEventType::PermissionCheck),
        ProtoAET::ApiKeyCreated => Some(AuditEventType::ApiKeyCreated),
        ProtoAET::ApiKeyRevoked => Some(AuditEventType::ApiKeyRevoked),
        ProtoAET::Create => Some(AuditEventType::Create),
        ProtoAET::Update => Some(AuditEventType::Update),
        ProtoAET::Delete => Some(AuditEventType::Delete),
        ProtoAET::Read => Some(AuditEventType::Read),
        ProtoAET::PermissionDenied => Some(AuditEventType::PermissionDenied),
        ProtoAET::SecretAccessed => Some(AuditEventType::SecretAccessed),
        ProtoAET::SecretRotated => Some(AuditEventType::SecretRotated),
    }
}

/// proto AuditResult i32 → ドメイン AuditResult に変換する。
fn proto_i32_to_domain_result(i: i32) -> Option<AuditResult> {
    use proto::k1s0::system::auth::v1::AuditResult as ProtoAR;
    match ProtoAR::try_from(i).unwrap_or(ProtoAR::Unspecified) {
        ProtoAR::Unspecified => None,
        ProtoAR::Success => Some(AuditResult::Success),
        ProtoAR::Failure => Some(AuditResult::Failure),
        ProtoAR::Partial => Some(AuditResult::Partial),
    }
}
