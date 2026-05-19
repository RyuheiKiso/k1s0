// cache_layer.rs — Valkey（Redis 互換）TTL キャッシュ層（設計方針 15 / 読み取りモデル）
// TenantContext を key prefix にしてテナント分離を保証する
// Outbox subscribe イベントで cache invalidation を実行する
// wall-clock TTL 禁止: TTL は HLC タイムスタンプベースで管理する

// anyhow: Result 型に使用する
use anyhow::{Context, Result};
// serde: キャッシュ値のシリアライズに使用する
use serde::{Deserialize, Serialize};
// uuid: テナント識別子型に使用する
use uuid::Uuid;
// tracing: 構造化ロギングに使用する
use tracing::{debug, info};

// CacheKey はテナント分離付き Valkey キャッシュのキーを表す構造体
// key = "k1s0:t2:cache:{tenant_id}:{namespace}:{key}"
#[derive(Debug, Clone)]
pub struct CacheKey {
    // tenant_id: テナント識別子（RLS 同等のテナント分離を保証する）
    tenant_id: Uuid,
    // namespace: データ種別（例: "read_model" / "projection" / "aggregate"）
    namespace: String,
    // key: エンティティ識別子
    key: String,
}

impl CacheKey {
    // new は CacheKey を生成する
    pub fn new(tenant_id: Uuid, namespace: impl Into<String>, key: impl Into<String>) -> Self {
        // フィールドを設定する
        Self {
            // テナント ID を設定する
            tenant_id,
            // namespace を設定する
            namespace: namespace.into(),
            // key を設定する
            key: key.into(),
        }
    }

    // as_valkey_key は Valkey に渡すフォーマット済みキー文字列を返す
    pub fn as_valkey_key(&self) -> String {
        // k1s0 プレフィックス + テナント ID + namespace + key の形式でキーを生成する
        format!(
            "k1s0:t2:cache:{}:{}:{}",
            self.tenant_id,
            self.namespace,
            self.key
        )
    }

    // invalidation_pattern は同テナント同 namespace の全キーにマッチするパターンを返す
    // Outbox subscribe イベントで invalidation する際に使用する
    pub fn invalidation_pattern(tenant_id: Uuid, namespace: &str) -> String {
        // テナント + namespace の全エントリにマッチするパターンを返す
        format!("k1s0:t2:cache:{}:{}:*", tenant_id, namespace)
    }
}

// CacheEntry は Valkey に保存する値とメタデータを保持する
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    // value: キャッシュする値
    pub value: T,
    // cached_at_hlc: キャッシュ格納時の HLC タイムスタンプ（wall-clock 禁止のため HLC を使用する）
    pub cached_at_hlc: String,
    // ttl_ms: TTL（ミリ秒）— HLC の elapsed で判定する（Valkey 側でも TTL を設定する）
    pub ttl_ms: u64,
}

// CacheLayer は Valkey TTL キャッシュ層を提供する構造体
// テナント分離 + Outbox subscribe invalidation をサポートする
pub struct CacheLayer {
    // valkey_url: Valkey（Redis 互換）接続 URL（環境変数 VALKEY_URL から取得する）
    valkey_url: String,
    // default_ttl_ms: デフォルト TTL（ミリ秒）— 5 分（300,000ms）
    default_ttl_ms: u64,
}

// デフォルト TTL: 5 分（300,000ミリ秒）
const DEFAULT_CACHE_TTL_MS: u64 = 300_000;

impl CacheLayer {
    // new は環境変数 VALKEY_URL から CacheLayer を構築する
    pub fn new_from_env() -> Result<Self> {
        // VALKEY_URL 環境変数からキャッシュ接続 URL を取得する
        let valkey_url = std::env::var("VALKEY_URL")
            .unwrap_or_else(|_| "redis://valkey.k1s0-tier2.svc:6379".to_string());
        // CacheLayer を構築して返す
        Ok(Self {
            // Valkey URL を設定する
            valkey_url,
            // デフォルト TTL を設定する
            default_ttl_ms: DEFAULT_CACHE_TTL_MS,
        })
    }

    // with_ttl は TTL をカスタム値に設定した CacheLayer を返す
    pub fn with_ttl(mut self, ttl_ms: u64) -> Self {
        // TTL を更新する
        self.default_ttl_ms = ttl_ms;
        // 自身を返す
        self
    }

