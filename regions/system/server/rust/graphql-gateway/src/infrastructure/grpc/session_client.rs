use std::time::Duration;

use chrono::{DateTime, Utc};
use tonic::transport::Channel;
use tracing::instrument;

use crate::domain::model::{Session, SessionStatus};
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
            pub mod session {
                pub mod v1 {
                    tonic::include_proto!("k1s0.system.session.v1");
                }
            }
        }
    }
}

use proto::k1s0::system::session::v1::session_service_client::SessionServiceClient;

pub struct SessionGrpcClient {
    client: SessionServiceClient<Channel>,
}

impl SessionGrpcClient {
    /// バックエンド設定からクライアントを生成する。
    /// connect_lazy() により起動時の接続確立を不要とし、実際のRPC呼び出し時に接続する。
    pub fn new(cfg: &BackendConfig) -> anyhow::Result<Self> {
        let channel = Channel::from_shared(cfg.address.clone())?
            .timeout(Duration::from_millis(cfg.timeout_ms))
            .connect_lazy();
        Ok(Self {
            client: SessionServiceClient::new(channel),
        })
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_session(&self, session_id: &str) -> anyhow::Result<Option<Session>> {
        let request = tonic::Request::new(proto::k1s0::system::session::v1::GetSessionRequest {
            session_id: session_id.to_owned(),
        });

        match self.client.clone().get_session(request).await {
            Ok(resp) => {
                let s = match resp.into_inner().session {
                    Some(s) => s,
                    None => return Ok(None),
                };
                Ok(Some(session_from_proto(s)))
            }
            Err(status) if status.code() == tonic::Code::NotFound => Ok(None),
            Err(e) => Err(anyhow::anyhow!("SessionService.GetSession failed: {}", e)),
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_user_sessions(&self, user_id: &str) -> anyhow::Result<Vec<Session>> {
        let request =
            tonic::Request::new(proto::k1s0::system::session::v1::ListUserSessionsRequest {
                user_id: user_id.to_owned(),
            });

        let resp = self
            .client
            .clone()
            .list_user_sessions(request)
            .await
            .map_err(|e| anyhow::anyhow!("SessionService.ListUserSessions failed: {}", e))?
            .into_inner();

        Ok(resp.sessions.into_iter().map(session_from_proto).collect())
    }

    #[allow(clippy::too_many_arguments)]
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn create_session(
        &self,
        user_id: &str,
        device_id: &str,
        device_name: Option<&str>,
        device_type: Option<&str>,
        user_agent: Option<&str>,
        ip_address: Option<&str>,
        ttl_seconds: Option<i32>,
    ) -> anyhow::Result<(Session, String)> {
        let request = tonic::Request::new(proto::k1s0::system::session::v1::CreateSessionRequest {
            user_id: user_id.to_owned(),
            device_id: device_id.to_owned(),
            device_name: device_name.map(|s| s.to_owned()),
            device_type: device_type.map(|s| s.to_owned()),
            user_agent: user_agent.map(|s| s.to_owned()),
            ip_address: ip_address.map(|s| s.to_owned()),
            ttl_seconds: ttl_seconds.map(|t| t as u32),
            max_devices: None,
            metadata: Default::default(),
        });

        let resp = self
            .client
            .clone()
            .create_session(request)
            .await
            .map_err(|e| anyhow::anyhow!("SessionService.CreateSession failed: {}", e))?
            .into_inner();

        let token = resp.token.clone();
        let session = Session {
            session_id: resp.session_id,
            user_id: resp.user_id,
            device_id: resp.device_id,
            device_name: resp.device_name.filter(|s| !s.is_empty()),
            device_type: resp.device_type.filter(|s| !s.is_empty()),
            user_agent: resp.user_agent.filter(|s| !s.is_empty()),
            ip_address: resp.ip_address.filter(|s| !s.is_empty()),
            // proto の i32 ステータス値をドメインモデルの SessionStatus に変換する
            status: proto_status_to_domain_str(&resp.status),
            expires_at: timestamp_to_rfc3339(resp.expires_at),
            created_at: timestamp_to_rfc3339(resp.created_at),
            last_accessed_at: None,
        };

        Ok((session, token))
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn refresh_session(
        &self,
        session_id: &str,
        ttl_seconds: Option<i32>,
    ) -> anyhow::Result<(Session, String)> {
        let request =
            tonic::Request::new(proto::k1s0::system::session::v1::RefreshSessionRequest {
                session_id: session_id.to_owned(),
                ttl_seconds: ttl_seconds.map(|t| t as u32),
            });

        let resp = self
            .client
            .clone()
            .refresh_session(request)
            .await
            .map_err(|e| anyhow::anyhow!("SessionService.RefreshSession failed: {}", e))?
            .into_inner();

        let token = resp.token.clone();
        let session = Session {
            session_id: resp.session_id,
            user_id: resp.user_id,
            device_id: resp.device_id,
            device_name: resp.device_name.filter(|s| !s.is_empty()),
            device_type: resp.device_type.filter(|s| !s.is_empty()),
            user_agent: resp.user_agent.filter(|s| !s.is_empty()),
            ip_address: resp.ip_address.filter(|s| !s.is_empty()),
            // proto の i32 ステータス値をドメインモデルの SessionStatus に変換する
            status: proto_status_to_domain_str(&resp.status),
            expires_at: timestamp_to_rfc3339(resp.expires_at),
            created_at: timestamp_to_rfc3339(resp.created_at),
            last_accessed_at: resp
                .last_accessed_at
                .map(|ts| timestamp_to_rfc3339(Some(ts))),
        };

        Ok((session, token))
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn revoke_session(&self, session_id: &str) -> anyhow::Result<bool> {
        let request = tonic::Request::new(proto::k1s0::system::session::v1::RevokeSessionRequest {
            session_id: session_id.to_owned(),
        });

        let resp = self
            .client
            .clone()
            .revoke_session(request)
            .await
            .map_err(|e| anyhow::anyhow!("SessionService.RevokeSession failed: {}", e))?
            .into_inner();

        Ok(resp.success)
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn revoke_all_sessions(&self, user_id: &str) -> anyhow::Result<u32> {
        let request =
            tonic::Request::new(proto::k1s0::system::session::v1::RevokeAllSessionsRequest {
                user_id: user_id.to_owned(),
            });

        let resp = self
            .client
            .clone()
            .revoke_all_sessions(request)
            .await
            .map_err(|e| anyhow::anyhow!("SessionService.RevokeAllSessions failed: {}", e))?
            .into_inner();

        Ok(resp.revoked_count)
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn health_check(&self) -> anyhow::Result<()> {
        self.list_user_sessions("__readyz__").await?;
        Ok(())
    }
}

/// proto の SessionStatus 値をドメインモデルの SessionStatus に変換するヘルパー関数。
/// 生成コードバージョンに依存しない型変換を提供する。
/// proto v1 (i32): 1 = Active、2 = Revoked
/// proto v0 (String): "SESSION_STATUS_ACTIVE" = Active、それ以外は Active
fn proto_status_to_domain_str(v: &str) -> SessionStatus {
    match v {
        "SESSION_STATUS_REVOKED" => SessionStatus::Revoked,
        _ => SessionStatus::Active,
    }
}

fn session_from_proto(s: proto::k1s0::system::session::v1::Session) -> Session {
    // status フィールドは proto 生成コードのバージョンによって String 型になる場合がある。
    // SessionStatus enum は文字列表現からドメインモデルに変換する。
    let status = proto_status_to_domain_str(&s.status);
    Session {
        session_id: s.session_id,
        user_id: s.user_id,
        device_id: s.device_id,
        device_name: s.device_name.filter(|v| !v.is_empty()),
        device_type: s.device_type.filter(|v| !v.is_empty()),
        user_agent: s.user_agent.filter(|v| !v.is_empty()),
        ip_address: s.ip_address.filter(|v| !v.is_empty()),
        // proto から変換した SessionStatus を使用する
        status,
        expires_at: timestamp_to_rfc3339(s.expires_at),
        created_at: timestamp_to_rfc3339(s.created_at),
        last_accessed_at: s.last_accessed_at.map(|ts| {
            DateTime::<Utc>::from_timestamp(ts.seconds, ts.nanos as u32)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_default()
        }),
    }
}

fn timestamp_to_rfc3339(ts: Option<proto::k1s0::system::common::v1::Timestamp>) -> String {
    ts.and_then(|ts| DateTime::<Utc>::from_timestamp(ts.seconds, ts.nanos as u32))
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_default()
}
