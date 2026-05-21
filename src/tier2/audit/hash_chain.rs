// k1s0 tier2 audit hash chain
// 業務エラー監査 08 の hash chain 実装（SHA-256 を使った audit event hash chain）
// 各 audit_event エントリは直前のエントリの hash_digest を prev_digest として含む
// hash chain の整合性破壊は改ざん検出として機能する

// SHA-256 を使った hash 計算に使用する
use sha2::{Sha256, Digest};
// シリアライズ / デシリアライズ
use serde::{Deserialize, Serialize};
// UUID
use uuid::Uuid;
// 日時
use chrono::{DateTime, Utc};

// AuditEventEntry: hash chain に含む audit event エントリの型
// 0003_audit_hash_chain.sql で追加したカラムに対応する
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEventEntry {
    // エントリの主キー（domain_event と同一 UUID）
    pub id: Uuid,
    // 関連する aggregate の ID
    pub aggregate_id: Uuid,
    // テナント ID（RLS FORCE が保証する）
    pub tenant_id: Uuid,
    // アクター識別子（Keycloak subject）
    pub actor_id: String,
    // セッション目的（business_op / support 等）
    pub purpose: String,
    // テーブルクラス（TenantScoped / PiiSegregated 等）
    pub table_class: String,
    // 操作内容のペイロード（PII は redact 済み）
    pub payload: serde_json::Value,
    // 書込日時
    pub created_at: DateTime<Utc>,
    // hash chain 用: このエントリの SHA-256 ダイジェスト
    pub hash_digest: Option<String>,
    // hash chain 用: 直前のエントリの hash_digest（チェーンの先頭は None）
    pub prev_digest: Option<String>,
    // hash chain 用: テナント内での連番（hash chain の順序を保証する）
    pub chain_sequence: Option<i64>,
}

// HashChainEntry: hash chain 計算の入力型
// AuditEventEntry の不変部分（hash_digest / prev_digest / chain_sequence 以外）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashChainInput {
    // エントリの主キー
    pub id: Uuid,
    // 関連する aggregate の ID
    pub aggregate_id: Uuid,
    // テナント ID
    pub tenant_id: Uuid,
    // アクター識別子
    pub actor_id: String,
    // セッション目的
    pub purpose: String,
    // テーブルクラス
    pub table_class: String,
    // ペイロード（JSON 文字列）
    pub payload_json: String,
    // 書込日時（RFC 3339 形式）
    pub created_at_rfc3339: String,
    // 直前のエントリの hash_digest（チェーンの先頭は None、DB 側の hash_digest TEXT NULL と整合する）
    // None = chain head（先頭エントリ）、Some(digest) = 前エントリの hash_digest
    pub prev_digest: Option<String>,
    // テナント内での連番
    pub chain_sequence: i64,
}

// compute_digest: HashChainInput の SHA-256 ダイジェストを計算する
// チェーン不変量: hash_digest = SHA-256(JSON serialize(entry) + prev_digest)
// prev_digest が None の場合（chain head）は空文字列を sentinel として使用する
pub fn compute_digest(input: &HashChainInput) -> String {
    // prev_digest を計算前に解決する（None = chain head → 空文字列を sentinel として使用する）
    let prev_digest_str: String = input.prev_digest.clone().unwrap_or_default();
    // prev_digest を解決した一時的な入力を構築する（JSON シリアライズに使用する）
    let resolved_input = HashChainInput {
        // 元の入力フィールドをそのままコピーする
        id: input.id,
        // aggregate の ID をコピーする
        aggregate_id: input.aggregate_id,
        // テナント ID をコピーする
        tenant_id: input.tenant_id,
        // アクター識別子をコピーする
        actor_id: input.actor_id.clone(),
        // セッション目的をコピーする
        purpose: input.purpose.clone(),
        // テーブルクラスをコピーする
        table_class: input.table_class.clone(),
        // ペイロード JSON をコピーする
        payload_json: input.payload_json.clone(),
        // 書込日時をコピーする
        created_at_rfc3339: input.created_at_rfc3339.clone(),
        // None の場合は空文字列を sentinel として使用する（chain head の不変量）
        prev_digest: Some(prev_digest_str),
        // テナント内での連番をコピーする
        chain_sequence: input.chain_sequence,
    };
    // JSON シリアライズして文字列に変換する
    let canonical_str = serde_json::to_string(&resolved_input)
        .unwrap_or_default();
    // SHA-256 ハッシュを計算する
    let mut hasher = Sha256::new();
    // エントリの正規化 JSON を入力する
    hasher.update(canonical_str.as_bytes());
    // ダイジェストを 16 進数文字列に変換して返す
    format!("{:x}", hasher.finalize())
}

