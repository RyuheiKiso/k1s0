// 本ファイルは t1-audit Pod 内で RetentionRunner を周期実行するための tokio task。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/10_Audit_Pii_API.md
//     - FR-T1-AUDIT-003 受け入れ基準:
//       * 1 年経過後、自動的に PostgreSQL → MinIO にエクスポート
//       * 7 年経過後、自動削除（削除操作も Audit ログに記録）
//
// 役割:
//   K8s CronJob 化せず、t1-audit Pod 内に常駐 task として retention を回す簡素な経路。
//   Pod が単一 StatefulSet replica（DS-SW-COMP-007）で稼働する前提なので、複数 Pod 競合は
//   起きない。スケールアウト時はリーダー選出が必要だが、そのケースは将来の対応とする。
//
// 設定:
//   - K1S0_AUDIT_RETENTION_INTERVAL  : tick 間隔（time::ParseDuration 互換、既定 24h）
//   - K1S0_AUDIT_RETENTION_BATCH     : 1 tick あたり 1 tier の最大処理件数（既定 1000、0 で無制限）
//   - K1S0_AUDIT_RETENTION_DISABLED  : "true" で起動しない（test / 単体検証経路）
//
// 失敗時挙動:
//   - tick ごとに run_once が単独で完了する。失敗は warn ログのみで継続（次 tick で再試行）。
//   - tenant 列挙が空 / Err の tick は skip ログを出して何もしない。
//
// shutdown:
//   返り値の JoinHandle を main 側で abort することで停止する（Pod 終了時の graceful path）。

use std::sync::Arc;
use std::time::Duration;

use crate::archival::{ArchivalError, RetentionRunner, RetentionStats};
use crate::archive::ArchiveSink;
use crate::retention::RetentionPolicy;
use crate::store::AuditStore;

/// 既定 tick 間隔（24 時間）。日次 cron 相当。
pub const DEFAULT_RETENTION_INTERVAL_SECS: u64 = 24 * 60 * 60;
/// 既定 1 tier あたり最大処理件数。
pub const DEFAULT_RETENTION_BATCH: usize = 1000;

/// `RetentionLoopConfig` は周期実行設定。env 由来 / test 注入の両方で使う。
#[derive(Debug, Clone)]
pub struct RetentionLoopConfig {
    /// tick 間隔。
    pub interval: Duration,
    /// 1 tick / 1 tier あたりの最大処理件数。0 で無制限。
    pub max_per_tier: usize,
    /// retention 閾値（既定 90 / 365 / 2555 日）。
    pub policy: RetentionPolicy,
}

impl Default for RetentionLoopConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(DEFAULT_RETENTION_INTERVAL_SECS),
            max_per_tier: DEFAULT_RETENTION_BATCH,
            policy: RetentionPolicy::DEFAULT,
        }
    }
}

impl RetentionLoopConfig {
    /// 環境変数から構築する。失敗値は既定にフォールバックして警告ログのみ出す。
    /// `K1S0_AUDIT_RETENTION_DISABLED=true` は呼び出し元の判定責務（本関数では参照しない）。
    pub fn from_env<F>(env: F) -> Self
    where
        F: Fn(&str) -> Option<String>,
    {
        let mut cfg = Self::default();
        if let Some(raw) = env("K1S0_AUDIT_RETENTION_INTERVAL") {
            match parse_duration(&raw) {
                Some(d) if d > Duration::ZERO => cfg.interval = d,
                _ => eprintln!(
                    "tier1/audit: invalid K1S0_AUDIT_RETENTION_INTERVAL={:?}, using default 24h",
                    raw
                ),
            }
        }
        if let Some(raw) = env("K1S0_AUDIT_RETENTION_BATCH") {
            match raw.parse::<usize>() {
                Ok(n) => cfg.max_per_tier = n,
                Err(e) => eprintln!(
                    "tier1/audit: invalid K1S0_AUDIT_RETENTION_BATCH={:?} ({}), using default 1000",
                    raw, e
                ),
            }
        }
        cfg
    }
}

/// `K1S0_AUDIT_RETENTION_DISABLED` の真偽を解釈する。"true" / "1" / "yes" は disabled。
pub fn is_disabled<F>(env: F) -> bool
where
    F: Fn(&str) -> Option<String>,
{
    match env("K1S0_AUDIT_RETENTION_DISABLED")
        .as_deref()
        .map(str::trim)
        .map(str::to_ascii_lowercase)
    {
        Some(s) if matches!(s.as_str(), "true" | "1" | "yes" | "on") => true,
        _ => false,
    }
}

