// 本ファイルは service.rs（DecisionService / DecisionAdminService 実装）の単体テスト。
//
// service.rs から `#[path = "service_tests.rs"] mod tests;` で取り込まれる。
// registry.rs と同じ分割パターン（src/CLAUDE.md: 1 ファイル 500 行以内）。

use super::*;
use k1s0_sdk_proto::k1s0::tier1::decision::v1::{
    BatchEvaluateRequest, EvaluateRequest, ListVersionsRequest, RegisterRuleRequest,
};
use k1s0_tier1_common::audit::{AuditEmitter, NoopAuditEmitter};

fn make_servers() -> (DecisionServer, DecisionAdminServer) {
    let r = Arc::new(RuleRegistry::new());
    let emitter: Arc<dyn AuditEmitter> = Arc::new(NoopAuditEmitter);
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
    })
    .to_string()
    .into_bytes()
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
