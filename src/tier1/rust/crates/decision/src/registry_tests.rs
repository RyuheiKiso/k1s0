// 本ファイルは registry.rs の単体テスト一式（500 行規約に従い分離）。
//
// registry.rs の `#[cfg(test)] mod tests;` 経由で取り込まれるため、
// `super::*` で registry.rs の private 項目（CompiledRule など）にもアクセスできる。

use super::*;

/// 最小 JDM: input → expression(tax = amount * 0.10) → output。
/// 業務担当者が gorules Editor で生成した形式と同等の 3 ノード 2 エッジ構造。
fn simple_jdm() -> Vec<u8> {
    serde_json::json!({
        "nodes": [
            {"id": "n_in", "name": "request", "type": "inputNode", "content": {}},
            {"id": "n_ex", "name": "calc", "type": "expressionNode", "content": {
                "expressions": [
                    {"id": "e1", "key": "tax", "value": "amount * 0.10"}
                ]
            }},
            {"id": "n_out", "name": "result", "type": "outputNode", "content": {}}
        ],
        "edges": [
            {"id": "ed1", "sourceId": "n_in",  "targetId": "n_ex", "type": "edge"},
            {"id": "ed2", "sourceId": "n_ex", "targetId": "n_out", "type": "edge"}
        ]
    })
    .to_string()
    .into_bytes()
}

/// テスト用 RegisterInput を 1 行で組み立てる helper。
fn input(tenant: &str, rule_id: &str) -> RegisterInput {
    RegisterInput {
        tenant_id: tenant.into(),
        rule_id: rule_id.into(),
        jdm_document: simple_jdm(),
        ..Default::default()
    }
}

