// 本ファイルは service.rs（DecisionService / DecisionAdminService 実装）の単体テスト。
//
// service.rs から `#[path = "service_tests.rs"] mod tests;` で取り込まれるため、
// `super::*` で service.rs の private 項目（resolve_tenant など）にもアクセスできる。
// registry.rs と同じ分割パターン（src/CLAUDE.md: 1 ファイル 500 行以内）。

use super::*;
use k1s0_sdk_proto::k1s0::tier1::common::v1::TenantContext;
use k1s0_sdk_proto::k1s0::tier1::decision::v1::{
    BatchEvaluateRequest, EvaluateRequest, GetRuleRequest, ListVersionsRequest,
    RegisterRuleRequest,
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

/// claims (JWT 由来) を `Request::extensions` に注入する helper。
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

fn ctx(tenant: &str) -> TenantContext {
    TenantContext {
        tenant_id: tenant.to_string(),
        ..Default::default()
    }
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
        .register_rule(req_with_claims(
            RegisterRuleRequest {
                rule_id: "tax-calc".into(),
                jdm_document: rule,
                context: Some(ctx("tenant-A")),
                ..Default::default()
            },
            "tenant-A",
        ))
        .await
        .unwrap();
    let resp = dec
        .evaluate(req_with_claims(
            EvaluateRequest {
                rule_id: "tax-calc".into(),
                rule_version: "v1".into(),
                input_json: br#"{"amount": 100}"#.to_vec(),
                include_trace: false,
                context: Some(ctx("tenant-A")),
            },
            "tenant-A",
        ))
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
        .register_rule(req_with_claims(
            RegisterRuleRequest {
                rule_id: "rid".into(),
                jdm_document: jdm_with_one_expression("y", "x * 2"),
                context: Some(ctx("t")),
                ..Default::default()
            },
            "t",
        ))
        .await
        .unwrap();
    let resp = dec
        .batch_evaluate(req_with_claims(
            BatchEvaluateRequest {
                rule_id: "rid".into(),
                rule_version: "v1".into(),
                inputs_json: vec![
                    br#"{"x": 1}"#.to_vec(),
                    br#"{"x": 2}"#.to_vec(),
                    br#"{"x": 3}"#.to_vec(),
                ],
                context: Some(ctx("t")),
            },
            "t",
        ))
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
        .evaluate(req_with_claims(
            EvaluateRequest {
                rule_id: "missing".into(),
                rule_version: "v1".into(),
                input_json: br#"{}"#.to_vec(),
                include_trace: false,
                context: Some(ctx("t")),
            },
            "t",
        ))
        .await;
    assert!(r.is_err());
    assert_eq!(r.err().unwrap().code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn list_versions_returns_registered() {
    let (_dec, admin) = make_servers();
    for _ in 0..3 {
        admin
            .register_rule(req_with_claims(
                RegisterRuleRequest {
                    rule_id: "rid".into(),
                    jdm_document: jdm_with_one_expression("y", "1"),
                    context: Some(ctx("t")),
                    ..Default::default()
                },
                "t",
            ))
            .await
            .unwrap();
    }
    let resp = admin
        .list_versions(req_with_claims(
            ListVersionsRequest {
                rule_id: "rid".into(),
                context: Some(ctx("t")),
            },
            "t",
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp.versions.len(), 3);
}

/// NFR-E-AC-003: claims (JWT) と body.context の tenant_id 不一致は PermissionDenied。
#[tokio::test]
async fn evaluate_rejects_cross_tenant() {
    let (dec, admin) = make_servers();
    // tenant-A で rule を登録。
    admin
        .register_rule(req_with_claims(
            RegisterRuleRequest {
                rule_id: "shared".into(),
                jdm_document: jdm_with_one_expression("y", "1"),
                context: Some(ctx("tenant-A")),
                ..Default::default()
            },
            "tenant-A",
        ))
        .await
        .unwrap();
    // tenant-B から（claims=B、body=A 偽装） → PermissionDenied。
    let err = dec
        .evaluate(req_with_claims(
            EvaluateRequest {
                rule_id: "shared".into(),
                rule_version: "v1".into(),
                input_json: br#"{}"#.to_vec(),
                include_trace: false,
                context: Some(ctx("tenant-A")),
            },
            "tenant-B",
        ))
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::PermissionDenied);
    assert!(
        err.message().contains("cross-tenant"),
        "expected 'cross-tenant' in error, got: {}",
        err.message()
    );
}

/// NFR-E-AC-003: tenant-A で登録した rule_id を tenant-B から evaluate すると NotFound。
/// （越境試行が claims=B, body=B で行われた場合の検証。同名 rule_id は構造的に分離。）
#[tokio::test]
async fn evaluate_isolates_rules_between_tenants() {
    let (dec, admin) = make_servers();
    admin
        .register_rule(req_with_claims(
            RegisterRuleRequest {
                rule_id: "shared".into(),
                jdm_document: jdm_with_one_expression("y", "1"),
                context: Some(ctx("tenant-A")),
                ..Default::default()
            },
            "tenant-A",
        ))
        .await
        .unwrap();
    // tenant-B から自己 tenant 内のリクエストとして呼ぶ → NotFound（構造的分離の確認）。
    let err = dec
        .evaluate(req_with_claims(
            EvaluateRequest {
                rule_id: "shared".into(),
                rule_version: "".into(),
                input_json: br#"{}"#.to_vec(),
                include_trace: false,
                context: Some(ctx("tenant-B")),
            },
            "tenant-B",
        ))
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::NotFound);
}

/// NFR-E-AC-003: GetRule も tenant 越境すると NotFound（情報漏洩防止）。
#[tokio::test]
async fn get_rule_isolates_between_tenants() {
    let (_dec, admin) = make_servers();
    admin
        .register_rule(req_with_claims(
            RegisterRuleRequest {
                rule_id: "shared".into(),
                jdm_document: jdm_with_one_expression("y", "1"),
                context: Some(ctx("tenant-A")),
                ..Default::default()
            },
            "tenant-A",
        ))
        .await
        .unwrap();
    // tenant-B 自身として GetRule。
    let err = admin
        .get_rule(req_with_claims(
            GetRuleRequest {
                rule_id: "shared".into(),
                rule_version: "v1".into(),
                context: Some(ctx("tenant-B")),
            },
            "tenant-B",
        ))
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::NotFound);
}

/// NFR-E-AC-003: ListVersions は当該 tenant 配下の登録のみ返す（漏洩防止）。
#[tokio::test]
async fn list_versions_isolates_between_tenants() {
    let (_dec, admin) = make_servers();
    // tenant-A に 2 件、tenant-B に 1 件。
    for _ in 0..2 {
        admin
            .register_rule(req_with_claims(
                RegisterRuleRequest {
                    rule_id: "shared".into(),
                    jdm_document: jdm_with_one_expression("y", "1"),
                    context: Some(ctx("tenant-A")),
                    ..Default::default()
                },
                "tenant-A",
            ))
            .await
            .unwrap();
    }
    admin
        .register_rule(req_with_claims(
            RegisterRuleRequest {
                rule_id: "shared".into(),
                jdm_document: jdm_with_one_expression("y", "1"),
                context: Some(ctx("tenant-B")),
                ..Default::default()
            },
            "tenant-B",
        ))
        .await
        .unwrap();
    // tenant-B からは 1 件しか見えない。
    let resp = admin
        .list_versions(req_with_claims(
            ListVersionsRequest {
                rule_id: "shared".into(),
                context: Some(ctx("tenant-B")),
            },
            "tenant-B",
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp.versions.len(), 1);
}
