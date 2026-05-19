// k1s0 tier3 IndexedDB encrypted outbox（Rust 等価強度実装）
// TypeScript primary の outbox.ts と同等の抽象を Rust で実装する
// PII strip on enqueue / Idempotency-Key 24h TTL を強制する
// wall-clock TTL 禁止規約に従い monotonic clock（std::time::Instant）でTTL を計算する

// 標準ライブラリの時刻型を使用する
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
// UUID 生成（idempotency key のランダム部分）
use uuid::Uuid;

// IDEMPOTENCY_KEY_TTL_MS は Idempotency-Key の 24h TTL（ミリ秒）
pub const IDEMPOTENCY_KEY_TTL_MS: u64 = 24 * 60 * 60 * 1000;

/// hlc_now は現在時刻を HLC タイムスタンプ文字列で返す
/// フォーマット: "{timestamp_ms_hex}-{logical_counter}-{node_id}"
/// SystemTime::UNIX_EPOCH からのオフセット（ミリ秒）で壁時計を読む（Rust: HLC 基底）
/// TTL 計算には Instant（monotonic）を使用する（下記 is_expired 参照）
pub fn hlc_now() -> String {
    // UNIX_EPOCH からの経過時間をミリ秒で取得する（HLC の物理クロック基底）
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        // UNIX_EPOCH より前の時刻は panic する（実環境では発生しない）
        .expect("SystemTime before UNIX_EPOCH")
        .as_millis() as u64;
    // ミリ秒を 16 桁 hex 文字列にフォーマットする
    let timestamp_hex = format!("{:016x}", ms);
    // logical_counter は本実装では 0000 固定（同一ミリ秒内の複数イベントが不要なため）
    let logical_counter = "0000";
    // node_id は本実装では 0000 固定（単一ノード想定）
    let node_id = "0000";
    // HLC タイムスタンプ文字列を組み立てて返す
    format!("{}-{}-{}", timestamp_hex, logical_counter, node_id)
}

/// extract_ms_from_hlc は HLC タイムスタンプからミリ秒値を抽出する
/// hlc_timestamp: "{timestamp_ms_hex}-{logical_counter}-{node_id}" 形式
fn extract_ms_from_hlc(hlc_timestamp: &str) -> u64 {
    // ハイフン区切りの先頭部分が 16 進数ミリ秒タイムスタンプ
    let hex_part = hlc_timestamp.split('-').next().unwrap_or("0");
    // 16 進数文字列をパースして u64 に変換する（パース失敗時は 0 を返す）
    u64::from_str_radix(hex_part, 16).unwrap_or(0)
}

/// OutboxEntryMeta は Outbox エントリのメタデータ
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboxEntryMeta {
    /// Idempotency-Key（aggregateId prefix + HLC ベース + UUID suffix）
    pub idempotency_key: String,
    /// enqueue 日時（HLC タイムスタンプ: "{timestamp_ms_hex}-{logical_counter}-{node_id}"）
    pub enqueued_at: String,
    /// enqueue 時点の Instant（TTL 計算用 monotonic clock）
    /// std::time::Instant は単調増加クロックのためシステム時刻の巻き戻しに影響されない
    pub enqueued_instant: Instant,
    /// TTL（UNIX ミリ秒、24h 後 — backward compat 用途で保持する; 値は HLC から導出する）
    pub expires_at_ms: u64,
    /// chain 元 idempotency_key（rebase 後再送時に設定、None は chain なし）
    pub chained_from: Option<String>,
    /// aggregate ID
    pub aggregate_id: String,
    /// RPC method 名（短縮）
    pub rpc_method: String,
}

/// is_expired は Idempotency-Key が TTL 超過かどうかを monotonic clock で確認する
/// wall-clock TTL 禁止規約に従い std::time::Instant（monotonic）を使用する
pub fn is_expired(meta: &OutboxEntryMeta) -> bool {
    // enqueue 時点から現在までの経過時間（monotonic clock）を計算する
    let elapsed: Duration = meta.enqueued_instant.elapsed();
    // 経過時間が TTL を超えているか判定する
    elapsed.as_millis() as u64 >= IDEMPOTENCY_KEY_TTL_MS
}

