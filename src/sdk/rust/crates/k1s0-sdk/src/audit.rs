// 本ファイルは k1s0-sdk の Audit 動詞統一 facade。
use crate::client::Client;
use crate::proto::k1s0::tier1::audit::v1::{
    audit_service_client::AuditServiceClient, AuditEvent, QueryAuditRequest, RecordAuditRequest,
};
use prost_types::Timestamp;
use std::collections::HashMap;
use std::time::SystemTime;
use tonic::{transport::Channel, Status};

/// AuditFacade は AuditService の動詞統一 facade。
pub struct AuditFacade {
    client: Client,
    raw: AuditServiceClient<Channel>,
}

impl AuditFacade {
    pub(crate) fn new(client: Client) -> Self {
        let raw = AuditServiceClient::new(client.channel());
        Self { client, raw }
    }

    /// record は監査イベント記録。audit_id を返す。
    pub async fn record(
        &mut self,
        actor: &str,
        action: &str,
        resource: &str,
        outcome: &str,
        attributes: HashMap<String, String>,
    ) -> Result<String, Status> {
        // 現在時刻を Timestamp に変換する。
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();
        let ts = Timestamp { seconds: now.as_secs() as i64, nanos: now.subsec_nanos() as i32 };

        let resp = self.raw.record(RecordAuditRequest {
            event: Some(AuditEvent {
                timestamp: Some(ts),
                actor: actor.to_string(),
                action: action.to_string(),
                resource: resource.to_string(),
                outcome: outcome.to_string(),
                attributes,
            }),
            context: Some(self.client.tenant_context()),
        }).await?.into_inner();
        Ok(resp.audit_id)
    }

    /// query は監査イベント検索（時刻範囲 + filter）。
    pub async fn query(
        &mut self,
        from_secs: i64,
        to_secs: i64,
        filters: HashMap<String, String>,
        limit: i32,
    ) -> Result<Vec<AuditEvent>, Status> {
        let from_ts = Timestamp { seconds: from_secs, nanos: 0 };
        let to_ts = Timestamp { seconds: to_secs, nanos: 0 };
        let resp = self.raw.query(QueryAuditRequest {
            from: Some(from_ts),
            to: Some(to_ts),
            filters,
            limit,
            context: Some(self.client.tenant_context()),
        }).await?.into_inner();
        Ok(resp.events)
    }
}
