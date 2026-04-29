// 本ファイルは tier1 Rust Pod 共通の runtime 構築ヘルパ。
//
// 設計正典:
//   docs/03_要件定義/00_共通規約.md §「認証認可 / レート制限 / 監査自動発火」
//
// 役割（Go 側 src/tier1/go/internal/common/runtime.go と等価）:
//   3 Pod の main 関数からまったく同じ手順で
//     - Authenticator（環境変数 AUTH_MODE 駆動）
//     - RateLimiter（既定 100 RPS / 200 burst）
//     - AuditEmitter（環境変数 AUDIT_MODE 駆動: noop / log）
//     - InMemoryIdempotencyCache（24h TTL）
//   を構築できるよう、薄い factory を提供する。
//
//   AuditEmitter は将来 gRPC client 経由で k1s0_audit Pod に送るモードを追加する
//   余地を残す（noop / log の 2 mode のみ本リリース）。

// 標準同期。
use std::sync::Arc;
// 標準時刻。
use std::time::Duration;

// 共通 module。
use crate::audit::{AuditEmitter, LogAuditEmitter, NoopAuditEmitter};
use crate::auth::Authenticator;
use crate::idempotency::{IdempotencyCache, InMemoryIdempotencyCache};
use crate::ratelimit::{RateLimitConfig, RateLimiter};

/// k1s0 共通 runtime に必要な依存物の束。
#[derive(Clone)]
pub struct CommonRuntime {
    /// JWT 認証器。
    pub auth: Arc<Authenticator>,
    /// テナント単位 rate limiter。
    pub rate_limiter: Arc<RateLimiter>,
    /// audit emitter。
    pub audit_emitter: Arc<dyn AuditEmitter>,
    /// idempotency cache（共通規約 §「冪等性と再試行」）。
    pub idempotency: Arc<dyn IdempotencyCache>,
}

impl CommonRuntime {
    /// 環境変数から CommonRuntime を構築する。
    /// すべての設定が任意なので、最小では `from_env()` 一発で動く（off auth / 既定 RPS）。
    pub fn from_env() -> Self {
        Self {
            auth: Arc::new(Authenticator::from_env()),
            rate_limiter: Arc::new(RateLimiter::new(RateLimitConfig::default())),
            audit_emitter: load_audit_emitter_from_env(),
            idempotency: Arc::new(InMemoryIdempotencyCache::new(Duration::ZERO)),
        }
    }
}

/// 環境変数 `AUDIT_MODE` から audit emitter を構築する。
/// 既定 / "noop" は NoopAuditEmitter、"log" は stderr に書く LogAuditEmitter。
pub fn load_audit_emitter_from_env() -> Arc<dyn AuditEmitter> {
    match std::env::var("AUDIT_MODE")
        .unwrap_or_default()
        .to_lowercase()
        .as_str()
    {
        "log" => Arc::new(LogAuditEmitter),
        _ => Arc::new(NoopAuditEmitter),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn from_env_constructs_all_components() {
        // 環境変数未設定でも構築できる（off auth / noop audit / 100rps）。
        let rt = CommonRuntime::from_env();
        // dev 既定 claims を返す。
        let c = rt.auth.verify_bearer(None).await.unwrap();
        assert_eq!(c.tenant_id, "demo-tenant");
        // rate limiter は最初の token を許可する。
        assert!(rt.rate_limiter.try_acquire("T1").await);
        // idempotency は空。
        assert!(rt.idempotency.lookup("k").await.is_none());
    }
}
