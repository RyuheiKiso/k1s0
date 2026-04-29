// 本ファイルは t1-decision Pod の HTTP/JSON gateway 用 JsonRpc 実装。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/00_tier1_API共通規約.md
//     §「HTTP/JSON 互換インタフェース共通仕様」
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/09_Decision_API.md
//
// 役割:
//   DecisionService（Evaluate / BatchEvaluate）と DecisionAdminService
//   （RegisterRule / ListVersions / GetRule）の 5 unary RPC を HTTP/JSON 経路
//   で公開する。bytes フィールドは protojson 慣例に従い base64 で扱う。

// 共通 gateway。
use k1s0_tier1_common::auth::AuthClaims;
use k1s0_tier1_common::http_gateway::JsonRpc;
// JSON 値型。
use serde_json::Value as JsonValue;
// registry。
use crate::registry::{RegisterInput, RegistryError, RuleRegistry};
// 標準。
use std::sync::Arc;

/// Decision HTTP gateway 用に共有する registry。
#[derive(Clone)]
pub struct DecisionHttpState {
    /// rule registry（Arc 共有）。
    pub registry: Arc<RuleRegistry>,
}

/// `Decision.Evaluate` adapter。
pub struct EvaluateRpc {
    pub state: DecisionHttpState,
}

#[async_trait::async_trait]
impl JsonRpc for EvaluateRpc {
    fn route(&self) -> &'static str {
        "decision/evaluate"
    }
    fn full_method(&self) -> &'static str {
        "/k1s0.tier1.decision.v1.DecisionService/Evaluate"
    }
    async fn invoke(
        &self,
        _claims: &AuthClaims,
        body: JsonValue,
    ) -> Result<JsonValue, tonic::Status> {
        let rule_id = pick_str(&body, "ruleId").unwrap_or_default();
        let rule_version = pick_str(&body, "ruleVersion").unwrap_or_default();
        let input_json = pick_bytes(&body, "inputJson")?;
        let include_trace = body
            .get("includeTrace")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let outcome = self
            .state
            .registry
            .evaluate(&rule_id, &rule_version, &input_json, include_trace)
            .await
            .map_err(|e| registry_err_to_status(e, "Evaluate"))?;
        Ok(serde_json::json!({
            "outputJson": base64_encode(&outcome.output_json),
            "traceJson": base64_encode(&outcome.trace_json),
            "elapsedUs": outcome.elapsed_us,
        }))
    }
}

/// `Decision.BatchEvaluate` adapter。
pub struct BatchEvaluateRpc {
    pub state: DecisionHttpState,
}

#[async_trait::async_trait]
impl JsonRpc for BatchEvaluateRpc {
    fn route(&self) -> &'static str {
        "decision/batchevaluate"
    }
    fn full_method(&self) -> &'static str {
        "/k1s0.tier1.decision.v1.DecisionService/BatchEvaluate"
    }
    async fn invoke(
        &self,
        _claims: &AuthClaims,
        body: JsonValue,
    ) -> Result<JsonValue, tonic::Status> {
        let rule_id = pick_str(&body, "ruleId").unwrap_or_default();
        let rule_version = pick_str(&body, "ruleVersion").unwrap_or_default();
        let inputs = body
            .get("inputsJson")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let mut outputs: Vec<String> = Vec::with_capacity(inputs.len());
        for input in inputs.iter() {
            let bytes = base64_decode(input.as_str().unwrap_or(""))?;
            let outcome = self
                .state
                .registry
                .evaluate(&rule_id, &rule_version, &bytes, false)
                .await
                .map_err(|e| registry_err_to_status(e, "BatchEvaluate"))?;
            outputs.push(base64_encode(&outcome.output_json));
        }
        Ok(serde_json::json!({ "outputsJson": outputs }))
    }
}

/// `DecisionAdmin.RegisterRule` adapter。
pub struct RegisterRuleRpc {
    pub state: DecisionHttpState,
}

