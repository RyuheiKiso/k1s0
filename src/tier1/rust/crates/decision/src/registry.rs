// 本ファイルは k1s0 簡易 Decision rule registry と evaluator。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/09_Decision_API.md
//   docs/02_構想設計/adr/ADR-RULE-001-zen-engine.md
//
// 役割:
//   - rule_id × rule_version → rule 文書（JSON）の保管（in-memory）
//   - rule の expression 群を `evalexpr` で評価し output_json を返す
//
// 簡易ルール形式（JDM expressionNode 互換、subset）:
//   ```
//   {
//     "expressions": [
//       {"key": "tax",   "value": "amount * 0.10"},
//       {"key": "total", "value": "amount * 1.10"}
//     ]
//   }
//   ```
//   - 入力 JSON は object 限定。各 top-level field が evalexpr の変数として注入される
//   - expression は evalexpr 構文。算術 / 論理 / 比較 / if-else を含む
//   - 出力は同 key で JSON object を組み上げて返す
//
// バージョニング:
//   バージョン文字列は呼出側採番。空 string は "latest"（最後に登録された version）に解決。
//
// 現時点 で zen-engine（loader / decisionTable / function 等の高機能）は環境依存
// （rquickjs の C++ 依存）で同梱しない。完全 JDM 互換は plan 04-08 後段で別 crate
// 直結に切り替える（本 registry の interface は維持）。

use std::collections::HashMap;
use std::sync::RwLock;

use evalexpr::{eval_with_context, ContextWithMutableVariables, HashMapContext, Value as ExprValue};
use serde_json::Value as JsonValue;

/// 内部 ID と version を 1 セットで保持するキー。
type RuleKey = (String, String);

/// registry のエラー型。
#[derive(Debug)]
pub enum RegistryError {
    /// JSON parse 失敗。
    InvalidJson(String),
    /// rule 形式不正（expressions 配列がない等）。
    InvalidRule(String),
    /// rule 未登録。
    NotFound { rule_id: String, rule_version: String },
    /// expression 評価失敗。
    EvalFailed(String),
    /// lock 失敗。
    LockPoisoned,
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryError::InvalidJson(s) => write!(f, "invalid json: {}", s),
            RegistryError::InvalidRule(s) => write!(f, "invalid rule: {}", s),
            RegistryError::NotFound { rule_id, rule_version } => {
                write!(f, "rule not found: {}/{}", rule_id, rule_version)
            }
            RegistryError::EvalFailed(s) => write!(f, "eval failed: {}", s),
            RegistryError::LockPoisoned => write!(f, "lock poisoned"),
        }
    }
}

impl std::error::Error for RegistryError {}

/// 1 件の expression（key + value 文字列）。
#[derive(Debug, Clone)]
struct Expression {
    key: String,
    value: String,
}

/// rule 文書を内部表現に compile 済の形で保管する（再評価で parse コスト削減）。
#[derive(Debug, Clone)]
struct CompiledRule {
    expressions: Vec<Expression>,
    /// 元の JDM JSON（GetRule で再返却するため保持）。
    raw_jdm_json: Vec<u8>,
    /// 登録時メタ情報。
    meta: RuleMeta,
}

/// 登録時メタ情報（proto RuleVersionMeta と等価）。
#[derive(Debug, Clone, Default)]
pub struct RuleMeta {
    pub rule_version: String,
    pub commit_hash: String,
    pub registered_at_ms: i64,
    pub registered_by: String,
}

/// 評価結果。
#[derive(Debug)]
pub struct EvalOutcome {
    pub output_json: Vec<u8>,
    pub elapsed_us: i64,
    pub trace_json: Vec<u8>,
}

/// register の入力。
#[derive(Debug, Clone, Default)]
pub struct RegisterInput {
    /// rule_id（tenant 内で一意）。
    pub rule_id: String,
    /// JDM 文書（JSON）。
    pub jdm_document: Vec<u8>,
    /// Git commit hash（任意、メタ情報）。
    pub commit_hash: String,
    /// 登録者（任意、認証 subject）。
    pub registered_by: String,
    /// 登録時刻（Unix ms、0 なら register 時に now() で埋める）。
    pub registered_at_ms: i64,
}

/// register の出力（次の rule_version + 発効時刻）。
#[derive(Debug, Clone)]
pub struct RegisterOutcome {
    pub rule_version: String,
    pub effective_at_ms: i64,
}

/// JDM ルール registry。
pub struct RuleRegistry {
    rules: RwLock<HashMap<RuleKey, CompiledRule>>,
    latest: RwLock<HashMap<String, String>>,
}

impl Default for RuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleRegistry {
    pub fn new() -> Self {
        RuleRegistry {
            rules: RwLock::new(HashMap::new()),
            latest: RwLock::new(HashMap::new()),
        }
    }

