// cache.rs — k1s0 tier1 Library: キャッシュ操作の L2* facade trait
// Valkey/Redis 等の OSS 型を公開 API に露出しない（L2* ラップ規約）。
// wall-clock TTL 禁止規約: TTL は HLC ミリ秒（u64）のみを受け付ける。
// 全 trait は Send + Sync を要求する（スレッド安全性の強制）。

// async_trait: async fn in trait を stable で使用するためのマクロ
use async_trait::async_trait;

// Cache は Valkey/Redis を L2* ラップするキャッシュ操作 facade trait。
// 公開 API シグネチャに OSS 型（redis::Client 等）を一切含まない。
// wall-clock TTL は禁止し、HLC ミリ秒のみを TTL として受け付ける。
#[async_trait]
pub trait Cache: Send + Sync {
    // get はキーに対応する値を取得する（存在しない場合は None を返す）。
    // キーが HLC 期限切れの場合も None を返す（wall-clock 判定禁止）。
    async fn get(&self, key: &str) -> crate::Result<Option<Vec<u8>>>;

    // put はキーと値を TTL 付きで格納する（TTL は HLC ミリ秒で指定する）。
    // ttl_hlc_ms が 0 の場合は無期限キャッシュとして保存する。
    // wall-clock TTL 禁止規約: std::time::Duration / DateTime::UtcNow 等の使用禁止。
    async fn put(&self, key: &str, val: Vec<u8>, ttl_hlc_ms: u64) -> crate::Result<()>;

    // del はキーを削除する（キーが存在しない場合はエラーにならない）。
    // idempotent な操作として設計する。
    async fn del(&self, key: &str) -> crate::Result<()>;
}