#[async_trait::async_trait]
impl JsonRpc for RegisterRuleRpc {
    fn route(&self) -> &'static str {
        "decision/registerrule"
    }
    fn full_method(&self) -> &'static str {
        "/k1s0.tier1.decision.v1.DecisionAdminService/RegisterRule"
    }
    async fn invoke(
        &self,
        claims: &AuthClaims,
        body: JsonValue,
    ) -> Result<JsonValue, tonic::Status> {
        let rule_id = pick_str(&body, "ruleId").unwrap_or_default();
        let jdm = pick_bytes(&body, "jdmDocument")?;
        let commit_hash = pick_str(&body, "commitHash").unwrap_or_default();
        let outcome = self
            .state
            .registry
            .register(RegisterInput {
                rule_id,
                jdm_document: jdm,
                commit_hash,
                registered_by: claims.subject.clone(),
                registered_at_ms: 0,
            })
            .map_err(|e| registry_err_to_status(e, "RegisterRule"))?;
        Ok(serde_json::json!({
            "ruleVersion": outcome.rule_version,
            "effectiveAtMs": outcome.effective_at_ms,
        }))
    }
}

/// `DecisionAdmin.ListVersions` adapter。
pub struct ListVersionsRpc {
    pub state: DecisionHttpState,
}

#[async_trait::async_trait]
impl JsonRpc for ListVersionsRpc {
    fn route(&self) -> &'static str {
        "decision/listversions"
    }
    fn full_method(&self) -> &'static str {
        "/k1s0.tier1.decision.v1.DecisionAdminService/ListVersions"
    }
    async fn invoke(
        &self,
        _claims: &AuthClaims,
        body: JsonValue,
    ) -> Result<JsonValue, tonic::Status> {
        let rule_id = pick_str(&body, "ruleId").unwrap_or_default();
        let metas = self
            .state
            .registry
            .list_versions(&rule_id)
            .map_err(|e| registry_err_to_status(e, "ListVersions"))?;
        let versions: Vec<JsonValue> = metas
            .iter()
            .map(|m| {
                serde_json::json!({
                    "ruleVersion": m.rule_version,
                    "commitHash": m.commit_hash,
                    "registeredAtMs": m.registered_at_ms,
                    "registeredBy": m.registered_by,
                    "deprecated": false,
                })
            })
            .collect();
        Ok(serde_json::json!({ "versions": versions }))
    }
}

/// `DecisionAdmin.GetRule` adapter。
pub struct GetRuleRpc {
    pub state: DecisionHttpState,
}

#[async_trait::async_trait]
impl JsonRpc for GetRuleRpc {
    fn route(&self) -> &'static str {
        "decision/getrule"
    }
    fn full_method(&self) -> &'static str {
        "/k1s0.tier1.decision.v1.DecisionAdminService/GetRule"
    }
    async fn invoke(
        &self,
        _claims: &AuthClaims,
        body: JsonValue,
    ) -> Result<JsonValue, tonic::Status> {
        let rule_id = pick_str(&body, "ruleId").unwrap_or_default();
        let rule_version = pick_str(&body, "ruleVersion").unwrap_or_default();
        let (jdm, meta) = self
            .state
            .registry
            .get_jdm_with_meta(&rule_id, &rule_version)
            .map_err(|e| registry_err_to_status(e, "GetRule"))?;
        Ok(serde_json::json!({
            "jdmDocument": base64_encode(&jdm),
            "meta": {
                "ruleVersion": meta.rule_version,
                "commitHash": meta.commit_hash,
                "registeredAtMs": meta.registered_at_ms,
                "registeredBy": meta.registered_by,
                "deprecated": false,
            },
        }))
    }
}

/// JSON object から文字列フィールドを取り出す（snake_case フォールバックも対応）。
fn pick_str(body: &JsonValue, camel: &str) -> Option<String> {
    body.get(camel)
        .or_else(|| body.get(camel_to_snake(camel)))
        .and_then(|v| v.as_str().map(String::from))
}

/// camelCase → snake_case 単純変換（"ruleId" → "rule_id"）。
fn camel_to_snake(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
    for (i, c) in s.chars().enumerate() {
        if c.is_ascii_uppercase() && i > 0 {
            out.push('_');
        }
        out.push(c.to_ascii_lowercase());
    }
    out
}

/// JSON 文字列（base64）→ Vec<u8>。protojson の bytes 仕様。
fn pick_bytes(body: &JsonValue, camel: &str) -> Result<Vec<u8>, tonic::Status> {
    let s = body
        .get(camel)
        .or_else(|| body.get(camel_to_snake(camel)))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    base64_decode(s)
}

