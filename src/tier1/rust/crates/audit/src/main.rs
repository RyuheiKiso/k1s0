// 本ファイルは t1-audit Pod の起動エントリポイント（plan 04-09 結線済）。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/01_tier1全体コンポーネント俯瞰.md
//     - DS-SW-COMP-007（t1-audit Pod、WORM 追記専用）
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/10_Audit_Pii_API.md
//
// 役割:
//   - :50001 で listen
//   - AuditService（Record / Query）を hash chain backed in-memory store で実装
//   - SIGINT / SIGTERM で graceful shutdown
//
// 永続化:
//   本リリースでは in-memory store のみ。Postgres 永続化は同 trait の別実装で
//   plan 04-09 の post-MVP として追加する予定。in-memory でも hash chain 改竄
//   検知（NFR-H-INT-001）は機能する。

use std::sync::Arc;

// SDK 公開 API の AuditService の Service trait / Server 型 / Request / Response 型を import。
use k1s0_sdk_proto::k1s0::tier1::audit::v1::{
    // AuditEvent / Request / Response 型。
    AuditEvent, QueryAuditRequest, QueryAuditResponse, RecordAuditRequest, RecordAuditResponse,
    // AuditService の trait と Server 型。
    audit_service_server::{AuditService, AuditServiceServer},
};
// store 層（lib.rs 経由）。
use k1s0_tier1_audit::store::{AppendInput, AuditEntry, AuditStore, InMemoryAuditStore, QueryInput};
// SIGTERM / SIGINT 受信。
use tokio::signal::unix::{SignalKind, signal};
// tonic ランタイム。
use tonic::{Request, Response, Status, transport::Server};

// EXPOSE 50001 規約。production の K8s Pod は単一 NetNS なので 50001 でぶつからないが、
// dev / 同一ホスト内で複数 Rust Pod を同時起動する場面は `LISTEN_ADDR` 環境変数で上書きする。
const DEFAULT_LISTEN: &str = "[::]:50001";

/// 環境変数 `LISTEN_ADDR` が設定されていればそれを使い、未設定なら DEFAULT_LISTEN を返す。
fn listen_addr() -> String {
    std::env::var("LISTEN_ADDR").unwrap_or_else(|_| DEFAULT_LISTEN.to_string())
}

// AuditServer は AuditService の trait 実装。
struct AuditServer {
    /// hash-chain backed store（trait object で他実装に swap 可能、production は別実装）。
    store: Arc<dyn AuditStore>,
}

// proto AuditEvent の attributes は HashMap<String,String>。
// store は順序固定の BTreeMap で保持するため、proto → store で並び替える。
fn proto_attrs_to_btree(
    attrs: std::collections::HashMap<String, String>,
) -> std::collections::BTreeMap<String, String> {
    let mut out = std::collections::BTreeMap::new();
    for (k, v) in attrs.into_iter() {
        out.insert(k, v);
    }
    out
}

// proto AuditEvent + tenant_id → AppendInput。
fn proto_to_append(event: &AuditEvent, tenant_id: String) -> AppendInput {
    let timestamp_ms = event
        .timestamp
        .as_ref()
        .map(|t| t.seconds * 1000 + i64::from(t.nanos / 1_000_000))
        .unwrap_or(0);
    AppendInput {
        timestamp_ms,
        actor: event.actor.clone(),
        action: event.action.clone(),
        resource: event.resource.clone(),
        outcome: event.outcome.clone(),
        attributes: proto_attrs_to_btree(event.attributes.clone()),
        tenant_id,
    }
}

// store の AuditEntry → proto AuditEvent。
fn entry_to_proto(e: &AuditEntry) -> AuditEvent {
    let mut attrs = std::collections::HashMap::new();
    for (k, v) in e.attributes.iter() {
        attrs.insert(k.clone(), v.clone());
    }
    AuditEvent {
        timestamp: Some(prost_types::Timestamp {
            seconds: e.timestamp_ms / 1000,
            nanos: ((e.timestamp_ms % 1000) * 1_000_000) as i32,
        }),
        actor: e.actor.clone(),
        action: e.action.clone(),
        resource: e.resource.clone(),
        outcome: e.outcome.clone(),
        attributes: attrs,
    }
}

#[tonic::async_trait]
impl AuditService for AuditServer {
    async fn record(
        &self,
        req: Request<RecordAuditRequest>,
    ) -> Result<Response<RecordAuditResponse>, Status> {
        let r = req.into_inner();
        let tenant_id = r
            .context
            .as_ref()
            .map(|c| c.tenant_id.clone())
            .unwrap_or_default();
        if tenant_id.is_empty() {
            return Err(Status::invalid_argument(
                "tier1/audit: tenant_id required in TenantContext",
            ));
        }
        let event = match r.event {
            Some(e) => e,
            None => {
                return Err(Status::invalid_argument(
                    "tier1/audit: event field required",
                ));
            }
        };
        let input = proto_to_append(&event, tenant_id);
        let id = self
            .store
            .append(input)
            .map_err(|e| Status::internal(format!("tier1/audit: store error: {}", e)))?;
        Ok(Response::new(RecordAuditResponse { audit_id: id }))
    }

