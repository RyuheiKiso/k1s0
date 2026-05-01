// 本ファイルは t1-audit Pod の起動エントリポイント（plan 04-09 結線済）。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/01_tier1全体コンポーネント俯瞰.md
//     - DS-SW-COMP-007（t1-audit Pod、WORM 追記専用）
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/10_Audit_Pii_API.md
//
// 役割:
//   - :50001 で listen
//   - AuditService 4 RPC（Record / Query / Export / VerifyChain）の実装は
//     `k1s0_tier1_audit::server::AuditServer` に集約済（500 行制限のため）。
//   - SIGINT / SIGTERM で graceful shutdown
//
// 永続化:
//   本リリースでは in-memory store のみ。Postgres 永続化は同 trait の別実装で
//   plan 04-09 の post-MVP として追加する予定。in-memory でも hash chain 改竄
//   検知（NFR-H-INT-001）は機能する。

use std::sync::Arc;

// SDK 公開 API。
use k1s0_sdk_proto::FILE_DESCRIPTOR_SET;
// HealthServiceServer: 共通 HealthService 実装を gRPC server に登録するための型。
use k1s0_sdk_proto::k1s0::tier1::health::v1::health_service_server::HealthServiceServer;
// AuditServiceServer は SDK proto 側に存在する Server 型。
use k1s0_sdk_proto::k1s0::tier1::audit::v1::audit_service_server::AuditServiceServer;
// 共通 HealthService 実装。
use k1s0_tier1_health::Service as HealthSvc;
// store / server 層（lib.rs 経由）。
use k1s0_tier1_audit::server::AuditServer;
use k1s0_tier1_audit::store::{AuditStore, InMemoryAuditStore};
// 共通 idempotency cache（共通規約 §「冪等性と再試行」）。
use k1s0_tier1_common::idempotency::{IdempotencyCache, InMemoryIdempotencyCache};
// 共通 gRPC interceptor Layer（auth / ratelimit / observability / audit auto-emit）。
use k1s0_tier1_common::grpc_layer::K1s0Layer;
// 共通 HTTP/JSON gateway。
use k1s0_tier1_common::http_gateway::{HttpGateway, JsonRpc, serve as serve_http};
// 共通 runtime（環境変数から auth / rate_limiter / audit_emitter / idempotency をまとめて構築）。
use k1s0_tier1_common::runtime::CommonRuntime;
// Audit HTTP/JSON gateway 用 adapter。
use k1s0_tier1_audit::http::{AuditHttpState, QueryRpc, RecordRpc, VerifyChainRpc};
// SIGTERM / SIGINT 受信。
use tokio::signal::unix::{SignalKind, signal};
// tonic ランタイム。
use tonic::transport::Server;

// EXPOSE 50001 規約。production の K8s Pod は単一 NetNS なので 50001 でぶつからないが、
// dev / 同一ホスト内で複数 Rust Pod を同時起動する場面は `LISTEN_ADDR` 環境変数で上書きする。
const DEFAULT_LISTEN: &str = "[::]:50001";

