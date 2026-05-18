// cache.rs — k1s0 tier1 Library backend: KeyValue/Cache L3 trait
// backend 専用カテゴリ（frontend には提供しない）。
// 公開 API に OSS 型（redis::Client / bb8::Pool / deadpool_redis::Pool 等）を露出しない。
// wall-clock TTL 禁止: TTL は HLC tick 数（u64）で渡す。
// 全 trait は Send + Sync を要求する。

// async_trait: async fn in trait を stable で使用するためのマクロ
use async_trait::async_trait;
// anyhow: エラーハンドリング（Result 型の統一）
use anyhow::Result;
// serde: キャッシュ値のシリアライズに使用する
use serde::{Deserialize, Serialize};
// AuthContext: キャッシュ操作に認証コンテキストを伝播する
use crate::core::auth::AuthContext;

// CacheEntry はキャッシュの 1 エントリを表す Library 独自型。
// OSS 固有の TTL 型（Duration 等）を使わず HLC tick 数で期限を管理する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    // key: キャッシュキー（テナント ID が prefix として埋め込まれる）
    pub key: String,
    // value_bytes: キャッシュ値のバイナリ表現（シリアライズ形式は実装が決定する）
    pub value_bytes: Vec<u8>,
    // expires_at_hlc_tick: 期限切れ HLC tick 数（0 は無期限）
    // wall-clock TTL 禁止のため HLC tick で表現する
    pub expires_at_hlc_tick: u64,
    // tenant_id: このエントリが属するテナントの識別子（マルチテナント分離）
    pub tenant_id: String,
}

// CacheGetResult はキャッシュ取得の結果を表す enum。
#[derive(Debug)]
pub enum CacheGetResult {
    // Hit: キャッシュヒット（CacheEntry を伴う）
    Hit(CacheEntry),
    // Miss: キャッシュミス（キーが存在しない）
    Miss,
    // Expired: 期限切れ（エントリは存在するが HLC tick が超過している）
    Expired,
}

// CacheSetOptions はキャッシュ書き込みオプションを表す struct。
#[derive(Debug, Clone)]
pub struct CacheSetOptions {
    // ttl_ticks: 生存 HLC tick 数（0 は無期限）
    // wall-clock TTL 禁止のため HLC tick で表現する
    pub ttl_ticks: u64,
    // overwrite: 既存エントリを上書きするかどうか（false は NX セマンティクス）
    pub overwrite: bool,
    // replicate: レプリカへの書き込みを待つかどうか（強整合性が必要な場合に使う）
    pub replicate: bool,
}

// CacheSetOptions のデフォルト値（TTL なし / 上書きあり / レプリカ待ちなし）
impl Default for CacheSetOptions {
    fn default() -> Self {
        // 汎用的なデフォルト設定
        Self {
            // ttl_ticks: 0（無期限）
            ttl_ticks: 0,
            // overwrite: true（既存エントリを上書きする）
            overwrite: true,
            // replicate: false（レプリカへの書き込みを待たない）
            replicate: false,
        }
    }
}

// Cache は KeyValue/Cache の L3 抽象 trait。
// Redis / Memcached 等の OSS を実装で切り替えられる。
// 公開 API に OSS 型を露出しない。
#[async_trait]
pub trait Cache: Send + Sync {
    // get はキーを受け取り、CacheGetResult を返す。
    // auth_ctx はアクセス制御と audit ログに使用する（tenant_id の境界を保証する）。
    async fn get(&self, key: &str, auth_ctx: &AuthContext) -> Result<CacheGetResult>;

    // set はキーと値と書き込みオプションを受け取り、CacheEntry を保存する。
    // auth_ctx はアクセス制御と audit ログに使用する。
    async fn set(
        &self,
        key: &str,
        value_bytes: Vec<u8>,
        options: CacheSetOptions,
        auth_ctx: &AuthContext,
    ) -> Result<()>;

    // delete はキーを受け取り、CacheEntry を削除する。
    // 存在しないキーを削除してもエラーにならない（idempotent）。
    async fn delete(&self, key: &str, auth_ctx: &AuthContext) -> Result<()>;

    // exists はキーが存在するかどうかを返す（期限切れは Miss 扱い）。
    async fn exists(&self, key: &str, auth_ctx: &AuthContext) -> Result<bool>;

    // flush_tenant は tenant_id のキャッシュを全削除する（テナント解約時等に使用する）。
    async fn flush_tenant(&self, auth_ctx: &AuthContext) -> Result<u64>;
}