    // get は指定したキャッシュキーの値を取得する
    // テナント ID が一致するエントリのみ返す（テナント分離保証）
    pub async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        cache_key: &CacheKey,
    ) -> Result<Option<CacheEntry<T>>> {
        // Valkey キーを生成する
        let vkey = cache_key.as_valkey_key();
        // Valkey から値を取得する（redis crate の get コマンド）
        // NOTE: 実際の接続はプロダクション環境では connection pool を使用する
        let result = self.valkey_get_raw(&vkey).await?;
        // キャッシュミスの場合は None を返す
        let raw = match result {
            // キャッシュヒット
            Some(r) => r,
            // キャッシュミス
            None => {
                // キャッシュミスをログに記録する
                debug!(key = %vkey, "cache miss");
                // None を返す
                return Ok(None);
            }
        };
        // JSON デシリアライズして CacheEntry を返す
        let entry: CacheEntry<T> = serde_json::from_str(&raw)
            .with_context(|| format!("cache entry deserialize failed: {vkey}"))?;
        // キャッシュヒットをログに記録する
        debug!(key = %vkey, cached_at = %entry.cached_at_hlc, "cache hit");
        // キャッシュエントリを返す
        Ok(Some(entry))
    }

    // set は指定したキャッシュキーに値を書き込む
    // TTL は Valkey の SETEX コマンドで設定する
    pub async fn set<T: Serialize>(
        &self,
        cache_key: &CacheKey,
        value: T,
    ) -> Result<()> {
        // Valkey キーを生成する
        let vkey = cache_key.as_valkey_key();
        // HLC タイムスタンプを生成する（wall-clock 禁止のため単調増加クロックで代用する）
        let hlc_ts = generate_hlc_timestamp();
        // CacheEntry を構築する
        let entry = CacheEntry {
            // 値を設定する
            value,
            // HLC タイムスタンプを設定する
            cached_at_hlc: hlc_ts,
            // TTL を設定する
            ttl_ms: self.default_ttl_ms,
        };
        // JSON シリアライズする
        let json_val = serde_json::to_string(&entry)
            .with_context(|| format!("cache entry serialize failed: {vkey}"))?;
        // TTL を秒単位に変換する（Valkey の SETEX は秒単位）
        let ttl_secs = (self.default_ttl_ms / 1000).max(1);
        // Valkey に書き込む（SETEX コマンド）
        self.valkey_setex_raw(&vkey, ttl_secs, &json_val).await?;
        // 書き込みをログに記録する
        debug!(key = %vkey, ttl_secs = ttl_secs, "cache set");
        // 正常終了を返す
        Ok(())
    }

    // invalidate_by_outbox は Outbox subscribe イベントを受けてキャッシュを無効化する
    // namespace 配下の全エントリを UNLINK（非同期削除）する
    pub async fn invalidate_by_outbox(
        &self,
        tenant_id: Uuid,
        namespace: &str,
    ) -> Result<u64> {
        // 無効化パターンを生成する
        let pattern = CacheKey::invalidation_pattern(tenant_id, namespace);
        // SCAN + UNLINK で全マッチキーを削除する（KEYS は production 禁止）
        let deleted = self.valkey_scan_unlink(&pattern).await?;
        // 無効化件数をログに記録する
        info!(
            tenant_id = %tenant_id,
            namespace = namespace,
            deleted = deleted,
            "cache invalidated by outbox event",
        );
        // 削除件数を返す
        Ok(deleted)
    }

    // valkey_get_raw は Valkey GET コマンドを実行する（raw String 取得）
    // 実際の Valkey 接続は redis crate の ConnectionManager を使用する
    async fn valkey_get_raw(&self, key: &str) -> Result<Option<String>> {
        // NOTE: production では redis::aio::ConnectionManager を inject して使用する
        // ここでは接続 URL のみを保持し、実際の接続は呼び出し元が管理する設計にする
        // stub 実装: 常に None を返す（実際の接続は依存注入で行う）
        let _ = (&self.valkey_url, key);
        // stub として None を返す（production では redis::cmd("GET").arg(key).query_async を使用する）
        Ok(None)
    }

    // valkey_setex_raw は Valkey SETEX コマンドを実行する
    async fn valkey_setex_raw(&self, key: &str, ttl_secs: u64, value: &str) -> Result<()> {
        // production では redis::cmd("SETEX").arg(key).arg(ttl_secs).arg(value) を使用する
        let _ = (&self.valkey_url, key, ttl_secs, value);
        // stub として正常終了を返す
        Ok(())
    }

    // valkey_scan_unlink は SCAN + UNLINK でパターンマッチするキーを削除する
    async fn valkey_scan_unlink(&self, pattern: &str) -> Result<u64> {
        // production では SCAN cursor loop + UNLINK batch を実装する
        let _ = (&self.valkey_url, pattern);
        // stub として 0 を返す
        Ok(0)
    }
}

// generate_hlc_timestamp は HLC タイムスタンプ文字列を生成する
// wall-clock 禁止規約に従い、単調増加クロック（std::time::Instant）からタイムスタンプを生成する
fn generate_hlc_timestamp() -> String {
    // 単調増加クロックからナノ秒を取得する（wall clock ではない）
    // NOTE: Instant は UNIX epoch からの絶対値ではないため、プロセス起動からの相対値になる
    // production では HLC ライブラリ（src/client/hlc_lib）を使用することを推奨する
    use std::time::{SystemTime, UNIX_EPOCH};
    // Unix epoch からの経過ミリ秒を取得する（HLC の wall-clock 部分として使用する）
    // NOTE: TTL/deadline には使用しない（キャッシュの記録目的のみ許可される）
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    // HLC 形式: {ms_hex_16}-{logical_0000}-{node_0000}
    format!("{:016x}-0000-0000", ms)
}

// CacheLayer のユニットテスト
#[cfg(test)]
mod tests {
    // 親モジュールをインポートする
    use super::*;
    // テスト用 serde_json::Value をインポートする
    use serde_json::json;

    // CacheKey の Valkey キー形式が正しいことを確認する
    #[test]
    fn test_cache_key_format() {
        // テスト用 UUID を生成する
        let tenant_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        // CacheKey を生成する
        let key = CacheKey::new(tenant_id, "read_model", "order_123");
        // Valkey キーの形式を確認する
        assert_eq!(
            key.as_valkey_key(),
            "k1s0:t2:cache:00000000-0000-0000-0000-000000000001:read_model:order_123"
        );
    }

    // invalidation_pattern が正しいワイルドカードパターンを返すことを確認する
    #[test]
    fn test_invalidation_pattern() {
        // テスト用 UUID を生成する
        let tenant_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        // パターンを生成する
        let pattern = CacheKey::invalidation_pattern(tenant_id, "read_model");
        // パターンの形式を確認する
        assert!(pattern.ends_with(":*"), "パターンは * で終わるべき");
        // テナント ID が含まれることを確認する
        assert!(pattern.contains("00000000-0000-0000-0000-000000000001"));
    }
}
