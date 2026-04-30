// 本ファイルは t1-audit Pod の AuditService trait 実装本体。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/01_tier1全体コンポーネント俯瞰.md
//     - DS-SW-COMP-007（t1-audit Pod、WORM 追記専用）
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/10_Audit_Pii_API.md
//
// 役割:
//   AuditService の 4 RPC（Record / Query / Export / VerifyChain）を実装する。
//   Record は共通規約 §「冪等性と再試行」に従う 24h TTL の dedup を行う
//   （hash chain への二重追記を防止）。

// 標準同期。
use std::sync::Arc;

// SDK 公開 API の AuditService 関連型。
use k1s0_sdk_proto::k1s0::tier1::audit::v1::{
    AuditEvent, ExportAuditChunk, ExportAuditRequest, ExportFormat, QueryAuditRequest,
    QueryAuditResponse, RecordAuditRequest, RecordAuditResponse, VerifyChainRequest,
    VerifyChainResponse,
    audit_service_server::AuditService,
};
// 共通 idempotency cache。
use k1s0_tier1_common::idempotency::{IdempotencyCache, idempotency_key};
// store 層。
use crate::store::{AppendInput, AuditEntry, AuditStore, QueryInput};
// Export RPC の chunk 整形ループ。
use crate::export::send_export_chunks;
// 非同期 channel（server streaming で chunk を receiver 側に push する）。
use tokio::sync::mpsc;
// tokio_stream::wrappers で mpsc::Receiver を Stream に変換する。
use tokio_stream::wrappers::ReceiverStream;
// tonic ランタイム。
use tonic::{Request, Response, Status};

/// `AuditServer` は AuditService の trait 実装。
///
/// HTTP/JSON gateway と gRPC server の両方が同 instance を共有して同 store /
/// idempotency cache に書き込めるよう、Arc<dyn> 済の field のみ持つ。
#[derive(Clone)]
pub struct AuditServer {
    /// hash-chain backed store（trait object で他実装に swap 可能）。
    pub store: Arc<dyn AuditStore>,
    /// 共通規約 §「冪等性と再試行」に従う 24h TTL の重複抑止 cache。
    /// 同一 idempotency_key に対する re-record で hash chain への二重追記を防ぐ。
    pub idempotency: Arc<dyn IdempotencyCache>,
}

/// proto AuditEvent の attributes は HashMap、store は順序固定の BTreeMap で保持する。
pub fn proto_attrs_to_btree(
    attrs: std::collections::HashMap<String, String>,
) -> std::collections::BTreeMap<String, String> {
    let mut out = std::collections::BTreeMap::new();
    for (k, v) in attrs.into_iter() {
        out.insert(k, v);
    }
    out
}

