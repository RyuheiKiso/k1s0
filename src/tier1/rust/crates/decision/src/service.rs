// 本ファイルは t1-decision Pod の DecisionService / DecisionAdminService 実装本体。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/01_tier1全体コンポーネント俯瞰.md
//     - DS-SW-COMP-008（t1-decision Pod、JDM 評価エンジン）
//   docs/02_構想設計/adr/ADR-RULE-001-zen-engine.md
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/09_Decision_API.md
//
// 役割:
//   - DecisionService（Evaluate / BatchEvaluate）と DecisionAdminService
//     （RegisterRule / ListVersions / GetRule）の trait 実装を集約する。
//   - main.rs の 500 行制限（src/CLAUDE.md）を維持するため、handler 実装と
//     audit emit の詳細を本ファイルに分離する。

// 標準同期。
use std::sync::Arc;

// SDK 公開 API の DecisionService / DecisionAdminService 関連型。
use k1s0_sdk_proto::k1s0::tier1::decision::v1::{
    BatchEvaluateRequest, BatchEvaluateResponse, EvaluateRequest, EvaluateResponse, GetRuleRequest,
    GetRuleResponse, ListVersionsRequest, ListVersionsResponse, RegisterRuleRequest,
    RegisterRuleResponse, RuleVersionMeta,
    decision_admin_service_server::DecisionAdminService,
    decision_service_server::DecisionService,
};
// FR-T1-DECISION-003: 評価ごとに rule_id / rule_version / input_hash / output_hash 付き
// audit を rich に発火するための型。
use k1s0_tier1_common::audit::{AuditEmitter, AuditRecord, outcome_from_code};
use k1s0_tier1_common::auth::{enforce_tenant_boundary, AuthClaims};
// proto 共通の TenantContext から body 側 tenant_id を抽出するため import。
use k1s0_sdk_proto::k1s0::tier1::common::v1::TenantContext;
// 内部 registry。
use crate::registry::{RegisterInput, RegistryError, RuleMeta, RuleRegistry};
// tonic ランタイム / 型。
use tonic::{Request, Response, Status};

/// 共有 registry を保持する DecisionService の Server 実装。
pub struct DecisionServer {
    /// 登録済 JDM ルールの in-memory registry。
    pub registry: Arc<RuleRegistry>,
    /// FR-T1-DECISION-003: 評価ごとに rich audit を出すための emitter。
    /// 共通 K1s0Layer も基本記録を出すが、本フィールド経由で rule_id /
    /// rule_version / input_hash / output_hash を attributes に詰めた追加 record を発火する。
    pub audit_emitter: Arc<dyn AuditEmitter>,
}

/// DecisionAdminService の Server 実装。registry への CRUD と RegisterRule audit 発火を担う。
pub struct DecisionAdminServer {
    /// 登録済 JDM ルールの in-memory registry（DecisionServer と Arc 共有）。
    pub registry: Arc<RuleRegistry>,
    /// RegisterRule 時に rule_id / rule_version / commit_hash を attributes に
    /// 詰めた追加 audit record を発火するための emitter（FR-T1-DECISION-003）。
    pub audit_emitter: Arc<dyn AuditEmitter>,
}

/// RuleMeta → proto RuleVersionMeta 変換。registry 内部表現と proto 表現の橋渡し。
pub fn rule_meta_to_proto(m: &RuleMeta) -> RuleVersionMeta {
    RuleVersionMeta {
        rule_version: m.rule_version.clone(),
        commit_hash: m.commit_hash.clone(),
        registered_at_ms: m.registered_at_ms,
        registered_by: m.registered_by.clone(),
        deprecated: false,
    }
}

/// 入力 / 出力 JSON の SHA-256 を URL-safe base64 (no padding) で算出する。
/// FR-T1-DECISION-003: 「個人情報を含む入力は PII マスキング連携で保護される」要件のため、
/// 生 input/output の代わりに hash を attributes に保存する（PII Mask が同 Pod に同居しない
/// アーキテクチャ前提では hash 化が最小限の構造的保護となる）。
pub fn hash_b64(bytes: &[u8]) -> String {
    use base64::Engine as _;
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(h.finalize())
}

/// req.extensions の AuthClaims を取り出す。K1s0Layer が auth を通している場合は
/// claims が存在し、auth=off の場合は default claims（empty tenant）を返す。
pub fn claims_from_req<T>(req: &Request<T>) -> AuthClaims {
    req.extensions()
        .get::<AuthClaims>()
        .cloned()
        .unwrap_or_default()
}

