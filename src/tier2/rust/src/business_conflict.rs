// k1s0 tier2 business_conflict detector — Rust 実装
// 10_テナント分離適合仕様.md §business_conflict_subtypes に基づく 4 subtype 検出ロジック
// src/tier2/business_conflict/subtypes.yaml の detector_rule を Rust で物理化する
// 409 レスポンスの conflict_subtype フィールドに設定する値を生成する

// serde: ConflictSubtype の JSON シリアライズに使用する
use serde::{Deserialize, Serialize};

// ConflictSubtype: 4 種の business conflict サブタイプを表す列挙型
// subtypes.yaml の 4 エントリ（stale_write / lost_update / supersede / concurrent_edit）と 1:1 対応する
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictSubtype {
    // stale_write: 書き込み時点でデータが陳腐化していた（楽観ロック失敗）
    // detector_rule: last_seen_version != current_version
    StaleWrite,
    // lost_update: 並行書き込みにより更新が消失した（フィールド差分重複）
    // detector_rule: concurrent_write_detected && field_overlap
    LostUpdate,
    // supersede: 後発の書き込みが先発を上書きした（同一アクターによる意図的な上書き）
    // detector_rule: supersede_flag == true
    Supersede,
    // concurrent_edit: 同一フィールドへの同時編集が検出された（プレゼンス情報ベース）
    // detector_rule: field_intersection_detected && concurrent_writers >= 2
    ConcurrentEdit,
}

// ConflictDetector: 4 subtype の detector_rule を評価するエンジン
// 引数を受け取り、subtypes.yaml の detector_rule 順序（優先度順）で評価する
pub struct ConflictDetector;

impl ConflictDetector {
    // detect: 引数から conflict subtype を判定して返す
    // 戻り値: ConflictSubtype が確定した場合は Some(subtype)、conflict がない場合は None
    //
    // 引数:
    //   last_seen_version  — クライアントが送信した base version（楽観ロックの基準点）
    //   current_version    — サーバー側の現在 version
    //   field_diff         — クライアントが変更したフィールド名のスライス（空 = 変更なし）
    //   concurrent_writers — 同一 aggregate を並行して編集しているアクター数（presence から取得）
    //   supersede_flag     — 同一アクターによる意図的な上書きフラグ（クライアントが設定）
    //
    // 評価順序: supersede → concurrent_edit → lost_update → stale_write
    // supersede は他の全 subtype より高い優先度を持つため最初に評価する
    pub fn detect(
        // クライアントが送信した base version（楽観ロックの基準点）
        last_seen_version: i64,
        // サーバー側の現在 version（DB から取得した最新値）
        current_version: i64,
        // クライアントが変更したフィールド名のリスト（overlap 検出に使用する）
        field_diff: &[String],
        // presence 情報から取得した並行ライター数（concurrent_edit 検出に使用する）
        concurrent_writers: usize,
        // 同一アクターによる意図的な上書きフラグ（supersede 検出に使用する）
        supersede_flag: bool,
    ) -> Option<ConflictSubtype> {
        // P1: supersede_flag が true の場合は Supersede を最優先で返す
        // detector_rule: supersede_flag == true（subtypes.yaml 参照）
        if supersede_flag {
            // 意図的な上書き: Supersede subtype を返す
            return Some(ConflictSubtype::Supersede);
        }

        // P2: 並行ライターが 2 人以上 かつ フィールド差分が非空の場合は ConcurrentEdit を返す
        // detector_rule: field_intersection_detected && concurrent_writers >= 2（subtypes.yaml 参照）
        // field_diff が非空 = クライアントが少なくとも 1 フィールドを変更している（intersection 有りとみなす）
        if concurrent_writers >= 2 && !field_diff.is_empty() {
            // 同時編集検出: ConcurrentEdit subtype を返す
            return Some(ConflictSubtype::ConcurrentEdit);
        }

        // P3: version 不一致 かつ フィールド差分が重複している場合は LostUpdate を返す
        // detector_rule: concurrent_write_detected && field_overlap（subtypes.yaml 参照）
        // concurrent_write_detected = last_seen_version != current_version かつ concurrent_writers >= 1
        // field_overlap = field_diff が非空（重複フィールドが存在するとみなす）
        if last_seen_version != current_version && concurrent_writers >= 1 && !field_diff.is_empty() {
            // 更新消失: LostUpdate subtype を返す
            return Some(ConflictSubtype::LostUpdate);
        }

        // P4: version のみ不一致の場合は StaleWrite を返す（最低優先度）
        // detector_rule: last_seen_version != current_version（subtypes.yaml 参照）
        if last_seen_version != current_version {
            // 陳腐化書込: StaleWrite subtype を返す
            return Some(ConflictSubtype::StaleWrite);
        }

        // 上記のいずれにも該当しない場合は conflict なし（None を返す）
        None
    }
}

