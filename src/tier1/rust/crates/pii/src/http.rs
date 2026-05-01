// 本ファイルは t1-pii Pod の HTTP/JSON gateway 用 JsonRpc 実装。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/00_tier1_API共通規約.md
//     §「HTTP/JSON 互換インタフェース共通仕様」
//
// 役割:
//   PiiService の Classify / Mask 2 RPC を HTTP/JSON 経路で公開する。
//   gRPC 経路の handler ロジック（PiiServer 相当）と同じ Masker を使い、
//   request body / response body は protojson 互換 camelCase の JSON で扱う。

// 共通 gateway。
use k1s0_tier1_common::auth::AuthClaims;
use k1s0_tier1_common::http_gateway::JsonRpc;
// JSON 値型。
use serde_json::Value as JsonValue;
// PII 検出 logic。
use crate::masker::{Finding, Masker};
// FR-T1-PII-002 仮名化純関数。
use crate::pseudonymize::{pseudonymize, PseudonymizeError};
// 標準同期。
use std::sync::Arc;

/// PII 検出 / マスキング共通 state（Masker は ZST 相当だが Arc で揃える）。
#[derive(Default, Clone)]
pub struct PiiHttpState {
    /// PII 検出器。
    pub masker: Arc<Masker>,
}

/// `Pii.Classify` の HTTP 用 adapter。
pub struct ClassifyRpc {
    /// 共有 masker 参照。
    pub state: PiiHttpState,
}

#[async_trait::async_trait]
impl JsonRpc for ClassifyRpc {
    fn route(&self) -> &'static str {
        "pii/classify"
    }
    fn full_method(&self) -> &'static str {
        "/k1s0.tier1.pii.v1.PiiService/Classify"
    }
    async fn invoke(
        &self,
        _claims: &AuthClaims,
        body: JsonValue,
    ) -> Result<JsonValue, tonic::Status> {
        let text = body
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let findings = self.state.masker.classify(text);
        Ok(classify_response(&findings))
    }
}

/// `Pii.Mask` の HTTP 用 adapter。
pub struct MaskRpc {
    /// 共有 masker 参照。
    pub state: PiiHttpState,
}

#[async_trait::async_trait]
impl JsonRpc for MaskRpc {
    fn route(&self) -> &'static str {
        "pii/mask"
    }
    fn full_method(&self) -> &'static str {
        "/k1s0.tier1.pii.v1.PiiService/Mask"
    }
    async fn invoke(
        &self,
        _claims: &AuthClaims,
        body: JsonValue,
    ) -> Result<JsonValue, tonic::Status> {
        let text = body
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let (masked, findings) = self.state.masker.mask(text);
        Ok(serde_json::json!({
            "maskedText": masked,
            "findings": findings_to_json(&findings),
        }))
    }
}

/// `Pii.Pseudonymize` の HTTP 用 adapter（FR-T1-PII-002）。
/// salt 由来の HMAC-SHA256 を URL-safe base64 で返す純関数 RPC。state は不要。
pub struct PseudonymizeRpc {}

#[async_trait::async_trait]
impl JsonRpc for PseudonymizeRpc {
    fn route(&self) -> &'static str {
        "pii/pseudonymize"
    }
    fn full_method(&self) -> &'static str {
        "/k1s0.tier1.pii.v1.PiiService/Pseudonymize"
    }
    async fn invoke(
        &self,
        _claims: &AuthClaims,
        body: JsonValue,
    ) -> Result<JsonValue, tonic::Status> {
        // protojson camelCase: fieldType / value / salt の 3 必須キーを取り出す。
        let field_type = body
            .get("fieldType")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let value = body
            .get("value")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let salt = body
            .get("salt")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let pseudonym = pseudonymize(field_type, value, salt).map_err(|e| match e {
            PseudonymizeError::EmptySalt => {
                tonic::Status::invalid_argument("tier1/pii: salt required")
            }
            PseudonymizeError::EmptyValue => {
                tonic::Status::invalid_argument("tier1/pii: value required")
            }
            PseudonymizeError::EmptyFieldType => {
                tonic::Status::invalid_argument("tier1/pii: field_type required")
            }
        })?;
        Ok(serde_json::json!({ "pseudonym": pseudonym }))
    }
}

/// Finding 配列 → JSON 配列（protojson camelCase）。
fn findings_to_json(findings: &[Finding]) -> Vec<JsonValue> {
    findings
        .iter()
        .map(|f| {
            serde_json::json!({
                "type": f.kind.as_str(),
                "start": f.start as i32,
                "end": f.end as i32,
                "confidence": f.confidence,
            })
        })
        .collect()
}

/// Classify 応答を整形する。
fn classify_response(findings: &[Finding]) -> JsonValue {
    serde_json::json!({
        "findings": findings_to_json(findings),
        "containsPii": !findings.is_empty(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn classify_returns_email_finding() {
        let s = PiiHttpState::default();
        let r = ClassifyRpc { state: s };
        let resp = r
            .invoke(
                &AuthClaims::default(),
                serde_json::json!({ "text": "contact me at user@example.com" }),
            )
            .await
            .unwrap();
        assert_eq!(resp["containsPii"], serde_json::json!(true));
        assert!(resp["findings"].as_array().unwrap().len() >= 1);
    }

    #[tokio::test]
    async fn pseudonymize_returns_deterministic_value() {
        let r = PseudonymizeRpc {};
        let body = serde_json::json!({
            "fieldType": "EMAIL",
            "value": "alice@example.com",
            "salt": "tenant-A",
        });
        let a = r
            .invoke(&AuthClaims::default(), body.clone())
            .await
            .unwrap();
        let b = r.invoke(&AuthClaims::default(), body).await.unwrap();
        assert_eq!(a["pseudonym"], b["pseudonym"]);
        assert_eq!(a["pseudonym"].as_str().unwrap().len(), 43);
    }

    #[tokio::test]
    async fn pseudonymize_rejects_empty_salt() {
        let r = PseudonymizeRpc {};
        let err = r
            .invoke(
                &AuthClaims::default(),
                serde_json::json!({
                    "fieldType": "EMAIL",
                    "value": "alice@example.com",
                    "salt": "",
                }),
            )
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::InvalidArgument);
    }

    #[tokio::test]
    async fn mask_returns_masked_text() {
        let s = PiiHttpState::default();
        let r = MaskRpc { state: s };
        let resp = r
            .invoke(
                &AuthClaims::default(),
                serde_json::json!({ "text": "email user@example.com here" }),
            )
            .await
            .unwrap();
        let masked = resp["maskedText"].as_str().unwrap();
        // 元 email が plaintext で残らない。
        assert!(!masked.contains("user@example.com"));
    }
}