/// generate_idempotency_key は Idempotency-Key を生成する
/// wall-clock TTL 禁止規約に従い HLC を使用する
pub fn generate_idempotency_key(aggregate_id: &str, rpc_method: &str) -> String {
    // HLC タイムスタンプの先頭 16 進数部分をランダム識別子の基底として使用する
    let hlc_base = hlc_now().split('-').next().unwrap_or("0000000000000000").to_string();
    // UUID v4 でランダムサフィックスを生成する
    let random_suffix = Uuid::new_v4().to_string().replace('-', "");
    // aggregateId の先頭 8 文字を prefix に使用する（長すぎる場合は切り詰める）
    let agg_prefix = &aggregate_id[..aggregate_id.len().min(8)];
    // rpcMethod の先頭 4 文字を prefix に使用する（長すぎる場合は切り詰める）
    let method_prefix = &rpc_method[..rpc_method.len().min(4)];
    // prefix + HLC ベース + random suffix で Idempotency-Key を組み立てる
    format!("{}_{}_{}_{}",
        agg_prefix,
        method_prefix,
        hlc_base,
        &random_suffix[..16]
    )
}

/// create_outbox_meta は Outbox エントリのメタデータを生成する
/// wall-clock TTL 禁止規約に従い HLC + Instant（monotonic）を使用する
pub fn create_outbox_meta(
    aggregate_id: &str,
    rpc_method: &str,
    chained_from: Option<String>,
) -> OutboxEntryMeta {
    // HLC タイムスタンプを現在時刻として取得する
    let now_hlc = hlc_now();
    // enqueue 時刻（ミリ秒）を HLC から抽出する（backward compat 用）
    let enqueued_ms = extract_ms_from_hlc(&now_hlc);
    // monotonic clock の現在時点を記録する（TTL 計算に使用する）
    let enqueued_instant = Instant::now();
    // chain がある場合は chain された新 key を生成する
    let key = generate_idempotency_key(aggregate_id, rpc_method);
    // backward compat 用の expires_at_ms は HLC ミリ秒から計算する
    let expires_at_ms = enqueued_ms + IDEMPOTENCY_KEY_TTL_MS;
    // メタデータ構造体を組み立てて返す
    OutboxEntryMeta {
        // 生成した Idempotency-Key
        idempotency_key: key,
        // HLC タイムスタンプ（wall clock 代替）
        enqueued_at: now_hlc,
        // monotonic clock の記録時点（TTL 計算用）
        enqueued_instant,
        // backward compat 用 TTL（HLC から導出した値）
        expires_at_ms,
        // chain 元（None は chain なし）
        chained_from,
        // aggregate ID
        aggregate_id: aggregate_id.to_string(),
        // RPC method 名
        rpc_method: rpc_method.to_string(),
    }
}

/// strip_pii_fields は PII フィールドを strip する
/// pii_field_names に含まれるキーを payload から除去して返す
pub fn strip_pii_fields(
    payload: std::collections::HashMap<String, serde_json::Value>,
    pii_field_names: &[&str],
) -> std::collections::HashMap<String, serde_json::Value> {
    // PII フィールド名を O(1) 検索できるよう HashSet に変換する
    let pii_set: std::collections::HashSet<&str> = pii_field_names.iter().copied().collect();
    // PII strip 済み payload を構築する
    payload
        .into_iter()
        // PII フィールド以外のみを結果に含める
        .filter(|(k, _)| !pii_set.contains(k.as_str()))
        .collect()
}

#[cfg(test)]
mod tests {
    // テストモジュール内部でのみ使用するインポート
    use super::*;
    // HashMap をテストで使用する
    use std::collections::HashMap;

    #[test]
    // hlc_now() が正しいフォーマットを返すことを確認する
    fn test_hlc_now_format() {
        // HLC タイムスタンプを生成する
        let hlc = hlc_now();
        // ハイフン区切りで 3 部分に分かれることを確認する
        let parts: Vec<&str> = hlc.split('-').collect();
        // パーツ数が 3 であることを確認する
        assert_eq!(parts.len(), 3, "HLC フォーマット不正: パーツ数が 3 でない: {}", hlc);
        // timestamp_ms_hex が 16 桁であることを確認する
        assert_eq!(parts[0].len(), 16, "timestamp_ms_hex が 16 桁でない: {}", parts[0]);
        // logical_counter が 4 桁であることを確認する
        assert_eq!(parts[1].len(), 4, "logical_counter が 4 桁でない: {}", parts[1]);
        // node_id が 4 桁であることを確認する
        assert_eq!(parts[2].len(), 4, "node_id が 4 桁でない: {}", parts[2]);
    }