// conflict_subtype_for_409: 409 レスポンスの conflict_subtype フィールド用文字列を返す
// ConflictSubtype を 409 レスポンスボディに埋め込む際に使用する
pub fn conflict_subtype_for_409(subtype: &ConflictSubtype) -> &'static str {
    // subtype に応じて文字列を返す（subtypes.yaml の subtype_id と 1:1 対応する）
    match subtype {
        // stale_write: 書込時点でデータが陳腐化していた
        ConflictSubtype::StaleWrite => "stale_write",
        // lost_update: 並行書込により更新が消失した
        ConflictSubtype::LostUpdate => "lost_update",
        // supersede: 後発書込が先発を上書きした
        ConflictSubtype::Supersede => "supersede",
        // concurrent_edit: 同一フィールドへの同時編集を検出
        ConflictSubtype::ConcurrentEdit => "concurrent_edit",
    }
}

#[cfg(test)]
mod tests {
    // テストモジュール内部のインポート
    use super::*;

    #[test]
    // supersede_flag が true の場合は Supersede を返すことを確認する
    fn test_detect_supersede() {
        // supersede_flag = true → Supersede が最優先で返ることを検証する
        let result = ConflictDetector::detect(1, 5, &["field_a".to_string()], 0, true);
        // Some(Supersede) が返ることを確認する
        assert_eq!(result, Some(ConflictSubtype::Supersede));
    }

    #[test]
    // concurrent_writers >= 2 かつ field_diff が非空の場合は ConcurrentEdit を返すことを確認する
    fn test_detect_concurrent_edit() {
        // 2 人のライターが同一フィールドを編集している場合を検証する
        let result = ConflictDetector::detect(3, 3, &["field_b".to_string()], 2, false);
        // Some(ConcurrentEdit) が返ることを確認する
        assert_eq!(result, Some(ConflictSubtype::ConcurrentEdit));
    }

    #[test]
    // version 不一致 かつ concurrent_writers >= 1 の場合は LostUpdate を返すことを確認する
    fn test_detect_lost_update() {
        // version が古く、並行ライターが存在し、フィールド差分がある場合を検証する
        let result = ConflictDetector::detect(2, 5, &["field_c".to_string()], 1, false);
        // Some(LostUpdate) が返ることを確認する
        assert_eq!(result, Some(ConflictSubtype::LostUpdate));
    }

    #[test]
    // version のみ不一致の場合は StaleWrite を返すことを確認する
    fn test_detect_stale_write() {
        // concurrent_writers = 0 かつ version が古い場合を検証する
        let result = ConflictDetector::detect(1, 3, &[], 0, false);
        // Some(StaleWrite) が返ることを確認する
        assert_eq!(result, Some(ConflictSubtype::StaleWrite));
    }

    #[test]
    // conflict がない場合は None を返すことを確認する
    fn test_detect_no_conflict() {
        // version 一致 かつ concurrent_writers = 0 かつ supersede_flag = false の場合を検証する
        let result = ConflictDetector::detect(5, 5, &[], 0, false);
        // None が返ることを確認する
        assert_eq!(result, None);
    }

    #[test]
    // conflict_subtype_for_409: 各 subtype が正しい文字列を返すことを確認する
    fn test_conflict_subtype_for_409_strings() {
        // StaleWrite → "stale_write" を確認する
        assert_eq!(conflict_subtype_for_409(&ConflictSubtype::StaleWrite), "stale_write");
        // LostUpdate → "lost_update" を確認する
        assert_eq!(conflict_subtype_for_409(&ConflictSubtype::LostUpdate), "lost_update");
        // Supersede → "supersede" を確認する
        assert_eq!(conflict_subtype_for_409(&ConflictSubtype::Supersede), "supersede");
        // ConcurrentEdit → "concurrent_edit" を確認する
        assert_eq!(conflict_subtype_for_409(&ConflictSubtype::ConcurrentEdit), "concurrent_edit");
    }
}
