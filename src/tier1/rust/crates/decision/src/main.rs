// 本ファイルは t1-decision Pod の起動エントリポイント（plan 04-08 結線済）。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/01_tier1全体コンポーネント俯瞰.md
//     - DS-SW-COMP-008（t1-decision Pod、JDM 評価エンジン）
//   docs/02_構想設計/adr/ADR-RULE-001-zen-engine.md
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/09_Decision_API.md
//
// 役割:
//   - :50001 で listen
//   - DecisionService（Evaluate / BatchEvaluate）と DecisionAdminService
//     （RegisterRule / ListVersions / GetRule）を registry-backed 実装で登録
//   - SIGINT / SIGTERM で graceful shutdown
//
// rule engine:
//   ZEN Engine 0.55+（gorules/zen）と JDM フォーマットを直接統合する（ADR-RULE-001 採用）。
//   登録された JDM は DecisionContent に parse + opcode キャッシュコンパイルされ、
//   評価時は DecisionEngine.evaluate_with_opts に委譲する。include_trace=true で
//   nodes 単位の評価トレースが返る（ADR-RULE-001 必須要件）。

use std::sync::Arc;

// SDK 公開 API の DecisionService / DecisionAdminService の Service trait と Server 型を import。
use k1s0_sdk_proto::FILE_DESCRIPTOR_SET;
// HealthServiceServer: 共通 HealthService 実装を gRPC server に登録するための型。
use k1s0_sdk_proto::k1s0::tier1::health::v1::health_service_server::HealthServiceServer;
use k1s0_sdk_proto::k1s0::tier1::decision::v1::{
    BatchEvaluateRequest, BatchEvaluateResponse, EvaluateRequest, EvaluateResponse, GetRuleRequest,
    GetRuleResponse, ListVersionsRequest, ListVersionsResponse, RegisterRuleRequest,
    RegisterRuleResponse, RuleVersionMeta,
    decision_admin_service_server::{DecisionAdminService, DecisionAdminServiceServer},
    decision_service_server::{DecisionService, DecisionServiceServer},
};
// 共通 HealthService 実装。
use k1s0_tier1_health::Service as HealthSvc;
// 共通 gRPC interceptor Layer（auth / ratelimit / observability / audit auto-emit）。
use k1s0_tier1_common::grpc_layer::K1s0Layer;
// Decision.Evaluate の評価結果を rule_id / rule_version / input_hash / output_hash 付き
// audit として詳細発火するための型 (FR-T1-DECISION-003)。
use k1s0_tier1_common::audit::{AuditEmitter, AuditRecord, outcome_from_code};
use k1s0_tier1_common::auth::AuthClaims;
// 共通 HTTP/JSON gateway。
use k1s0_tier1_common::http_gateway::{HttpGateway, JsonRpc, serve as serve_http};
// 共通 runtime（環境変数から共通リソースを構築）。
use k1s0_tier1_common::runtime::CommonRuntime;
// HTTP/JSON gateway 用 adapter。
use k1s0_tier1_decision::http::{
    BatchEvaluateRpc, DecisionHttpState, EvaluateRpc, GetRuleRpc, ListVersionsRpc,
    RegisterRuleRpc,
};
// 内部 registry。
use k1s0_tier1_decision::registry::{RegisterInput, RegistryError, RuleMeta, RuleRegistry};

// RuleMeta → proto RuleVersionMeta 変換。
fn rule_meta_to_proto(m: &RuleMeta) -> RuleVersionMeta {
    RuleVersionMeta {
        rule_version: m.rule_version.clone(),
        commit_hash: m.commit_hash.clone(),
        registered_at_ms: m.registered_at_ms,
        registered_by: m.registered_by.clone(),
        deprecated: false,
    }
}
// SIGTERM / SIGINT 受信用。
use tokio::signal::unix::{SignalKind, signal};
// tonic ランタイム / 型。
use tonic::{Request, Response, Status, transport::Server};

// EXPOSE 50001 規約。production の K8s Pod は単一 NetNS なので 50001 でぶつからないが、
// dev / 同一ホスト内で複数 Rust Pod を同時起動する場面は `LISTEN_ADDR` 環境変数で上書きする。
const DEFAULT_LISTEN: &str = "[::]:50001";