/// proto AuditEvent + tenant_id → AppendInput 変換。
pub fn proto_to_append(event: &AuditEvent, tenant_id: String) -> AppendInput {
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

/// store の AuditEntry → proto AuditEvent 変換。
pub fn entry_to_proto(e: &AuditEntry) -> AuditEvent {
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
        // NFR-E-AC-003 二重防御: K1s0Layer が extensions に格納した claims (JWT 由来
        // tenant_id) と body.context.tenant_id を比較し、不一致なら PermissionDenied。
        // claims が無い場合 (auth=off) は body 由来をそのまま採用 (旧挙動維持)。
        let claims = req
            .extensions()
            .get::<k1s0_tier1_common::auth::AuthClaims>()
            .cloned()
            .unwrap_or_default();
        let r = req.into_inner();
        let body_tid = r
            .context
            .as_ref()
            .map(|c| c.tenant_id.clone())
            .unwrap_or_default();
        let tenant_id =
            k1s0_tier1_common::auth::enforce_tenant_boundary(&claims, &body_tid, "Audit.Record")?;
        let event = match r.event {
            Some(e) => e,
            None => {
                return Err(Status::invalid_argument(
                    "tier1/audit: event field required",
                ));
            }
        };
        // 共通規約 §「冪等性と再試行」: idempotency_key 指定時は cache hit で
        // 二重追記を回避する（hash chain は再書込で末尾の id がズレるため必須）。
        let idem_key = idempotency_key(&tenant_id, "Audit.Record", &r.idempotency_key);
        if !idem_key.is_empty() {
            if let Some(cached) = self.idempotency.lookup(&idem_key).await {
                if let Ok(audit_id) = String::from_utf8(cached) {
                    return Ok(Response::new(RecordAuditResponse { audit_id }));
                }
            }
        }
        let input = proto_to_append(&event, tenant_id);
        let id = self
            .store
            .append(input)
            .map_err(|e| Status::internal(format!("tier1/audit: store error: {}", e)))?;
        if !idem_key.is_empty() {
            // 成功時のみ cache に保存（失敗は再試行可能であるべき）。
            self.idempotency
                .store(&idem_key, id.as_bytes().to_vec())
                .await;
        }
        Ok(Response::new(RecordAuditResponse { audit_id: id }))
    }

    async fn query(
        &self,
        req: Request<QueryAuditRequest>,
    ) -> Result<Response<QueryAuditResponse>, Status> {
        // NFR-E-AC-003 二重防御: claims と body の tenant_id 一致を検査。
        let claims = req
            .extensions()
            .get::<k1s0_tier1_common::auth::AuthClaims>()
            .cloned()
            .unwrap_or_default();
        let r = req.into_inner();
        let body_tid = r
            .context
            .as_ref()
            .map(|c| c.tenant_id.clone())
            .unwrap_or_default();
        let tenant_id =
            k1s0_tier1_common::auth::enforce_tenant_boundary(&claims, &body_tid, "Audit.Query")?;
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

    /// 監査ログ Export の server-streaming 実装で利用する関連型。
    type ExportStream = ReceiverStream<Result<ExportAuditChunk, Status>>;

    /// FR-T1-AUDIT-002 疑似 IF "Audit.Export"。
    async fn export(
        &self,
        req: Request<ExportAuditRequest>,
    ) -> Result<Response<Self::ExportStream>, Status> {
        // NFR-E-AC-003 二重防御: claims と body の tenant_id 一致を検査。
        let claims = req
            .extensions()
            .get::<k1s0_tier1_common::auth::AuthClaims>()
            .cloned()
            .unwrap_or_default();
        let r = req.into_inner();
        let body_tid = r
            .context
            .as_ref()
            .map(|c| c.tenant_id.clone())
            .unwrap_or_default();
        let tenant_id =
            k1s0_tier1_common::auth::enforce_tenant_boundary(&claims, &body_tid, "Audit.Export")?;
        let from_ms = r
            .from
            .as_ref()
            .map(|t| t.seconds * 1000 + i64::from(t.nanos / 1_000_000));
        let to_ms = r
            .to
            .as_ref()
            .map(|t| t.seconds * 1000 + i64::from(t.nanos / 1_000_000));
        // フォーマット解決（UNSPECIFIED は NDJSON にフォールバック）。
        let format = match ExportFormat::try_from(r.format) {
            Ok(ExportFormat::Csv) => ExportFormat::Csv,
            Ok(ExportFormat::JsonArray) => ExportFormat::JsonArray,
            _ => ExportFormat::Ndjson,
        };
        // chunk_bytes の上下限を仕様通りクランプする。
        let chunk_bytes: usize = match r.chunk_bytes {
            n if n <= 0 => 65_536,
            n if n > 1_048_576 => 1_048_576,
            n => n as usize,
        };
        let q = QueryInput {
            from_ms,
            to_ms,
            filters: std::collections::BTreeMap::new(),
            limit: usize::MAX,
            tenant_id,
        };
        let entries = self
            .store
            .query(q)
            .map_err(|e| Status::internal(format!("tier1/audit: store error: {}", e)))?;
        let (tx, rx) = mpsc::channel::<Result<ExportAuditChunk, Status>>(16);
        tokio::spawn(async move {
            send_export_chunks(tx, entries, format, chunk_bytes).await;
        });
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    /// FR-T1-AUDIT-002: ハッシュチェーン整合性検証。
    async fn verify_chain(
        &self,
        req: Request<VerifyChainRequest>,
    ) -> Result<Response<VerifyChainResponse>, Status> {
        // NFR-E-AC-003 二重防御: claims と body の tenant_id 一致を検査。
        let claims = req
            .extensions()
            .get::<k1s0_tier1_common::auth::AuthClaims>()
            .cloned()
            .unwrap_or_default();
        let r = req.into_inner();
        let body_tid = r
            .context
            .as_ref()
            .map(|c| c.tenant_id.clone())
            .unwrap_or_default();
        let tenant_id = k1s0_tier1_common::auth::enforce_tenant_boundary(
            &claims,
            &body_tid,
            "Audit.VerifyChain",
        )?;
        let from_ms = r
            .from
            .as_ref()
            .map(|t| t.seconds * 1000 + i64::from(t.nanos / 1_000_000));
        let to_ms = r
            .to
            .as_ref()
            .map(|t| t.seconds * 1000 + i64::from(t.nanos / 1_000_000));
        let outcome = self
            .store
            .verify_chain_detail(&tenant_id, from_ms, to_ms)
            .map_err(|e| Status::internal(format!("tier1/audit: store error: {}", e)))?;
        Ok(Response::new(VerifyChainResponse {
            valid: outcome.valid,
            checked_count: outcome.checked_count,
            first_bad_sequence: outcome.first_bad_sequence,
            reason: outcome.reason,
        }))
    }
}

#[cfg(test)]
mod handler_tests {
    use super::*;
    use crate::store::InMemoryAuditStore;
    use k1s0_sdk_proto::k1s0::tier1::audit::v1::AuditEvent;
    use k1s0_sdk_proto::k1s0::tier1::common::v1::TenantContext;
    use k1s0_tier1_common::auth::AuthClaims;
    use k1s0_tier1_common::idempotency::InMemoryIdempotencyCache;

    /// テスト用 AuditServer を構築する。in-memory store + 空 idempotency cache。
    fn server() -> AuditServer {
        AuditServer {
            store: Arc::new(InMemoryAuditStore::new()),
            idempotency: Arc::new(InMemoryIdempotencyCache::new(std::time::Duration::ZERO)),
        }
    }

    /// claims (tenant-A) を `Request::extensions` に注入する helper。
    /// K1s0Layer が production で行う処理を bypass してテストから直接 inject する。
    fn req_with_claims<T>(payload: T, tenant_id: &str) -> Request<T> {
        let mut r = Request::new(payload);
        r.extensions_mut().insert(AuthClaims {
            tenant_id: tenant_id.to_string(),
            subject: "alice".to_string(),
            ..Default::default()
        });
        r
    }

    fn sample_event() -> AuditEvent {
        AuditEvent {
            actor: "alice".to_string(),
            action: "Read".to_string(),
            resource: "secret/foo".to_string(),
            outcome: "SUCCESS".to_string(),
            ..Default::default()
        }
    }

    /// G3 / H1 regression: claims (JWT) と body の tenant_id が不一致 → handler が
    /// PermissionDenied で reject。
    #[tokio::test]
    async fn record_rejects_cross_tenant() {
        let s = server();
        let req = req_with_claims(
            RecordAuditRequest {
                event: Some(sample_event()),
                context: Some(TenantContext {
                    tenant_id: "tenant-B".to_string(), // 攻撃側の偽装値
                    ..Default::default()
                }),
                idempotency_key: String::new(),
            },
            "tenant-A", // claims (JWT 由来)
        );
        let err = s.record(req).await.unwrap_err();
        assert_eq!(err.code(), tonic::Code::PermissionDenied);
        assert!(
            err.message().contains("cross-tenant"),
            "error should mention 'cross-tenant', got: {}",
            err.message()
        );
        assert!(
            err.message().contains("tenant-A") && err.message().contains("tenant-B"),
            "error should expose jwt + body tenants, got: {}",
            err.message()
        );
    }

    /// 一致する tenant_id なら通常通り record される (auth_id を返す)。
    #[tokio::test]
    async fn record_accepts_matching_tenant() {
        let s = server();
        let req = req_with_claims(
            RecordAuditRequest {
                event: Some(sample_event()),
                context: Some(TenantContext {
                    tenant_id: "tenant-A".to_string(),
                    ..Default::default()
                }),
                idempotency_key: String::new(),
            },
            "tenant-A",
        );
        let resp = s.record(req).await.expect("matching tenant should pass");
        assert!(
            !resp.into_inner().audit_id.is_empty(),
            "successful record should return non-empty audit_id"
        );
    }

    /// claims が無い (auth=off / HTTP gateway 経由) なら body の tenant_id を採用する
    /// (旧挙動互換、H1 の仕様)。
    #[tokio::test]
    async fn record_without_claims_uses_body_tenant() {
        let s = server();
        // claims を入れずに request 構築
        let req = Request::new(RecordAuditRequest {
            event: Some(sample_event()),
            context: Some(TenantContext {
                tenant_id: "any-tenant".to_string(),
                ..Default::default()
            }),
            idempotency_key: String::new(),
        });
        let resp = s.record(req).await.expect("auth=off path should accept body");
        assert!(!resp.into_inner().audit_id.is_empty());
    }

    /// query / verify_chain も同じ cross-tenant 防御を持つこと。
    #[tokio::test]
    async fn query_rejects_cross_tenant() {
        let s = server();
        let req = req_with_claims(
            QueryAuditRequest {
                context: Some(TenantContext {
                    tenant_id: "tenant-B".to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            },
            "tenant-A",
        );
        let err = s.query(req).await.unwrap_err();
        assert_eq!(err.code(), tonic::Code::PermissionDenied);
    }

    #[tokio::test]
    async fn verify_chain_rejects_cross_tenant() {
        let s = server();
        let req = req_with_claims(
            VerifyChainRequest {
                context: Some(TenantContext {
                    tenant_id: "tenant-B".to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            },
            "tenant-A",
        );
        let err = s.verify_chain(req).await.unwrap_err();
        assert_eq!(err.code(), tonic::Code::PermissionDenied);
    }
}