    /// rule 文書（JSON）を内部 CompiledRule に変換する。
    /// 期待形式: `{"expressions": [{"key": "...", "value": "..."}, ...]}`
    fn compile(jdm_json: &[u8]) -> Result<CompiledRule, RegistryError> {
        let v: JsonValue = serde_json::from_slice(jdm_json)
            .map_err(|e| RegistryError::InvalidJson(e.to_string()))?;
        let arr = v
            .get("expressions")
            .and_then(|x| x.as_array())
            .ok_or_else(|| RegistryError::InvalidRule("missing 'expressions' array".into()))?;
        let mut exprs = Vec::with_capacity(arr.len());
        for item in arr {
            let key = item
                .get("key")
                .and_then(|x| x.as_str())
                .ok_or_else(|| RegistryError::InvalidRule("expression missing 'key'".into()))?;
            let val = item
                .get("value")
                .and_then(|x| x.as_str())
                .ok_or_else(|| RegistryError::InvalidRule("expression missing 'value'".into()))?;
            exprs.push(Expression {
                key: key.to_string(),
                value: val.to_string(),
            });
        }
        Ok(CompiledRule {
            expressions: exprs,
            raw_jdm_json: Vec::new(),
            meta: RuleMeta::default(),
        })
    }

    /// JDM 文書を登録し、自動採番された rule_version を返す。
    /// 既存 rule_id への登録なら次の連番（"v2", "v3", ...）、新規なら "v1"。
    pub fn register(&self, input: RegisterInput) -> Result<RegisterOutcome, RegistryError> {
        let mut compiled = Self::compile(&input.jdm_document)?;
        compiled.raw_jdm_json = input.jdm_document.clone();
        // 次のバージョン番号を決定（既存最大 +1）。
        let new_version = {
            let rules = self.rules.read().map_err(|_| RegistryError::LockPoisoned)?;
            let max_n: u32 = rules
                .keys()
                .filter(|(rid, _)| rid == &input.rule_id)
                .filter_map(|(_, ver)| ver.strip_prefix('v').and_then(|s| s.parse::<u32>().ok()))
                .max()
                .unwrap_or(0);
            format!("v{}", max_n + 1)
        };
        // メタ情報を組み立て。
        let now_ms = if input.registered_at_ms > 0 {
            input.registered_at_ms
        } else {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0)
        };
        compiled.meta = RuleMeta {
            rule_version: new_version.clone(),
            commit_hash: input.commit_hash,
            registered_at_ms: now_ms,
            registered_by: input.registered_by,
        };
        // 排他で書き込み + latest 更新。
        let key = (input.rule_id.clone(), new_version.clone());
        let mut rules = self.rules.write().map_err(|_| RegistryError::LockPoisoned)?;
        rules.insert(key, compiled);
        let mut latest = self.latest.write().map_err(|_| RegistryError::LockPoisoned)?;
        latest.insert(input.rule_id, new_version.clone());
        Ok(RegisterOutcome {
            rule_version: new_version,
            effective_at_ms: now_ms,
        })
    }

    fn resolve_version(&self, rule_id: &str, rule_version: &str) -> Result<String, RegistryError> {
        if !rule_version.is_empty() {
            return Ok(rule_version.to_string());
        }
        let latest = self.latest.read().map_err(|_| RegistryError::LockPoisoned)?;
        latest
            .get(rule_id)
            .cloned()
            .ok_or_else(|| RegistryError::NotFound {
                rule_id: rule_id.to_string(),
                rule_version: "(latest)".to_string(),
            })
    }

    /// 登録済 rule の RuleMeta 一覧を登録時刻昇順で返す。
    pub fn list_versions(&self, rule_id: &str) -> Result<Vec<RuleMeta>, RegistryError> {
        let rules = self.rules.read().map_err(|_| RegistryError::LockPoisoned)?;
        let mut metas: Vec<RuleMeta> = rules
            .iter()
            .filter(|((rid, _), _)| rid == rule_id)
            .map(|(_, c)| c.meta.clone())
            .collect();
        metas.sort_by_key(|m| m.registered_at_ms);
        Ok(metas)
    }

    /// 特定 rule_id × rule_version の元 JDM JSON と meta を返す（GetRule 用）。
    pub fn get_jdm_with_meta(
        &self,
        rule_id: &str,
        rule_version: &str,
    ) -> Result<(Vec<u8>, RuleMeta), RegistryError> {
        let resolved = self.resolve_version(rule_id, rule_version)?;
        let rules = self.rules.read().map_err(|_| RegistryError::LockPoisoned)?;
        let rule = rules
            .get(&(rule_id.to_string(), resolved.clone()))
            .ok_or(RegistryError::NotFound {
                rule_id: rule_id.to_string(),
                rule_version: resolved,
            })?;
        Ok((rule.raw_jdm_json.clone(), rule.meta.clone()))
    }

    /// rule を評価する。
    pub fn evaluate(
        &self,
        rule_id: &str,
        rule_version: &str,
        input_json: &[u8],
        include_trace: bool,
    ) -> Result<EvalOutcome, RegistryError> {
        let resolved = self.resolve_version(rule_id, rule_version)?;
        let rule = {
            let rules = self.rules.read().map_err(|_| RegistryError::LockPoisoned)?;
            rules
                .get(&(rule_id.to_string(), resolved.clone()))
                .cloned()
                .ok_or(RegistryError::NotFound {
                    rule_id: rule_id.to_string(),
                    rule_version: resolved,
                })?
        };
        let input: JsonValue = serde_json::from_slice(input_json)
            .map_err(|e| RegistryError::InvalidJson(e.to_string()))?;

        // input の top-level field を evalexpr context に注入する。
        let mut ctx = HashMapContext::new();
        if let Some(obj) = input.as_object() {
            for (k, v) in obj {
                let ev = json_to_expr_value(v).map_err(RegistryError::InvalidJson)?;
                ctx.set_value(k.clone(), ev)
                    .map_err(|e| RegistryError::InvalidRule(format!("ctx set: {}", e)))?;
            }
        }
        // 各 expression を順次評価。前 expression の出力も後続から参照できるよう
        // 評価結果を context に逐次追加する。
        let started = std::time::Instant::now();
        let mut output_map = serde_json::Map::new();
        let mut trace_steps: Vec<JsonValue> = Vec::new();

        for expr in rule.expressions.iter() {
            let result = eval_with_context(&expr.value, &ctx)
                .map_err(|e| RegistryError::EvalFailed(format!("{}: {}", expr.key, e)))?;
            // 後続 expression が参照できるよう context に追加。
            ctx.set_value(expr.key.clone(), result.clone())
                .map_err(|e| RegistryError::InvalidRule(format!("ctx set: {}", e)))?;
            // output に追加。
            let out_v = expr_value_to_json(&result);
            if include_trace {
                trace_steps.push(serde_json::json!({
                    "key": expr.key,
                    "expression": expr.value,
                    "result": out_v.clone(),
                }));
            }
            output_map.insert(expr.key.clone(), out_v);
        }
        let elapsed_us = started.elapsed().as_micros() as i64;

        let output_json = serde_json::to_vec(&JsonValue::Object(output_map))
            .map_err(|e| RegistryError::InvalidJson(e.to_string()))?;
        let trace_json = if include_trace {
            serde_json::to_vec(&JsonValue::Array(trace_steps))
                .map_err(|e| RegistryError::InvalidJson(e.to_string()))?
        } else {
            Vec::new()
        };
        Ok(EvalOutcome {
            output_json,
            elapsed_us,
            trace_json,
        })
    }
}