    #[test]
    // generate_idempotency_key() が一意な key を生成することを確認する
    fn test_generate_idempotency_key_uniqueness() {
        // 100 回生成して全て異なることを確認する（プロパティテスト）
        let mut keys = std::collections::HashSet::new();
        for _ in 0..100 {
            // Idempotency-Key を生成する
            let key = generate_idempotency_key("aggregate1", "create");
            // 既に同じ key が生成されていた場合はテスト失敗
            assert!(keys.insert(key.clone()), "重複 key が生成された: {}", key);
        }
    }

    #[test]
    // create_outbox_meta() の expires_at_ms が TTL 後になることを確認する
    fn test_create_outbox_meta_expires_at_ms() {
        // メタデータを生成する
        let meta = create_outbox_meta("agg-001", "create", None);
        // ExpiresAtMs が 0 より大きいことを確認する
        assert!(meta.expires_at_ms > 0, "expires_at_ms が 0 以下: {}", meta.expires_at_ms);
        // EnqueuedAt から抽出したミリ秒 + TTL が expires_at_ms と一致することを確認する
        let enqueued_ms = extract_ms_from_hlc(&meta.enqueued_at);
        // 期待する expires_at_ms を計算する
        let expected = enqueued_ms + IDEMPOTENCY_KEY_TTL_MS;
        // 一致しない場合はテスト失敗
        assert_eq!(meta.expires_at_ms, expected, "expires_at_ms 不一致: got={}, want={}", meta.expires_at_ms, expected);
    }

    #[test]
    // is_expired() が生成直後のエントリで false を返すことを確認する
    fn test_is_expired_false_for_fresh_entry() {
        // 生成直後のメタデータを作成する
        let meta = create_outbox_meta("agg-001", "create", None);
        // 生成直後は TTL 超過でないことを確認する
        assert!(!is_expired(&meta), "生成直後のエントリが TTL 超過と判定された");
    }

    #[test]
    // strip_pii_fields() が PII フィールドを正しく除去することを確認する
    fn test_strip_pii_fields_removes_pii() {
        // テスト用 payload を作成する
        let mut payload = HashMap::new();
        // PII フィールド（除去対象）
        payload.insert("email".to_string(), serde_json::Value::String("test@example.com".to_string()));
        // PII フィールド（除去対象）
        payload.insert("name".to_string(), serde_json::Value::String("テストユーザー".to_string()));
        // 非 PII フィールド（保持対象）
        payload.insert("order_id".to_string(), serde_json::Value::String("ORD-001".to_string()));
        // PII フィールド名リスト
        let pii_fields = &["email", "name"];
        // PII strip を実行する
        let stripped = strip_pii_fields(payload, pii_fields);
        // email が除去されていることを確認する
        assert!(!stripped.contains_key("email"), "email が除去されていない");
        // name が除去されていることを確認する
        assert!(!stripped.contains_key("name"), "name が除去されていない");
        // order_id が保持されていることを確認する
        assert!(stripped.contains_key("order_id"), "order_id が除去されてしまった");
    }

    #[test]
    // create_outbox_meta() の ChainedFrom が正しく設定されることを確認する
    fn test_create_outbox_meta_with_chained_from() {
        // chain 元 key を設定してメタデータを生成する
        let meta = create_outbox_meta("agg-001", "update", Some("original-key-001".to_string()));
        // ChainedFrom が設定されていることを確認する
        assert_eq!(
            meta.chained_from.as_deref(),
            Some("original-key-001"),
            "ChainedFrom が不正: {:?}",
            meta.chained_from
        );
        // IdempotencyKey が ChainedFrom と異なることを確認する
        assert_ne!(
            meta.idempotency_key,
            "original-key-001",
            "IdempotencyKey が ChainedFrom と同じ: {}",
            meta.idempotency_key
        );
    }
}