#[tokio::test]
async fn register_and_evaluate() {
    let r = RuleRegistry::new();
    r.register(input("tenant-A", "tax-calc")).unwrap();
    let outcome = r
        .evaluate("tenant-A", "tax-calc", "v1", br#"{"amount": 100}"#, false)
        .await
        .unwrap();
    let out: JsonValue = serde_json::from_slice(&outcome.output_json).unwrap();
    assert_eq!(out["tax"], serde_json::json!(10));
}

#[tokio::test]
async fn evaluate_with_trace() {
    let r = RuleRegistry::new();
    r.register(input("t", "rid")).unwrap();
    let outcome = r
        .evaluate("t", "rid", "v1", br#"{"amount": 50}"#, true)
        .await
        .unwrap();
    // ZEN Engine は trace を node 単位の HashMap で返す。最低 1 件含まれることを確認。
    assert!(!outcome.trace_json.is_empty());
    let trace: JsonValue = serde_json::from_slice(&outcome.trace_json).unwrap();
    assert!(trace.is_object());
}

#[tokio::test]
async fn evaluate_resolves_latest_when_version_empty() {
    let r = RuleRegistry::new();
    r.register(input("t", "rid")).unwrap();
    let outcome = r
        .evaluate("t", "rid", "", br#"{"amount": 100}"#, false)
        .await
        .unwrap();
    let out: JsonValue = serde_json::from_slice(&outcome.output_json).unwrap();
    assert_eq!(out["tax"], serde_json::json!(10));
}

#[test]
fn list_versions_returns_registered() {
    let r = RuleRegistry::new();
    r.register(input("t", "rid")).unwrap();
    r.register(input("t", "rid")).unwrap();
    let v = r.list_versions("t", "rid").unwrap();
    assert_eq!(v.len(), 2);
    assert!(v.iter().any(|m| m.rule_version == "v1"));
    assert!(v.iter().any(|m| m.rule_version == "v2"));
}

#[test]
fn register_invalid_json_returns_error() {
    let r = RuleRegistry::new();
    let e = r
        .register(RegisterInput {
            tenant_id: "t".into(),
            rule_id: "rid".into(),
            jdm_document: b"not-json".to_vec(),
            ..Default::default()
        })
        .unwrap_err();
    match e {
        RegistryError::InvalidJson(_) => {}
        other => panic!("expected InvalidJson, got {:?}", other),
    }
}

#[test]
fn register_empty_graph_returns_invalid_rule() {
    let r = RuleRegistry::new();
    let empty = serde_json::json!({"nodes": [], "edges": []})
        .to_string()
        .into_bytes();
    let e = r
        .register(RegisterInput {
            tenant_id: "t".into(),
            rule_id: "rid".into(),
            jdm_document: empty,
            ..Default::default()
        })
        .unwrap_err();
    match e {
        RegistryError::InvalidRule(_) => {}
        other => panic!("expected InvalidRule, got {:?}", other),
    }
}

#[tokio::test]
async fn evaluate_unknown_rule_returns_not_found() {
    let r = RuleRegistry::new();
    let e = r
        .evaluate("t", "missing", "v1", br#"{}"#, false)
        .await
        .unwrap_err();
    match e {
        RegistryError::NotFound { .. } => {}
        other => panic!("expected NotFound, got {:?}", other),
    }
}

#[tokio::test]
async fn evaluate_supports_boolean_logic() {
    let r = RuleRegistry::new();
    let rule = serde_json::json!({
        "nodes": [
            {"id": "n_in", "name": "in", "type": "inputNode", "content": {}},
            {"id": "n_ex", "name": "flags", "type": "expressionNode", "content": {
                "expressions": [
                    {"id": "e1", "key": "is_premium", "value": "amount >= 100"},
                    {"id": "e2", "key": "passes_kyc", "value": "score > 0.7 and verified == true"}
                ]
            }},
            {"id": "n_out", "name": "out", "type": "outputNode", "content": {}}
        ],
        "edges": [
            {"id": "ed1", "sourceId": "n_in",  "targetId": "n_ex", "type": "edge"},
            {"id": "ed2", "sourceId": "n_ex", "targetId": "n_out", "type": "edge"}
        ]
    }).to_string().into_bytes();
    r.register(RegisterInput {
        tenant_id: "t".into(),
        rule_id: "flags".into(),
        jdm_document: rule,
        ..Default::default()
    })
    .unwrap();
    let resp = r
        .evaluate(
            "t",
            "flags",
            "v1",
            br#"{"amount": 150, "score": 0.9, "verified": true}"#,
            false,
        )
        .await
        .unwrap();
    let out: JsonValue = serde_json::from_slice(&resp.output_json).unwrap();
    assert_eq!(out["is_premium"], serde_json::json!(true));
    assert_eq!(out["passes_kyc"], serde_json::json!(true));
}

#[test]
fn get_jdm_returns_serialized_rule() {
    let r = RuleRegistry::new();
    r.register(input("t", "rid")).unwrap();
    let (bytes, meta) = r.get_jdm_with_meta("t", "rid", "v1").unwrap();
    let v: JsonValue = serde_json::from_slice(&bytes).unwrap();
    // 元の JDM が保持されていることを確認。
    assert!(v.get("nodes").is_some());
    assert_eq!(meta.rule_version, "v1");
}

/// NFR-E-AC-003: tenant-A で登録した rule_id を tenant-B から evaluate しても
/// NotFound にならなければならない（rule_id は tenant 内で一意、別 tenant の
/// 同名 rule_id は構造的に分離される）。
#[tokio::test]
async fn evaluate_isolates_rules_between_tenants() {
    let r = RuleRegistry::new();
    r.register(input("tenant-A", "shared-name")).unwrap();
    let err = r
        .evaluate(
            "tenant-B",
            "shared-name",
            "",
            br#"{"amount": 100}"#,
            false,
        )
        .await
        .unwrap_err();
    match err {
        RegistryError::NotFound { tenant_id, rule_id, .. } => {
            assert_eq!(tenant_id, "tenant-B");
            assert_eq!(rule_id, "shared-name");
        }
        other => panic!("expected NotFound for cross-tenant, got {:?}", other),
    }
}

/// NFR-E-AC-003: 同じ rule_id を複数 tenant で登録しても、各 tenant 配下で v1 から
/// 独立採番される（tenant-A の v3 と tenant-B の v1 は別物）。
#[test]
fn version_numbering_is_per_tenant() {
    let r = RuleRegistry::new();
    let oa1 = r.register(input("tenant-A", "shared")).unwrap();
    let oa2 = r.register(input("tenant-A", "shared")).unwrap();
    let ob1 = r.register(input("tenant-B", "shared")).unwrap();
    assert_eq!(oa1.rule_version, "v1");
    assert_eq!(oa2.rule_version, "v2");
    assert_eq!(ob1.rule_version, "v1");
    // list_versions も tenant 単位。
    let a_versions = r.list_versions("tenant-A", "shared").unwrap();
    let b_versions = r.list_versions("tenant-B", "shared").unwrap();
    assert_eq!(a_versions.len(), 2);
    assert_eq!(b_versions.len(), 1);
}

/// NFR-E-AC-003: get_jdm_with_meta も tenant 境界で分離される。
#[test]
fn get_jdm_isolates_between_tenants() {
    let r = RuleRegistry::new();
    r.register(input("tenant-A", "rid")).unwrap();
    // tenant-A は読み出せる。
    let (_jdm, meta) = r.get_jdm_with_meta("tenant-A", "rid", "v1").unwrap();
    assert_eq!(meta.rule_version, "v1");
    // tenant-B からは NotFound。
    let err = r.get_jdm_with_meta("tenant-B", "rid", "v1").unwrap_err();
    match err {
        RegistryError::NotFound { tenant_id, .. } => assert_eq!(tenant_id, "tenant-B"),
        other => panic!("expected NotFound, got {:?}", other),
    }
}
