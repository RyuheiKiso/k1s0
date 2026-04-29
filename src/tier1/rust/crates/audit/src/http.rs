// 本ファイルは t1-audit Pod の HTTP/JSON gateway 用 JsonRpc 実装。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/00_tier1_API共通規約.md
//     §「HTTP/JSON 互換インタフェース共通仕様」
//
// 役割:
//   AuditService の Record / Query / VerifyChain 3 RPC を HTTP/JSON 経路で
//   公開する。Export は server-streaming のため HTTP/JSON 単発応答に収まらず
//   非対応（gRPC 経路を使う運用、Go 側 HTTP gateway と同じ方針）。
//
//   gRPC handler ロジック（AuditService impl）と同じ store / idempotency cache を
//   共有するため、AuditServer を Arc で渡して in-process で呼ぶ。

// 共通 gateway。
use k1s0_tier1_common::auth::AuthClaims;
use k1s0_tier1_common::http_gateway::JsonRpc;
// JSON 値型。
use serde_json::Value as JsonValue;
// SDK proto 型。
use k1s0_sdk_proto::k1s0::tier1::audit::v1::{
    AuditEvent, QueryAuditRequest, RecordAuditRequest, VerifyChainRequest,
    audit_service_server::AuditService,
};
use k1s0_sdk_proto::k1s0::tier1::common::v1::TenantContext;
// AuditServer 実装。
use crate::server::AuditServer;
// 標準。
use std::collections::HashMap;
use std::sync::Arc;

/// Audit HTTP gateway 用に共有する AuditServer 参照。
#[derive(Clone)]
pub struct AuditHttpState {
    /// in-process gRPC handler を保持する。
    pub server: Arc<AuditServer>,
}

/// `Audit.Record` adapter。
pub struct RecordRpc {
    pub state: AuditHttpState,
}

#[async_trait::async_trait]
impl JsonRpc for RecordRpc {
    fn route(&self) -> &'static str {
        "audit/record"
    }
    fn full_method(&self) -> &'static str {
        "/k1s0.tier1.audit.v1.AuditService/Record"
    }
    async fn invoke(
        &self,
        claims: &AuthClaims,
        body: JsonValue,
    ) -> Result<JsonValue, tonic::Status> {
        // tenant_id は Auth claims を信頼する（HTTP body 由来は spoof 可能）。
        let ctx = tenant_ctx_for(claims);
        let event_json = body.get("event").cloned().unwrap_or(JsonValue::Null);
        let event = parse_audit_event(&event_json)?;
        let idempotency_key = body
            .get("idempotencyKey")
            .or_else(|| body.get("idempotency_key"))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let req = RecordAuditRequest {
            event: Some(event),
            context: Some(ctx),
            idempotency_key,
        };
        let resp = self
            .state
            .server
            .record(tonic::Request::new(req))
            .await?
            .into_inner();
        Ok(serde_json::json!({ "auditId": resp.audit_id }))
    }
}

/// `Audit.Query` adapter。
pub struct QueryRpc {
    pub state: AuditHttpState,
}

#[async_trait::async_trait]
impl JsonRpc for QueryRpc {
    fn route(&self) -> &'static str {
        "audit/query"
    }
    fn full_method(&self) -> &'static str {
        "/k1s0.tier1.audit.v1.AuditService/Query"
    }
    async fn invoke(
        &self,
        claims: &AuthClaims,
        body: JsonValue,
    ) -> Result<JsonValue, tonic::Status> {
        let ctx = tenant_ctx_for(claims);
        let from = parse_timestamp(body.get("from"));
        let to = parse_timestamp(body.get("to"));
        let filters = parse_string_map(body.get("filters"));
        let limit = body.get("limit").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let req = QueryAuditRequest {
            from,
            to,
            filters,
            limit,
            context: Some(ctx),
        };
        let resp = self
            .state
            .server
            .query(tonic::Request::new(req))
            .await?
            .into_inner();
        let events: Vec<JsonValue> = resp.events.iter().map(audit_event_to_json).collect();
        Ok(serde_json::json!({ "events": events }))
    }
}

/// `Audit.VerifyChain` adapter。
pub struct VerifyChainRpc {
    pub state: AuditHttpState,
}

#[async_trait::async_trait]
impl JsonRpc for VerifyChainRpc {
    fn route(&self) -> &'static str {
        "audit/verifychain"
    }
    fn full_method(&self) -> &'static str {
        "/k1s0.tier1.audit.v1.AuditService/VerifyChain"
    }
    async fn invoke(
        &self,
        claims: &AuthClaims,
        body: JsonValue,
    ) -> Result<JsonValue, tonic::Status> {
        let ctx = tenant_ctx_for(claims);
        let from = parse_timestamp(body.get("from"));
        let to = parse_timestamp(body.get("to"));
        let req = VerifyChainRequest {
            from,
            to,
            context: Some(ctx),
        };
        let resp = self
            .state
            .server
            .verify_chain(tonic::Request::new(req))
            .await?
            .into_inner();
        Ok(serde_json::json!({
            "valid": resp.valid,
            "checkedCount": resp.checked_count,
            "firstBadSequence": resp.first_bad_sequence,
            "reason": resp.reason,
        }))
    }
}

