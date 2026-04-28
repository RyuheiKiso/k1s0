// 本ファイルは k1s0 Decision rule registry。ZEN Engine（JDM 評価器）を直接統合する。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/09_Decision_API.md
//   docs/02_構想設計/adr/ADR-RULE-001-zen-engine.md
//     - "ルールエンジンは ZEN Engine（Rust、MIT）+ JDM フォーマットを採用する"
//     - "ZEN Engine 0.30+（Rust 実装、C FFI と Go/Python/Node バインディング提供）"
//     - "JDM（JSON Decision Model）を標準フォーマット"
//
// 役割:
//   - rule_id × rule_version → JDM DecisionContent の保管（in-memory）
//   - 登録時に JDM を `serde_json::from_slice::<DecisionContent>` で検証
//   - 評価は ZEN Engine の DecisionEngine + Decision.evaluate_with_opts に委譲
//   - 評価トレースは ADR-RULE-001 必須要件（"include_trace オプションで公開"）
//
// バージョニング:
//   バージョン文字列は registry が "v<N>" 連番で自動採番する。空 string 評価は
//   "latest"（最後に登録された version）に解決。

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use serde_json::Value as JsonValue;
use zen_engine::model::DecisionContent;
use zen_engine::{DecisionEngine, EvaluationOptions};

// ZEN Engine の Decision::evaluate_with_opts が返す Future は内部で std::cell::OnceCell
// を保持するため `!Send`。tonic は handler future の Send を要求するため、評価は
// `spawn_blocking` で別スレッドに退避し、そのスレッド内で current-thread tokio runtime を
// 立ち上げて future を回す。in-memory 評価のみなので実質ブロッキングにはならず、
// runtime 起動コストは数百 μs（ADR-RULE-001 の p99 < 50ms 予算内）に収まる。

/// 内部 ID と version を 1 セットで保持するキー。
type RuleKey = (String, String);

/// registry のエラー型。
#[derive(Debug)]
pub enum RegistryError {
    /// JDM JSON parse 失敗。
    InvalidJson(String),
    /// JDM 構造不正（ZEN Engine の評価でグラフ構造異常）。
    InvalidRule(String),
    /// rule 未登録。
    NotFound { rule_id: String, rule_version: String },
    /// expression / 評価グラフ実行失敗。
    EvalFailed(String),
    /// lock 失敗。
    LockPoisoned,
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryError::InvalidJson(s) => write!(f, "invalid json: {}", s),
            RegistryError::InvalidRule(s) => write!(f, "invalid rule: {}", s),
            RegistryError::NotFound {
                rule_id,
                rule_version,
            } => {
                write!(f, "rule not found: {}/{}", rule_id, rule_version)
            }
            RegistryError::EvalFailed(s) => write!(f, "eval failed: {}", s),
            RegistryError::LockPoisoned => write!(f, "lock poisoned"),
        }
    }
}

impl std::error::Error for RegistryError {}

/// 1 件のコンパイル済 JDM。
struct CompiledRule {
    /// ZEN Engine の DecisionContent（コンパイル済 opcode キャッシュ込み）。
    content: Arc<DecisionContent>,
    /// 元の JDM JSON bytes（GetRule で再返却するため保持）。
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
    pub rule_id: String,
    pub jdm_document: Vec<u8>,
    pub commit_hash: String,
    pub registered_by: String,
    /// 0 なら register 時に now() で埋める。
    pub registered_at_ms: i64,
}

/// register の出力。
#[derive(Debug, Clone)]
pub struct RegisterOutcome {
    pub rule_version: String,
    pub effective_at_ms: i64,
}

/// JDM ルール registry。
pub struct RuleRegistry {
    /// rule_id × rule_version → 登録済 JDM。
    rules: RwLock<HashMap<RuleKey, Arc<CompiledRule>>>,
    /// rule_id → 最新 rule_version。空 rule_version 解決に使う。
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