/// 環境変数 `LISTEN_ADDR` が設定されていればそれを使い、未設定なら DEFAULT_LISTEN を返す。
fn listen_addr() -> String {
    std::env::var("LISTEN_ADDR").unwrap_or_else(|_| DEFAULT_LISTEN.to_string())
}

// 共有 registry を保持する Server。
struct DecisionServer {
    registry: Arc<RuleRegistry>,
    /// FR-T1-DECISION-003: 評価ごとに rich audit を出すための emitter。
    /// 共通 K1s0Layer も基本記録を出すが、本フィールド経由で rule_id /
    /// rule_version / input_hash / output_hash を attributes に詰めた追加 record を発火する。
    audit_emitter: Arc<dyn AuditEmitter>,
}

struct DecisionAdminServer {
    registry: Arc<RuleRegistry>,
    /// RegisterRule 時に rule_id / rule_version / commit_hash を attributes に
    /// 詰めた追加 audit record を発火するための emitter（FR-T1-DECISION-003）。
    audit_emitter: Arc<dyn AuditEmitter>,
}

/// 入力 / 出力 JSON の SHA-256 を URL-safe base64 (no padding) で算出する。
/// FR-T1-DECISION-003: 「個人情報を含む入力は PII マスキング連携で保護される」要件のため、
/// 生 input/output の代わりに hash を attributes に保存する（PII Mask が同 Pod に同居しない
/// アーキテクチャ前提では hash 化が最小限の構造的保護となる）。
fn hash_b64(bytes: &[u8]) -> String {
    use base64::Engine as _;
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(h.finalize())
}

/// req.extensions の AuthClaims を取り出す。K1s0Layer が auth を通している場合は
/// claims が存在し、auth=off の場合は default claims（empty tenant）を返す。
fn claims_from_req<T>(req: &Request<T>) -> AuthClaims {
    req.extensions()
        .get::<AuthClaims>()
        .cloned()
        .unwrap_or_default()
}

// RegistryError → tonic::Status 翻訳。
fn registry_err_to_status(e: RegistryError, rpc: &str) -> Status {
    match e {
        RegistryError::InvalidJson(msg) | RegistryError::InvalidRule(msg) => {
            Status::invalid_argument(format!("tier1/decision: {}: {}", rpc, msg))
        }
        RegistryError::NotFound { rule_id, rule_version } => Status::not_found(format!(
            "tier1/decision: {}: rule {}/{} not found",
            rpc, rule_id, rule_version
        )),
        RegistryError::EvalFailed(msg) => {
            Status::internal(format!("tier1/decision: {}: eval failed: {}", rpc, msg))
        }
        RegistryError::LockPoisoned => {
            Status::internal(format!("tier1/decision: {}: lock poisoned", rpc))
        }
    }
}

#[tonic::async_trait]
impl DecisionService for DecisionServer {
    async fn evaluate(
        &self,
        req: Request<EvaluateRequest>,
    ) -> Result<Response<EvaluateResponse>, Status> {
        let claims = claims_from_req(&req);
        let r = req.into_inner();
        // 評価実行。FR-T1-DECISION-003 の audit 連携用に rule_id / rule_version /
        // input/output hash を保持する。
        let outcome_result = self
            .registry
            .evaluate(&r.rule_id, &r.rule_version, &r.input_json, r.include_trace)
            .await;
        // 評価成否を audit attributes に詰めて発火する（FR-T1-DECISION-003）。
        let mut attrs = std::collections::BTreeMap::new();
        attrs.insert("decision_table_name".to_string(), r.rule_id.clone());
        // 空 rule_version は registry が "(latest)" を解決済みのはずだが、ここでは
        // request に来た値をそのまま記録する（latest 解決後の version は別途記録）。
        attrs.insert("rule_version".to_string(), r.rule_version.clone());
        attrs.insert("input_hash".to_string(), hash_b64(&r.input_json));
        attrs.insert("input_bytes".to_string(), r.input_json.len().to_string());
        let (resp_or_err, code) = match outcome_result {
            Ok(outcome) => {
                attrs.insert("output_hash".to_string(), hash_b64(&outcome.output_json));
                attrs.insert(
                    "output_bytes".to_string(),
                    outcome.output_json.len().to_string(),
                );
                attrs.insert("elapsed_us".to_string(), outcome.elapsed_us.to_string());
                let resp = EvaluateResponse {
                    output_json: outcome.output_json,
                    trace_json: outcome.trace_json,
                    elapsed_us: outcome.elapsed_us,
                };
                (Ok(Response::new(resp)), tonic::Code::Ok)
            }
            Err(e) => {
                let st = registry_err_to_status(e, "Evaluate");
                let code = st.code();
                (Err(st), code)
            }
        };
        // 詳細 audit record を fire（K1s0Layer の基本 record とは別軸の補足）。
        let rec = AuditRecord {
            tenant_id: claims.tenant_id.clone(),
            actor: if claims.subject.is_empty() {
                "unknown".to_string()
            } else {
                claims.subject.clone()
            },
            action: "k1s0.tier1.decision.v1.DecisionService/Evaluate".to_string(),
            resource: format!("decision:{}", r.rule_id),
            outcome: outcome_from_code(code).to_string(),
            code: code as i32,
            attributes: attrs,
        };
        self.audit_emitter.emit(rec).await;
        resp_or_err
    }

