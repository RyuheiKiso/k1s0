// 本ファイルは t1-audit Pod 内で hash chain の整合性検証を周期実行する常駐 task。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/10_Audit_Pii_API.md
//     - FR-T1-AUDIT-002 受け入れ基準:
//       * 1 テナント × 1 日分のチェーン検証が 5 分以内に完了
//       * 改ざん検知時にアラート発報
//       * 定期検証（日次）の結果を Grafana で可視化
//
// 役割:
//   日次（既定 24h）に 1 度、`AuditStore::verify_chain_detail("", None, None)` を呼んで
//   グローバル hash chain の連鎖整合性を検証する。invalid 検出時は stderr に WARN を出し、
//   将来 OTel emitter が wire される時点で audit alert event を発火させる起点となる。
//   検証結果は stats として呼び出し元に返し、Grafana 可視化用 metric の足場とする。
//
// 設定:
//   - K1S0_AUDIT_VERIFY_INTERVAL  : tick 間隔（time.ParseDuration 互換、既定 24h）
//   - K1S0_AUDIT_VERIFY_DISABLED  : "true" で起動しない（test / 単体検証経路）
//
// 失敗時挙動:
//   - tick ごとに verify が単独で完了する。失敗（不整合）は warn ログのみで継続し、
//     次 tick で再検証する。永続的な不整合状態は外部監視（log scrape）で alert を発火する。

use std::sync::Arc;
use std::time::Duration;

use crate::store::{AuditStore, StoreError, VerifyOutcome};

/// 既定 tick 間隔（24 時間）。日次検証相当。
pub const DEFAULT_VERIFY_INTERVAL_SECS: u64 = 24 * 60 * 60;

/// `VerifyLoopConfig` は周期実行設定。env 由来 / test 注入の両方で使う。
#[derive(Debug, Clone)]
pub struct VerifyLoopConfig {
    /// tick 間隔。
    pub interval: Duration,
}

impl Default for VerifyLoopConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(DEFAULT_VERIFY_INTERVAL_SECS),
        }
    }
}

impl VerifyLoopConfig {
    /// 環境変数から構築する。失敗値は既定にフォールバックして警告ログのみ出す。
    pub fn from_env<F>(env: F) -> Self
    where
        F: Fn(&str) -> Option<String>,
    {
        let mut cfg = Self::default();
        if let Some(raw) = env("K1S0_AUDIT_VERIFY_INTERVAL") {
            match crate::retention_loop::parse_duration_public(&raw) {
                Some(d) if d > Duration::ZERO => cfg.interval = d,
                _ => eprintln!(
                    "tier1/audit: invalid K1S0_AUDIT_VERIFY_INTERVAL={:?}, using default 24h",
                    raw
                ),
            }
        }
        cfg
    }
}

/// `K1S0_AUDIT_VERIFY_DISABLED` の真偽を解釈する。"true" / "1" / "yes" / "on" は disabled。
pub fn is_disabled<F>(env: F) -> bool
where
    F: Fn(&str) -> Option<String>,
{
    match env("K1S0_AUDIT_VERIFY_DISABLED")
        .as_deref()
        .map(str::trim)
        .map(str::to_ascii_lowercase)
    {
        Some(s) if matches!(s.as_str(), "true" | "1" | "yes" | "on") => true,
        _ => false,
    }
}

/// 1 tick の verify pass を実行する。tenant 不問で global チェーンを末尾まで検証する。
/// テストはこの関数を直接呼んで決定論的に検証できる。
pub fn run_pass(store: Arc<dyn AuditStore>) -> Result<VerifyOutcome, StoreError> {
    store.verify_chain_detail("", None, None)
}

/// 周期実行を spawn する。返り値の JoinHandle を main で abort することで停止できる。
pub fn spawn(store: Arc<dyn AuditStore>, cfg: VerifyLoopConfig) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(cfg.interval);
        // 1 回目の tick は Instant::now() で発火するため skip（Pod 起動直後の負荷を避ける）。
        ticker.tick().await;
        loop {
            ticker.tick().await;
            match run_pass(store.clone()) {
                Ok(outcome) if outcome.valid => {
                    eprintln!(
                        "tier1/audit: verify_chain pass: valid=true checked_count={}",
                        outcome.checked_count
                    );
                }
                Ok(outcome) => {
                    // 不整合を検出。WARN として残し、外部監視（Grafana log alerting）で
                    // ピックアップされる前提。FR-T1-AUDIT-002 「改ざん検知時にアラート発報」。
                    eprintln!(
                        "tier1/audit: ALERT chain integrity violation: first_bad_sequence={} reason={} checked_count={}",
                        outcome.first_bad_sequence, outcome.reason, outcome.checked_count
                    );
                }
                Err(e) => {
                    eprintln!("tier1/audit: verify_chain pass failed: {}", e);
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{AppendInput, InMemoryAuditStore};

    #[test]
    fn run_pass_returns_valid_for_clean_chain() {
        let store: Arc<dyn AuditStore> = Arc::new(InMemoryAuditStore::new());
        // 空 store は valid（検証対象 0 件）。
        let outcome = run_pass(store.clone()).unwrap();
        assert!(outcome.valid);
        assert_eq!(outcome.checked_count, 0);
    }

    #[test]
    fn run_pass_traverses_all_appended_entries() {
        let concrete = Arc::new(InMemoryAuditStore::new());
        for i in 1..=3 {
            concrete
                .append(AppendInput {
                    timestamp_ms: i * 1000,
                    actor: format!("user{}", i),
                    action: "READ".into(),
                    resource: "r".into(),
                    outcome: "SUCCESS".into(),
                    attributes: std::collections::BTreeMap::new(),
                    tenant_id: "T".into(),
                })
                .unwrap();
        }
        let store: Arc<dyn AuditStore> = concrete;
        let outcome = run_pass(store).unwrap();
        assert!(outcome.valid);
        assert_eq!(outcome.checked_count, 3);
        assert_eq!(outcome.first_bad_sequence, 0);
        assert!(outcome.reason.is_empty());
    }

    #[test]
    fn from_env_falls_back_on_invalid() {
        let env = |k: &str| match k {
            "K1S0_AUDIT_VERIFY_INTERVAL" => Some("zzz".into()),
            _ => None,
        };
        let cfg = VerifyLoopConfig::from_env(env);
        assert_eq!(cfg.interval, Duration::from_secs(DEFAULT_VERIFY_INTERVAL_SECS));
    }

    #[test]
    fn from_env_applies_valid_overrides() {
        let env = |k: &str| match k {
            "K1S0_AUDIT_VERIFY_INTERVAL" => Some("12h".into()),
            _ => None,
        };
        let cfg = VerifyLoopConfig::from_env(env);
        assert_eq!(cfg.interval, Duration::from_secs(12 * 3600));
    }

    #[test]
    fn is_disabled_recognizes_truthy_values() {
        for v in ["true", "1", "yes", "ON", " True "] {
            assert!(is_disabled(|_k| Some(v.into())), "should disable for {v:?}");
        }
        for v in ["false", "0", "", "no", "off"] {
            assert!(!is_disabled(|_k| Some(v.into())), "should not disable for {v:?}");
        }
        assert!(!is_disabled(|_k| None));
    }
}