/// AuthClaims から TenantContext を作る。
fn tenant_ctx_for(claims: &AuthClaims) -> TenantContext {
    TenantContext {
        tenant_id: claims.tenant_id.clone(),
        subject: claims.subject.clone(),
        ..Default::default()
    }
}

/// `{"seconds": N, "nanos": N}` または ISO8601 文字列を Timestamp に変換する。
/// 本リリースでは `{seconds, nanos}` 形式のみサポート（protojson 互換は将来）。
fn parse_timestamp(v: Option<&JsonValue>) -> Option<prost_types::Timestamp> {
    let v = v?;
    if v.is_null() {
        return None;
    }
    let seconds = v.get("seconds").and_then(|s| s.as_i64()).unwrap_or(0);
    let nanos = v.get("nanos").and_then(|n| n.as_i64()).unwrap_or(0) as i32;
    Some(prost_types::Timestamp { seconds, nanos })
}

/// `{"k": "v", ...}` → HashMap<String, String>。値が文字列以外は to_string で正規化。
fn parse_string_map(v: Option<&JsonValue>) -> HashMap<String, String> {
    let mut out = HashMap::new();
    if let Some(JsonValue::Object(map)) = v {
        for (k, val) in map.iter() {
            let s = val.as_str().map(String::from).unwrap_or_else(|| val.to_string());
            out.insert(k.clone(), s);
        }
    }
    out
}

/// JSON → AuditEvent。
fn parse_audit_event(v: &JsonValue) -> Result<AuditEvent, tonic::Status> {
    if v.is_null() {
        return Err(tonic::Status::invalid_argument(
            "tier1/audit/http: event field required",
        ));
    }
    Ok(AuditEvent {
        timestamp: parse_timestamp(v.get("timestamp")),
        actor: v.get("actor").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        action: v.get("action").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        resource: v
            .get("resource")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string(),
        outcome: v
            .get("outcome")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string(),
        attributes: parse_string_map(v.get("attributes")),
    })
}

/// AuditEvent → protojson 互換 JSON。
fn audit_event_to_json(e: &AuditEvent) -> JsonValue {
    let ts = e.timestamp.as_ref().map(|t| {
        serde_json::json!({ "seconds": t.seconds, "nanos": t.nanos })
    });
    let attrs: serde_json::Map<String, JsonValue> = e
        .attributes
        .iter()
        .map(|(k, v)| (k.clone(), JsonValue::String(v.clone())))
        .collect();
    serde_json::json!({
        "timestamp": ts,
        "actor": e.actor,
        "action": e.action,
        "resource": e.resource,
        "outcome": e.outcome,
        "attributes": attrs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::InMemoryAuditStore;
    use k1s0_tier1_common::idempotency::InMemoryIdempotencyCache;
    use std::time::Duration;

    fn make_state() -> AuditHttpState {
        AuditHttpState {
            server: Arc::new(AuditServer {
                store: Arc::new(InMemoryAuditStore::new()),
                idempotency: Arc::new(InMemoryIdempotencyCache::new(Duration::ZERO)),
            }),
        }
    }

    fn claims_for(tenant: &str) -> AuthClaims {
        AuthClaims {
            tenant_id: tenant.to_string(),
            subject: "u1".to_string(),
        }
    }

    #[tokio::test]
    async fn record_returns_audit_id() {
        let s = make_state();
        let rpc = RecordRpc { state: s.clone() };
        let body = serde_json::json!({
            "event": {
                "timestamp": { "seconds": 100, "nanos": 0 },
                "actor": "u1", "action": "WRITE",
                "resource": "k1s0:tenant:T1:resource:r/id",
                "outcome": "SUCCESS",
                "attributes": {}
            }
        });
        let resp = rpc.invoke(&claims_for("T1"), body).await.unwrap();
        assert_eq!(resp["auditId"].as_str().unwrap().len(), 64);
    }

    #[tokio::test]
    async fn record_dedups_on_idempotency_key() {
        let s = make_state();
        let rpc = RecordRpc { state: s.clone() };
        let body = || {
            serde_json::json!({
                "idempotencyKey": "kx",
                "event": {
                    "timestamp": { "seconds": 100, "nanos": 0 },
                    "actor": "u1", "action": "WRITE",
                    "resource": "r", "outcome": "SUCCESS"
                }
            })
        };
        let id1 = rpc.invoke(&claims_for("T1"), body()).await.unwrap()["auditId"]
            .as_str()
            .unwrap()
            .to_string();
        let id2 = rpc.invoke(&claims_for("T1"), body()).await.unwrap()["auditId"]
            .as_str()
            .unwrap()
            .to_string();
        assert_eq!(id1, id2);
    }

    #[tokio::test]
    async fn query_isolates_tenants() {
        let s = make_state();
        let rec = RecordRpc { state: s.clone() };
        let q = QueryRpc { state: s.clone() };
        rec.invoke(
            &claims_for("T1"),
            serde_json::json!({ "event": { "actor": "u1", "action": "R" } }),
        )
        .await
        .unwrap();
        rec.invoke(
            &claims_for("T2"),
            serde_json::json!({ "event": { "actor": "u2", "action": "R" } }),
        )
        .await
        .unwrap();
        let resp = q
            .invoke(&claims_for("T1"), serde_json::json!({ "limit": 10 }))
            .await
            .unwrap();
        let events = resp["events"].as_array().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["actor"], serde_json::json!("u1"));
    }
}