/// 1 tick の retention pass を実行する。tenant 列挙 → run_once → ログ出力までを 1 関数に集約。
/// テストはこの関数を直接呼んで決定論的に検証できる。
pub fn run_pass(
    store: Arc<dyn AuditStore>,
    sink: Arc<dyn ArchiveSink>,
    cfg: &RetentionLoopConfig,
    now_ms: i64,
) -> Result<RetentionStats, ArchivalError> {
    let tenants = store.list_tenants().unwrap_or_else(|e| {
        eprintln!("tier1/audit: retention: list_tenants failed: {}", e);
        Vec::new()
    });
    if tenants.is_empty() {
        // 空 tenant 集合は no-op（warm に何も無いだけ、または store が discovery 未対応）。
        return Ok(RetentionStats::default());
    }
    let runner = RetentionRunner {
        store: store.clone(),
        sink,
        policy: cfg.policy,
        max_per_tier: cfg.max_per_tier,
    };
    runner.run_once(store.as_ref(), &tenants, now_ms)
}

/// 周期実行を spawn する。返り値の JoinHandle を main で abort することで停止できる。
/// disabled=true なら spawn せず None を返す。
pub fn spawn(
    store: Arc<dyn AuditStore>,
    sink: Arc<dyn ArchiveSink>,
    cfg: RetentionLoopConfig,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        // 起動直後に 1 tick 走らせず、interval 経過後から開始する（Pod 起動直後の負荷を避ける）。
        let mut ticker = tokio::time::interval(cfg.interval);
        // 1 回目の tick は Instant::now() で発火するため skip する。
        ticker.tick().await;
        loop {
            ticker.tick().await;
            let now_ms = current_unix_ms();
            match run_pass(store.clone(), sink.clone(), &cfg, now_ms) {
                Ok(stats) => {
                    if stats.warm_to_cold_archived
                        + stats.cold_to_expired_deleted
                        + stats.failures
                        > 0
                    {
                        // 0 件以外の tick はログを残す（運用での観測点）。
                        eprintln!(
                            "tier1/audit: retention pass: archived={} deleted={} expired_audit={} failures={}",
                            stats.warm_to_cold_archived,
                            stats.cold_to_expired_deleted,
                            stats.expired_audit_emitted,
                            stats.failures,
                        );
                    }
                }
                Err(e) => {
                    eprintln!("tier1/audit: retention pass failed: {}", e);
                }
            }
        }
    })
}

/// 現在の Unix milliseconds を返す。`std::time::SystemTime` 失敗（時計取得不可）は 0 を返す。
fn current_unix_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// `parse_duration` の crate-internal 公開ラッパ。verify_loop 等の他 module から再利用される。
pub fn parse_duration_public(raw: &str) -> Option<Duration> {
    parse_duration(raw)
}

