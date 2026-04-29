// 本ファイルは Rust 共通の自動 audit 発火 interceptor。
//
// 設計正典:
//   docs/03_要件定義/00_共通規約.md §「監査自動発火」
//   FR-T1-AUDIT-001: tier1 配下の特権 RPC 全件 audit
//
// 役割（Go 側 src/tier1/go/internal/common/audit.go と等価）:
//   3 Pod の gRPC server 全 RPC で `AuditEmitter::emit` を呼び、特権 RPC については
//   成功 / 失敗を WORM 監査ログに記録する。emitter は in-process と
//   gRPC 経由（k1s0_audit Pod 結線）の 2 種類を切替可能。

// 標準同期。
use std::collections::HashSet;

// 認証クレーム（actor 用）。
use crate::auth::AuthClaims;

/// 監査記録 1 件分のイベント。
#[derive(Debug, Clone)]
pub struct AuditRecord {
    /// テナント。
    pub tenant_id: String,
    /// 操作主体。
    pub actor: String,
    /// 操作種別（"State.Set" など）。
    pub action: String,
    /// 対象リソース URN。
    pub resource: String,
    /// 結果文字列（"SUCCESS" / "DENIED" / "ERROR"）。
    pub outcome: String,
    /// gRPC ステータスコード（int 化）。
    pub code: i32,
}

/// audit 発火器の trait。
#[async_trait::async_trait]
pub trait AuditEmitter: Send + Sync + 'static {
    /// 1 件記録する。失敗は fail-soft（drop）で構わない。
    async fn emit(&self, rec: AuditRecord);
}

/// dev 用 noop emitter。
#[derive(Default)]
pub struct NoopAuditEmitter;

#[async_trait::async_trait]
impl AuditEmitter for NoopAuditEmitter {
    async fn emit(&self, _rec: AuditRecord) {}
}

/// stderr に 1 行 JSON で書き出す簡易 emitter（dev 用）。
#[derive(Default)]
pub struct LogAuditEmitter;

#[async_trait::async_trait]
impl AuditEmitter for LogAuditEmitter {
    async fn emit(&self, rec: AuditRecord) {
        // RFC8259 JSON line。
        let json = serde_json::json!({
            "tenant_id": rec.tenant_id,
            "actor": rec.actor,
            "action": rec.action,
            "resource": rec.resource,
            "outcome": rec.outcome,
            "code": rec.code,
        });
        eprintln!("k1s0.audit {}", json);
    }
}

/// gRPC ステータスコードから outcome 文字列を導出する。
pub fn outcome_from_code(code: tonic::Code) -> &'static str {
    match code {
        tonic::Code::Ok => "SUCCESS",
        tonic::Code::PermissionDenied | tonic::Code::Unauthenticated => "DENIED",
        _ => "ERROR",
    }
}