    async fn batch_evaluate(
        &self,
        req: Request<BatchEvaluateRequest>,
    ) -> Result<Response<BatchEvaluateResponse>, Status> {
        let claims = claims_from_req(&req);
        let r = req.into_inner();
        let mut outputs: Vec<Vec<u8>> = Vec::with_capacity(r.inputs_json.len());
        let mut total_input_bytes: usize = 0;
        let mut total_output_bytes: usize = 0;
        for input in r.inputs_json.iter() {
            total_input_bytes += input.len();
            let outcome = self
                .registry
                .evaluate(&r.rule_id, &r.rule_version, input, false)
                .await
                .map_err(|e| registry_err_to_status(e, "BatchEvaluate"))?;
            total_output_bytes += outcome.output_json.len();
            outputs.push(outcome.output_json);
        }
        // FR-T1-DECISION-003: BatchEvaluate も詳細 audit を発火する。個別 input/output の hash は
        // 1 record に集約できないため、件数 + 合算 size のみを attributes に詰める。
        let mut attrs = std::collections::BTreeMap::new();
        attrs.insert("decision_table_name".to_string(), r.rule_id.clone());
        attrs.insert("rule_version".to_string(), r.rule_version.clone());
        attrs.insert("batch_size".to_string(), r.inputs_json.len().to_string());
        attrs.insert("input_bytes".to_string(), total_input_bytes.to_string());
        attrs.insert("output_bytes".to_string(), total_output_bytes.to_string());
        let rec = AuditRecord {
            tenant_id: claims.tenant_id.clone(),
            actor: if claims.subject.is_empty() {
                "unknown".to_string()
            } else {
                claims.subject.clone()
            },
            action: "k1s0.tier1.decision.v1.DecisionService/BatchEvaluate".to_string(),
            resource: format!("decision:{}", r.rule_id),
            outcome: outcome_from_code(tonic::Code::Ok).to_string(),
            code: tonic::Code::Ok as i32,
            attributes: attrs,
        };
        self.audit_emitter.emit(rec).await;
        Ok(Response::new(BatchEvaluateResponse {
            outputs_json: outputs,
        }))
    }
}

#[tonic::async_trait]
impl DecisionAdminService for DecisionAdminServer {
    async fn register_rule(
        &self,
        req: Request<RegisterRuleRequest>,
    ) -> Result<Response<RegisterRuleResponse>, Status> {
        let claims = claims_from_req(&req);
        let r = req.into_inner();
        let registered_by = r
            .context
            .as_ref()
            .map(|c| c.subject.clone())
            .unwrap_or_default();
        let rule_id_for_audit = r.rule_id.clone();
        let commit_hash_for_audit = r.commit_hash.clone();
        let jdm_hash = hash_b64(&r.jdm_document);
        let outcome = self
            .registry
            .register(RegisterInput {
                rule_id: r.rule_id,
                jdm_document: r.jdm_document,
                commit_hash: r.commit_hash,
                registered_by,
                registered_at_ms: 0, // now() で埋める
            })
            .map_err(|e| registry_err_to_status(e, "RegisterRule"))?;
        // FR-T1-DECISION-002 / 003: 登録された rule_version を audit attributes に詰める。
        let mut attrs = std::collections::BTreeMap::new();
        attrs.insert("decision_table_name".to_string(), rule_id_for_audit.clone());
        attrs.insert("rule_version".to_string(), outcome.rule_version.clone());
        attrs.insert("commit_hash".to_string(), commit_hash_for_audit);
        attrs.insert("jdm_hash".to_string(), jdm_hash);
        let rec = AuditRecord {
            tenant_id: claims.tenant_id.clone(),
            actor: if claims.subject.is_empty() {
                "unknown".to_string()
            } else {
                claims.subject.clone()
            },
            action: "k1s0.tier1.decision.v1.DecisionAdminService/RegisterRule".to_string(),
            resource: format!("decision:{}", rule_id_for_audit),
            outcome: outcome_from_code(tonic::Code::Ok).to_string(),
            code: tonic::Code::Ok as i32,
            attributes: attrs,
        };
        self.audit_emitter.emit(rec).await;
        Ok(Response::new(RegisterRuleResponse {
            rule_version: outcome.rule_version,
            effective_at_ms: outcome.effective_at_ms,
        }))
    }