    async fn query(
        &self,
        req: Request<QueryAuditRequest>,
    ) -> Result<Response<QueryAuditResponse>, Status> {
        let r = req.into_inner();
        let tenant_id = r
            .context
            .as_ref()
            .map(|c| c.tenant_id.clone())
            .unwrap_or_default();
        if tenant_id.is_empty() {
            return Err(Status::invalid_argument(
                "tier1/audit: tenant_id required in TenantContext",
            ));
        }
        let from_ms = r
            .from
            .as_ref()
            .map(|t| t.seconds * 1000 + i64::from(t.nanos / 1_000_000));
        let to_ms = r
            .to
            .as_ref()
            .map(|t| t.seconds * 1000 + i64::from(t.nanos / 1_000_000));
        let q = QueryInput {
            from_ms,
            to_ms,
            filters: proto_attrs_to_btree(r.filters),
            limit: r.limit as usize,
            tenant_id,
        };
        let entries = self
            .store
            .query(q)
            .map_err(|e| Status::internal(format!("tier1/audit: store error: {}", e)))?;
        let events: Vec<AuditEvent> = entries.iter().map(entry_to_proto).collect();
        Ok(Response::new(QueryAuditResponse { events }))
    }
}

async fn shutdown_signal() {
    let mut sigterm = signal(SignalKind::terminate()).expect("install SIGTERM handler");
    let mut sigint = signal(SignalKind::interrupt()).expect("install SIGINT handler");
    tokio::select! {
        _ = sigterm.recv() => {
            eprintln!("tier1/audit: received SIGTERM, shutting down");
        },
        _ = sigint.recv() => {
            eprintln!("tier1/audit: received SIGINT, shutting down");
        },
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listen = listen_addr();
    let addr = listen.parse()?;
    eprintln!("tier1/audit: gRPC server listening on {}", listen);
    let store: Arc<dyn AuditStore> = Arc::new(InMemoryAuditStore::new());
    let server = AuditServer { store };
    Server::builder()
        .add_service(AuditServiceServer::new(server))
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_event(ts_sec: i64, actor: &str, action: &str) -> AuditEvent {
        AuditEvent {
            timestamp: Some(prost_types::Timestamp {
                seconds: ts_sec,
                nanos: 0,
            }),
            actor: actor.to_string(),
            action: action.to_string(),
            resource: "k1s0:tenant:T:resource:secret/db".to_string(),
            outcome: "SUCCESS".to_string(),
            attributes: Default::default(),
        }
    }

    fn make_ctx(tenant: &str) -> Option<k1s0_sdk_proto::k1s0::tier1::common::v1::TenantContext> {
        Some(k1s0_sdk_proto::k1s0::tier1::common::v1::TenantContext {
            tenant_id: tenant.to_string(),
            ..Default::default()
        })
    }

    fn make_server() -> AuditServer {
        AuditServer {
            store: Arc::new(InMemoryAuditStore::new()),
        }
    }

    #[tokio::test]
    async fn record_returns_audit_id() {
        let s = make_server();
        let resp = s
            .record(Request::new(RecordAuditRequest {
                event: Some(make_event(100, "u", "READ")),
                context: make_ctx("T"),
            }))
            .await
            .unwrap()
            .into_inner();
        assert!(!resp.audit_id.is_empty());
        // SHA-256 hex は 64 文字。
        assert_eq!(resp.audit_id.len(), 64);
    }

    #[tokio::test]
    async fn record_requires_tenant() {
        let s = make_server();
        let r = s
            .record(Request::new(RecordAuditRequest {
                event: Some(make_event(100, "u", "READ")),
                context: None,
            }))
            .await;
        assert!(r.is_err());
        assert_eq!(r.err().unwrap().code(), tonic::Code::InvalidArgument);
    }

    #[tokio::test]
    async fn query_returns_recorded_events() {
        let s = make_server();
        for i in 1..=3 {
            s.record(Request::new(RecordAuditRequest {
                event: Some(make_event(i, "u", "R")),
                context: make_ctx("T"),
            }))
            .await
            .unwrap();
        }
        let resp = s
            .query(Request::new(QueryAuditRequest {
                from: None,
                to: None,
                filters: Default::default(),
                limit: 10,
                context: make_ctx("T"),
            }))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(resp.events.len(), 3);
        // 時刻昇順。
        for w in resp.events.windows(2) {
            assert!(w[0].timestamp.as_ref().unwrap().seconds <= w[1].timestamp.as_ref().unwrap().seconds);
        }
    }

    #[tokio::test]
    async fn query_isolates_tenants() {
        let s = make_server();
        s.record(Request::new(RecordAuditRequest {
            event: Some(make_event(1, "u1", "R")),
            context: make_ctx("T1"),
        }))
        .await
        .unwrap();
        s.record(Request::new(RecordAuditRequest {
            event: Some(make_event(2, "u2", "R")),
            context: make_ctx("T2"),
        }))
        .await
        .unwrap();
        let r = s
            .query(Request::new(QueryAuditRequest {
                limit: 10,
                context: make_ctx("T1"),
                ..Default::default()
            }))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(r.events.len(), 1);
        assert_eq!(r.events[0].actor, "u1");
    }
}
