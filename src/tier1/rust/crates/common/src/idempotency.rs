// 本ファイルは Rust 共通の idempotency cache 実装。
//
// 設計正典:
//   docs/03_要件定義/00_共通規約.md §「冪等性と再試行」
//
// 役割:
//   Go 側 `src/tier1/go/internal/common/idempotency.go` と等価の機能を提供する。
//   tier1 内部 RPC のうち、副作用を伴うもの（Audit.Record / PubSub.Publish 等）が
//   `idempotency_key` を受け取った場合、24 時間 TTL の in-memory cache に
//   初回応答を記録し、同一キーの再試行で副作用を再実行せずに同一応答を返す。
//
// API 設計:
//   Rust の async trait + クロージャ引数は dyn 互換性の制約が強いため、本実装は
//   `lookup` / `store` の単純な 2 段階に分け、呼出側が自分で singleflight を
//   組み立てる素朴な API とする。Audit Pod の handler は本 cache を内部 Mutex 配下で
//   操作することで、Go 側 `GetOrCompute` と等価な保証を満たせる。
//
// 永続化:
//   in-memory のみ。Pod 再起動で揮発する（共通規約上、TTL 24h は best-effort）。

// 標準同期型。
use std::collections::HashMap;
// 単調時刻（cache TTL）。
use std::time::{Duration, Instant};

// 非同期 RwLock（cache map 全体の保護）。
use tokio::sync::RwLock;

/// `IdempotencyCache` は副作用付き RPC の重複抑止 cache trait。
///
/// `key` の名前空間衝突を避けるため、呼び出し側は `tenant_id + ":" + rpc + ":" + req_key`
/// の形で文字列を構成して渡すこと（`idempotency_key` helper を使う）。
///
/// 値型は実装単純化のため `Vec<u8>`（任意 bytes）。proto Message を encode した bytes を
/// 格納し、再現時に decode することを呼び出し側で行う。
#[async_trait::async_trait]
pub trait IdempotencyCache: Send + Sync + 'static {
    /// 値を lookup する。期限切れ / 未存在は `None`。
    async fn lookup(&self, key: &str) -> Option<Vec<u8>>;

    /// 値を保存する。同一 key の上書きは許容（再書込で TTL は更新される）。
    async fn store(&self, key: &str, value: Vec<u8>);
}

/// `InMemoryIdempotencyCache` は in-memory backend の `IdempotencyCache` 実装。
///
/// TTL 既定値は 24h（Go 側 `defaultIdempotencyTTL` と一致）。
pub struct InMemoryIdempotencyCache {
    /// cache 本体（key → (value, expires_at)）。
    inner: RwLock<HashMap<String, CacheEntry>>,
    /// エントリ TTL。
    ttl: Duration,
}

/// cache に入る 1 件分のエントリ。
struct CacheEntry {
    /// 保存値。
    value: Vec<u8>,
    /// この時刻を過ぎたら無効（GC 対象）。
    expires_at: Instant,
}

impl InMemoryIdempotencyCache {
    /// 新規 cache を作成する。`ttl` が 0 の場合は 24 時間扱い。
    pub fn new(ttl: Duration) -> Self {
        let effective = if ttl.is_zero() {
            Duration::from_secs(24 * 3600)
        } else {
            ttl
        };
        Self {
            inner: RwLock::new(HashMap::new()),
            ttl: effective,
        }
    }

    /// cache の有効エントリ数（テスト用）。
    pub async fn len(&self) -> usize {
        let now = Instant::now();
        let g = self.inner.read().await;
        g.values().filter(|e| e.expires_at > now).count()
    }

    /// 期限切れエントリを物理削除する（GC、明示呼出のみ）。
    pub async fn sweep_expired(&self) {
        let now = Instant::now();
        let mut w = self.inner.write().await;
        w.retain(|_, e| e.expires_at > now);
    }
}

#[async_trait::async_trait]
impl IdempotencyCache for InMemoryIdempotencyCache {
    async fn lookup(&self, key: &str) -> Option<Vec<u8>> {
        let now = Instant::now();
        let g = self.inner.read().await;
        g.get(key)
            .filter(|e| e.expires_at > now)
            .map(|e| e.value.clone())
    }

    async fn store(&self, key: &str, value: Vec<u8>) {
        let entry = CacheEntry {
            value,
            expires_at: Instant::now() + self.ttl,
        };
        self.inner.write().await.insert(key.to_string(), entry);
    }
}

/// 共通規約 §「冪等性と再試行」: 名前空間衝突を避けるため
/// `tenant_id:rpc:req_key` で完全修飾する helper。
///
/// `req_key` が空文字なら空文字を返し、呼出元が dedup を skip できるようにする
/// （Go 側 `common.IdempotencyKey` と完全互換）。
pub fn idempotency_key(tenant_id: &str, rpc: &str, req_key: &str) -> String {
    if req_key.is_empty() {
        return String::new();
    }
    format!("{}:{}:{}", tenant_id, rpc, req_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn idempotency_key_returns_empty_on_empty_req_key() {
        assert_eq!(idempotency_key("T", "Rpc", ""), "");
    }

    #[tokio::test]
    async fn idempotency_key_concatenates_components() {
        assert_eq!(idempotency_key("T", "Rpc", "k"), "T:Rpc:k");
    }

    #[tokio::test]
    async fn store_and_lookup_round_trip() {
        let c = InMemoryIdempotencyCache::new(Duration::from_secs(60));
        assert!(c.lookup("k").await.is_none());
        c.store("k", b"v1".to_vec()).await;
        assert_eq!(c.lookup("k").await.as_deref(), Some(&b"v1"[..]));
        // 上書き保存は最新値を返す。
        c.store("k", b"v2".to_vec()).await;
        assert_eq!(c.lookup("k").await.as_deref(), Some(&b"v2"[..]));
    }

    #[tokio::test]
    async fn cache_expires_after_ttl() {
        let c = InMemoryIdempotencyCache::new(Duration::from_millis(50));
        c.store("k", b"v1".to_vec()).await;
        tokio::time::sleep(Duration::from_millis(80)).await;
        assert!(c.lookup("k").await.is_none());
    }

    #[tokio::test]
    async fn sweep_expired_removes_old_entries() {
        let c = InMemoryIdempotencyCache::new(Duration::from_millis(50));
        c.store("k1", b"v1".to_vec()).await;
        c.store("k2", b"v2".to_vec()).await;
        tokio::time::sleep(Duration::from_millis(80)).await;
        c.sweep_expired().await;
        assert_eq!(c.len().await, 0);
    }
}