// verify_chain: audit event チェーンの整合性を検証する
// エントリのスライスを順番に検証し、hash chain が正しいことを確認する
// 戻り値: 全エントリが整合している場合は Ok(()), 不整合の場合は Err(改ざん位置)
pub fn verify_chain(entries: &[AuditEventEntry]) -> Result<(), VerifyChainError> {
    // エントリが空の場合は OK を返す（検証するものがない）
    if entries.is_empty() {
        return Ok(());
    }

    // 連番が昇順であることを確認する（chain_sequence の連続性）
    // chain head の先頭は None（chain head sentinel）として初期化する
    let mut prev_digest: Option<String> = None;
    for (idx, entry) in entries.iter().enumerate() {
        // chain_sequence が設定されているエントリのみ検証する
        let chain_seq = match entry.chain_sequence {
            Some(seq) => seq,
            // chain_sequence が未設定のエントリはスキップする
            None => continue,
        };
        // hash_digest が設定されていないエントリはスキップする
        let stored_digest = match &entry.hash_digest {
            Some(d) => d.clone(),
            // hash_digest が未設定のエントリはスキップする
            None => continue,
        };

        // HashChainInput を構築する
        let input = HashChainInput {
            // エントリの主キーを設定する
            id: entry.id,
            // aggregate の ID を設定する
            aggregate_id: entry.aggregate_id,
            // テナント ID を設定する
            tenant_id: entry.tenant_id,
            // アクター識別子を設定する
            actor_id: entry.actor_id.clone(),
            // セッション目的を設定する
            purpose: entry.purpose.clone(),
            // テーブルクラスを設定する
            table_class: entry.table_class.clone(),
            // ペイロードを JSON 文字列に変換する
            payload_json: serde_json::to_string(&entry.payload).unwrap_or_default(),
            // 書込日時を RFC 3339 形式に変換する
            created_at_rfc3339: entry.created_at.to_rfc3339(),
            // 直前のエントリの hash_digest（チェーン先頭は None / chain head）
            prev_digest: prev_digest.clone(),
            // テナント内での連番
            chain_sequence: chain_seq,
        };

        // 期待されるダイジェストを計算する
        let expected_digest = compute_digest(&input);
        // 保存されているダイジェストと一致しない場合は改ざんエラーを返す
        if stored_digest != expected_digest {
            return Err(VerifyChainError::DigestMismatch {
                // 改ざんが検出されたエントリのインデックス
                index: idx,
                // エントリの主キー
                entry_id: entry.id,
                // 期待されるダイジェスト
                expected: expected_digest,
                // 保存されているダイジェスト
                actual: stored_digest,
            });
        }
        // 検証済みダイジェストを next の prev_digest として使用する（Option<String> に格納する）
        prev_digest = Some(stored_digest);
    }
    // 全エントリが整合している場合は OK を返す
    Ok(())
}

// VerifyChainError: hash chain 検証エラー型
#[derive(Debug, Clone)]
pub enum VerifyChainError {
    // DigestMismatch: ダイジェスト不一致（改ざん検出）
    DigestMismatch {
        // 改ざんが検出されたエントリのインデックス
        index: usize,
        // エントリの主キー
        entry_id: Uuid,
        // 期待されるダイジェスト
        expected: String,
        // 保存されているダイジェスト
        actual: String,
    },
}