/// `time.ParseDuration` 互換の解釈（"24h" / "30m" / "5s" / "100ms"）。
/// 失敗は None。負値 / 0 / 単位なし数値も None として既定にフォールバックさせる。
fn parse_duration(raw: &str) -> Option<Duration> {
    let s = raw.trim();
    if s.is_empty() {
        return None;
    }
    // 後方一致で単位を検出する。
    let (num_str, unit_factor_ns): (&str, u128) = if let Some(rest) = s.strip_suffix("ms") {
        (rest, 1_000_000)
    } else if let Some(rest) = s.strip_suffix("us") {
        (rest, 1_000)
    } else if let Some(rest) = s.strip_suffix("ns") {
        (rest, 1)
    } else if let Some(rest) = s.strip_suffix('s') {
        (rest, 1_000_000_000)
    } else if let Some(rest) = s.strip_suffix('m') {
        (rest, 60 * 1_000_000_000)
    } else if let Some(rest) = s.strip_suffix('h') {
        (rest, 3_600 * 1_000_000_000)
    } else {
        // 単位なしの bare number は誤設定（"86400" は秒なのか分なのか曖昧）として拒否する。
        return None;
    };
    let n: u64 = num_str.trim().parse().ok()?;
    if n == 0 {
        return None;
    }
    let total_ns: u128 = (n as u128).checked_mul(unit_factor_ns)?;
    // u64 ns に収まる範囲のみ許容（最大 ~584 年）。
    let secs: u64 = (total_ns / 1_000_000_000) as u64;
    let nanos: u32 = (total_ns % 1_000_000_000) as u32;
    Some(Duration::new(secs, nanos))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::archive::InMemoryArchiveSink;
    use crate::store::{AppendInput, InMemoryAuditStore};

    fn append_at(store: &InMemoryAuditStore, tenant: &str, ts_ms: i64, actor: &str) {
        let mut attrs = std::collections::BTreeMap::new();
        attrs.insert("seq".into(), actor.into());
        store
            .append(AppendInput {
                timestamp_ms: ts_ms,
                actor: actor.into(),
                action: "READ".into(),
                resource: "r".into(),
                outcome: "SUCCESS".into(),
                attributes: attrs,
                tenant_id: tenant.into(),
            })
            .unwrap();
    }

    #[test]
    fn run_pass_handles_empty_tenants_gracefully() {
        let store: Arc<dyn AuditStore> = Arc::new(InMemoryAuditStore::new());
        let sink: Arc<dyn ArchiveSink> = Arc::new(InMemoryArchiveSink::new());
        let cfg = RetentionLoopConfig::default();
        let stats = run_pass(store, sink, &cfg, 10_000_000_000_000).unwrap();
        // tenant が居なければ何も処理しない。
        assert_eq!(stats.warm_to_cold_archived, 0);
        assert_eq!(stats.cold_to_expired_deleted, 0);
        assert_eq!(stats.failures, 0);
    }

    #[test]
    fn run_pass_archives_old_entries() {
        // 直接 InMemoryAuditStore を共有する（trait object 経由ではなく Arc<concrete> で
        // 初期 append → trait object に再 wrap という流れ）。
        let concrete = Arc::new(InMemoryAuditStore::new());
        let now = 10_000_000_000_000_i64;
        append_at(
            concrete.as_ref(),
            "T",
            now - 400 * RetentionPolicy::ONE_DAY_MS,
            "old",
        );
        append_at(
            concrete.as_ref(),
            "T",
            now - 30 * RetentionPolicy::ONE_DAY_MS,
            "recent",
        );
        let store: Arc<dyn AuditStore> = concrete.clone();
        let sink: Arc<dyn ArchiveSink> = Arc::new(InMemoryArchiveSink::new());
        let cfg = RetentionLoopConfig::default();
        let stats = run_pass(store, sink, &cfg, now).unwrap();
        // 1 件 old だけが移行される。
        assert_eq!(stats.warm_to_cold_archived, 1);
        assert_eq!(stats.warm_to_cold_deleted, 1);
        assert_eq!(stats.failures, 0);
    }

    #[test]
    fn parse_duration_accepts_common_units() {
        assert_eq!(parse_duration("1h"), Some(Duration::from_secs(3600)));
        assert_eq!(parse_duration("30m"), Some(Duration::from_secs(1800)));
        assert_eq!(parse_duration("15s"), Some(Duration::from_secs(15)));
        assert_eq!(parse_duration("250ms"), Some(Duration::from_millis(250)));
    }

    #[test]
    fn parse_duration_rejects_invalid() {
        assert!(parse_duration("").is_none());
        assert!(parse_duration("0h").is_none());
        assert!(parse_duration("12").is_none()); // 単位なしは曖昧として拒否
        assert!(parse_duration("12d").is_none()); // 未対応単位
        assert!(parse_duration("abc").is_none());
    }

    #[test]
    fn from_env_falls_back_on_invalid() {
        let env = |k: &str| match k {
            "K1S0_AUDIT_RETENTION_INTERVAL" => Some("zzz".into()),
            "K1S0_AUDIT_RETENTION_BATCH" => Some("not-a-number".into()),
            _ => None,
        };
        let cfg = RetentionLoopConfig::from_env(env);
        // 無効値は既定にフォールバック。
        assert_eq!(cfg.interval, Duration::from_secs(DEFAULT_RETENTION_INTERVAL_SECS));
        assert_eq!(cfg.max_per_tier, DEFAULT_RETENTION_BATCH);
    }

    #[test]
    fn from_env_applies_valid_overrides() {
        let env = |k: &str| match k {
            "K1S0_AUDIT_RETENTION_INTERVAL" => Some("6h".into()),
            "K1S0_AUDIT_RETENTION_BATCH" => Some("500".into()),
            _ => None,
        };
        let cfg = RetentionLoopConfig::from_env(env);
        assert_eq!(cfg.interval, Duration::from_secs(6 * 3600));
        assert_eq!(cfg.max_per_tier, 500);
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