/// JSON Value を evalexpr Value に変換する。
fn json_to_expr_value(v: &JsonValue) -> Result<ExprValue, String> {
    match v {
        JsonValue::Null => Ok(ExprValue::Empty),
        JsonValue::Bool(b) => Ok(ExprValue::Boolean(*b)),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(ExprValue::Int(i))
            } else if let Some(f) = n.as_f64() {
                Ok(ExprValue::Float(f))
            } else {
                Err(format!("unsupported number: {}", n))
            }
        }
        JsonValue::String(s) => Ok(ExprValue::String(s.clone())),
        // 配列・オブジェクトは evalexpr が直接サポートしない。文字列化して保持する。
        other => Ok(ExprValue::String(other.to_string())),
    }
}

/// evalexpr Value を JSON Value に変換する。
fn expr_value_to_json(v: &ExprValue) -> JsonValue {
    match v {
        ExprValue::Empty => JsonValue::Null,
        ExprValue::Boolean(b) => JsonValue::Bool(*b),
        ExprValue::Int(i) => JsonValue::Number(serde_json::Number::from(*i)),
        ExprValue::Float(f) => serde_json::Number::from_f64(*f)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null),
        ExprValue::String(s) => JsonValue::String(s.clone()),
        ExprValue::Tuple(values) => {
            JsonValue::Array(values.iter().map(expr_value_to_json).collect())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_rule() -> &'static [u8] {
        br#"{
          "expressions": [
            {"key": "tax",   "value": "amount * 0.10"},
            {"key": "total", "value": "amount + tax"}
          ]
        }"#
    }

    #[test]
    fn register_and_evaluate() {
        let r = RuleRegistry::new();
        r.register(RegisterInput {
            rule_id: "tax-calc".into(),
            jdm_document: simple_rule().to_vec(),
            ..Default::default()
        })
        .unwrap();
        let outcome = r
            .evaluate("tax-calc", "v1", br#"{"amount": 100}"#, false)
            .unwrap();
        let out: JsonValue = serde_json::from_slice(&outcome.output_json).unwrap();
        // tax = 10.0、total = 110.0（tax を含む式の例: 後 expression が前を参照）。
        assert_eq!(out["tax"], serde_json::json!(10.0));
        assert_eq!(out["total"], serde_json::json!(110.0));
    }

    #[test]
    fn evaluate_with_trace() {
        let r = RuleRegistry::new();
        r.register(RegisterInput {
            rule_id: "rid".into(),
            jdm_document: simple_rule().to_vec(),
            ..Default::default()
        })
        .unwrap();
        let outcome = r
            .evaluate("rid", "v1", br#"{"amount": 50}"#, true)
            .unwrap();
        assert!(!outcome.trace_json.is_empty());
        let trace: JsonValue = serde_json::from_slice(&outcome.trace_json).unwrap();
        // 2 ステップが trace に含まれる。
        assert_eq!(trace.as_array().unwrap().len(), 2);
    }

    #[test]
    fn evaluate_resolves_latest_when_version_empty() {
        let r = RuleRegistry::new();
        r.register(RegisterInput {
            rule_id: "rid".into(),
            jdm_document: simple_rule().to_vec(),
            ..Default::default()
        })
        .unwrap();
        let outcome = r.evaluate("rid", "", br#"{"amount": 100}"#, false).unwrap();
        let out: JsonValue = serde_json::from_slice(&outcome.output_json).unwrap();
        assert_eq!(out["tax"], serde_json::json!(10.0));
    }

    #[test]
    fn list_versions_returns_registered() {
        let r = RuleRegistry::new();
        r.register(RegisterInput {
            rule_id: "rid".into(),
            jdm_document: simple_rule().to_vec(),
            ..Default::default()
        })
        .unwrap();
        r.register(RegisterInput {
            rule_id: "rid".into(),
            jdm_document: simple_rule().to_vec(),
            ..Default::default()
        })
        .unwrap();
        let v = r.list_versions("rid").unwrap();
        assert_eq!(v.len(), 2);
        assert!(v.iter().any(|m| m.rule_version == "v1"));
        assert!(v.iter().any(|m| m.rule_version == "v2"));
    }

    #[test]
    fn register_invalid_json_returns_error() {
        let r = RuleRegistry::new();
        let e = r
            .register(RegisterInput {
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
    fn register_invalid_rule_returns_error() {
        let r = RuleRegistry::new();
        let e = r
            .register(RegisterInput {
                rule_id: "rid".into(),
                jdm_document: br#"{"foo": 1}"#.to_vec(),
                ..Default::default()
            })
            .unwrap_err();
        match e {
            RegistryError::InvalidRule(_) => {}
            other => panic!("expected InvalidRule, got {:?}", other),
        }
    }

    #[test]
    fn evaluate_unknown_rule_returns_not_found() {
        let r = RuleRegistry::new();
        let e = r.evaluate("missing", "v1", br#"{}"#, false).unwrap_err();
        match e {
            RegistryError::NotFound { .. } => {}
            other => panic!("expected NotFound, got {:?}", other),
        }
    }

    #[test]
    fn evaluate_supports_boolean_logic_x() {
        let r = RuleRegistry::new();
        let rule = br#"{
          "expressions": [
            {"key": "is_premium", "value": "amount >= 100"},
            {"key": "passes_kyc", "value": "score > 0.7 && verified == true"}
          ]
        }"#;
        r.register(RegisterInput {
            rule_id: "flags".into(),
            jdm_document: rule.to_vec(),
            ..Default::default()
        })
        .unwrap();
        let resp = r
            .evaluate(
                "flags",
                "v1",
                br#"{"amount": 150, "score": 0.9, "verified": true}"#,
                false,
            )
            .unwrap();
        let out: JsonValue = serde_json::from_slice(&resp.output_json).unwrap();
        assert_eq!(out["is_premium"], serde_json::json!(true));
        assert_eq!(out["passes_kyc"], serde_json::json!(true));
    }

    #[test]
    fn get_jdm_returns_serialized_rule() {
        let r = RuleRegistry::new();
        r.register(RegisterInput {
            rule_id: "rid".into(),
            jdm_document: simple_rule().to_vec(),
            ..Default::default()
        })
        .unwrap();
        let (bytes, meta) = r.get_jdm_with_meta("rid", "v1").unwrap();
        let v: JsonValue = serde_json::from_slice(&bytes).unwrap();
        let arr = v.get("expressions").and_then(|x| x.as_array()).unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(meta.rule_version, "v1");
    }
}