/// 共通の tenant 境界検証 helper。`claims_from_req` で取り出した claims と body の
/// `TenantContext.tenant_id` を `enforce_tenant_boundary` で照合し、確定 tenant_id を返す。
/// claims が空（auth=off）の場合は body 側を採用する（AuditServer と同じ挙動）。
pub fn resolve_tenant<T>(req: &Request<T>, body_ctx: Option<&TenantContext>, rpc: &str) -> Result<String, Status> {
    let claims = claims_from_req(req);
    let body_tid = body_ctx.map(|c| c.tenant_id.clone()).unwrap_or_default();
    enforce_tenant_boundary(&claims, &body_tid, rpc)
}

/// RegistryError → tonic::Status 翻訳。NotFound / InvalidJson 系を gRPC code に正しく落とす。
pub fn registry_err_to_status(e: RegistryError, rpc: &str) -> Status {
    match e {
        RegistryError::InvalidJson(msg) | RegistryError::InvalidRule(msg) => {
            Status::invalid_argument(format!("tier1/decision: {}: {}", rpc, msg))
        }
        RegistryError::NotFound {
            tenant_id,
            rule_id,
            rule_version,
        } => Status::not_found(format!(
            "tier1/decision: {}: rule tenant={} {}/{} not found",
            rpc, tenant_id, rule_id, rule_version
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
        // claims を audit attributes に詰めるため、into_inner の前に取り出す。
        let claims = claims_from_req(&req);
        // NFR-E-AC-003: claims (JWT) と body.context.tenant_id の不一致を構造的に拒否する。
        let tenant_id = resolve_tenant(&req, req.get_ref().context.as_ref(), "Decision.Evaluate")?;
        let r = req.into_inner();
        // 評価実行。FR-T1-DECISION-003 の audit 連携用に rule_id / rule_version /
        // input/output hash を保持する。
        let outcome_result = self
            .registry
            .evaluate(
                &tenant_id,
                &r.rule_id,
                &r.rule_version,
                &r.input_json,
                r.include_trace,
            )
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
        // 監査用に claims を取り出す。
        let claims = claims_from_req(&req);
        // NFR-E-AC-003: tenant_id 越境防止。
        let tenant_id = resolve_tenant(&req, req.get_ref().context.as_ref(), "Decision.BatchEvaluate")?;
        let r = req.into_inner();
        // 結果バッファと size 集計を初期化。
        let mut outputs: Vec<Vec<u8>> = Vec::with_capacity(r.inputs_json.len());
        let mut total_input_bytes: usize = 0;
        let mut total_output_bytes: usize = 0;
        for input in r.inputs_json.iter() {
            total_input_bytes += input.len();
            let outcome = self
                .registry
                .evaluate(&tenant_id, &r.rule_id, &r.rule_version, input, false)
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
        // 監査用に claims を取り出してから本体を進める。
        let claims = claims_from_req(&req);
        // NFR-E-AC-003: tenant_id 越境防止。RegisterRule は新規登録なので fallback 無し
        // （body 由来 tenant に登録される、claims と一致しない場合は拒否）。
        let tenant_id = resolve_tenant(&req, req.get_ref().context.as_ref(), "Decision.RegisterRule")?;
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
                tenant_id: tenant_id.clone(),
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
        // NFR-E-AC-003: 当該 tenant 配下の rule のみ列挙する（情報漏洩防止）。
        let tenant_id = resolve_tenant(&req, req.get_ref().context.as_ref(), "Decision.ListVersions")?;
        let r = req.into_inner();
        let metas = self
            .registry
            .list_versions(&tenant_id, &r.rule_id)
            .map_err(|e| registry_err_to_status(e, "ListVersions"))?;
        Ok(Response::new(ListVersionsResponse {
            versions: metas.iter().map(rule_meta_to_proto).collect(),
        }))
    }

    async fn get_rule(
        &self,
        req: Request<GetRuleRequest>,
    ) -> Result<Response<GetRuleResponse>, Status> {
        // NFR-E-AC-003: 別 tenant の rule を読み出せないよう必須検証。
        let tenant_id = resolve_tenant(&req, req.get_ref().context.as_ref(), "Decision.GetRule")?;
        let r = req.into_inner();
        let (jdm, meta) = self
            .registry
            .get_jdm_with_meta(&tenant_id, &r.rule_id, &r.rule_version)
            .map_err(|e| registry_err_to_status(e, "GetRule"))?;
        Ok(Response::new(GetRuleResponse {
            jdm_document: jdm,
            meta: Some(rule_meta_to_proto(&meta)),
        }))
    }
}

// テストは行数規約（src/CLAUDE.md: 1 ファイル 500 行以内）に従い別ファイルに切り出す。
#[cfg(test)]
#[path = "service_tests.rs"]
mod tests;