/// 自前 base64 encoder（外部 crate を増やさないため）。
/// 入力 bytes を Standard alphabet で encode し、padding ('=') 込みの ASCII 文字列を返す。
fn base64_encode(input: &[u8]) -> String {
    const ALPH: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(((input.len() + 2) / 3) * 4);
    let chunks = input.chunks(3);
    for chunk in chunks {
        let b0 = chunk[0];
        let b1 = chunk.get(1).copied().unwrap_or(0);
        let b2 = chunk.get(2).copied().unwrap_or(0);
        let n = ((b0 as u32) << 16) | ((b1 as u32) << 8) | (b2 as u32);
        out.push(ALPH[((n >> 18) & 0x3f) as usize] as char);
        out.push(ALPH[((n >> 12) & 0x3f) as usize] as char);
        if chunk.len() >= 2 {
            out.push(ALPH[((n >> 6) & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() == 3 {
            out.push(ALPH[(n & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

/// 自前 base64 decoder。空文字は空 Vec を返す。
fn base64_decode(s: &str) -> Result<Vec<u8>, tonic::Status> {
    if s.is_empty() {
        return Ok(Vec::new());
    }
    let bytes = s.as_bytes();
    if bytes.len() % 4 != 0 {
        return Err(tonic::Status::invalid_argument(
            "tier1/decision/http: base64 length must be multiple of 4",
        ));
    }
    let val = |c: u8| -> Result<u32, tonic::Status> {
        match c {
            b'A'..=b'Z' => Ok((c - b'A') as u32),
            b'a'..=b'z' => Ok((c - b'a' + 26) as u32),
            b'0'..=b'9' => Ok((c - b'0' + 52) as u32),
            b'+' => Ok(62),
            b'/' => Ok(63),
            b'=' => Ok(0),
            _ => Err(tonic::Status::invalid_argument(
                "tier1/decision/http: invalid base64 character",
            )),
        }
    };
    let mut out = Vec::with_capacity((bytes.len() / 4) * 3);
    for chunk in bytes.chunks(4) {
        let n = (val(chunk[0])? << 18)
            | (val(chunk[1])? << 12)
            | (val(chunk[2])? << 6)
            | val(chunk[3])?;
        out.push((n >> 16) as u8);
        if chunk[2] != b'=' {
            out.push((n >> 8) as u8);
        }
        if chunk[3] != b'=' {
            out.push(n as u8);
        }
    }
    Ok(out)
}

/// RegistryError → tonic::Status。
fn registry_err_to_status(e: RegistryError, rpc: &str) -> tonic::Status {
    match e {
        RegistryError::InvalidJson(msg) | RegistryError::InvalidRule(msg) => {
            tonic::Status::invalid_argument(format!("tier1/decision: {}: {}", rpc, msg))
        }
        RegistryError::NotFound { rule_id, rule_version } => tonic::Status::not_found(format!(
            "tier1/decision: {}: rule {}/{} not found",
            rpc, rule_id, rule_version
        )),
        RegistryError::EvalFailed(msg) => {
            tonic::Status::internal(format!("tier1/decision: {}: eval failed: {}", rpc, msg))
        }
        RegistryError::LockPoisoned => {
            tonic::Status::internal(format!("tier1/decision: {}: lock poisoned", rpc))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_round_trip() {
        let data: &[&[u8]] = &[
            b"",
            b"a",
            b"ab",
            b"abc",
            b"hello, world",
            b"\x00\x01\x02\x03\xfe\xff",
        ];
        for d in data {
            let enc = base64_encode(d);
            let dec = base64_decode(&enc).unwrap();
            assert_eq!(dec, *d, "round-trip failed for {:?}", d);
        }
    }

    #[test]
    fn camel_to_snake_handles_words() {
        assert_eq!(camel_to_snake("ruleId"), "rule_id");
        assert_eq!(camel_to_snake("includeTrace"), "include_trace");
        assert_eq!(camel_to_snake("inputsJson"), "inputs_json");
    }

    #[test]
    fn pick_str_uses_camel_or_snake() {
        let v = serde_json::json!({ "rule_id": "r1" });
        assert_eq!(pick_str(&v, "ruleId").as_deref(), Some("r1"));
        let v2 = serde_json::json!({ "ruleId": "r2" });
        assert_eq!(pick_str(&v2, "ruleId").as_deref(), Some("r2"));
    }
}