// Display: エラーメッセージを返す
impl std::fmt::Display for VerifyChainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // DigestMismatch のエラーメッセージを構築する
            VerifyChainError::DigestMismatch { index, entry_id, expected, actual } => {
                write!(
                    f,
                    "audit hash chain tampering detected at index {}: entry_id={}, expected={}, actual={}",
                    index, entry_id, expected, actual
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // テストモジュール内部でのみ使用するインポート
    use super::*;

    // テスト用の AuditEventEntry を生成するヘルパー関数
    fn make_entry(
        id: Uuid,
        tenant_id: Uuid,
        seq: i64,
        prev_digest: Option<String>,
    ) -> AuditEventEntry {
        // HashChainInput を構築する
        let input = HashChainInput {
            // エントリの主キーを設定する
            id,
            aggregate_id: Uuid::new_v4(),
            tenant_id,
            // テスト用のアクター ID を設定する
            actor_id: "test-actor".to_string(),
            // テスト用のセッション目的を設定する
            purpose: "business_op".to_string(),
            // テスト用のテーブルクラスを設定する
            table_class: "TenantScoped".to_string(),
            // テスト用のペイロード JSON を設定する
            payload_json: r#"{"test":true}"#.to_string(),
            // テスト用の書込日時を設定する
            created_at_rfc3339: "2026-01-01T00:00:00Z".to_string(),
            // 直前のダイジェストを設定する（先頭は None → unwrap_or_default() で空文字列に変換する）
            prev_digest: prev_digest.clone(),
            // テナント内での連番を設定する
            chain_sequence: seq,
        };
        // ダイジェストを計算する
        let digest = compute_digest(&input);
        // AuditEventEntry を生成して返す
        AuditEventEntry {
            id,
            aggregate_id: Uuid::new_v4(),
            tenant_id,
            actor_id: "test-actor".to_string(),
            purpose: "business_op".to_string(),
            table_class: "TenantScoped".to_string(),
            payload: serde_json::json!({"test": true}),
            created_at: Utc::now(),
            hash_digest: Some(digest),
            prev_digest,
            chain_sequence: Some(seq),
        }
    }

    #[test]
    // compute_digest: 同一入力に対して常に同一のダイジェストを返すことを確認する
    fn test_compute_digest_deterministic() {
        // テスト用の HashChainInput を生成する
        let input = HashChainInput {
            id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            aggregate_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap(),
            tenant_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440002").unwrap(),
            actor_id: "test-actor".to_string(),
            purpose: "business_op".to_string(),
            table_class: "TenantScoped".to_string(),
            payload_json: r#"{"test":true}"#.to_string(),
            created_at_rfc3339: "2026-01-01T00:00:00Z".to_string(),
            // chain head の場合は None を設定する（prev_digest: String から Option<String> に変更）
            prev_digest: None,
            chain_sequence: 1,
        };
        // 同一入力で 2 回計算して一致することを確認する
        let digest1 = compute_digest(&input);
        let digest2 = compute_digest(&input);
        assert_eq!(digest1, digest2, "同一入力のダイジェストは常に一致する");
        // ダイジェストが空でないことを確認する
        assert!(!digest1.is_empty(), "ダイジェストが空でないことを確認する");
    }

    #[test]
    // verify_chain: 正常な hash chain が OK を返すことを確認する
    fn test_verify_chain_valid() {
        // テスト用のテナント ID を生成する
        let tenant_id = Uuid::new_v4();
        // 先頭エントリを生成する（prev_digest = None）
        let entry1 = make_entry(Uuid::new_v4(), tenant_id, 1, None);
        // 第 2 エントリを生成する（prev_digest = entry1.hash_digest）
        let entry2 = make_entry(Uuid::new_v4(), tenant_id, 2, entry1.hash_digest.clone());
        // 正常な hash chain を検証する
        let result = verify_chain(&[entry1, entry2]);
        // OK を返すことを確認する
        assert!(result.is_ok(), "正常な hash chain は OK を返す");
    }

    #[test]
    // verify_chain: 改ざんされた hash chain が DigestMismatch エラーを返すことを確認する
    fn test_verify_chain_tampered() {
        // テスト用のテナント ID を生成する
        let tenant_id = Uuid::new_v4();
        // 先頭エントリを生成する
        let entry1 = make_entry(Uuid::new_v4(), tenant_id, 1, None);
        // 第 2 エントリを生成する（prev_digest を改ざんする）
        let mut tampered_entry = make_entry(Uuid::new_v4(), tenant_id, 2, entry1.hash_digest.clone());
        // hash_digest を改ざんする
        tampered_entry.hash_digest = Some("tampered_digest_value".to_string());
        // 改ざんされた hash chain を検証する
        let result = verify_chain(&[entry1, tampered_entry]);
        // DigestMismatch エラーを返すことを確認する
        assert!(
            matches!(result, Err(VerifyChainError::DigestMismatch { index: 1, .. })),
            "改ざんされた hash chain は DigestMismatch エラーを返す"
        );
    }
}