/// 共通規約 §「監査自動発火」: WORM 化対象の特権 RPC 集合。
/// Go 側 `privilegedRPCs`（src/tier1/go/internal/common/audit.go）と完全一致するよう保つ。
/// docs NFR-E-MON-001 / NFR-E-MON-002 / NFR-E-MON-004 を満たす。
///
/// 対象選定基準:
///   - Secret アクセス（Get/BulkGet/GetDynamic/Rotate）: NFR-E-MON-002
///   - Decision 評価 + Rule 登録: NFR-E-MON-001 「Decision 評価」+ NFR-E-MON-004
///   - Feature Flag 定義変更（RegisterFlag のみ。Evaluate は高頻度のため除外）: NFR-E-MON-004
///   - Binding 外部送信: NFR-E-MON-001
///   - Workflow 状態変更（Start/Signal/Cancel/Terminate）: NFR-E-MON-001
///   - State 書込（Set/Delete/Transact）: NFR-E-MON-001 「tier1 API 呼び出し」
///   - PubSub 発行（Publish/BulkPublish）: NFR-E-MON-001 同上
pub fn privileged_rpcs() -> HashSet<&'static str> {
    [
        // State 系書込（NFR-E-MON-001）。読取（Get/BulkGet）は高頻度のため除外。
        "k1s0.tier1.state.v1.StateService/Set",
        "k1s0.tier1.state.v1.StateService/Delete",
        "k1s0.tier1.state.v1.StateService/Transact",
        // PubSub 発行（NFR-E-MON-001）。Subscribe は受信側のため除外。
        "k1s0.tier1.pubsub.v1.PubSubService/Publish",
        "k1s0.tier1.pubsub.v1.PubSubService/BulkPublish",
        // Secrets 全アクセス（NFR-E-MON-002）。Set RPC は IDL に存在しないため対象外。
        "k1s0.tier1.secrets.v1.SecretsService/Get",
        "k1s0.tier1.secrets.v1.SecretsService/BulkGet",
        "k1s0.tier1.secrets.v1.SecretsService/GetDynamic",
        "k1s0.tier1.secrets.v1.SecretsService/Rotate",
        // Decision 評価 + 定義変更（NFR-E-MON-001 / NFR-E-MON-004）。
        "k1s0.tier1.decision.v1.DecisionService/Evaluate",
        "k1s0.tier1.decision.v1.DecisionService/BatchEvaluate",
        "k1s0.tier1.decision.v1.DecisionAdminService/RegisterRule",
        // Feature 定義変更（NFR-E-MON-004）。Evaluate* は高頻度のため除外。
        "k1s0.tier1.feature.v1.FeatureAdminService/RegisterFlag",
        // Workflow 状態変更（NFR-E-MON-001）。Query / GetStatus は読取のため除外。
        "k1s0.tier1.workflow.v1.WorkflowService/Start",
        "k1s0.tier1.workflow.v1.WorkflowService/Signal",
        "k1s0.tier1.workflow.v1.WorkflowService/Cancel",
        "k1s0.tier1.workflow.v1.WorkflowService/Terminate",
        // Binding 外部送信（NFR-E-MON-001）。
        "k1s0.tier1.binding.v1.BindingService/Invoke",
        // Audit 自身は loop 防止のため対象外（自動発火しない）。
    ]
    .into_iter()
    .collect()
}

/// 認証クレーム + RPC 情報から `AuditRecord` を組み立てる。
/// `service_method` は `<service>/<method>` 形式（gRPC FullMethod 後半）。
pub fn build_record(
    claims: &AuthClaims,
    service_method: &str,
    code: tonic::Code,
    resource: &str,
) -> AuditRecord {
    AuditRecord {
        tenant_id: claims.tenant_id.clone(),
        actor: if claims.subject.is_empty() {
            "unknown".to_string()
        } else {
            claims.subject.clone()
        },
        action: service_method.to_string(),
        resource: resource.to_string(),
        outcome: outcome_from_code(code).to_string(),
        code: code as i32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outcome_mapping() {
        assert_eq!(outcome_from_code(tonic::Code::Ok), "SUCCESS");
        assert_eq!(outcome_from_code(tonic::Code::PermissionDenied), "DENIED");
        assert_eq!(outcome_from_code(tonic::Code::Unauthenticated), "DENIED");
        assert_eq!(outcome_from_code(tonic::Code::Internal), "ERROR");
    }

    #[test]
    fn privileged_set_contains_state_writes() {
        let s = privileged_rpcs();
        assert!(s.contains("k1s0.tier1.state.v1.StateService/Set"));
        assert!(s.contains("k1s0.tier1.audit.v1.AuditService/Record") == false);
    }

    #[tokio::test]
    async fn noop_emitter_does_not_panic() {
        let e = NoopAuditEmitter;
        e.emit(AuditRecord {
            tenant_id: "T".into(),
            actor: "u".into(),
            action: "x".into(),
            resource: "r".into(),
            outcome: "SUCCESS".into(),
            code: 0,
        })
        .await;
    }
}
