// 本ファイルは Rust 共通のテナント単位 token bucket rate limit。
//
// 設計正典:
//   docs/03_要件定義/00_共通規約.md §「レート制限」
//   NFR-A-SLA-*: テナント間の interference 防止
//
// 役割（Go 側 src/tier1/go/internal/common/ratelimit.go と等価）:
//   各テナントに固有の token bucket を持ち、RPC 1 件あたり 1 token を消費する。
//   未消費 token があれば accept、無ければ `tonic::Status::resource_exhausted` を返す。
//   bucket は最終アクセスから一定時間経過するとアイドル GC で破棄される。

// 標準同期。
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

// 非同期 RwLock。
use tokio::sync::RwLock;

/// rate limiter 設定。
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// 1 秒あたりに補充される token 数（持続スループット）。
    pub rps: f64,
    /// バケット最大保持量（バースト）。
    pub burst: f64,
    /// 最終アクセスからこの時間経過した bucket は GC する。
    pub idle_eviction: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            // 既定 100 RPS（Go 側の既定と一致）。
            rps: 100.0,
            // 既定 200 burst。
            burst: 200.0,
            // 既定 5 分のアイドル後に bucket 破棄。
            idle_eviction: Duration::from_secs(300),
        }
    }
}

/// テナント単位の token bucket rate limiter。
pub struct RateLimiter {
    /// 設定。
    cfg: RateLimitConfig,
    /// tenant_id → bucket（並行アクセスのため Arc + Mutex で 1 件をロックする）。
    buckets: RwLock<HashMap<String, Arc<tokio::sync::Mutex<Bucket>>>>,
}

/// 単一テナントの bucket。
#[derive(Debug)]
struct Bucket {
    /// 現在の token 数。
    tokens: f64,
    /// 最終補充 / アクセス時刻。
    last: Instant,
}

impl RateLimiter {
    /// 新規 limiter。`cfg` は既定値を上書きしたい呼出元が渡す。
    pub fn new(cfg: RateLimitConfig) -> Self {
        Self {
            cfg,
            buckets: RwLock::new(HashMap::new()),
        }
    }

    /// 1 トークン消費を試みる。`true` なら accept、`false` なら reject。
    pub async fn try_acquire(&self, tenant_id: &str) -> bool {
        let bucket = self.bucket_for(tenant_id).await;
        let mut g = bucket.lock().await;
        let now = Instant::now();
        let elapsed = now.duration_since(g.last).as_secs_f64();
        // 経過時間に応じて補充する。
        g.tokens = (g.tokens + elapsed * self.cfg.rps).min(self.cfg.burst);
        g.last = now;
        if g.tokens >= 1.0 {
            g.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    /// 該当テナントの bucket（無ければ新規作成）。
    async fn bucket_for(&self, tenant_id: &str) -> Arc<tokio::sync::Mutex<Bucket>> {
        if let Some(b) = self.buckets.read().await.get(tenant_id).cloned() {
            return b;
        }
        let mut w = self.buckets.write().await;
        w.entry(tenant_id.to_string())
            .or_insert_with(|| {
                Arc::new(tokio::sync::Mutex::new(Bucket {
                    tokens: self.cfg.burst,
                    last: Instant::now(),
                }))
            })
            .clone()
    }

    /// アイドルバケットを GC する（明示呼出 / cron task 用）。
    pub async fn evict_idle(&self) {
        let now = Instant::now();
        // 削除候補を集める（lock を抱えたまま削除しないよう 2 phase）。
        let to_remove: Vec<String> = {
            let g = self.buckets.read().await;
            let mut out = Vec::new();
            for (k, b) in g.iter() {
                if let Ok(bg) = b.try_lock() {
                    if now.duration_since(bg.last) > self.cfg.idle_eviction {
                        out.push(k.clone());
                    }
                }
            }
            out
        };
        if to_remove.is_empty() {
            return;
        }
        let mut w = self.buckets.write().await;
        for k in to_remove {
            w.remove(&k);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn allows_within_burst() {
        let l = RateLimiter::new(RateLimitConfig {
            rps: 1.0,
            burst: 5.0,
            idle_eviction: Duration::from_secs(60),
        });
        for _ in 0..5 {
            assert!(l.try_acquire("T1").await);
        }
        // 6 件目は reject（burst 超過）。
        assert!(!l.try_acquire("T1").await);
    }

    #[tokio::test]
    async fn isolates_tenants() {
        let l = RateLimiter::new(RateLimitConfig {
            rps: 1.0,
            burst: 1.0,
            idle_eviction: Duration::from_secs(60),
        });
        assert!(l.try_acquire("T1").await);
        // T1 は枯渇しているが T2 は別 bucket。
        assert!(l.try_acquire("T2").await);
        assert!(!l.try_acquire("T1").await);
    }
}