    /// JDM JSON を ZEN Engine の DecisionContent に parse する。
    /// parse 失敗時は InvalidJson、最低限の構造（nodes / edges）が無い場合 InvalidRule を返す。
    fn compile(jdm_json: &[u8]) -> Result<Arc<DecisionContent>, RegistryError> {
        let mut content: DecisionContent = serde_json::from_slice(jdm_json)
            .map_err(|e| RegistryError::InvalidJson(e.to_string()))?;
        // ZEN Engine の評価には最低 1 ノード必要。0 ノードの空グラフは登録段階で弾く。
        if content.nodes.is_empty() {
            return Err(RegistryError::InvalidRule("nodes is empty".into()));
        }
        // expressionNode / decisionTableNode のバイトコードを事前コンパイルしてキャッシュする。
        // 評価レイテンシが p99 < 50ms（ADR-RULE-001）の達成に必要。
        content.compile();
        Ok(Arc::new(content))
    }

    /// JDM 文書を登録し、自動採番された rule_version を返す。
    /// 既存 rule_id への登録なら次の連番（"v2", "v3", ...）、新規なら "v1"。
    pub fn register(&self, input: RegisterInput) -> Result<RegisterOutcome, RegistryError> {
        let content = Self::compile(&input.jdm_document)?;
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
        let now_ms = if input.registered_at_ms > 0 {
            input.registered_at_ms
        } else {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0)
        };
        let meta = RuleMeta {
            rule_version: new_version.clone(),
            commit_hash: input.commit_hash,
            registered_at_ms: now_ms,
            registered_by: input.registered_by,
        };
        let compiled = Arc::new(CompiledRule {
            content,
            raw_jdm_json: input.jdm_document,
            meta,
        });
        let key = (input.rule_id.clone(), new_version.clone());
        let mut rules = self.rules.write().map_err(|_| RegistryError::LockPoisoned)?;
        rules.insert(key, compiled);
        let mut latest = self
            .latest
            .write()
            .map_err(|_| RegistryError::LockPoisoned)?;
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

    /// JDM ルールを ZEN Engine で評価する。include_trace=true の時は trace を JSON で詰める。
    pub async fn evaluate(
        &self,
        rule_id: &str,
        rule_version: &str,
        input_json: &[u8],
        include_trace: bool,
    ) -> Result<EvalOutcome, RegistryError> {
        let resolved = self.resolve_version(rule_id, rule_version)?;
        let compiled = {
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

        let started = std::time::Instant::now();
        let opts = EvaluationOptions {
            trace: include_trace,
            max_depth: 10,
        };
        let content = compiled.content.clone();
        // ZEN Engine の評価 future および DecisionGraphResponse 内の Variable は内部に
        // Rc / OnceCell を保持するため `!Send`。tonic は handler future の Send を要求するので、
        // spawn_blocking で別スレッドに退避し、評価 → JSON への変換まで同スレッドで完結させ、
        // Send-safe な (output_bytes, trace_bytes) のみを呼出元に返す。
        // 実 I/O は無いため runtime 起動コストは数百 μs（ADR-RULE-001 p99 < 50ms 予算内）。
        let (output_json, trace_json) =
            tokio::task::spawn_blocking(move || -> Result<(Vec<u8>, Vec<u8>), RegistryError> {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|e| RegistryError::EvalFailed(format!("rt build: {}", e)))?;
                let engine = DecisionEngine::default();
                let decision = engine.create_decision(content);
                rt.block_on(async move {
                    let resp = decision
                        .evaluate_with_opts(input.into(), opts)
                        .await
                        .map_err(|e| RegistryError::EvalFailed(e.to_string()))?;
                    // Variable → JsonValue → bytes（JsonValue は Send-safe）。
                    let result_value: JsonValue = resp.result.into();
                    let out = serde_json::to_vec(&result_value)
                        .map_err(|e| RegistryError::InvalidJson(e.to_string()))?;
                    let trace = if include_trace {
                        match resp.trace.as_ref() {
                            Some(t) => serde_json::to_vec(t)
                                .map_err(|e| RegistryError::InvalidJson(e.to_string()))?,
                            None => Vec::new(),
                        }
                    } else {
                        Vec::new()
                    };
                    Ok::<(Vec<u8>, Vec<u8>), RegistryError>((out, trace))
                })
            })
            .await
            .map_err(|e| RegistryError::EvalFailed(format!("join: {}", e)))??;
        let elapsed_us = started.elapsed().as_micros() as i64;
        Ok(EvalOutcome {
            output_json,
            elapsed_us,
            trace_json,
        })
    }
}

// テストは行数規約（src/CLAUDE.md: 1 ファイル 500 行以内）に従い別ファイルに切り出す。
#[cfg(test)]
#[path = "registry_tests.rs"]
mod tests;