    async fn list_versions(
        &self,
        req: Request<ListVersionsRequest>,
    ) -> Result<Response<ListVersionsResponse>, Status> {
        let r = req.into_inner();
        let metas = self
            .registry
            .list_versions(&r.rule_id)
            .map_err(|e| registry_err_to_status(e, "ListVersions"))?;
        Ok(Response::new(ListVersionsResponse {
            versions: metas.iter().map(rule_meta_to_proto).collect(),
        }))
    }

    async fn get_rule(
        &self,
        req: Request<GetRuleRequest>,
    ) -> Result<Response<GetRuleResponse>, Status> {
        let r = req.into_inner();
        let (jdm, meta) = self
            .registry
            .get_jdm_with_meta(&r.rule_id, &r.rule_version)
            .map_err(|e| registry_err_to_status(e, "GetRule"))?;
        Ok(Response::new(GetRuleResponse {
            jdm_document: jdm,
            meta: Some(rule_meta_to_proto(&meta)),
        }))
    }
}

async fn shutdown_signal() {
    let mut sigterm = signal(SignalKind::terminate()).expect("install SIGTERM handler");
    let mut sigint = signal(SignalKind::interrupt()).expect("install SIGINT handler");
    tokio::select! {
        _ = sigterm.recv() => { eprintln!("tier1/decision: received SIGTERM, shutting down"); },
        _ = sigint.recv() => { eprintln!("tier1/decision: received SIGINT, shutting down"); },
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // DS-SW-COMP-109: 共通 OTel 初期化。OTEL_EXPORTER_OTLP_ENDPOINT が設定済なら OTLP gRPC
    // 直送、未設定なら fmt layer のみ。Guard は main 関数の生存期間中保持する。
    let _otel_guard = k1s0_tier1_otel::init("t1-decision", "k1s0-tier1");
    let listen = listen_addr();
    let addr = listen.parse()?;
    eprintln!("tier1/decision: gRPC server listening on {}", listen);
    let registry = Arc::new(RuleRegistry::new());
    // 評価 / 登録時の rich audit emit に使う共通 emitter を main 側で確保する。
    // CommonRuntime::from_env() が下流で呼ばれるが、emitter のみ先に組み立てる。
    let audit_emitter = k1s0_tier1_common::runtime::load_audit_emitter_from_env();

    // FR-T1-DECISION-004: ConfigMap mount された JDM ファイル群を hot reload する。
    // 環境変数 `DECISION_JDM_DIR` で監視ディレクトリを指定する（既定 /etc/k1s0/decisions）。
    // 起動時にディレクトリ全件を register、以後 fs イベントで自動再登録する。
    // 値が "off" の場合は hot reload 機能を無効化する（test / 単体起動経路）。
    let _hot_reload_handle: Option<tokio::task::JoinHandle<()>> = {
        let dir_env = std::env::var("DECISION_JDM_DIR")
            .unwrap_or_else(|_| "/etc/k1s0/decisions".to_string());
        if dir_env == "off" {
            eprintln!("tier1/decision: hot-reload disabled (DECISION_JDM_DIR=off)");
            None
        } else {
            let dir = std::path::PathBuf::from(dir_env);
            // 起動時 load: 失敗ファイルは warn ログのみで継続。
            match k1s0_tier1_decision::loader::load_initial(&registry, &dir, "hot-reload") {
                Ok((ok, errors)) => {
                    eprintln!(
                        "tier1/decision: initial load loaded={} failed={} dir={}",
                        ok,
                        errors.len(),
                        dir.display()
                    );
                    for (path, err) in &errors {
                        eprintln!(
                            "tier1/decision: initial load error path={} error={}",
                            path.display(),
                            err
                        );
                    }
                }
                Err(e) => {
                    eprintln!(
                        "tier1/decision: initial load failed dir={} error={}",
                        dir.display(),
                        e
                    );
                }
            }
            // watcher 起動: ディレクトリ未存在時は失敗するため warn して None で継続。
            match k1s0_tier1_decision::loader::spawn_watcher(
                registry.clone(),
                dir.clone(),
                "hot-reload".to_string(),
            ) {
                Ok(h) => {
                    eprintln!(
                        "tier1/decision: hot-reload watcher started dir={}",
                        dir.display()
                    );
                    Some(h)
                }
                Err(e) => {
                    eprintln!(
                        "tier1/decision: hot-reload watcher start failed dir={} error={}",
                        dir.display(),
                        e
                    );
                    None
                }
            }
        }
    };
    let dec = DecisionServer {
        registry: registry.clone(),
        audit_emitter: audit_emitter.clone(),
    };
    let admin = DecisionAdminServer {
        registry,
        audit_emitter: audit_emitter.clone(),
    };
    // gRPC Server Reflection を有効化する（grpcurl の `list` / `describe` 対応、
    // Go Pod 側の reflection.Register と機能等価）。
    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build_v1()?;
    // 共通 HealthService を構築する。decision Pod 自体は ZEN engine in-memory のため
    // 依存先 probe は空（リリース時点）。Postgres backed registry に切替時は probe 追加予定。
    let health = HealthSvc::new(env!("CARGO_PKG_VERSION").to_string(), vec![]);
    // docs §共通規約 に従う interceptor chain を構築。
    let rt = CommonRuntime::from_env();
    let layer = K1s0Layer::new(rt.auth.clone(), rt.rate_limiter.clone(), rt.audit_emitter.clone());

    // HTTP/JSON gateway（TIER1_HTTP_LISTEN_ADDR が設定されている場合のみ起動）。
    // 共通規約 §「HTTP/JSON 互換」: DecisionService 2 RPC + DecisionAdminService 3 RPC を
    // JSON で公開する（5 unary RPC、bytes フィールドは base64 で表現）。
    let http_handle: Option<tokio::task::JoinHandle<()>> =
        match std::env::var("TIER1_HTTP_LISTEN_ADDR").ok().filter(|s| !s.is_empty()) {
            Some(http_addr) => {
                let http_state = DecisionHttpState {
                    registry: dec.registry.clone(),
                };
                let gateway = HttpGateway::new(
                    rt.auth.clone(),
                    rt.rate_limiter.clone(),
                    rt.audit_emitter.clone(),
                )
                .register(Arc::new(EvaluateRpc { state: http_state.clone() }) as Arc<dyn JsonRpc>)
                .register(Arc::new(BatchEvaluateRpc { state: http_state.clone() }) as Arc<dyn JsonRpc>)
                .register(Arc::new(RegisterRuleRpc { state: http_state.clone() }) as Arc<dyn JsonRpc>)
                .register(Arc::new(ListVersionsRpc { state: http_state.clone() }) as Arc<dyn JsonRpc>)
                .register(Arc::new(GetRuleRpc { state: http_state }) as Arc<dyn JsonRpc>);
                let router = gateway.into_router();
                eprintln!("tier1/decision: HTTP/JSON gateway listening on {}", http_addr);
                let addr_for_task = http_addr.clone();
                Some(tokio::spawn(async move {
                    if let Err(e) = serve_http(&addr_for_task, router).await {
                        eprintln!("tier1/decision: HTTP gateway error: {}", e);
                    }
                }))
            }
            None => None,
        };

    // 標準 grpc.health.v1.Health プロトコル登録（K8s grpc liveness/readiness probe 用）。
    let (mut health_reporter, health_svc) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<DecisionServiceServer<DecisionServer>>()
        .await;

    Server::builder()
        .layer(layer)
        .add_service(DecisionServiceServer::new(dec))
        .add_service(DecisionAdminServiceServer::new(admin))
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

    fn make_servers() -> (DecisionServer, DecisionAdminServer) {
        let r = Arc::new(RuleRegistry::new());
        let emitter: Arc<dyn AuditEmitter> =
            Arc::new(k1s0_tier1_common::audit::NoopAuditEmitter);
        (
            DecisionServer {
                registry: r.clone(),
                audit_emitter: emitter.clone(),
            },
            DecisionAdminServer {
                registry: r,
                audit_emitter: emitter,
            },
        )
    }

    /// 最小 JDM: input → expression(key = expr) → output。
    /// 業務担当者が gorules Editor で生成する 3 ノード 2 エッジの構造。
    fn jdm_with_one_expression(key: &str, expr: &str) -> Vec<u8> {
        serde_json::json!({
            "nodes": [
                {"id": "n_in", "name": "in", "type": "inputNode", "content": {}},
                {"id": "n_ex", "name": "calc", "type": "expressionNode", "content": {
                    "expressions": [
                        {"id": "e1", "key": key, "value": expr}
                    ]
                }},
                {"id": "n_out", "name": "out", "type": "outputNode", "content": {}}
            ],
            "edges": [
                {"id": "ed1", "sourceId": "n_in",  "targetId": "n_ex", "type": "edge"},
                {"id": "ed2", "sourceId": "n_ex", "targetId": "n_out", "type": "edge"}
            ]
        }).to_string().into_bytes()
    }

    #[tokio::test]
    async fn register_then_evaluate_roundtrip() {
        let (dec, admin) = make_servers();
        let rule = jdm_with_one_expression("tax", "amount * 0.10");
        admin
            .register_rule(Request::new(RegisterRuleRequest {
                rule_id: "tax-calc".into(),
                jdm_document: rule,
                ..Default::default()
            }))
            .await
            .unwrap();
        let resp = dec
            .evaluate(Request::new(EvaluateRequest {
                rule_id: "tax-calc".into(),
                rule_version: "v1".into(),
                input_json: br#"{"amount": 100}"#.to_vec(),
                include_trace: false,
                ..Default::default()
            }))
            .await
            .unwrap()
            .into_inner();
        let out: serde_json::Value = serde_json::from_slice(&resp.output_json).unwrap();
        assert_eq!(out["tax"], serde_json::json!(10));
    }

    #[tokio::test]
    async fn batch_evaluate_processes_all_inputs() {
        let (dec, admin) = make_servers();
        admin
            .register_rule(Request::new(RegisterRuleRequest {
                rule_id: "rid".into(),
                jdm_document: jdm_with_one_expression("y", "x * 2"),
                ..Default::default()
            }))
            .await
            .unwrap();
        let resp = dec
            .batch_evaluate(Request::new(BatchEvaluateRequest {
                rule_id: "rid".into(),
                rule_version: "v1".into(),
                inputs_json: vec![
                    br#"{"x": 1}"#.to_vec(),
                    br#"{"x": 2}"#.to_vec(),
                    br#"{"x": 3}"#.to_vec(),
                ],
                ..Default::default()
            }))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(resp.outputs_json.len(), 3);
        let v0: serde_json::Value = serde_json::from_slice(&resp.outputs_json[0]).unwrap();
        let v2: serde_json::Value = serde_json::from_slice(&resp.outputs_json[2]).unwrap();
        assert_eq!(v0["y"], serde_json::json!(2));
        assert_eq!(v2["y"], serde_json::json!(6));
    }

    #[tokio::test]
    async fn evaluate_unknown_rule_returns_not_found() {
        let (dec, _admin) = make_servers();
        let r = dec
            .evaluate(Request::new(EvaluateRequest {
                rule_id: "missing".into(),
                rule_version: "v1".into(),
                input_json: br#"{}"#.to_vec(),
                include_trace: false,
                ..Default::default()
            }))
            .await;
        assert!(r.is_err());
        assert_eq!(r.err().unwrap().code(), tonic::Code::NotFound);
    }

    #[tokio::test]
    async fn list_versions_returns_registered() {
        let (_dec, admin) = make_servers();
        for _ in 0..3 {
            admin
                .register_rule(Request::new(RegisterRuleRequest {
                    rule_id: "rid".into(),
                    jdm_document: jdm_with_one_expression("y", "1"),
                    ..Default::default()
                }))
                .await
                .unwrap();
        }
        let resp = admin
            .list_versions(Request::new(ListVersionsRequest {
                rule_id: "rid".into(),
                ..Default::default()
            }))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(resp.versions.len(), 3);
    }
}
