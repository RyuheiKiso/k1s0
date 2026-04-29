// 本ファイルは k1s0-sdk の Audit 動詞統一 facade。
use crate::client::Client;
use crate::proto::k1s0::tier1::audit::v1::{
    AuditEvent, QueryAuditRequest, RecordAuditRequest, VerifyChainRequest,
    audit_service_client::AuditServiceClient,
};
use prost_types::Timestamp;
use std::collections::HashMap;
use std::time::SystemTime;
use tonic::{Status, transport::Channel};

/// AuditFacade は AuditService の動詞統一 facade。
pub struct AuditFacade {
    client: Client,
    raw: AuditServiceClient<Channel>,
}

/// VerifyChain（FR-T1-AUDIT-002）の応答を SDK 利用者向けに整理した型。
#[derive(Debug, Clone)]
pub struct VerifyChainResult {
    /// チェーン整合性が取れていれば true。
    pub valid: bool,
    /// 検証対象だったイベント件数。
    pub checked_count: i64,
    /// 不整合検出時、最初に失敗した sequence_number（1-based）。Valid 時は 0。
    pub first_bad_sequence: i64,
    /// 不整合の理由。Valid 時は空文字。
    pub reason: String,
}

impl AuditFacade {
    pub(crate) fn new(client: Client) -> Self {
        let raw = AuditServiceClient::new(client.channel());
        Self { client, raw }
    }

    /// record は監査イベント記録。audit_id を返す。
    /// idempotency_key が空でなければ tier1 が 24h dedup（hash chain 二重追記防止）する。
    pub async fn record(
        &mut self,
        actor: &str,
        action: &str,
        resource: &str,
        outcome: &str,
        attributes: HashMap<String, String>,
        idempotency_key: &str,
    ) -> Result<String, Status> {
        // 現在時刻を Timestamp に変換する。
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();
        let ts = Timestamp {
            seconds: now.as_secs() as i64,
            nanos: now.subsec_nanos() as i32,
        };

        let resp = self
            .raw
            .record(RecordAuditRequest {
                event: Some(AuditEvent {
                    timestamp: Some(ts),
                    actor: actor.to_string(),
                    action: action.to_string(),
                    resource: resource.to_string(),
                    outcome: outcome.to_string(),
                    attributes,
                }),
                idempotency_key: idempotency_key.to_string(),
                context: Some(self.client.tenant_context()),
            })
            .await?
            .into_inner();
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
        let from_ts = Timestamp {
            seconds: from_secs,
            nanos: 0,
        };
        let to_ts = Timestamp {
            seconds: to_secs,
            nanos: 0,
        };
        let resp = self
            .raw
            .query(QueryAuditRequest {
                from: Some(from_ts),
                to: Some(to_ts),
                filters,
                limit,
                context: Some(self.client.tenant_context()),
            })
            .await?
            .into_inner();
        Ok(resp.events)
    }

    /// verify_chain は監査ハッシュチェーンの整合性を検証する（FR-T1-AUDIT-002）。
    /// from_secs / to_secs に 0 を渡すと全範囲（gRPC 側 zero Timestamp 扱い）。
    pub async fn verify_chain(
        &mut self,
        from_secs: i64,
        to_secs: i64,
    ) -> Result<VerifyChainResult, Status> {
        // 0 を None として送ると tier1 側で「未指定」扱いになる。
        let from = if from_secs > 0 {
            Some(Timestamp {
                seconds: from_secs,
                nanos: 0,
            })
        } else {
            None
        };
        let to = if to_secs > 0 {
            Some(Timestamp {
                seconds: to_secs,
                nanos: 0,
            })
        } else {
            None
        };
        // RPC 呼出。
        let resp = self
            .raw
            .verify_chain(VerifyChainRequest {
                from,
                to,
                context: Some(self.client.tenant_context()),
            })
            .await?
            .into_inner();
        // SDK 型に詰め替える。
        Ok(VerifyChainResult {
            valid: resp.valid,
            checked_count: resp.checked_count,
            first_bad_sequence: resp.first_bad_sequence,
            reason: resp.reason,
        })
    }
}