/// 環境変数 `LISTEN_ADDR` が設定されていればそれを使い、未設定なら DEFAULT_LISTEN を返す。
fn listen_addr() -> String {
    std::env::var("LISTEN_ADDR").unwrap_or_else(|_| DEFAULT_LISTEN.to_string())
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
    // env `K1S0_AUDIT_PG_DSN` が設定されていれば Postgres-backed WORM ストアを使う
    // （CNPG cluster の k1s0-postgres-rw を想定、NFR-H-INT-001 永続化）。
    // 未設定 / 接続失敗時は InMemoryAuditStore に fallback（dev / CI 用）。
    let store: Arc<dyn AuditStore> = if let Ok(dsn) = std::env::var("K1S0_AUDIT_PG_DSN") {
        match k1s0_tier1_audit::postgres_store::PostgresAuditStore::connect(&dsn).await {
            Ok(s) => {
                eprintln!("tier1/audit: PostgresAuditStore connected (DSN ok)");
                Arc::new(s)
            }
            Err(e) => {
                eprintln!(
                    "tier1/audit: PostgresAuditStore connect failed ({}), falling back to in-memory",
                    e
                );
                Arc::new(InMemoryAuditStore::new())
            }
        }
    } else {
        eprintln!("tier1/audit: K1S0_AUDIT_PG_DSN not set, using InMemoryAuditStore (dev mode)");
        Arc::new(InMemoryAuditStore::new())
    };
    // 共通規約 §「冪等性と再試行」: 24h TTL（既定）の重複抑止 cache を有効化。
    let idempotency: Arc<dyn IdempotencyCache> =
        Arc::new(InMemoryIdempotencyCache::new(std::time::Duration::ZERO));
    let server = AuditServer { store, idempotency };
    // gRPC Server Reflection（Go Pod 側の reflection.Register と機能等価）。
    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build_v1()?;
    // 共通 HealthService を構築する。in-memory store のみのためリリース時点は probe 空。
    // Postgres backed store に切替時は WAL / replication 状態 probe を追加予定。
    let health = HealthSvc::new(env!("CARGO_PKG_VERSION").to_string(), vec![]);
    // docs §共通規約 に従う interceptor chain を環境変数駆動で構築する。
    // AUTH_MODE / AUTH_HMAC_SECRET / AUTH_JWKS_URL / AUDIT_MODE で挙動を切替。
    let rt = CommonRuntime::from_env();
    let layer = K1s0Layer::new(rt.auth.clone(), rt.rate_limiter.clone(), rt.audit_emitter.clone());

    // HTTP/JSON gateway（TIER1_HTTP_LISTEN_ADDR が設定されている場合のみ起動）。
    // 共通規約 §「HTTP/JSON 互換」: AuditService の 3 unary RPC を JSON で公開する。
    // Export は server-streaming のため非対応（gRPC 経路を使う）。
    let http_handle: Option<tokio::task::JoinHandle<()>> =
        match std::env::var("TIER1_HTTP_LISTEN_ADDR").ok().filter(|s| !s.is_empty()) {
            Some(http_addr) => {
                let http_state = AuditHttpState {
                    server: Arc::new(server.clone()),
                };
                let gateway = HttpGateway::new(
                    rt.auth.clone(),
                    rt.rate_limiter.clone(),
                    rt.audit_emitter.clone(),
                )
                .register(Arc::new(RecordRpc { state: http_state.clone() }) as Arc<dyn JsonRpc>)
                .register(Arc::new(QueryRpc { state: http_state.clone() }) as Arc<dyn JsonRpc>)
                .register(Arc::new(VerifyChainRpc { state: http_state }) as Arc<dyn JsonRpc>);
                let router = gateway.into_router();
                eprintln!("tier1/audit: HTTP/JSON gateway listening on {}", http_addr);
                let addr_for_task = http_addr.clone();
                Some(tokio::spawn(async move {
                    if let Err(e) = serve_http(&addr_for_task, router).await {
                        eprintln!("tier1/audit: HTTP gateway error: {}", e);
                    }
                }))
            }
            None => None,
        };

    // 標準 grpc.health.v1.Health プロトコル登録（K8s grpc liveness/readiness probe 用）。
    let (mut health_reporter, health_svc) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<AuditServiceServer<AuditServer>>()
        .await;

    Server::builder()
        .layer(layer)
        .add_service(AuditServiceServer::new(server))
        .add_service(HealthServiceServer::new(health))
        .add_service(health_svc)
        .add_service(reflection)
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;
    if let Some(h) = http_handle {
        h.abort();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use k1s0_sdk_proto::k1s0::tier1::audit::v1::audit_service_server::AuditService;
    use k1s0_sdk_proto::k1s0::tier1::audit::v1::{
        AuditEvent, QueryAuditRequest, RecordAuditRequest, VerifyChainRequest,
    };

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
            idempotency: Arc::new(InMemoryIdempotencyCache::new(std::time::Duration::ZERO)),
        }
    }

    #[tokio::test]
    async fn record_returns_audit_id() {
        let s = make_server();
        let resp = s
            .record(tonic::Request::new(RecordAuditRequest {
                event: Some(make_event(100, "u", "READ")),
                context: make_ctx("T"),
                ..Default::default()
            }))
            .await
            .unwrap()
            .into_inner();
        assert!(!resp.audit_id.is_empty());
        // FR-T1-AUDIT-001: SHA-256 を URL-safe base64（padding なし）で表現 → 43 文字。
        assert_eq!(resp.audit_id.len(), 43);
    }

    #[tokio::test]
    async fn record_requires_tenant() {
        let s = make_server();
        let r = s
            .record(tonic::Request::new(RecordAuditRequest {
                event: Some(make_event(100, "u", "READ")),
                context: None,
                ..Default::default()
            }))
            .await;
        assert!(r.is_err());
        assert_eq!(r.err().unwrap().code(), tonic::Code::InvalidArgument);
    }

    #[tokio::test]
    async fn record_dedups_by_idempotency_key() {
        let s = make_server();
        // 同一 idempotency_key で 2 回 record を呼ぶ。2 回目は cache hit で
        // 同じ audit_id を返し、hash chain には 1 件のみ書かれる。
        let req = || RecordAuditRequest {
            event: Some(make_event(100, "u", "WRITE")),
            context: make_ctx("T"),
            idempotency_key: "k1".into(),
        };
        let id1 = s
            .record(tonic::Request::new(req()))
            .await
            .unwrap()
            .into_inner()
            .audit_id;
        let id2 = s
            .record(tonic::Request::new(req()))
            .await
            .unwrap()
            .into_inner()
            .audit_id;
        assert_eq!(id1, id2);
        // verify_chain で event 件数 1 を確認する（dedup の証跡）。
        let v = s
            .verify_chain(tonic::Request::new(VerifyChainRequest {
                from: None,
                to: None,
                context: make_ctx("T"),
            }))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(v.checked_count, 1);
        assert!(v.valid);
    }

    #[tokio::test]
    async fn record_with_different_keys_creates_distinct_entries() {
        let s = make_server();
        let id1 = s
            .record(tonic::Request::new(RecordAuditRequest {
                event: Some(make_event(100, "u", "WRITE")),
                context: make_ctx("T"),
                idempotency_key: "k1".into(),
            }))
            .await
            .unwrap()
            .into_inner()
            .audit_id;
        let id2 = s
            .record(tonic::Request::new(RecordAuditRequest {
                event: Some(make_event(100, "u", "WRITE")),
                context: make_ctx("T"),
                idempotency_key: "k2".into(),
            }))
            .await
            .unwrap()
            .into_inner()
            .audit_id;
        assert_ne!(id1, id2);
    }

    #[tokio::test]
    async fn query_returns_recorded_events() {
        let s = make_server();
        for i in 1..=3 {
            s.record(tonic::Request::new(RecordAuditRequest {
                event: Some(make_event(i, "u", "R")),
                context: make_ctx("T"),
                ..Default::default()
            }))
            .await
            .unwrap();
        }
        let resp = s
            .query(tonic::Request::new(QueryAuditRequest {
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
        for w in resp.events.windows(2) {
            assert!(
                w[0].timestamp.as_ref().unwrap().seconds
                    <= w[1].timestamp.as_ref().unwrap().seconds
            );
        }
    }

    #[tokio::test]
    async fn verify_chain_returns_valid_after_appends() {
        let s = make_server();
        for i in 1..=3 {
            s.record(tonic::Request::new(RecordAuditRequest {
                event: Some(make_event(i, "u", "R")),
                context: make_ctx("T"),
                ..Default::default()
            }))
            .await
            .unwrap();
        }
        let resp = s
            .verify_chain(tonic::Request::new(VerifyChainRequest {
                from: None,
                to: None,
                context: make_ctx("T"),
            }))
            .await
            .unwrap()
            .into_inner();
        assert!(resp.valid);
        assert_eq!(resp.checked_count, 3);
        assert_eq!(resp.first_bad_sequence, 0);
        assert!(resp.reason.is_empty());
    }

    #[tokio::test]
    async fn verify_chain_requires_tenant() {
        let s = make_server();
        let r = s
            .verify_chain(tonic::Request::new(VerifyChainRequest {
                from: None,
                to: None,
                context: None,
            }))
            .await;
        assert!(r.is_err());
        assert_eq!(r.err().unwrap().code(), tonic::Code::InvalidArgument);
    }

    #[tokio::test]
    async fn query_isolates_tenants() {
        let s = make_server();
        s.record(tonic::Request::new(RecordAuditRequest {
            event: Some(make_event(1, "u1", "R")),
            context: make_ctx("T1"),
            ..Default::default()
        }))
        .await
        .unwrap();
        s.record(tonic::Request::new(RecordAuditRequest {
            event: Some(make_event(2, "u2", "R")),
            context: make_ctx("T2"),
            ..Default::default()
        }))
        .await
        .unwrap();
        let r = s
            .query(tonic::Request::new(QueryAuditRequest {
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
