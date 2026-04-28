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
use k1s0_sdk_proto::k1s0::tier1::decision::v1::{
    BatchEvaluateRequest, BatchEvaluateResponse, EvaluateRequest, EvaluateResponse, GetRuleRequest,
    GetRuleResponse, ListVersionsRequest, ListVersionsResponse, RegisterRuleRequest,
    RegisterRuleResponse, RuleVersionMeta,
    decision_admin_service_server::{DecisionAdminService, DecisionAdminServiceServer},
    decision_service_server::{DecisionService, DecisionServiceServer},
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
}

struct DecisionAdminServer {
    registry: Arc<RuleRegistry>,
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
        let r = req.into_inner();
        let outcome = self
            .registry
            .evaluate(&r.rule_id, &r.rule_version, &r.input_json, r.include_trace)
            .await
            .map_err(|e| registry_err_to_status(e, "Evaluate"))?;
        Ok(Response::new(EvaluateResponse {
            output_json: outcome.output_json,
            trace_json: outcome.trace_json,
            elapsed_us: outcome.elapsed_us,
        }))
    }

    async fn batch_evaluate(
        &self,
        req: Request<BatchEvaluateRequest>,
    ) -> Result<Response<BatchEvaluateResponse>, Status> {
        let r = req.into_inner();
        let mut outputs: Vec<Vec<u8>> = Vec::with_capacity(r.inputs_json.len());
        for input in r.inputs_json.iter() {
            let outcome = self
                .registry
                .evaluate(&r.rule_id, &r.rule_version, input, false)
                .await
                .map_err(|e| registry_err_to_status(e, "BatchEvaluate"))?;
            outputs.push(outcome.output_json);
        }
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
        let r = req.into_inner();
        let registered_by = r
            .context
            .as_ref()
            .map(|c| c.subject.clone())
            .unwrap_or_default();
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
    let listen = listen_addr();
    let addr = listen.parse()?;
    eprintln!("tier1/decision: gRPC server listening on {}", listen);
    let registry = Arc::new(RuleRegistry::new());
    let dec = DecisionServer {
        registry: registry.clone(),
    };
    let admin = DecisionAdminServer { registry };
    // gRPC Server Reflection を有効化する（grpcurl の `list` / `describe` 対応、
    // Go Pod 側の reflection.Register と機能等価）。
    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build_v1()?;
    Server::builder()
        .add_service(DecisionServiceServer::new(dec))
        .add_service(DecisionAdminServiceServer::new(admin))
        .add_service(reflection)
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_servers() -> (DecisionServer, DecisionAdminServer) {
        let r = Arc::new(RuleRegistry::new());
        (
            DecisionServer {
                registry: r.clone(),
            },
            DecisionAdminServer { registry: r },
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
